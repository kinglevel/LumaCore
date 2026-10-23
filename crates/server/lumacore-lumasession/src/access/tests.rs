//  █████╗  ██████╗ ██████╗███████╗███████╗███████╗    ████████╗███████╗███████╗████████╗███████╗
// ██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██╔════╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████║██║     ██║     █████╗  ███████╗███████╗       ██║   █████╗  ███████╗   ██║   ███████╗
// ██╔══██║██║     ██║     ██╔══╝  ╚════██║╚════██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║  ██║╚██████╗╚██████╗███████╗███████║███████║       ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═╝ ╚═════╝ ╚═════╝╚══════╝╚══════╝╚══════╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: access tests

//! Tests of `access`.
//!
//! Owns:          the unit tests of access.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §30 "12d part 4", §34 19.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn nic(name: &str, ip: [u8; 4]) -> Nic {
    Nic {
        name: name.into(),
        ip: Ipv4Addr::from(ip),
        netmask: Ipv4Addr::new(255, 255, 255, 0),
        broadcast: Ipv4Addr::new(ip[0], ip[1], ip[2], 255),
        index: None,
    }
}

#[test]
fn services_follow_the_station_and_loopback_always_serves() {
    let nics = vec![nic("en0", [10, 0, 0, 5]), nic("en1", [10, 1, 0, 5])];
    let mut st = StationSettings::default();
    st.services.insert(
        "en1".into(),
        lumacore_tree::NicServices {
            lumasession: false,
            shell: true,
            http: false,
        },
    );
    let a = ServiceAccess::compute(&st, &nics);
    assert!(a.lumasession_from("10.0.0.77".parse().unwrap()));
    assert!(!a.lumasession_from("10.1.0.77".parse().unwrap()));
    assert!(a.shell_at("10.1.0.5".parse().unwrap()));
    assert!(!a.http_at("10.1.0.5".parse().unwrap()));
    assert!(a.http_at("127.0.0.1".parse().unwrap()));
    // LumaSession off: only loopback.
    st.online = false;
    let a = ServiceAccess::compute(&st, &nics);
    assert!(!a.lumasession_from("10.0.0.77".parse().unwrap()));
    assert!(!a.shell_at("10.0.0.5".parse().unwrap()));
    assert!(a.shell_at("127.0.0.1".parse().unwrap()));
    assert!(a.lumasession_from("127.0.0.1".parse().unwrap()));
}
