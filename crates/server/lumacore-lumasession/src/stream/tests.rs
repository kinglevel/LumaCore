// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `stream`.
//!
//! Owns:          the unit tests of stream.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §21 "6b", "6c", §34 19.3.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

#[test]
fn a_file_travels_by_hash_and_a_damaged_one_is_refused() {
    let bytes = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x02\0\0\0\x02stream-test".to_vec();
    let sha = lumacore_luma::media::store(&bytes, "png").unwrap();
    let want = lumacore_objects::session_link::WantedFile { sha256: sha.clone(), ext: "png".into() };
    let Some(StreamMsg::File { sha256, ext, data }) = file_frame(&want) else {
        panic!("the master has the file");
    };
    assert!(keep_file(&sha256, &ext, &data).is_ok());
    let err = keep_file(&"cd".repeat(32), &ext, &data).unwrap_err();
    assert!(err.contains("damaged"), "{err}");
    assert!(keep_file(&sha256, "exe", &data).is_err(), "only show file types");
}

#[test]
fn frames_round_trip_compressed() {
    let msg = StreamMsg::Command {
        id: 7,
        user: "LD".into(),
        line: "Fixture 1 At 50".repeat(100),
        gesture: None,
    };
    let mut buf = Vec::new();
    write_frame(&mut buf, &msg).unwrap();
    // zstd shrinks the repeated line well below its JSON size.
    assert!(buf.len() < 300, "{} bytes", buf.len());
    match read_frame(&mut &buf[..]).unwrap() {
        StreamMsg::Command { id, user, line, .. } => {
            assert_eq!((id, user.as_str()), (7, "LD"));
            assert!(line.starts_with("Fixture 1 At 50"));
        }
        other => panic!("{other:?}"),
    }
}

/// A member's upload says which library it is for: a fixture file (`gdtf`) goes to the master's
/// `library/gdtf/`, anything else to `library/media/`.
#[test]
fn an_upload_says_whether_it_is_a_fixture_file() {
    let msg = StreamMsg::Upload { name: "Spot.gdtf".into(), data: "AA==".into(), gdtf: true };
    let mut buf = Vec::new();
    write_frame(&mut buf, &msg).unwrap();
    assert!(matches!(read_frame(&mut &buf[..]).unwrap(), StreamMsg::Upload { gdtf: true, .. }));
}

#[test]
fn oversized_frames_are_refused() {
    let mut buf = (u32::MAX).to_be_bytes().to_vec();
    buf.extend_from_slice(&[0; 8]);
    assert!(read_frame(&mut &buf[..]).is_err());
}

/// A reader that counts how much a caller asked it for at once.
struct Counting<'a> {
    data: &'a [u8],
    biggest_ask: usize,
}

impl Read for Counting<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.biggest_ask = self.biggest_ask.max(buf.len());
        self.data.read(buf)
    }
}

/// Hands out at most one piece per read, and lets "time pass" between reads:
/// each read finds out whether the reader counted the last piece as
/// heard, then makes the link look silent again.
struct Trickle<'a> {
    data: &'a [u8],
    stats: &'a LinkStats,
    reads: usize,
    unheard: usize,
}

impl Read for Trickle<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.reads > 0 && self.stats.last_rx_ms.load(Ordering::Relaxed) == 0 {
            self.unheard += 1;
        }
        self.reads += 1;
        self.stats.last_rx_ms.store(0, Ordering::Relaxed);
        let n = buf.len().min(CHUNK);
        self.data.read(&mut buf[..n])
    }
}

#[test]
fn a_big_frame_trickling_in_counts_as_heard() {
    // ~1 MB that zstd can't shrink much: many reads.
    let mut x = 0x2545_f491_4f6c_dd1du64;
    let data: String = (0..1 << 20)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            char::from(b'a' + (x % 26) as u8)
        })
        .collect();
    let msg = StreamMsg::File { sha256: "x".into(), ext: "png".into(), data };
    let mut buf = Vec::new();
    write_frame(&mut buf, &msg).unwrap();
    assert!(buf.len() > 4 * CHUNK, "{} bytes", buf.len());
    let stats = LinkStats::default();
    let mut r = Trickle { data: &buf, stats: &stats, reads: 0, unheard: 0 };
    assert!(matches!(read_frame_heard(&mut r, &stats).unwrap(), StreamMsg::File { .. }));
    assert!(r.reads > 4);
    // Every piece that arrived was counted before the next read.
    assert_eq!(r.unheard, 0, "{} of {} reads found the link silent", r.unheard, r.reads);
    assert!(stats.silent_for() < Duration::from_secs(1), "heard at the end");
}

#[test]
fn a_header_claiming_a_gigabyte_then_eof_allocates_nothing() {
    // 1 GiB announced (the cap), then the stream ends.
    let header = (MAX_FRAME as u32).to_be_bytes();
    let mut r = Counting { data: &header, biggest_ask: 0 };
    assert!(read_frame(&mut r).is_err());
    // Nothing was set aside for the announced size.
    assert!(r.biggest_ask < 1 << 20, "asked for {} bytes at once", r.biggest_ask);
}
