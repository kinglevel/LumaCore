//! Making, copying, renaming and deleting profiles (decisions §30 "12c2").
//!
//! ```text
//! Store UserProfile                          asks: name, start from (Default / copy of… / import file…)
//! Store UserProfile ["name"] [/CopyOf n|"name"]  a copy of Default (or of a profile); never blank
//! Copy UserProfile n At m|"name"             a copy with new GUIDs
//! Label UserProfile n ["name"]               rename (bare asks)
//! Delete UserProfile n [/NoConfirmation]     asks; refused while a user works in it
//! ```
//!
//! UserProfile "Default" is the locked template: it
//! can't be changed, renamed or deleted, and nobody works in it. Duplicate
//! names get the next free ("Bob 2"). Admin only (the profile's rights,
//! `ObjectDef::rights`); a profile's pools (`UserProfile 2 Image 3`) stay its
//! users' too.

use crate::AppState;
use crate::commands::users::{find_profile, free_profile_name, new_profile};
use crate::errors::render_code;
use crate::text::render_success;

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// `Store UserProfile "x" /CopyOf <profile>` (decisions §31: declared on the UserProfile object).
pub const COPY_OF: crate::commands::flags::FlagDef =
    crate::commands::flags::FlagDef::new("CopyOf", "Start as a copy of this profile").value("profile");

/// The refusal for the template (E-USER-002; the rule is dispatch's).
pub use super::dispatch::DEFAULT_LOCKED;

fn label_of(state: &AppState, no: u32) -> String {
    state
        .root
        .show_data
        .user_profiles
        .get(&no)
        .and_then(|p| p.meta.identity.name.clone())
        .unwrap_or_else(|| format!("UserProfile {no}"))
}

/// The profiles a new one may start from (Default first), for a pick list.
fn source_items(state: &AppState) -> Vec<lumacore_wire::PickItem> {
    state
        .root
        .show_data
        .user_profiles
        .iter()
        .map(|(no, p)| lumacore_wire::PickItem {
            no: *no,
            name: p.meta.identity.name.clone().unwrap_or_default(),
            note: if p.locked { "the template".into() } else { String::new() },
            guid: p.meta.identity.guid.to_string(),
        })
        .collect()
}

/// Bare `Store UserProfile`: one ask — the name and what it starts from.
fn ask_new_profile(state: &mut AppState) -> String {
    let name = free_profile_name(state, "Profile");
    state.asking = Some(
        crate::dialogs::Ask::new("New profile", "A profile starts from something that works — never blank.")
            .meta("Store UserProfile", None)
            .input("name", "Name", name, lumacore_wire::InputKind::Text)
            .selector(
                "from",
                "Start from",
                &[
                    ("From Default", "Store UserProfile \"{name}\""),
                    ("Copy of…", "Store UserProfile \"{name}\" /CopyOf"),
                    ("Import file…", "Import UserProfile /As \"{name}\""),
                ],
                "From Default",
            )
            .button("Create", lumacore_wire::ButtonKind::Primary, "{from}")
            .cancel("Cancel")
            .flags("the name: Store UserProfile \"name\" [/CopyOf n]"),
    );
    render_code("Store UserProfile needs a name: Store UserProfile \"name\" [/CopyOf n]", "E-CMD-006")
}

/// `Store UserProfile ["name"] [/CopyOf n|"name"]` — args after the object word.
pub fn store(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return ask_new_profile(state);
    }
    use crate::commands::flags as F;
    // `/CopyOf` with no profile asks with the pick list.
    let copy_of = F::has(args, &COPY_OF).then(|| F::value(args, &COPY_OF));
    let words = F::positional(args, &[&COPY_OF]);
    if let Some(extra) = words.get(1) {
        return render_code(format!("Unexpected '{extra}'"), "E-CMD-006");
    }
    let name = words.first().map(|w| unquote(w));
    let name = name.filter(|n| !n.is_empty()).unwrap_or_else(|| "Profile".to_string());
    let from = match copy_of {
        None => None,
        // `/CopyOf` without a profile: the pick list.
        Some(None) => {
            let items = source_items(state);
            let first = items.first().map(|i| i.no.to_string()).unwrap_or_default();
            let pick = lumacore_wire::Pick { kind: "UserProfile".into(), items, ..Default::default() };
            state.asking = Some(
                crate::dialogs::Ask::new(format!("New profile \"{name}\""), "Which profile does it start as a copy of?")
                    .input("profile", "Copy of", first, lumacore_wire::InputKind::Link { pick })
                    .button(
                        "Create",
                        lumacore_wire::ButtonKind::Primary,
                        format!("Store UserProfile \"{name}\" /CopyOf {{profile}}"),
                    )
                    .cancel("Cancel")
                    .flags("a profile: /CopyOf <n|\"name\">"),
            );
            return render_code("/CopyOf needs a profile (n or \"name\")", "E-CMD-006");
        }
        Some(Some(spec)) => match find_profile(state, &spec) {
            Some(no) => Some(no),
            None => return render_code(format!("No UserProfile \"{spec}\""), "E-CMD-007"),
        },
    };
    let source = from.map_or_else(|| lumacore_tree::DEFAULT_PROFILE.to_string(), |no| label_of(state, no));
    match new_profile(state, &name, from) {
        Ok(no) => render_success(&format!(
            "UserProfile {no} \"{}\" — a copy of \"{source}\"",
            label_of(state, no)
        )),
        Err(e) => crate::errors::reply(e, "E-CMD-010"),
    }
}

/// `Copy UserProfile n At m|"name"` — args after the object word.
pub fn copy(state: &mut AppState, args: &[&str]) -> String {
    let usage = || render_code("Usage: Copy UserProfile n|\"name\" At m|\"name\"", "E-CMD-006");
    let Some(at) = args.iter().position(|a| a.eq_ignore_ascii_case("at")) else {
        return usage();
    };
    let (source, target) = (&args[..at], &args[at + 1..]);
    let ([source], [target]) = (source, target) else {
        return usage();
    };
    let Some(from) = find_profile(state, source) else {
        return render_code(format!("No UserProfile {source}"), "E-CMD-007");
    };
    let base = label_of(state, from);
    let (name, number) = match unquote(target).parse::<u32>() {
        Ok(0) => return render_code("Profile numbers start at 1", "E-CMD-008"),
        Ok(n) if state.root.show_data.user_profiles.contains_key(&n) => {
            return render_code(format!("UserProfile {n} exists (\"{}\").", label_of(state, n)), "E-CMD-010");
        }
        Ok(n) => (base, Some(n)),
        Err(_) => (unquote(target), None),
    };
    match new_profile(state, &name, Some(from)) {
        Ok(no) => {
            let no = match number {
                Some(n) if n != no => {
                    let show = &mut state.root.show_data;
                    let p = show.user_profiles.remove(&no).expect("just made");
                    show.user_profiles.insert(n, p);
                    n
                }
                _ => no,
            };
            render_success(&format!("UserProfile {no} \"{}\" — a copy of \"{}\"", label_of(state, no), label_of(state, from)))
        }
        Err(e) => crate::errors::reply(e, "E-CMD-010"),
    }
}

/// `Label UserProfile n ["name"]` — args after the object word. Bare asks.
pub fn label(state: &mut AppState, args: &[&str]) -> String {
    let Some((first, rest)) = args.split_first() else {
        return render_code("Usage: Label UserProfile n \"name\"", "E-CMD-006");
    };
    let Some(no) = find_profile(state, first) else {
        return render_code(format!("No UserProfile {first}"), "E-CMD-007");
    };
    if state.root.show_data.is_default_profile(no) {
        return render_code(DEFAULT_LOCKED, "E-USER-002");
    }
    let old = label_of(state, no);
    let name = unquote(&rest.join(" "));
    if name.is_empty() {
        state.asking = Some(
            crate::dialogs::Ask::new(format!("Rename profile \"{old}\""), "Its users, screens and views stay as they are.")
                .input("name", "Name", old.clone(), lumacore_wire::InputKind::Text)
                .button("Rename", lumacore_wire::ButtonKind::Primary, format!("Label UserProfile {no} \"{{name}}\""))
                .cancel("Cancel")
                .flags("the name: Label UserProfile n \"name\""),
        );
        return render_code("Label UserProfile needs a name", "E-CMD-006");
    }
    if name.eq_ignore_ascii_case(lumacore_tree::DEFAULT_PROFILE) {
        return render_code("\"Default\" is the template's name.", "E-USER-009");
    }
    if find_profile(state, &format!("\"{name}\"")).is_some_and(|n| n != no) {
        return render_code(format!("UserProfile \"{name}\" exists."), "E-USER-007");
    }
    let show = &mut state.root.show_data;
    show.profile_mut(no).expect("found").meta.identity.name = Some(name.clone());
    state.show_dirty = true;
    render_success(&format!("UserProfile {no} \"{old}\" → \"{name}\""))
}

/// `Delete UserProfile n [/NoConfirmation]` — args after the object word.
pub fn delete(state: &mut AppState, args: &[&str]) -> String {
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let [spec] = words.as_slice() else {
        return render_code("Usage: Delete UserProfile n|\"name\" [/NoConfirmation]", "E-CMD-006");
    };
    let Some(no) = find_profile(state, spec) else {
        return render_code(format!("No UserProfile {spec}"), "E-CMD-007");
    };
    let show = &state.root.show_data;
    if show.is_default_profile(no) {
        return render_code(DEFAULT_LOCKED, "E-USER-002");
    }
    let p = &show.user_profiles[&no];
    let name = p.meta.identity.name.clone().unwrap_or_else(|| format!("UserProfile {no}"));
    let users: Vec<&str> = show
        .users
        .values()
        .filter(|u| u.profile == p.meta.identity.guid)
        .map(|u| u.login.as_str())
        .collect();
    if !users.is_empty() {
        return render_code(
            format!("\"{name}\" is used by {} — move them first (Set User \"x\" \"Profile\" …).", users.join(", ")),
            "E-USER-006",
        );
    }
    if !confirmed {
        let screens: usize = p.content.screen_configs.values().map(|c| c.screens.len()).sum();
        let presets: usize = p.content.window_presets.values().map(|t| t.len()).sum();
        state.asking = Some(
            crate::dialogs::Ask::new(
                format!("Delete UserProfile \"{name}\""),
                format!("Nobody uses \"{name}\". Its screens, views, keys, presets and look go with it."),
            )
            .meta("one Oops step", Some(lumacore_wire::Tone::Warn))
            .row("Screens / views", format!("{screens} / {}", p.content.views.len()))
            .row("Window presets", presets.to_string())
            .row("Keys / pictures", format!("{} / {}", p.content.ui_keys.len(), p.content.images.len() + p.content.icons.len()))
            .button("Delete", lumacore_wire::ButtonKind::Danger, format!("Delete UserProfile {no} /NoConfirmation"))
            .cancel("Cancel")
            .flags("/NoConfirmation"),
        );
        return render_code(format!("Delete UserProfile \"{name}\" asks first: add /NoConfirmation."), "E-CMD-010");
    }
    state.root.show_data.user_profiles.remove(&no);
    state.show_dirty = true;
    render_success(&format!("Deleted UserProfile {no} \"{name}\""))
}

/// Bare `Import UserProfile [/As "name"]`: pick a file of
/// `library/userprofiles/` (and the new profile's name).
pub fn ask_import(state: &mut AppState, name: &str) -> String {
    // The station's "which file?" ask (decisions §38 5): the files element on userprofiles/,
    // the drives on its rail.
    state.asking = Some(
        crate::dialogs::Ask::new("Import UserProfile", "A profile file becomes a new profile (new GUIDs when you name it).")
            .file("file", "File", "userprofiles")
            .input("name", "Name (empty = the file's)", name, lumacore_wire::InputKind::Text)
            .button("Import", lumacore_wire::ButtonKind::Primary, "Import UserProfile {file} /As \"{name}\"")
            .cancel("Cancel")
            .flags("the file: Import UserProfile \"file\" [/As \"name\"] [/Drive \"usb1\"]"),
    );
    render_code("Import UserProfile needs a file: Import UserProfile \"file\" [/As \"name\"]", "E-CMD-006")
}

/// The profile's own verbs (Store, Copy, Delete, Label, Import) are
/// Admin's; reading it and its look (`Set UserProfile Theme…`, the property
/// levels) are anyone's; its pools (`UserProfile 2 Image 3`, `… WindowPreset
/// 1`) and `Reset UserProfile` (its windows) are Playback, like the user's
/// own screens — which profile is checked by the pool verbs (its users and
/// Admins).
pub fn rights(verb: crate::objects::Verb, args: &[&str]) -> crate::commands::registry::Rights {
    use crate::commands::registry::Rights;
    use crate::objects::Verb;
    let pool = |w: &&str| {
        crate::commands::media::IMAGE.matches(w)
            || crate::commands::media::ICON.matches(w)
            || crate::commands::window_presets::WINDOW_PRESET.matches(w)
    };
    let names_pool = args.iter().take(2).any(pool);
    match verb {
        // Edit: your attribute board (12f-a) — each change it sends is checked.
        Verb::List | Verb::Info | Verb::Dump | Verb::Call | Verb::Set | Verb::Edit => Rights::Guest,
        Verb::Store | Verb::Copy | Verb::Delete | Verb::Label | Verb::Import if !names_pool => Rights::Admin,
        _ => Rights::Playback,
    }
}
