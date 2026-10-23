// ███████╗██╗  ██╗███████╗██╗     ██╗         ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██║  ██║██╔════╝██║     ██║         ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████╗███████║█████╗  ██║     ██║            ██║   █████╗  ███████╗   ██║   ███████╗
// ╚════██║██╔══██║██╔══╝  ██║     ██║            ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████║██║  ██║███████╗███████╗███████╗       ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: shell tests

//! Tests of `shell`.
//!
//! Owns:          the unit tests of shell.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §23 "Step 8", §30 "12b", R1-67, Q40, §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[test]
fn a_reported_client_address_is_believed_only_from_loopback() {
    let lo = "127.0.0.1".parse().unwrap();
    let far = "10.0.0.9".parse().unwrap();
    assert_eq!(super::bridged_client(lo, Some("10.0.0.5")), "10.0.0.5".parse().ok());
    assert_eq!(super::bridged_client(far, Some("127.0.0.1")), None);
    assert_eq!(super::bridged_client(lo, None), None);
}

/// A 1 MB line in small chunks is searched once, not once per chunk
/// (R1-19); two lines in one chunk and a split line still come out whole.
#[test]
fn a_long_line_in_many_chunks_is_read_in_linear_time() {
    use lumacore_wire::ShellClientPacket;
    let hello = |user: String| ShellClientPacket::Hello {
        user,
        width: 80,
        height: 24,
        password: None,
        token: None,
        client_addr: None,
        protocol: 0,
        device: None,
    };
    let mut bytes = serde_json::to_vec(&hello("x".repeat(1 << 20))).unwrap();
    bytes.push(b'\n');
    bytes.extend(serde_json::to_vec(&hello("short".into())).unwrap());
    bytes.push(b'\n');
    let mut buf = Vec::new();
    let mut packets = Vec::new();
    let started = std::time::Instant::now();
    for chunk in bytes.chunks(64) {
        super::take_packets(&mut buf, chunk, &mut packets).unwrap();
    }
    assert!(started.elapsed() < std::time::Duration::from_secs(2), "{:?}", started.elapsed());
    assert!(buf.is_empty());
    let users: Vec<usize> = packets
        .iter()
        .map(|p| match p {
            ShellClientPacket::Hello { user, .. } => user.len(),
            _ => 0,
        })
        .collect();
    assert_eq!(users, [1 << 20, 5]);
}
