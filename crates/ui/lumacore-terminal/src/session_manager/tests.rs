// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `sessions`.
//!
//! Owns:          the unit tests of sessions.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §23 "Step 8", §30 "12f"

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn make_client(width: u16, height: u16, secs: u64) -> ClientHandle {
    ClientHandle {
        id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        user: "Admin".to_string(),
        remote_addr: "127.0.0.1:47654".parse().expect("socket addr"),
        width,
        height,
        attached_at: SystemTime::UNIX_EPOCH,
        last_seen_at: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs),
        view: ClientView::default(),
    }
}

#[test]
fn smallest_policy_uses_smallest_attached_client() {
    let c1 = make_client(120, 40, 1);
    let c2 = make_client(80, 24, 2);
    let (width, height) = select_size(ShellSizePolicy::Smallest, 100, 30, vec![&c1, &c2]);
    assert_eq!((width, height), (80, 24));
}

#[test]
fn latest_policy_uses_most_recent_client() {
    let c1 = make_client(120, 40, 1);
    let c2 = make_client(80, 24, 2);
    let (width, height) = select_size(ShellSizePolicy::Latest, 100, 30, vec![&c1, &c2]);
    assert_eq!((width, height), (80, 24));
}
