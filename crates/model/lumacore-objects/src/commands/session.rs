//! Session commands: login, connect, and LumaSession (decisions §11
//! "Menus, LumaSession and staged patch"): `Join LumaSession "Festival"`,
//! `Join LumaSession 10.0.0.5`, `Leave LumaSession`, `Invite Station "X"`,
//! `Dismiss Station "X"`.

use crate::session_link::SessionRequest;
use crate::AppState;
use crate::claims::Claims;

/// `Login "name"` for a user with a password, without one: the password
/// dialog (decisions §30 "12b") — a masked input; Please runs `Login "name"
/// "<password>"` (masked in dispatch). A one-shot path is told to add it.
pub fn login(state: &mut AppState, args: &[&str]) -> crate::DispatchResult {
    let unquote = |s: &str| s.trim_matches('"').trim_matches('\'').to_string();
    let rest: Vec<&str> = match args.first() {
        Some(a) if a.eq_ignore_ascii_case("user") => args[1..].to_vec(),
        _ => args.to_vec(),
    };
    if let [name] = rest.as_slice()
        && let Some(user) = state.root.show_data.user(&unquote(name)).cloned()
        && crate::passwords::has_password(state, &user.login)
        && crate::passwords::may_log_in_from(state, &user.login, state.remote_client).is_ok()
    {
        let n = &user.login;
        return crate::DispatchResult::Ask(Box::new(
            crate::dialogs::Ask::new(format!("Login {n}"), format!("{n} has a password."))
                .input("password", "Password", "", lumacore_wire::InputKind::Password)
                .button("Please", lumacore_wire::ButtonKind::Please, format!("Login \"{n}\" \"{{password}}\""))
                .cancel("Cancel")
                .flags(format!("the password: Login \"{n}\" \"password\"")),
        ));
    }
    crate::objects::out(cmd_login(state, args))
}

/// Handle: `Login [User] "name" ["password"]`
///
/// Switch to an existing user on this server — no auto-create (an Admin
/// creates users). A user with a password must give it; a UI client that
/// leaves it out gets a password prompt instead. Releases the previous user's
/// claims. In a remote shell the client then moves to the new user's session
/// (their command line, programmer and screens); the local REPL switches
/// directly. The password is masked in echo, history and logs.
pub fn cmd_login(state: &mut AppState, args: &[&str]) -> String {
    use crate::errors::render_code;
    use crate::text::render_success;

    let args: Vec<&str> = match args.first() {
        Some(a) if a.eq_ignore_ascii_case("user") => args[1..].to_vec(),
        _ => args.to_vec(),
    };
    let unquote = |s: &str| s.trim_matches('"').trim_matches('\'').to_string();
    let Some(name) = args.first().map(|a| unquote(a)).filter(|n| !n.is_empty()) else {
        return render_code("Usage: Login [User] \"name\" [\"password\"]", "E-CMD-006");
    };
    let password = args.get(1).map(|a| unquote(a));

    if name.eq_ignore_ascii_case(crate::INTERNAL_ROOT_USER) {
        return render_code("That name is reserved for the internal runtime user.", "E-CMD-008");
    }
    let Some(user) = state.root.show_data.user(&name).cloned() else {
        return render_code(
            format!(
                "No user \"{name}\". An Admin creates users with Store User."
            ),
            "E-CMD-011",
        );
    };
    if let Err(refused) =
        crate::passwords::may_log_in_from(state, &user.login, state.remote_client)
    {
        return crate::errors::reply(&refused, "E-RIGHTS-002");
    }
    if crate::passwords::has_password(state, &user.login) {
        match &password {
            None => {
                return render_code(
                    format!(
                        "User {} has a password: Login \"{}\" \"password\"",
                        user.login, user.login
                    ),
                    "E-CMD-006",
                );
            }
            Some(pw) if !crate::passwords::check_password(&user, Some(pw)) => {
                return render_code(format!("Wrong password for {}.", user.login), "E-CMD-008");
            }
            Some(_) => {}
        }
    }
    let old_user = state.effective_user_name().to_string();
    if old_user.eq_ignore_ascii_case(&user.login) {
        return render_success(&format!("Already logged in as {}.", user.login));
    }

    // Release all claims held by the old user
    state.release_claims_of(&old_user);
    state.context = crate::ReplContext::Fixture;
    state.navigation_path.clear();

    if state.acting_user.is_some() {
        // Remote shell: the session manager moves this client to the user's session.
        state.pending_user_switch = Some(user.login.clone());
    } else {
        state.user = user.login.clone();
    }
    // Every login asks for the Meow License before it is finished (decisions 0290).
    if let Some(holder) = crate::consent::holder(state) {
        crate::consent::begin(state, holder, &user.login, None);
    }
    // Attributes waiting in the user's profile: their dialog (§30 "12f-a").
    crate::commands::attributes::ask_waiting(state, &user.login);
    render_success(&format!("Logged in as {}.", user.login))
}

/// Handle: `Connect "Station"` | `Connect [IP] host[:port] [User "name" ["password"]]`
///
/// Switch this window of the LumaCore app to another server (a LumaCore
/// divergence: a desk usually cannot connect to external servers). The app resolves a
/// station name through discovery, logs in (asking for a password when the
/// user has one and none is given) and opens the screens there. Only the app
/// can do this; `lumacore-cli` connects with `CMDLINE <ip>`.
pub fn cmd_connect(state: &mut AppState, args: &[&str]) -> String {
    use crate::errors::render_code;
    use crate::text::render_success;

    const USAGE: &str =
        "Usage: Connect \"Station\" | Connect [IP] host[:port] [User \"name\" [\"password\"]]";
    let unquote = |s: &str| s.trim_matches('"').trim_matches('\'').to_string();
    let mut args: &[&str] = args;
    if args.first().is_some_and(|a| a.eq_ignore_ascii_case("ip")) {
        args = &args[1..];
    }
    let Some(target) = args.first().map(|a| unquote(a)).filter(|t| !t.is_empty()) else {
        return render_code(USAGE, "E-CMD-006");
    };
    let (user, password) = match &args[1..] {
        [] => (None, None),
        [kw, name, rest @ ..] if kw.eq_ignore_ascii_case("user") && rest.len() <= 1 => {
            (Some(unquote(name)), rest.first().map(|p| unquote(p)))
        }
        _ => return render_code(USAGE, "E-CMD-006"),
    };
    if !state.has_ui() {
        return render_code(
            "Connect switches servers in the LumaCore app. In lumacore-cli use CMDLINE <ip>.",
            "E-CMD-010",
        );
    }
    let who = user
        .as_deref()
        .map(|u| format!(" as {u}"))
        .unwrap_or_default();
    state
        .ui_requests
        .push(lumacore_wire::UiEvent::Connect {
            target: target.clone(),
            user,
            password,
        });
    render_success(&format!("Connecting to {target}{who}…"))
}

/// Ask for a join: the link to the master starts after this command (no
/// network I/O in a command) and tells the user how it went (decisions §21
/// "6b"). The station keeps its show until the master's show has arrived
/// complete and checked.
pub fn join_at(state: &mut AppState, host: &str, port: u16, invited_by: Option<String>) -> String {
    let user = state.effective_user_name().to_string();
    state.link_requests.push(crate::session_link::LinkRequest {
        host: host.to_string(),
        port,
        user: user.clone(),
        reply_to: state.reply_to.clone(),
        invited_by,
        patient: false,
        merge: false,
    });
    crate::text::render_success(&format!(
        "Joining the LumaSession at {host}:{port} as {user}…"
    ))
}

/// Join the master that invited this station.
pub fn join_lumasession_at(
    state: &mut AppState,
    host: &str,
    stream_port: u16,
    invited_by: Option<String>,
) -> String {
    if state.link.is_some() {
        return crate::errors::render_code(
            "This station is already in a LumaSession.",
            "E-CMD-010",
        );
    }
    join_at(state, host, stream_port, invited_by)
}

pub fn leave_lumasession(state: &mut AppState) -> String {
    use crate::session_link::Role;
    use crate::errors::render_code;
    use crate::text::render_success;
    // Cancel on a fenced master / an IdleMaster that found its session:
    // stay alone (IdleMaster, output only with "output when alone").
    if let Role::Fenced { by } | Role::IdleMaster { lost: by, .. } = state.session_role.clone() {
        state.session_role = Role::IdleMaster {
            diverged: true,
            lost: by.clone(),
            quiet: true,
        };
        state.decision = None;
        state.show_status = crate::session_link::ShowStatus::Loaded;
        state.protocols_dirty = true;
        return render_success(&format!(
            "Cancel: this station stays alone (IdleMaster, apart from {by}); Join LumaSession to go back."
        ));
    }
    let Some(left) = crate::session_link::leave(state) else {
        return render_code("This station isn't in a LumaSession.", "E-CMD-010");
    };
    state.is_authority = true;
    state.root.station_settings.output_held = true;
    state.local_settings.store_from(&state.root.station_settings);
    state.protocols_dirty = true;
    format!(
        "{}\n{}",
        render_success(&left),
        render_success("DMX output is held: resume it with Set Station \"HoldOutput\" Off.")
    )
}

/// The discovered station called `name` (not this one).
fn find_station<'a>(
    state: &'a AppState,
    name: &str,
) -> Option<&'a lumacore_tree::DiscoveryPeerRuntime> {
    state
        .root
        .runtime
        .luma_sessions
        .discovery
        .values()
        .filter(|p| p.cmd_port != 0 && p.ip != "this station")
        .find(|p| p.name.eq_ignore_ascii_case(name))
}

/// `Invite Station "FOH-2"`: a master (or standalone station) invites a
/// station into its LumaSession.
pub fn cmd_invite_station(state: &mut AppState, args: &[&str]) -> String {
    use crate::errors::render_code;
    use crate::text::render_success;
    let Some(name) = args.first().map(|a| a.trim_matches('"').to_string()) else {
        return render_code("Usage: Invite Station \"name\"", "E-CMD-006");
    };
    if crate::session_link::status_word(state) == "Member" {
        return render_code(
            "Only a master invites: this station is a member of a LumaSession.",
            "E-CMD-010",
        );
    }
    let Some(peer) = find_station(state, &name).cloned() else {
        return render_code(
            format!(
                "No station \"{name}\" found on the network (List LumaSession)."
            ),
            "E-CMD-007",
        );
    };
    if peer.status == "Member" {
        return render_code(format!("{name} is already in a LumaSession."), "E-CMD-010");
    }
    let station = &state.root.station_settings;
    let request = lumacore_wire::RemoteRequest::Invite {
        lumasession: station.lumasession.clone(),
        from: station.station_name.clone(),
        stream_port: state.lumasession_ports.2,
    };
    // Sent after the job (no network I/O in a command); the answer comes
    // to this line's client.
    state.session_requests.push(SessionRequest::Send {
        station: name.clone(),
        ip: peer.ip.clone(),
        port: peer.cmd_port,
        request,
        reply_to: state.reply_to.clone(),
    });
    render_success(&format!("Inviting {name}…"))
}

/// `Dismiss Station "FOH-2"`: the master sends a member away.
pub fn cmd_dismiss_station(state: &mut AppState, args: &[&str]) -> String {
    use crate::errors::render_code;
    use crate::text::render_success;
    let Some(name) = args.first().map(|a| a.trim_matches('"').to_string()) else {
        return render_code("Usage: Dismiss Station \"name\"", "E-CMD-006");
    };
    let members = &state.root.runtime.luma_sessions.members;
    let Some(member) = members
        .values()
        .find(|m| m.station.eq_ignore_ascii_case(&name))
        .cloned()
    else {
        return render_code(format!("{name} isn't a member of this LumaSession."), "E-CMD-007");
    };
    let from = state.root.station_settings.station_name.clone();
    state.root.runtime.luma_sessions.members.remove(&member.station);
    state.session_requests.push(SessionRequest::Send {
        station: name.clone(),
        ip: member.ip.clone(),
        port: member.cmd_port,
        request: lumacore_wire::RemoteRequest::Dismiss { from },
        reply_to: state.reply_to.clone(),
    });
    render_success(&format!("{name} removed from the members; telling it…"))
}

pub fn parse_host_and_port(target: &str, default_port: u16) -> (String, u16) {
    if let Some((host, port)) = target.rsplit_once(':')
        && let Ok(port) = port.parse::<u16>()
    {
        return (host.to_string(), port);
    }
    (target.to_string(), default_port)
}
