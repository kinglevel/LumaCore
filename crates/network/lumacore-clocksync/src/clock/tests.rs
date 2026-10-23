//  ██████╗██╗      ██████╗  ██████╗██╗  ██╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██║     ██╔═══██╗██╔════╝██║ ██╔╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║     ██║     ██║   ██║██║     █████╔╝        ██║   █████╗  ███████╗   ██║   ███████╗
// ██║     ██║     ██║   ██║██║     ██╔═██╗        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ╚██████╗███████╗╚██████╔╝╚██████╗██║  ██╗       ██║   ███████╗███████║   ██║   ███████║
//  ╚═════╝╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: clock tests

//! Tests of `clock`.
//!
//! Owns:          the unit tests of clock.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §24 "Session clock (researched)", §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

/// A master running `ppm` fast and `offset` µs ahead of local time.
fn sample(local: u64, offset: i64, ppm: f64, rtt: u64) -> Sample {
    Sample {
        local,
        delta: offset + (local as f64 * ppm / 1e6) as i64,
        rtt,
    }
}

#[test]
fn regression_finds_frequency_and_median_the_offset() {
    let mut f = Filter::default();
    let mut est = Estimate::default();
    for i in 0..32u64 {
        let local = i * 500_000;
        // Every 5th ping waited behind something: a long round trip.
        let rtt = if i % 5 == 0 { 9_000 } else { 400 + (i % 3) * 20 };
        if let Some(e) = f.add(sample(local, 1_000_000, 50.0, rtt), local) {
            est = e;
        }
    }
    let now = 31 * 500_000;
    assert!((est.frequency_ppb - 50_000).abs() < 2_000, "frequency {}", est.frequency_ppb);
    let expected = 1_000_000 + (now as f64 * 50.0 / 1e6) as i64;
    assert!((est.delta_now - expected).abs() < 50, "offset {} vs {expected}", est.delta_now);
    assert!(est.used < 32, "slow pings are gated out");
    assert_eq!(est.path_delay_us, 200);
}

#[test]
fn a_single_outlier_is_dropped_the_second_kept() {
    let mut f = Filter::default();
    for i in 0..10u64 {
        f.add(sample(i * 500_000, 0, 0.0, 400), i * 500_000);
    }
    assert!(f.add(sample(5_000_000, 0, 0.0, 50_000), 5_000_000).is_none());
    assert!(f.add(sample(5_500_000, 0, 0.0, 50_000), 5_500_000).is_some());
}

#[test]
fn corrections_are_bounded_and_never_reverse_time() {
    assert_eq!(correction_for(50), 0, "dead band");
    assert_eq!(correction_for(1_000), 500_000); // 1 ms over 2 s = 500 ppm
    assert_eq!(correction_for(15_000), MAX_CORRECTION_PPB);
    assert_eq!(correction_for(-500_000), -CATCH_UP_PPB);
    // The slowest the clock runs is 98 % of real time.
    const { assert!(1_000_000_000 - CATCH_UP_PPB - MAX_FREQUENCY_PPB > 0) };
}

#[test]
fn follows_a_master_over_udp() {
    use std::sync::Arc;
    let master = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = master.local_addr().unwrap();
    let board = Arc::new(HealthBoard::default());
    let served = board.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let serving = stop.clone();
    master.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
    // The "master": its session clock is 3 s ahead of ours.
    let server = thread::spawn(move || {
        let mut buf = [0u8; 2048];
        while !serving.load(Ordering::Relaxed) {
            let Ok((n, from)) = master.recv_from(&mut buf) else { continue };
            let t2 = clock::local_us() + 3_000_000;
            if let Ok(RemoteRequest::Time { seq, t1, .. }) = serde_json::from_slice(&buf[..n]) {
                let reply = TimeReply { seq, t1, t2, t3: clock::local_us() + 3_000_000, master: true };
                let _ = master.send_to(&serde_json::to_vec(&reply).unwrap(), from);
            }
            let _ = &served;
        }
    });
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(ANSWER_WAIT)).unwrap();
    let target = Target {
        addr,
        master: "M".into(),
        epoch: 1,
        from: "B".into(),
    };
    let mut f = Filter::default();
    let mut est = None;
    for seq in 1..=16 {
        if let Some(s) = ping(&socket, &target, seq) {
            est = f.add(s, clock::local_us()).or(est);
        }
    }
    stop.store(true, Ordering::Relaxed);
    server.join().unwrap();
    let e = est.expect("answers");
    assert!((e.delta_now - 3_000_000).abs() < 2_000, "delta {}", e.delta_now);
    assert!(e.path_delay_us < 2_000);
}
