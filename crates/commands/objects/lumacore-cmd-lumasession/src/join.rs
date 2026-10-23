//      ██╗ ██████╗ ██╗███╗   ██╗
//      ██║██╔═══██╗██║████╗  ██║
//      ██║██║   ██║██║██╔██╗ ██║
// ██   ██║██║   ██║██║██║╚██╗██║
// ╚█████╔╝╚██████╔╝██║██║ ╚████║
//  ╚════╝  ╚═════╝ ╚═╝╚═╝  ╚═══╝
// FILE: join

//! `Join LumaSession "Festival" | <host[:port]> [/Merge]` (decisions §21 "6b"): no network I/O in
//! the command — it queues a link request; a fenced master or an IdleMaster keeps its show first.
//!
//! Owns:          Join LumaSession: an address or a name found through discovery, /Merge, what keeps this show first.
//! Doesn't own:   the link request (the kernel's session::join_at), the stream (lumacore-lumasession).
//! Entry points:  cmd_join_lumasession.
//! Decisions:     §21 "6b" "6d".

use lumacore_objects::AppState;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// A LumaSession target as typed: an address (`10.0.0.5`, `host:47653`,
/// `localhost`) or a name.
fn is_address(target: &str) -> bool {
    let host = target.rsplit_once(':').map_or(target, |(h, _)| h);
    host.eq_ignore_ascii_case("localhost") || host.parse::<std::net::Ipv4Addr>().is_ok()
}

/// The master of the LumaSession called `name`, from discovery (a master
/// first, else a standalone station of that name): its address and stream
/// port.
fn find_lumasession(state: &AppState, name: &str) -> Option<(String, u16)> {
    let peers: Vec<&lumacore_tree::DiscoveryPeerRuntime> = state
        .root
        .runtime
        .luma_sessions
        .discovery
        .values()
        .filter(|p| p.stream_port != 0 && p.ip != "this station")
        .filter(|p| p.lumasession.eq_ignore_ascii_case(name))
        .collect();
    peers
        .iter()
        .find(|p| p.status == "Master")
        .or_else(|| peers.iter().find(|p| p.status == "Standalone"))
        .map(|p| (p.ip.clone(), p.stream_port))
}

/// `Join LumaSession "Festival"` | `Join LumaSession 10.0.0.5[:stream_port]`
/// `[/Merge]` (§21 "6d": merge this station's show into the session's).
pub fn cmd_join_lumasession(state: &mut AppState, args: &[&str]) -> String {
    use lumacore_objects::errors::render_code;
    let merge =
        lumacore_objects::commands::flags::has(args, &lumacore_objects::commands::flags::MERGE);
    let Some(target) = args
        .iter()
        .find(|a| !a.starts_with('/'))
        .map(|a| a.trim_matches('"').to_string())
    else {
        return render_code(
            "Usage: Join LumaSession \"name\" | Join LumaSession <host[:port]> [/Merge]",
            "E-CMD-006",
        );
    };
    if !state.root.station_settings.online {
        return render_code(
            "LumaSession is off on this station: Set Station \"Online\" On",
            "E-CMD-010",
        );
    }
    if state.link.is_some() || !state.link_requests.is_empty() {
        return render_code(
            "This station is already in a LumaSession: Leave LumaSession first.",
            "E-CMD-010",
        );
    }
    if !state.root.runtime.luma_sessions.members.is_empty() {
        return render_code(
            "This station is a master with members: Dismiss them before joining another LumaSession.",
            "E-CMD-010",
        );
    }
    // Use Master / Merge from a fenced master / an IdleMaster: keep this
    // show first.
    let kept = if state.session_role != lumacore_objects::session_link::Role::Normal {
        let reason = if merge { "before-merge" } else { "use-master" };
        match lumacore_objects::session_link::autosave_copy(state, reason) {
            Ok(saved) => saved
                .map(|p| format!("This show is kept in {p}.\n"))
                .unwrap_or_default(),
            // The session's show would replace this one: not without a copy.
            Err(e) => return lumacore_objects::session_link::not_kept(&e),
        }
    } else {
        String::new()
    };
    let found = if is_address(&target) {
        Some(lumacore_objects::commands::session::parse_host_and_port(
            &target,
            lumacore_wire::REMOTE_STREAM_PORT,
        ))
    } else {
        find_lumasession(state, &target)
    };
    match found {
        Some((host, port)) => {
            let out = lumacore_objects::commands::session::join_at(state, &host, port, None);
            if let Some(request) = state.link_requests.last_mut() {
                request.merge = merge;
            }
            format!("{kept}{out}")
        }
        None => render_code(
            format!(
                "No LumaSession \"{target}\" found on the network (List LumaSession shows the stations)."
            ),
            "E-CMD-007",
        ),
    }
}
