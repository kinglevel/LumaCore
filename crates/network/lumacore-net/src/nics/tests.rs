// ███╗   ██╗██╗ ██████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ████╗  ██║██║██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██╔██╗ ██║██║██║     ███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║╚██╗██║██║██║     ╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║ ╚████║██║╚██████╗███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═══╝╚═╝ ╚═════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: nics tests

//! Tests of `nics`.
//!
//! Owns:          the unit tests of nics.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn nic(name: &str, ip: [u8; 4]) -> Nic {
    let ip = Ipv4Addr::from(ip);
    let netmask = Ipv4Addr::new(255, 255, 255, 0);
    Nic {
        name: name.into(),
        ip,
        netmask,
        broadcast: directed_broadcast(ip, netmask),
        index: None,
    }
}

#[test]
fn mappings_find_by_name_ip_or_subnet() {
    let nics = vec![nic("en0", [192, 0, 2, 5]), nic("en1", [10, 33, 33, 9])];
    assert_eq!(
        resolve("EN1", &nics).unwrap().ip,
        Ipv4Addr::new(10, 33, 33, 9)
    );
    assert_eq!(resolve("192.0.2.5", &nics).unwrap().name, "en0");
    assert_eq!(resolve("10.33.33.0/24", &nics).unwrap().name, "en1");
    assert!(resolve("10.9.0.0/16", &nics).is_none());
    assert_eq!(
        resolve("en0", &nics).unwrap().broadcast,
        Ipv4Addr::new(192, 0, 2, 255)
    );
}
