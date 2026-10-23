// ██╗  ██╗ █████╗ ███╗   ██╗██████╗ ██╗     ███████╗██████╗
// ██║  ██║██╔══██╗████╗  ██║██╔══██╗██║     ██╔════╝██╔══██╗
// ███████║███████║██╔██╗ ██║██║  ██║██║     █████╗  ██████╔╝
// ██╔══██║██╔══██║██║╚██╗██║██║  ██║██║     ██╔══╝  ██╔══██╗
// ██║  ██║██║  ██║██║ ╚████║██████╔╝███████╗███████╗██║  ██║
// ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚══════╝╚═╝  ╚═╝
// FILE: handler

//!
//! Owns:          the UDP command port's requests on the state thread: Health, Login, commands as a user, LumaSession requests, Shutdown; the reply.
//! Doesn't own:   the socket and its thread (spawn), what a LumaSession request does (lumacore-lumasession).
//! Entry points:  handle_request, execute_remote_command, make_reply.
//! Decisions:     §10, §23 "Step 8", §34 24.2.

use std::path::PathBuf;

use lumacore_objects::commands::dispatch;
use lumacore_objects::DispatchResult;

use crate::context::ServerContext;
use crate::context::{drain_sysmon_pending, sysmon};
use lumacore_wire::{RemoteReply, RemoteRequest};
use lumacore_objects::claims::Claims;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn visible_active_user(state: &lumacore_objects::AppState) -> Option<String> {
    if state.user == lumacore_objects::INTERNAL_ROOT_USER {
        None
    } else {
        Some(state.user.clone())
    }
}

pub(crate) fn make_reply(ctx: &ServerContext, ok: bool, message: String) -> RemoteReply {
    RemoteReply {
        ok,
        message,
        server_pid: std::process::id(),
        active_user: visible_active_user(&ctx.state),
        show_path: ctx.show_path.clone(),
        debug_mode: ctx.state.debug_mode,
        build_id: ctx.state.root.diagnostics.build_info.build_id.clone(),
        station_name: Some(ctx.state.root.station_settings.station_name.clone()),
        session_count: Some(ctx.state.root.runtime.users.len() as u32),
        station_priority: Some(ctx.state.root.station_settings.station_priority.to_string()),
        lumasession: Some(ctx.state.root.station_settings.lumasession.clone()),
        kind: lumacore_wire::StationKind::Server,
        client_protocol: lumacore_wire::CLIENT_PROTOCOL,
        session_protocol: lumacore_wire::SESSION_PROTOCOL,
        clear_screen: false,
    }
}

pub(crate) fn user_exists(state: &lumacore_objects::AppState, user: &str) -> bool {
    if user.trim().is_empty()
        || user
            .trim()
            .eq_ignore_ascii_case(lumacore_objects::INTERNAL_ROOT_USER)
    {
        return false;
    }
    state.root.show_data.user(user).is_some()
        || state.root.show_data.profile_id_for_user(user).is_some()
}

pub fn handle_request(ctx: &mut ServerContext, request: RemoteRequest) -> (RemoteReply, bool) {
    match request {
        RemoteRequest::Health => {
            ctx.refresh_runtime();
            sysmon(ctx, "Server", "Health check from client");
            (
                RemoteReply {
                    ok: true,
                    message: format!(
                        "lumacore-server running ({})",
                        ctx.state.root.diagnostics.build_info.build_id
                    ),
                    server_pid: std::process::id(),
                    active_user: visible_active_user(&ctx.state),
                    show_path: ctx.show_path.clone(),
                    debug_mode: ctx.state.debug_mode,
                    build_id: ctx.state.root.diagnostics.build_info.build_id.clone(),
                    station_name: Some(ctx.state.root.station_settings.station_name.clone()),
                    session_count: Some(ctx.state.root.runtime.users.len() as u32),
                    station_priority: Some(
                        ctx.state.root.station_settings.station_priority.to_string(),
                    ),
                    lumasession: Some(ctx.state.root.station_settings.lumasession.clone()),
                    kind: lumacore_wire::StationKind::Server,
                    client_protocol: lumacore_wire::CLIENT_PROTOCOL,
                    session_protocol: lumacore_wire::SESSION_PROTOCOL,
                    clear_screen: false,
                },
                false,
            )
        }
        RemoteRequest::Login { user } => {
            let ok = user_exists(&ctx.state, &user);
            if ok {
                ctx.state.navigation_path.clear();
                ctx.state.context = lumacore_objects::ReplContext::Fixture;
            }
            ctx.refresh_runtime();
            sysmon(
                ctx,
                "Session",
                if ok {
                    format!("login accepted for visible user '{user}'")
                } else {
                    format!("login rejected for unknown/reserved user '{user}'")
                },
            );
            let message = if ok {
                format!("login ok for {user}")
            } else {
                format!("unknown user '{user}'")
            };
            (make_reply(ctx, ok, message), false)
        }
        RemoteRequest::Shutdown => {
            ctx.refresh_runtime();
            sysmon(ctx, "Shutdown", "Shutdown requested");
            (
                RemoteReply {
                    ok: true,
                    message: "lumacore-server shutting down".to_string(),
                    server_pid: std::process::id(),
                    active_user: visible_active_user(&ctx.state),
                    show_path: ctx.show_path.clone(),
                    debug_mode: ctx.state.debug_mode,
                    build_id: ctx.state.root.diagnostics.build_info.build_id.clone(),
                    station_name: Some(ctx.state.root.station_settings.station_name.clone()),
                    session_count: Some(ctx.state.root.runtime.users.len() as u32),
                    station_priority: Some(
                        ctx.state.root.station_settings.station_priority.to_string(),
                    ),
                    lumasession: Some(ctx.state.root.station_settings.lumasession.clone()),
                    kind: lumacore_wire::StationKind::Server,
                    client_protocol: lumacore_wire::CLIENT_PROTOCOL,
                    session_protocol: lumacore_wire::SESSION_PROTOCOL,
                    clear_screen: false,
                },
                true,
            )
        }
        RemoteRequest::Invite {
            lumasession,
            from,
            stream_port,
        } => (
            lumacore_lumasession::handle_invite(ctx, lumasession, from, stream_port),
            false,
        ),
        RemoteRequest::Dismiss { from } => (lumacore_lumasession::handle_dismiss(ctx, from), false),
        // Answered at the socket (net::spawn), never here.
        RemoteRequest::Time { .. } => (
            make_reply(ctx, false, "Time pings are answered by the command port".to_string()),
            false,
        ),
        RemoteRequest::Yield {
            lumasession: _,
            from,
            epoch,
            stream_port,
        } => (
            lumacore_lumasession::election::handle_yield(ctx, from, epoch, stream_port),
            false,
        ),
        RemoteRequest::Command { user, cmd } => {
            // Who sent it: a station discovery knows by its address, else
            // the address (decisions §30 "Remote station edits": the
            // target logs the sender; no field on the wire for it).
            let from = ctx.request_from.map(|a| a.ip().to_string()).unwrap_or_default();
            let place = ctx
                .state
                .root
                .runtime
                .luma_sessions
                .discovery
                .values()
                .find(|p| !from.is_empty() && p.ip == from)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("udp {from}").trim().to_string());
            sysmon(
                ctx,
                "Command",
                format!(
                    "command request from '{user}@{place}': {}",
                    lumacore_objects::passwords::mask_secrets(cmd.trim())
                ),
            );
            ctx.state.remote_from = Some(format!("{user}@{place}"));
            let (reply, should_exit) = execute_remote_command(ctx, &user, &cmd);
            ctx.state.remote_from = None;
            drain_sysmon_pending(ctx);
            (reply, should_exit)
        }
    }
}

pub(crate) fn execute_remote_command(
    ctx: &mut ServerContext,
    user: &str,
    cmd: &str,
) -> (RemoteReply, bool) {
    execute_remote_command_from(ctx, user, None, cmd)
}

/// Like `execute_remote_command`, for a line that came from a device (an
/// HTTP REST call naming its page's device, R1-42): while it runs the device
/// is the current one, so a locked device's lines are refused (E-LOCK-001)
/// like its shell's.
pub(crate) fn execute_remote_command_from(
    ctx: &mut ServerContext,
    user: &str,
    device: Option<&str>,
    cmd: &str,
) -> (RemoteReply, bool) {
    let previous = std::mem::replace(&mut ctx.state.current_device, device.map(str::to_string));
    let reply = execute_as(ctx, user, cmd);
    ctx.state.current_device = previous;
    reply
}

fn execute_as(ctx: &mut ServerContext, user: &str, cmd: &str) -> (RemoteReply, bool) {
    let input = cmd.trim().to_string();
    if input.is_empty() {
        return (
            make_reply(ctx, false, "command is empty".to_string()),
            false,
        );
    }
    let Some(rights) = ctx.state.rights_of(user) else {
        sysmon(
            ctx,
            "Command",
            format!("rejecting command for unknown/reserved user '{user}'"),
        );
        return (
            make_reply(ctx, false, format!("unknown user '{user}'")),
            false,
        );
    };
    // A gesture's moves stay quiet on the master (decisions §30 "12f").
    if ctx.state.gesture.is_none() {
        sysmon(
            ctx,
            "Command",
            format!("resolved rights for '{user}': {}", rights.label()),
        );
    }

    // Rights are checked by dispatch, like for every other transport.
    let commands: Vec<&str> = input.split(';').map(|s| s.trim()).collect();

    let Some(acting_user) = ctx.state.begin_acting_as_existing_user(user) else {
        sysmon(
            ctx,
            "Command",
            format!("failed to enter acting-user scope for '{user}'"),
        );
        return (
            make_reply(ctx, false, format!("unknown user '{user}'")),
            false,
        );
    };
    // A gesture's moves stay quiet on the master (decisions §30 "12f").
    if ctx.state.gesture.is_none() {
        sysmon(
            ctx,
            "Command",
            format!("entered acting-user scope '{acting_user}'"),
        );
    }

    // HissMe aborts on purpose: only once the rights gate lets it through.
    if input.eq_ignore_ascii_case("hissme")
        && lumacore_objects::rights::check(&ctx.state, "hissme").is_ok()
    {
        sysmon(
            ctx,
            "Command",
            format!("executing HissMe for '{acting_user}'"),
        );
        emit_hissme_report(ctx, &acting_user);
    }

    let mut outputs = Vec::new();
    let mut cleared = false;
    let mut error = None;
    let mut should_exit = false;

    for command in commands {
        if command.is_empty() {
            continue;
        }

        let result = dispatch(&mut ctx.state, command);
        let shown = lumacore_objects::passwords::mask_secrets(command);
        let via = ctx
            .state
            .remote_from
            .as_deref()
            .and_then(|f| f.split_once('@'))
            .map_or_else(|| "udp".to_string(), |(_, place)| place.to_string());
        if ctx.state.gesture.is_none() {
            lumacore_objects::commands::sysmon::log_command(&mut ctx.state, command, &result, Some(&via));
        }
        crate::context::drain_sysmon_pending(ctx);
        match result {
            // Refused by the rights gate: the request fails.
            DispatchResult::Output(output)
                if output.code.as_deref().is_some_and(|c| c.starts_with("E-RIGHTS-")) =>
            {
                sysmon(
                    ctx,
                    "Command",
                    format!("refused '{shown}' for '{acting_user}'"),
                );
                error = Some(lumacore_objects::text::strip_ansi(&output).trim().to_string());
                break;
            }
            DispatchResult::Output(output) => {
                sysmon(
                    ctx,
                    "Command",
                    format!("executed '{shown}' for '{acting_user}'"),
                );
                if output == "\x1B[2J\x1B[1;1H" {
                    cleared = true;
                    outputs.push(lumacore_objects::text::render_success(
                        "Terminal buffer cleared",
                    ));
                } else if !output.is_empty() {
                    outputs.push(String::from(output));
                }
            }
            DispatchResult::Exit => {
                sysmon(
                    ctx,
                    "Command",
                    format!("received exit/shutdown from '{acting_user}'"),
                );
                outputs.push("Server shutting down.".to_string());
                should_exit = true;
                break;
            }
            DispatchResult::Logout => {
                sysmon(
                    ctx,
                    "Command",
                    format!("logout requested by '{acting_user}'"),
                );
                outputs
                    .push("Logout is only available from an attached shell session.".to_string());
                break;
            }
            DispatchResult::ContextSwitch(ctx_switch) => {
                ctx.state.context = ctx_switch;
                sysmon(
                    ctx,
                    "Command",
                    format!("context switched for '{acting_user}'"),
                );
            }
            DispatchResult::Ask(ask) => {
                sysmon(
                    ctx,
                    "Command",
                    format!("'{acting_user}' was asked \"{}\" on a one-shot path", ask.title()),
                );
                error = Some(ask.refusal());
                break;
            }
        }
    }

    ctx.state.clear_acting_user();
    sysmon(
        ctx,
        "Command",
        format!("cleared acting-user scope for '{acting_user}'"),
    );
    ctx.refresh_runtime();

    let reply = if let Some(message) = error {
        make_reply(ctx, false, message)
    } else {
        let message = if outputs.is_empty() {
            format!("command accepted for {acting_user}")
        } else {
            outputs.join("\n")
        };
        RemoteReply {
            clear_screen: cleared,
            ..make_reply(ctx, true, message)
        }
    };
    (reply, should_exit)
}

fn emit_hissme_report(ctx: &ServerContext, acting_user: &str) -> ! {
    let programmer = ctx.state.current_programmer().cloned();
    let selected_count = programmer
        .as_ref()
        .map(|p| p.selected_fixtures.len())
        .unwrap_or(0);
    let value_count = programmer.as_ref().map(|p| p.values.len()).unwrap_or(0);
    let active_value_count = programmer
        .as_ref()
        .map(|p| p.values.values().filter(|pv| pv.active).count())
        .unwrap_or(0);

    let report = format!(
        "HissMe intentional crash\n\
        pid: {}\n\
        active_user: {}\n\
        show_path: {}\n\
        debug_mode: {}\n\
        protocols_running: {:?}\n\
        user_profiles: {}\n\
        fixtures: {}\n\
        universes: {}\n\
        selected_fixtures: {}\n\
        programmer_values: {}\n\
        active_programmer_values: {}\n",
        std::process::id(),
        acting_user,
        ctx.show_path.as_deref().unwrap_or("<none>"),
        ctx.state.debug_mode,
        ctx.state.protocols_running,
        ctx.state.root.show_data.user_profiles.len(),
        ctx.state.root.show_data.devices.dmx.patch.fixtures.len(),
        ctx.state.root.show_data.devices.dmx.patch.universes.len(),
        selected_count,
        value_count,
        active_value_count,
    );

    let path = write_hissme_report(&report);
    let final_report = if let Some(path) = path {
        format!("{report}report_path: {}\n", path.display())
    } else {
        report
    };

    log::error!("{final_report}");
    eprintln!("{final_report}");
    // A real crash on purpose: abort, so no panic guard can catch it.
    std::process::abort();
}

fn write_hissme_report(report: &str) -> Option<PathBuf> {
    let mut dir = PathBuf::from("library/logs/Crash");
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    dir.push(format!("hissme_{ts}.log"));
    std::fs::write(&dir, report).ok()?;
    Some(dir)
}
