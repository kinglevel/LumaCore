//! Users (`ShowData.Users`): login identities pointing at a profile.
//! Rights live on the user only (decisions §30 "12c2"): one level of
//! Guest < Playback < Presets < Program < Setup < Admin (§30 "12d").
//!
//! ```text
//! Store User                                    asks: name, rights, profile, password, colour
//! Store User "LD" [/Rights Guest|Playback|Presets|Program|Setup|Admin] [/Profile New|n|"name"]
//!                 [/Password "pw"] [/Colour Teal|#rrggbb]
//! Set User "LD" Name "y" | Password ["pw"] | Rights Setup | RemoteLogin Off
//!               | Profile New|n|"name" | ScreenConfig n | Colour Teal
//! Delete User "LD" [/WithProfile] [/NoConfirmation]   asks what goes
//! User                                          list users
//! ```
//!
//! Without `/Profile` (or with `/Profile New`) a new user gets their own
//! profile: a copy of the locked template "Default", named after them (the
//! next free name on a clash). Several users can share one profile; nobody
//! holds Default. Passwords are masked in echo, history and logs.

use crate::commands::flags as F;
use lumacore_tree::{TreeNode, User, UserRights};

use crate::AppState;
use crate::errors::{render_code, with_code};
use crate::passwords::hash_password;
use crate::text::render_success;

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').trim_matches('\'').to_string()
}

/// Changing another user's password or colour is Admin's (an ownership
/// rule: the declared level of `Set User … Password|Colour` is anyone's,
/// for their own).
fn require_admin(state: &AppState, what: &str) -> Result<(), String> {
    crate::rights::require(state, crate::commands::registry::Rights::Admin, what)
}

/// A rights level by name (E-CMD-008 names the six).
fn parse_rights(s: &str) -> Result<UserRights, String> {
    UserRights::parse(s).ok_or_else(|| {
        let all: Vec<&str> = UserRights::ALL.iter().map(|r| r.label()).collect();
        with_code(format!("Unknown rights level '{s}' ({})", all.join(", ")), "E-CMD-008")
    })
}

/// The levels as a pick list, lowest first.
fn levels() -> Vec<String> {
    UserRights::ALL.iter().map(|r| r.label().to_string()).collect()
}

/// A profile by number or name.
pub fn find_profile(state: &AppState, spec: &str) -> Option<u32> {
    let spec = unquote(spec);
    let profiles = &state.root.show_data.user_profiles;
    spec.parse::<u32>()
        .ok()
        .filter(|n| profiles.contains_key(n))
        .or_else(|| {
            profiles
                .iter()
                .find(|(_, p)| p.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(&spec)))
                .map(|(&id, _)| id)
        })
}

fn admins(state: &AppState) -> usize {
    state
        .root
        .show_data
        .users
        .values()
        .filter(|u| u.rights == UserRights::Admin)
        .count()
}

/// Errors carry their code already (`with_code`); anything else is E-CMD-010.
fn reply(result: Result<String, String>) -> String {
    result.map_or_else(|e| crate::errors::reply(e, "E-CMD-010"), |m| render_success(&m))
}

/// `#rrggbb` from a colour name of [`lumacore_tree::USER_COLOR_NAMES`] or a hex value.
fn parse_colour(value: &str) -> Result<String, String> {
    let value = unquote(value);
    if let Some(i) = lumacore_tree::USER_COLOR_NAMES
        .iter()
        .position(|n| n.eq_ignore_ascii_case(&value))
    {
        return Ok(lumacore_tree::USER_COLORS[i].to_string());
    }
    let hex = value.trim_start_matches('#');
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(with_code(
            format!(
                "'{value}' is not a colour: #rrggbb or {}",
                lumacore_tree::USER_COLOR_NAMES.join(", ")
            ),
            "E-CMD-008",
        ));
    }
    Ok(format!("#{}", hex.to_ascii_lowercase()))
}

/// The first user colour nobody has yet: (name, #rrggbb).
fn next_free_colour(state: &AppState) -> (&'static str, &'static str) {
    let users = &state.root.show_data.users;
    let used = |c: &str| users.values().any(|u| u.color.eq_ignore_ascii_case(c));
    let i = (0..lumacore_tree::USER_COLORS.len())
        .find(|&i| !used(lumacore_tree::USER_COLORS[i]))
        .unwrap_or(users.len() % lumacore_tree::USER_COLORS.len());
    (lumacore_tree::USER_COLOR_NAMES[i], lumacore_tree::USER_COLORS[i])
}

/// `base`, else `base 2`, `base 3`, … — the first profile name nobody has.
pub fn free_profile_name(state: &AppState, base: &str) -> String {
    let taken = |n: &str| find_profile(state, &format!("\"{n}\"")).is_some() || n.eq_ignore_ascii_case(lumacore_tree::DEFAULT_PROFILE);
    if !taken(base) {
        return base.to_string();
    }
    (2..).map(|i| format!("{base} {i}")).find(|n| !taken(n)).expect("a free name")
}

/// A new profile: a copy of `from` (else of Default) with new GUIDs, named
/// `name` (the next free name on a clash). Returns its number.
pub fn new_profile(state: &mut AppState, name: &str, from: Option<u32>) -> Result<u32, String> {
    let name = free_profile_name(state, name);
    let show = &mut state.root.show_data;
    show.ensure_default_profile();
    let source = match from {
        Some(no) => show.user_profiles.get(&no).ok_or_else(|| with_code(format!("No UserProfile {no}"), "E-CMD-007"))?,
        None => show.default_profile().map(|(_, p)| p).expect("ensured above"),
    };
    let no = show.user_profiles.keys().next_back().map_or(1, |n| n + 1);
    let copy = source.copy_renewed(Some(name));
    show.user_profiles.insert(no, std::sync::Arc::new(copy));
    state.show_dirty = true;
    // Its attribute layer: built from the show's groups when it starts from
    // Default; a copy keeps the other profile's and learns what it lacks.
    crate::commands::attributes::adopt_profile(state, no, from.is_none());
    Ok(no)
}

/// The profiles a user may hold (never Default), for a pick list; the
/// first row "Own — a copy of Default" (no 0) stands for `/Profile New`.
fn profile_items(state: &AppState, own_row: bool) -> Vec<lumacore_wire::PickItem> {
    let show = &state.root.show_data;
    let mut items = Vec::new();
    if own_row {
        items.push(lumacore_wire::PickItem {
            no: 0,
            name: "Own — a copy of Default".into(),
            note: "a new profile named after the user".into(),
            guid: String::new(),
        });
    }
    items.extend(show.user_profiles.iter().filter(|(_, p)| !p.locked).map(|(no, p)| {
        let used: Vec<&str> = show
            .users
            .values()
            .filter(|u| u.profile == p.meta.identity.guid)
            .map(|u| u.login.as_str())
            .collect();
        lumacore_wire::PickItem {
            no: *no,
            name: p.meta.identity.name.clone().unwrap_or_default(),
            note: if used.is_empty() { "nobody".into() } else { format!("used by {}", used.join(", ")) },
            guid: p.meta.identity.guid.to_string(),
        }
    }));
    items
}

/// Bare `Store User` (decisions §30 "12c2"): one ask with every input.
fn ask_new_user(state: &mut AppState) -> String {
    let (colour, _) = next_free_colour(state);
    let name = (1..)
        .map(|i| format!("User {i}"))
        .find(|n| state.root.show_data.user(n).is_none())
        .expect("a free name");
    let pick = lumacore_wire::Pick { kind: "UserProfile".into(), items: profile_items(state, true), ..Default::default() };
    let levels: Vec<(&str, &str)> = UserRights::ALL.iter().map(|r| (r.label(), r.label())).collect();
    state.asking = Some(
        crate::dialogs::Ask::new(
            "New user",
            "A new user always gets a profile: their own copy of Default, or one you share with others.",
        )
        .meta("Store User", None)
        .input("name", "Name", name, lumacore_wire::InputKind::Text)
        .selector("rights", "Rights", &levels, UserRights::Program.label())
        .input("profile", "Profile", "0", lumacore_wire::InputKind::Link { pick })
        .value_text("profile", "0", "New")
        .input("password", "Password (optional)", "", lumacore_wire::InputKind::Password)
        .input("colour", "Colour", colour, lumacore_wire::InputKind::Text)
        .button(
            "Create",
            lumacore_wire::ButtonKind::Primary,
            "Store User \"{name}\" /Rights {rights} /Profile {profile} /Password \"{password}\" /Colour {colour}",
        )
        .cancel("Cancel")
        .flags("the name: Store User \"name\" [/Rights r] [/Profile New|p] [/Password \"pw\"] [/Colour c]"),
    );
    render_code("Store User needs a name: Store User \"name\" [/Rights r] [/Profile New|p]", "E-CMD-006")
}

// Store User's options (decisions §31: declared on the User object).
const RIGHTS: F::FlagDef = F::FlagDef::new("Rights", "The rights level").value("level");
const PROFILE: F::FlagDef = F::FlagDef::new("Profile", "The profile it uses (New, n or \"name\")").value("profile");
const PASSWORD: F::FlagDef = F::FlagDef::new("Password", "Its password").value("password");
const COLOUR: F::FlagDef = F::FlagDef::new("Colour", "Its colour").alias(&["Color"]).value("colour");
/// `Set User "x" "Password" ""` (no password).
const NO_PASSWORD: F::FlagDef = F::FlagDef::new("None", "No password");

/// `Store User "name" [/Rights r] [/Profile New|p] [/Password "pw"] [/Colour c]` — args after `user`.
pub fn cmd_store_user(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return ask_new_user(state);
    }
    // `/Profile` without a value asks with the pick list (decisions §30
    // "12c": the pick lists left from 12b, one ask path).
    if let Some(i) = args.iter().position(|a| F::is(a, &PROFILE))
        && F::value(args, &PROFILE).is_none()
    {
        let rest: Vec<String> = args.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, a)| crate::objects::quote(a)).collect();
        let pick = lumacore_wire::Pick { kind: "UserProfile".into(), items: profile_items(state, true), ..Default::default() };
        state.asking = Some(
            crate::dialogs::Ask::new("Store User", "Which profile does the new user work in?")
                .input("profile", "Profile", "0", lumacore_wire::InputKind::Link { pick })
                .value_text("profile", "0", "New")
                .button("Store", lumacore_wire::ButtonKind::Primary, format!("Store User {} /Profile {{profile}}", rest.join(" ")))
                .cancel("Cancel")
                .flags("a profile: /Profile New|<n|\"name\">"),
        );
        return render_code("/Profile needs a profile (New, n or \"name\")", "E-CMD-006");
    }
    reply((|| {
        let rights = match (F::has(args, &RIGHTS), F::value(args, &RIGHTS)) {
            (false, _) => UserRights::Program,
            (true, Some(level)) => parse_rights(&level)?,
            (true, None) => return Err(with_code("/Rights needs a level", "E-CMD-006")),
        };
        let profile = F::value(args, &PROFILE);
        // An empty password (the ask's empty field) is none.
        let password = F::value(args, &PASSWORD).filter(|p| !p.is_empty());
        let colour = match (F::has(args, &COLOUR), F::value(args, &COLOUR)) {
            (false, _) => None,
            (true, Some(c)) => Some(parse_colour(&c)?),
            (true, None) => return Err(with_code("/Colour needs a value", "E-CMD-006")),
        };
        let words = F::positional(args, &[&RIGHTS, &PROFILE, &PASSWORD, &COLOUR]);
        if let Some(extra) = words.get(1) {
            return Err(with_code(format!("Unexpected '{extra}'"), "E-CMD-006"));
        }
        let name = words.first().map(|w| unquote(w));
        let name = name.filter(|n| !n.is_empty()).ok_or_else(|| {
            with_code("Usage: Store User \"name\" [/Rights r] [/Profile New|p] [/Password \"pw\"] [/Colour c]", "E-CMD-006")
        })?;
        if name.eq_ignore_ascii_case(crate::INTERNAL_ROOT_USER) {
            return Err(with_code("That name is reserved for the internal runtime user.", "E-USER-009"));
        }
        if state.root.show_data.user(&name).is_some() {
            return Err(with_code(format!("User \"{name}\" already exists (use Set User)"), "E-USER-007"));
        }
        // The profile: an existing one to share (never Default), else the
        // user's own copy of Default.
        let shared = match profile.as_deref() {
            None => None,
            Some(p) if p.eq_ignore_ascii_case("new") || p == "0" => None,
            Some(p) => {
                let no = find_profile(state, p).ok_or_else(|| with_code(format!("No profile \"{p}\""), "E-CMD-007"))?;
                if state.root.show_data.is_default_profile(no) {
                    return Err(with_code(
                        "Default is only a template: nobody works in it. Use /Profile New (a copy of Default) or another profile.",
                        "E-USER-008",
                    ));
                }
                Some(no)
            }
        };
        let (profile, new_profile) = match shared {
            Some(no) => (no, false),
            None => (new_profile(state, &name, None)?, true),
        };
        let colour = colour.unwrap_or_else(|| next_free_colour(state).1.to_string());
        let show = &mut state.root.show_data;
        let no = show.users.keys().next_back().map_or(1, |n| n + 1);
        let mut user = User::new(no, name.clone(), rights, show.user_profiles[&profile].meta.identity.guid);
        user.password_hash = password.map(|p| hash_password(&p));
        user.color = colour;
        show.users.insert(no, user);
        let profile_name = show.user_profiles[&profile].meta.identity.name.clone().unwrap_or_default();
        state.show_dirty = true;
        Ok(format!(
            "User {no} \"{name}\" ({}) → profile {profile} \"{profile_name}\"{}",
            rights.label(),
            if new_profile { " (new, a copy of Default)" } else { " (shared)" }
        ))
    })())
}

/// `EditSetting User "name"` (decisions §30 "12b2", the Users & Profiles
/// window): rights, profile (pick list), colour, remote login — each a
/// `Set User` line. Name and password are their own actions (station asks).
pub fn user_fields(
    state: &AppState,
    args: &[&str],
) -> Result<(String, Vec<lumacore_wire::SettingField>), String> {
    use lumacore_wire::{FieldInput, SettingField};
    let name = unquote(&args.join(" "));
    let show = &state.root.show_data;
    let user = show
        .user(&name)
        .ok_or_else(|| crate::errors::render_code(format!("No user \"{name}\""), "E-CMD-011"))?;
    let profile = show.profile_of(user);
    let line = |prop: &str| format!("Set User \"{}\" \"{prop}\" {{value}}", user.login);
    let field = |name: &str, input: FieldInput, value: String, prop: &str| SettingField {
        name: name.to_string(),
        key: String::new(),
        changed: false,
        reset: String::new(),
        section: String::new(),
        input,
        hint: String::new(),
        value,
        line: line(prop),
        pick: None,
        home: "user".into(),
        unit: String::new(),
        rights: if prop == "Rights" { "Admin".into() } else { String::new() },
        readonly: String::new(),
    };
    let mut profile_field = field(
        "Profile",
        FieldInput::Link,
        profile.map(|(no, p)| format!("{no} {}", p.meta.identity.name.clone().unwrap_or_default())).unwrap_or_default(),
        "Profile",
    );
    profile_field.pick = Some(lumacore_wire::Pick {
        kind: "UserProfile".into(),
        items: profile_items(state, false),
        optional: false,
        follow: None,
        missing: None,
        add: None,
        add_no: None,
        by_guid: false,
    });
    Ok((
        format!("User \"{}\"", user.login),
        vec![
            field("Rights", FieldInput::Choice { options: levels() }, user.rights.label().to_string(), "Rights"),
            profile_field,
            field("Colour", FieldInput::Colour, user.color.clone(), "Paint"),
            field("RemoteLogin", FieldInput::Toggle, if user.remote_login { "Yes" } else { "No" }.into(), "RemoteLogin"),
        ],
    ))
}

/// `Set User "x" "Password" Ask` / `"Name" Ask`: a station ask (masked
/// for the password) — the window's Password… and Rename buttons.
fn ask_user_value(state: &mut AppState, name: &str, prop: &str) -> String {
    let password = prop == "password";
    let (title, label, kind) = if password {
        (format!("Password of {name}"), "New password (empty = none)", lumacore_wire::InputKind::Password)
    } else {
        (format!("Rename user {name}"), "Name", lumacore_wire::InputKind::Text)
    };
    let value = if password { String::new() } else { name.to_string() };
    let word = if password { "Password" } else { "Name" };
    let mut ask = crate::dialogs::Ask::new(title, if password {
        "Every login token of the user ends when the password changes."
    } else {
        "The user keeps their programmer, selection and profile; links follow (GUID)."
    })
    .input("value", label, value, kind)
    .button("Set", lumacore_wire::ButtonKind::Primary, format!("Set User \"{name}\" \"{word}\" \"{{value}}\""));
    if password {
        ask = ask.or_when_empty(format!("Set User \"{name}\" \"Password\" \"\""));
    }
    state.asking = Some(ask.cancel("Cancel").flags(format!("the value: Set User \"{name}\" \"{word}\" \"…\"")));
    render_code(format!("Set User \"{name}\" \"{word}\" asks for the value"), "E-CMD-006")
}

/// `Set User "name" "<property>" <value>` — args after `user` (the property bare: the registry
/// passes it so).
pub fn cmd_set_user(state: &mut AppState, args: &[&str]) -> String {
    if let [name, prop] = args
        && matches!(prop.to_lowercase().as_str(), "password" | "name")
    {
        let name = unquote(name);
        let Some(real) = state.root.show_data.user(&name).map(|u| u.login.clone()) else {
            return render_code(format!("No user \"{name}\""), "E-CMD-011");
        };
        let own = state.effective_user_name().eq_ignore_ascii_case(&real);
        let prop = prop.to_lowercase();
        if !own
            && let Err(e) = require_admin(state, &format!("Set User \"{real}\" {}", if prop == "password" { "Password" } else { "Name" }))
        {
            return e;
        }
        return ask_user_value(state, &real, &prop);
    }
    reply((|| {
        let (name, prop, value) = match args {
            [name, prop, value @ ..] => (
                unquote(name),
                prop.to_lowercase(),
                unquote(&value.join(" ")),
            ),
            _ => return Err(with_code(
                "Usage: Set User \"name\" \"Name\"|\"Password\"|\"Rights\"|\"RemoteLogin\"|\"Profile\"|\"ScreenConfig\"|\"Paint\" <value>",
                "E-CMD-006",
            )),
        };
        // Users may change their own password and colour; another user's
        // are Admin's (the other properties are declared Admin: `user_rights`).
        let own = state.effective_user_name().eq_ignore_ascii_case(&name);
        if !own && matches!(prop.as_str(), "password" | "color" | "colour") {
            require_admin(state, &format!("Set User \"{name}\" {prop}"))?;
        }
        if state.root.show_data.user(&name).is_none() {
            return Err(with_code(format!("No user \"{name}\""), "E-CMD-011"));
        }
        let profile = if prop == "profile" {
            if value.eq_ignore_ascii_case("new") {
                let no = new_profile(state, &name, None)?;
                Some((no, state.root.show_data.user_profiles[&no].meta.identity.guid))
            } else {
                let no = find_profile(state, &value).ok_or_else(|| with_code(format!("No profile \"{value}\""), "E-CMD-007"))?;
                if state.root.show_data.is_default_profile(no) {
                    return Err(with_code(
                        "Default is only a template: nobody works in it. Use Profile New (a copy of Default) or another profile.",
                        "E-USER-008",
                    ));
                }
                Some((no, state.root.show_data.user_profiles[&no].meta.identity.guid))
            }
        } else {
            None
        };
        // `ScreenConfig <n>`: a screen config of the user's profile.
        let screen_config = if prop == "screenconfig" {
            let no = value
                .parse::<u32>()
                .map_err(|_| with_code("ScreenConfig needs a number", "E-CMD-008"))?;
            let show = &state.root.show_data;
            let guid = show
                .user(&name)
                .and_then(|u| show.profile_of(u))
                .and_then(|(_, p)| p.content.screen_configs.get(&no))
                .map(|c| c.meta.identity.guid)
                .ok_or_else(|| with_code(format!("The profile of \"{name}\" has no ScreenConfig {no}"), "E-CMD-007"))?;
            Some((no, guid))
        } else {
            None
        };
        if prop == "name" {
            if value.is_empty() {
                return Err(with_code("Set User \"x\" \"Name\" needs a name", "E-CMD-006"));
            }
            if crate::passwords::is_admin_user(&name) {
                return Err(with_code("Admin always exists under its own name.", "E-USER-005"));
            }
            if value.eq_ignore_ascii_case(crate::INTERNAL_ROOT_USER) {
                return Err(with_code("That name is reserved for the internal runtime user.", "E-USER-009"));
            }
            if crate::passwords::is_admin_user(&value)
                || state.root.show_data.user(&value).is_some_and(|u| !u.login.eq_ignore_ascii_case(&name))
            {
                return Err(with_code(format!("User \"{value}\" already exists."), "E-USER-007"));
            }
        }
        let colour = if matches!(prop.as_str(), "color" | "colour") { Some(parse_colour(&value)?) } else { None };
        let admin_count = admins(state);
        let user = state
            .root
            .show_data
            .user_mut(&name)
            .expect("checked above");
        let old_name = user.login.clone();
        let msg = match prop.as_str() {
            "password" if crate::passwords::is_admin_user(&user.login) => {
                return Err(with_code(
                    "Admin's password is always \"admin\" and can't be changed.",
                    "E-USER-001",
                ));
            }
            "password" => {
                let value = if F::is(&value, &NO_PASSWORD) { String::new() } else { value };
                user.password_hash = (!value.is_empty()).then(|| hash_password(&value));
                if value.is_empty() {
                    "password removed".to_string()
                } else {
                    "password set".to_string()
                }
            }
            "name" => {
                user.login = value.clone();
                format!("renamed to \"{value}\"")
            }
            "rights" => {
                let rights = parse_rights(&value)?;
                if crate::passwords::is_admin_user(&user.login) && rights != UserRights::Admin {
                    return Err(with_code("The user Admin is always Admin.", "E-USER-005"));
                }
                if user.rights == UserRights::Admin && rights != UserRights::Admin && admin_count <= 1 {
                    return Err(with_code("At least one Admin must remain.", "E-USER-004"));
                }
                user.rights = rights;
                format!("rights {}", rights.label())
            }
            "color" | "colour" => {
                user.color = colour.expect("parsed above");
                format!("colour {}", user.color)
            }
            "remotelogin" => {
                user.remote_login =
                    matches!(value.to_lowercase().as_str(), "on" | "yes" | "true" | "1");
                format!(
                    "remote login {}",
                    if user.remote_login { "on" } else { "off" }
                )
            }
            "profile" => {
                let (no, guid) = profile.expect("parsed above");
                user.profile = guid;
                format!("profile {no}")
            }
            "screenconfig" => {
                let (no, guid) = screen_config.expect("parsed above");
                user.default_screen_config = Some(guid);
                format!("default screen config {no}")
            }
            _ => return Err(with_code(format!("Unknown user property '{prop}'"), "E-SET-001")),
        };
        let label = user.login.clone();
        if prop == "name" {
            rename_runtime(state, &old_name, &label);
        }
        state.show_dirty = true;
        Ok(format!("User \"{label}\": {msg}"))
    })())
}

/// A renamed user keeps their claims and runtime (modes, Oops list).
fn rename_runtime(state: &mut AppState, old: &str, new: &str) {
    let runtime = &mut state.root.runtime;
    for claim in runtime.claims.values_mut() {
        if claim.user.eq_ignore_ascii_case(old) {
            claim.user = new.to_string();
        }
    }
    if let Some(mut user) = runtime.users.remove(old) {
        user.user = new.to_string();
        runtime.users.insert(new.to_string(), user);
    }
}

/// Where `name` is logged in: this station (with its screens) and members.
fn logged_in_where(state: &AppState, name: &str) -> Vec<String> {
    let mut at = Vec::new();
    let runtime = &state.root.runtime;
    if runtime.users.get(name).is_some_and(|u| u.attached) {
        let screens = runtime
            .live_screens
            .read()
            .map(|s| s.for_user(name).count())
            .unwrap_or(0);
        let station = &state.root.station_settings.station_name;
        at.push(match screens {
            0 => station.clone(),
            1 => format!("{station} · 1 screen"),
            n => format!("{station} · {n} screens"),
        });
    }
    for member in runtime.luma_sessions.members.values() {
        if member.users.iter().any(|u| u.eq_ignore_ascii_case(name)) && !at.iter().any(|s| s.starts_with(&member.station)) {
            at.push(member.station.clone());
        }
    }
    at
}

/// `Delete User "name" [/WithProfile] [/NoConfirmation]` — args after
/// `user`. Asks what goes first (decisions §30 "12c2"); the user is logged
/// out everywhere and their claims are released. One Oops step brings the
/// user, their programmer, selection and (with /WithProfile) the profile back.
/// `Delete User … /WithProfile` (decisions §31: declared on the User object).
const WITH_PROFILE: F::FlagDef = F::FlagDef::new("WithProfile", "Delete its profile too");

pub fn cmd_delete_user(state: &mut AppState, args: &[&str]) -> String {
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    let with_profile = crate::commands::flags::has(args, &WITH_PROFILE);
    // Other flags never get here: dispatch refuses an undeclared one (E-CMD-018).
    let name = unquote(&crate::commands::flags::positional(args, &[]).join(" "));
    if name.is_empty() {
        return render_code("Usage: Delete User \"name\" [/WithProfile] [/NoConfirmation]", "E-CMD-006");
    }
    if name.eq_ignore_ascii_case(crate::INTERNAL_ROOT_USER) {
        return render_code("The station's internal runtime user can't be deleted.", "E-USER-009");
    }
    if crate::passwords::is_admin_user(&name) {
        return render_code("Admin always exists.", "E-USER-005");
    }
    let show = &state.root.show_data;
    let Some(user) = show.user(&name) else {
        return render_code(format!("No user \"{name}\""), "E-CMD-011");
    };
    let name = user.login.clone();
    if state.effective_user_name().eq_ignore_ascii_case(&name) {
        return render_code("You can't delete yourself.", "E-USER-003");
    }
    if user.rights == UserRights::Admin && admins(state) <= 1 {
        return render_code("At least one Admin must remain.", "E-USER-004");
    }
    // The profile goes too only when nobody else works in it.
    let profile = show.profile_of(user).map(|(no, p)| (no, p.meta.identity.name.clone().unwrap_or_default(), p.locked));
    let others: Vec<String> = show
        .users
        .values()
        .filter(|u| u.profile == user.profile && u.meta.identity.guid != user.meta.identity.guid)
        .map(|u| u.login.clone())
        .collect();
    let profile_alone = profile.as_ref().is_some_and(|(_, _, locked)| !locked) && others.is_empty();
    let claims: Vec<String> = state
        .root
        .runtime
        .claims
        .values()
        .filter(|c| c.user.eq_ignore_ascii_case(&name))
        .map(|c| c.label.clone())
        .collect();
    if !confirmed {
        let values = user.programmer.values.len();
        let selected = user.programmer.selected_fixtures.len();
        let at = logged_in_where(state, &name);
        let plural = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
        let mut ask = crate::dialogs::Ask::new(
            format!("Delete User \"{name}\""),
            if at.is_empty() {
                format!("Deleting {name} removes their work:")
            } else {
                format!("{name} is logged in. Deleting logs {name} out everywhere and removes their work:")
            },
        )
        .meta("one Oops step", Some(lumacore_wire::Tone::Warn));
        if !at.is_empty() {
            ask = ask.row("Logged in", format!("{} · logged out now", at.join(", ")));
        }
        ask = ask
            .row("Programmer", format!("{} — go with {name}", plural(values, "value", "values")))
            .row("Selection", format!("{} — goes with {name}", plural(selected, "fixture", "fixtures")))
            .row(
                "Claims",
                if claims.is_empty() { "none".to_string() } else { format!("{} — released", claims.join(", ")) },
            );
        let lines = osc_lines_of(state, &name);
        if !lines.is_empty() {
            ask = ask.row("Lines", format!("used by {} — stop (no user) until another user is set", lines.join(", ")));
        }
        if let Some((_, pname, _)) = &profile {
            if profile_alone {
                ask = ask.toggle(
                    "withprofile",
                    &format!("Also delete profile \"{pname}\" (nobody else uses it)"),
                    true,
                    "/WithProfile",
                );
            } else if !others.is_empty() {
                ask = ask.row("Profile", format!("\"{pname}\" stays — used by {}", others.join(", ")));
            }
        }
        let toggle = if profile_alone { " {withprofile}" } else { "" };
        state.asking = Some(
            ask.button(
                &format!("Delete {name}"),
                lumacore_wire::ButtonKind::Danger,
                format!("Delete User \"{name}\"{toggle} /NoConfirmation"),
            )
            .cancel("Cancel")
            .note(format!("Oops brings {name}, the programmer and the profile back (not the session or claims)."))
            .flags("/NoConfirmation [/WithProfile]"),
        );
        return render_code(format!("Delete User \"{name}\" asks first: add /NoConfirmation."), "E-CMD-010");
    }
    // Go: the user (and their own profile), then everything of their session.
    let lines = osc_lines_of(state, &name);
    let show = &mut state.root.show_data;
    let no = show.user_no(&name).expect("found above");
    let user = show.users.remove(&no).expect("found above");
    let mut note = String::new();
    if with_profile && let Some((pno, pname, _)) = &profile {
        if profile_alone {
            show.user_profiles.remove(pno);
            note = format!(", profile {pno} \"{pname}\" deleted");
        } else {
            note = format!(", profile \"{pname}\" kept (used by {})", others.join(", "));
        }
    } else if let Some((pno, pname, _)) = &profile {
        note = format!(", profile {pno} \"{pname}\" kept");
    }
    end_sessions(state, &user.login);
    state.show_dirty = true;
    let stopped = if lines.is_empty() {
        String::new()
    } else {
        format!("\n{} stopped: no user (Set Input <n> \"User\" <name>)", lines.join(", "))
    };
    render_success(&format!(
        "Deleted user {no} \"{}\"{note}{}{stopped}",
        user.login,
        if claims.is_empty() { String::new() } else { format!("; released {}", claims.join(", ")) },
    ))
}

/// A deleted user is logged out everywhere (decisions §30 "12c2"): claims
/// released (askers are told), claim asks dropped, runtime ended, every
/// token revoked. Their shells close when they next render (the user is
/// gone from the show — `lumacore_shell`), on every station.
fn end_sessions(state: &mut AppState, name: &str) {
    let released = state.release_claims_of(name);
    if released > 0 {
        state.sysmon_pending.push((
            "Claims".to_string(),
            format!("[{name}] {released} claim(s) released: the user was deleted"),
        ));
    }
    state
        .root
        .runtime
        .claim_asks
        .retain(|a| !a.from.eq_ignore_ascii_case(name) && !a.owner.eq_ignore_ascii_case(name));
    state.root.runtime.users.retain(|u, _| !u.eq_ignore_ascii_case(name));
    crate::passwords::revoke_user(state, name);
    state.sysmon_pending.push(("Session".to_string(), format!("User {name} deleted: logged out everywhere")));
}

/// Bare `User` — list users.
pub fn cmd_list_users(state: &AppState) -> String {
    let show = &state.root.show_data;
    let mut out = String::from("Users:\n");
    for (no, u) in &show.users {
        let (profile_no, profile) = show
            .profile_of(u)
            .map_or((0, "?"), |(n, p)| (n, p.name().unwrap_or("?")));
        out.push_str(&format!(
            "  {no:>3}  {:<16} {:<11} profile {} \"{}\"{}{}\n",
            u.login,
            u.rights.label(),
            profile_no,
            profile,
            if u.password_hash.is_some() || crate::passwords::is_admin_user(&u.login) {
                "  🔒"
            } else {
                ""
            },
            if u.remote_login { "" } else { "  (local only)" },
        ));
    }
    out
}

// ── Object: User (see terminal/objects) ─────────────────────────────────────

use crate::objects::{ObjectDef, Verb, filter, out, quote};
use crate::DispatchResult;
use crate::claims::Claims;

fn user_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(cmd_list_users(state))
}
fn user_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_store_user(state, args))
}
fn user_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    // Several users (`Delete User "A" + "B"`): one ask for all (decisions
    // §30 "12d"); one user keeps its own ask (what goes with it).
    crate::commands::set_rows::delete_rows(state, &USER_ROWS, args, |s, a| out(cmd_delete_user(s, a)))
}

/// Deleting users: one is asked by its handler (what goes with them);
/// several get dispatch's one ask.
static USER_ROWS: crate::commands::set_rows::RowDelete = crate::commands::set_rows::RowDelete {
    keyword: "User",
    noun: "user",
    numbers: |s| s.root.show_data.users.keys().copied().collect(),
    refused: |_, _| None,
    affects: |state, row| {
        let lines = osc_lines_of(state, &user_of_row(state, row)?);
        (!lines.is_empty()).then(|| format!("used by {} — stops (no user)", lines.join(", ")))
    },
    own_ask: true,
};
/// A Delete row (a number or a name) as the user's name.
fn user_of_row(state: &AppState, row: &str) -> Option<String> {
    let show = &state.root.show_data;
    match row.parse::<u32>() {
        Ok(no) => show.users.get(&no).map(|u| u.login.clone()),
        Err(_) => show.user(&unquote(row)).map(|u| u.login.clone()),
    }
}

/// The lines that run as this user ("Osc 3"; decisions §31 "OSC line users").
fn osc_lines_of(state: &AppState, name: &str) -> Vec<String> {
    let show = &state.root.show_data;
    let Some(guid) = show.user(name).map(|u| u.meta.identity.guid) else {
        return Vec::new();
    };
    show.protocols
        .all_lines()
        .filter(|(_, _, line)| line.user == Some(guid))
        .map(|(protocol, no, _)| format!("{protocol} {no}"))
        .collect()
}

/// `Info User "LD"` — one user (no password hash, of course).
fn user_info(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let name = unquote(&args.join(" "));
    if name.is_empty() {
        return out(cmd_list_users(state));
    }
    let show = &state.root.show_data;
    let no = show.user_no(&name).unwrap_or_default();
    let Some(u) = show.user(&name) else {
        return out(render_code(format!("No user \"{name}\""), "E-CMD-011"));
    };
    let (profile_no, profile) = show
        .profile_of(u)
        .map_or((0, "?"), |(n, p)| (n, p.name().unwrap_or("?")));
    let screen_config = show
        .profile_of(u)
        .and_then(|(_, p)| {
            p.content.screen_configs
                .iter()
                .find(|(_, c)| Some(c.meta.identity.guid) == u.default_screen_config)
                .map(|(&n, _)| n.to_string())
        })
        .unwrap_or_else(|| "first".to_string());
    out(format!(
        "User {} \"{}\"\n  Rights         {}\n  Profile        {} \"{}\"\n  Password       {}\n  Remote login   {}\n  Screen config  {}\n  Selection      {} fixtures\n  Programmer     {} values ({} active)\n",
        no,
        u.login,
        u.rights.label(),
        profile_no,
        profile,
        if u.password_hash.is_some() || crate::passwords::is_admin_user(&u.login) {
            "set"
        } else {
            "none"
        },
        if u.remote_login {
            "allowed"
        } else {
            "this machine only"
        },
        screen_config,
        u.programmer.selected_fixtures.len(),
        u.programmer.values.len(),
        u.programmer.values.values().filter(|v| v.active).count(),
    ))
}
fn user_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(
        state.root.show_data.users.values().map(|u| quote(&u.login)),
        prefix,
    )
}

pub static USER: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "User",
        lum: crate::objects::LumShape::ids(1).children(&[&crate::objects::system::PROGRAMMER.decl]),
        default_verb: crate::objects::Verb::Info,
        flags: &[RIGHTS.on(&[Verb::Store]), PROFILE.on(&[Verb::Store]), PASSWORD.on(&[Verb::Store]), COLOUR.on(&[Verb::Store]), WITH_PROFILE.on(&[Verb::Delete]), NO_PASSWORD.on(&[Verb::Set])],
        aliases: &["users"],
        doc: crate::commands::registry::Doc {
            display: "User",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "User | Store User [\"name\" [/Rights r] [/Profile New|p] [/Password \"pw\"] [/Colour c]] | Set User \"name\" \"Name\"|\"Password\"|\"Rights\"|\"Profile\"|\"Paint\"|\"RemoteLogin\"|\"ScreenConfig\" <value> (\"Password\" Ask asks) | Delete User \"name\" [/WithProfile] [/NoConfirmation]",
            description: "Users (ShowData.Users). A user owns their programmer and selection, \
                          has one rights level (Guest < Playback < Presets < Program < Setup < \
                          Admin, one ladder — rights come only from the user) \
                          and works in a profile (views, screens, keys, settings). A new user gets \
                          their own profile, a copy of the locked template Default, or shares one \
                          (never Default). A bare Store User asks; Delete User asks what goes \
                          (logs the user out everywhere, releases their claims; one Oops step). \
                          Store / Delete User and Set (but your own password and colour) need Admin; \
                          the user Admin is always Admin.",
            examples: &[
                "User                                   -- list users",
                "Store User                             -- asks: name, rights, profile, password, colour",
                "Store User \"LD\" /Rights Program /Password \"pw\" /Colour Teal",
                "Set User \"LD\" \"Rights\" Setup -- only an Admin grants levels",
                "Store User \"Phone\" /Profile \"LD\"    -- share LD's views and screens",
                "Set User \"LD\" \"Name\" \"Anna\"",
                "Set User \"LD\" \"Password\" Ask -- asks (masked)",
                "Delete User \"LD\" /WithProfile /NoConfirmation",
            ],
            related: &["login", "logout"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: Some(crate::commands::registry::CommandFamily::Session),
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Session,
        scope: crate::objects::on_master,
        reach: crate::objects::show_reach,
        rights: user_rights,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Store, user_store),
        (Verb::Delete, user_delete),
        (Verb::List, user_list),
        (Verb::Info, user_info),
    ],
    delete_ask: Some(|s, a| crate::commands::set_rows::delete_ask(s, &USER_ROWS, a)),
    complete: Some(user_complete),
    props: Some(&USER_SETTINGS),
    tree: Some(crate::objects::TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::Users,
        children: users_children,
        object: user_view,
    }),
};

// ── A user's properties (decisions §33 58; `"Password" Ask` — 17) ────────────

/// One property of user `no` (the tree member), written by `cmd_set_user`; `Ask` opens the
/// station's dialog for a name or password.
fn user_write(state: &mut AppState, no: &str, prop: &str, value: &str) -> Result<String, String> {
    let name = no
        .parse::<u32>()
        .ok()
        .and_then(|n| state.root.show_data.users.get(&n))
        .map(|u| format!("\"{}\"", u.login))
        .ok_or_else(|| render_code(format!("No User {no}."), "E-CMD-007"))?;
    let text = if value == "Ask" { cmd_set_user(state, &[&name, prop]) } else { cmd_set_user(state, &[&name, prop, &format!("\"{value}\"")]) };
    crate::props::from_reply(text)
}

fn user_get(state: &AppState, no: &str, f: impl Fn(&AppState, &lumacore_tree::User) -> String) -> String {
    no.parse::<u32>().ok().and_then(|n| state.root.show_data.users.get(&n)).map(|u| f(state, u)).unwrap_or_default()
}

macro_rules! user_prop {
    ($name:literal, $prop:literal, $kind:expr, $named:expr, $rights:ident, $doc:literal, $get:expr) => {
        crate::props::Property {
            name: $name,
            kind: $kind,
            named: $named,
            rights: crate::commands::registry::Rights::$rights,
            section: "User",
            doc: $doc,
            access: crate::props::Access::Keyed { get: |s, k| user_get(s, k, $get), set: |s, k, v| user_write(s, k, $prop, v) },
            ..crate::props::Property::BASE
        }
    };
}

static USER_PROPS: [crate::props::Property; 7] = [
    user_prop!("Name", "name", crate::props::PropKind::Text, &["Ask"], Admin, "the user's name (Ask opens the rename dialog)", |_, u| u.login.clone()),
    user_prop!("Password", "password", crate::props::PropKind::Text, &["Ask"], Guest, "your own, or another user's as Admin; empty = none; Ask opens the dialog", |_, _| String::new()),
    user_prop!("Rights", "rights", crate::props::PropKind::Choice(lumacore_tree::UserRights::LABELS), &[], Admin, "the user's level", |_, u| u.rights.label().to_string()),
    user_prop!("Paint", "colour", crate::props::PropKind::Text, &[], Guest, "the user's colour (a name or #rrggbb; your own, or another's as Admin)", |_, u| u.color.clone()),
    user_prop!("RemoteLogin", "remotelogin", crate::props::PropKind::OnOff, &[], Admin, "may log in from another machine", |_, u| if u.remote_login { "On" } else { "Off" }.to_string()),
    user_prop!("Profile", "profile", crate::props::PropKind::Text, &["New"], Admin, "the profile the user works in (a name or number; New = a copy of Default)", |s, u| s.root.show_data.profile_of(u).map(|(no, _)| no.to_string()).unwrap_or_default()),
    user_prop!("ScreenConfig", "screenconfig", crate::props::PropKind::Int { min: 1, max: 9999 }, &[], Admin, "the screen config a new device of the user starts on", |_, _| String::new()),
];

static USER_SETTINGS: crate::props::Props = crate::props::Props { lists: &[&USER_PROPS], ..crate::props::Props::BASE };

/// Store and Delete User are Admin's (decisions §30 "12c2"); Set checks
/// per property (your own password and colour are yours).
/// Users are Admin's (decisions §30 "12c2", "12d"): Store, Delete and `Set
/// User` of the name, rights, profile, remote login and screen config.
/// Your own password and colour are anyone's (another user's are Admin's,
/// checked by `Set User`); reading is anyone's.
fn user_rights(verb: Verb, args: &[&str]) -> crate::commands::registry::Rights {
    use crate::commands::registry::Rights;
    match verb {
        Verb::Store | Verb::Delete => Rights::Admin,
        Verb::Set => match args.get(1).map(|p| p.trim_matches('"').to_lowercase()) {
            // tidy: not-keywords (Set User's own properties)
            None => Rights::Guest,
            Some(p) if matches!(p.as_str(), "password" | "paint" | "color" | "colour") => Rights::Guest,
            Some(_) => Rights::Admin,
        },
        _ => Rights::Guest,
    }
}

fn users_children(root: &lumacore_tree::Root) -> Vec<crate::vfs::DynChild> {
    root.show_data
        .users
        .iter()
        .map(|(&no, u)| crate::vfs::DynChild {
            display_name: format!(
                "{no} \"{}\"  ({}, profile {})",
                u.login,
                u.rights.label(),
                u.profile
            ),
            id: no,
            node_type: "User",
            is_leaf: true,
            child_count: 0,
            name: Some(u.login.clone()),
            guid: None,
        })
        .collect()
}
/// A user's view — never the password hash, only whether one is set.
fn user_view(root: &lumacore_tree::Root, id: u32) -> Option<crate::vfs::NodeView> {
    let u = root.show_data.users.get(&id)?;
    let json = serde_json::json!({
        "no": id,
        "name": u.login,
        "rights": u.rights.label(),
        "profile": u.profile,
        "has_password": u.password_hash.is_some() || crate::passwords::is_admin_user(&u.login),
        "remote_login": u.remote_login,
        "color": u.color,
        "default_screen_config": u.default_screen_config,
        "programmer": {
            "selected": u.programmer.selected_fixtures.len(),
            "values": u.programmer.values.len(),
            "active": u.programmer.values.values().filter(|v| v.active).count(),
        },
    });
    Some(crate::vfs::NodeView::Object {
        json,
        node_type: "User",
        id,
        guid: String::new(),
        name: Some(u.login.clone()),
    })
}
