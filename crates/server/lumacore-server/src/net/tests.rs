// ███╗   ██╗███████╗████████╗    ████████╗███████╗███████╗████████╗███████╗
// ████╗  ██║██╔════╝╚══██╔══╝    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██╔██╗ ██║█████╗     ██║          ██║   █████╗  ███████╗   ██║   ███████╗
// ██║╚██╗██║██╔══╝     ██║          ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ██║ ╚████║███████╗   ██║          ██║   ███████╗███████║   ██║   ███████║
// ╚═╝  ╚═══╝╚══════╝   ╚═╝          ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: net tests

//! Tests of `net`.
//!
//! Owns:          the unit tests of net.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §23 "Step 8", §26, §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use crate::net::handler::handle_request;
use crate::context::ServerContext;
use lumacore_shell::ShellHost;
use lumacore_wire::{REMOTE_CMD_PORT, REMOTE_SHELL_PORT, RemoteRequest, SESSION_DISCOVERY_PORT};
use lumacore_tree::{UserProfile, UserRights};

fn test_context() -> ServerContext {
    let mut state = lumacore_objects::AppState::new();

    // Rights come from users only (decisions §30 "12c2").
    for (no, name, role) in [(10, "Guest1", UserRights::Guest), (11, "LD", UserRights::Program)] {
        let profile = UserProfile::new(Some(name.to_string()));
        let guid = profile.meta.identity.guid;
        state.root.show_data.user_profiles.insert(no, std::sync::Arc::new(profile));
        state
            .root
            .show_data
            .users
            .insert(no, lumacore_tree::User::new(no, name, role, guid));
    }

    state.switch_to_internal_root();
    ServerContext::new(
        state,
        Some("library/shows/demo.luma".to_string()),
        false,
        REMOTE_CMD_PORT,
        REMOTE_SHELL_PORT,
        SESSION_DISCOVERY_PORT,
    )
}

/// R1-42: a REST line naming a locked device is refused like its
/// shell's; a line from no device runs.
#[test]
fn a_locked_devices_rest_lines_are_refused() {
    use crate::net::handler::execute_remote_command_from;
    let mut ctx = test_context();
    ctx.state.devices.entry("tab-1".into()).or_default().locked = true;
    let (reply, _) = execute_remote_command_from(&mut ctx, "LD", Some("tab-1"), "Store View 3");
    assert!(reply.message.contains("E-LOCK-001"), "{}", reply.message);
    assert!(ctx.state.current_device.is_none(), "the device is only current while its line runs");
    let (reply, _) = execute_remote_command_from(&mut ctx, "Admin", Some("tab-1"), "On DeskLock Station");
    assert!(!reply.message.contains("E-LOCK-001"), "the lock keys pass: {}", reply.message);
    let (reply, _) = execute_remote_command_from(&mut ctx, "LD", None, "List View");
    assert!(!reply.message.contains("E-LOCK-001"), "a line from no device runs: {}", reply.message);
    // While the station is locked, a device it hasn't seen is locked too.
    let (reply, _) = execute_remote_command_from(&mut ctx, "LD", Some("tab-2"), "List View");
    assert!(reply.message.contains("E-LOCK-001"), "{}", reply.message);
}

#[test]
fn health_hides_internal_root_user() {
    let mut ctx = test_context();
    let (reply, should_exit) = handle_request(&mut ctx, RemoteRequest::Health);
    assert!(reply.ok);
    assert!(!should_exit);
    assert_eq!(reply.active_user, None);
}

#[test]
fn login_rejects_internal_root_user() {
    let mut ctx = test_context();
    let (reply, _) = handle_request(
        &mut ctx,
        RemoteRequest::Login {
            user: lumacore_objects::INTERNAL_ROOT_USER.to_string(),
        },
    );
    assert!(!reply.ok);
    assert!(reply.message.contains("unknown user"));
}

#[test]
fn guest_can_run_read_only_command() {
    let mut ctx = test_context();
    let (reply, _) = handle_request(
        &mut ctx,
        RemoteRequest::Command {
            user: "Guest1".to_string(),
            cmd: "list userprofile".to_string(),
        },
    );
    assert!(reply.ok, "{}", reply.message);
    assert_eq!(ctx.state.user, lumacore_objects::INTERNAL_ROOT_USER);
}

#[test]
fn guest_cannot_run_programmer_command() {
    let mut ctx = test_context();
    let (reply, _) = handle_request(
        &mut ctx,
        RemoteRequest::Command {
            user: "Guest1".to_string(),
            cmd: "fixture 1 at full".to_string(),
        },
    );
    assert!(!reply.ok);
    assert!(reply.message.contains("needs Playback — you are Guest"), "{}", reply.message);
    assert_eq!(ctx.state.user, lumacore_objects::INTERNAL_ROOT_USER);
}

#[test]
fn programmer_cannot_run_admin_command() {
    let mut ctx = test_context();
    let (reply, _) = handle_request(
        &mut ctx,
        RemoteRequest::Command {
            user: "LD".to_string(),
            cmd: "debug".to_string(),
        },
    );
    assert!(!reply.ok);
    assert!(reply.message.contains("needs Admin — you are Program"), "{}", reply.message);
    assert_eq!(ctx.state.user, lumacore_objects::INTERNAL_ROOT_USER);
}

#[test]
fn admin_cannot_run_hissme_over_remote_command_path() {
    let mut ctx = test_context();
    let (reply, _) = handle_request(
        &mut ctx,
        RemoteRequest::Command {
            user: "Admin".to_string(),
            cmd: "HissMe".to_string(),
        },
    );
    assert!(!reply.ok);
    assert!(reply.message.contains("internal root monitor"));
    assert_eq!(ctx.state.user, lumacore_objects::INTERNAL_ROOT_USER);
}

#[test]
fn refresh_populates_session_network_runtime() {
    let mut ctx = test_context();
    let addr = "127.0.0.1:55000".parse().expect("socket addr");
    let attach = ctx.attach_shell_client("Admin".to_string(), addr, 120, 40);
    assert!(!attach.reused_existing_session);

    ctx.refresh_runtime();

    assert!(
        ctx.state
            .root
            .runtime
            .luma_sessions
            .listeners
            .contains_key("Cmdline")
    );
    assert!(
        ctx.state
            .root
            .runtime
            .luma_sessions
            .listeners
            .contains_key("Shell")
    );
    assert!(
        ctx.state
            .root
            .runtime
            .network_engines
            .contains_key("SessionNetwork")
    );
    assert!(
        ctx.state
            .root
            .runtime
            .session_engines
            .contains_key("SessionAuthority")
    );

    let session = ctx
        .state
        .root
        .runtime
        .users
        .get("Admin")
        .expect("admin session");
    assert!(session.attached);
    assert_eq!(
        session.authority,
        ctx.state.root.station_settings.station_name
    );
    assert_eq!(session.clients.len(), 1);
    assert!(
        session
            .stations
            .contains_key(&ctx.state.root.station_settings.station_name)
    );
}

/// LumaSession requests (decisions §11): invites follow the station's
/// Invite setting and defer the join (to the master's stream port); only
/// the master may dismiss. Membership itself is the stream
/// (`link_tests`).
#[test]
fn lumasession_invite_and_dismiss() {
    let mut ctx = test_context();
    ctx.request_from = Some("10.0.0.9:47653".parse().unwrap());
    let invite = |ctx: &mut ServerContext| {
        handle_request(
            ctx,
            RemoteRequest::Invite {
                lumasession: "Festival".into(),
                from: "FOH".into(),
                stream_port: 47658,
            },
        )
        .0
    };
    ctx.state.root.station_settings.invite = false;
    let r = invite(&mut ctx);
    assert!(!r.ok && r.message.contains("Invite is off"), "{}", r.message);
    assert!(ctx.lumasession.deferred.is_empty());
    ctx.state.root.station_settings.invite = true;
    let r = invite(&mut ctx);
    assert!(r.ok, "{}", r.message);
    assert!(matches!(
        &ctx.lumasession.deferred[..],
        [lumacore_lumasession::Deferred::Join { host, stream_port: 47658, .. }] if host == "10.0.0.9"
    ));
    ctx.lumasession.deferred.clear();

    // Dismiss only from this station's master.
    let r = handle_request(&mut ctx, RemoteRequest::Dismiss { from: "X".into() }).0;
    assert!(!r.ok);
    assert!(ctx.lumasession.deferred.is_empty());
}

#[test]
fn monitor_hello_roundtrip() {
    let packet = lumacore_wire::ShellClientPacket::MonitorHello { protocol: 1 };
    let json = serde_json::to_string(&packet).unwrap();
    let parsed: lumacore_wire::ShellClientPacket = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, lumacore_wire::ShellClientPacket::MonitorHello { protocol: 1 }));
}

#[test]
fn shutdown_command_over_udp_asks_first() {
    // `Shutdown` as a command line asks "Shut down LumaCore?"; a one-shot
    // UDP request can't answer, so it is refused and nothing stops (a
    // station never shuts down unless someone really wants it). The
    // command port's own `RemoteRequest::Shutdown` is the way to stop it.
    let mut ctx = test_context();
    let (reply, should_exit) = handle_request(
        &mut ctx,
        RemoteRequest::Command {
            user: "Admin".to_string(),
            cmd: "shutdown".to_string(),
        },
    );
    assert!(!reply.ok, "shutdown over UDP must ask first: {}", reply.message);
    assert!(
        reply.message.contains("can't answer") && reply.message.contains("/NoConfirmation"),
        "{}",
        reply.message
    );
    assert!(!should_exit, "a refused shutdown must not stop the server");
}

#[test]
fn shutdown_via_remote_request() {
    let mut ctx = test_context();
    let (reply, should_exit) = handle_request(&mut ctx, RemoteRequest::Shutdown);
    assert!(reply.ok);
    assert!(
        should_exit,
        "RemoteRequest::Shutdown must signal server exit"
    );
}
