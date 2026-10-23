// ██████╗ ██╗   ██╗████████╗████████╗ ██████╗ ███╗   ██╗███████╗
// ██╔══██╗██║   ██║╚══██╔══╝╚══██╔══╝██╔═══██╗████╗  ██║██╔════╝
// ██████╔╝██║   ██║   ██║      ██║   ██║   ██║██╔██╗ ██║███████╗
// ██╔══██╗██║   ██║   ██║      ██║   ██║   ██║██║╚██╗██║╚════██║
// ██████╔╝╚██████╔╝   ██║      ██║   ╚██████╔╝██║ ╚████║███████║
// ╚═════╝  ╚═════╝    ╚═╝      ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚══════╝
// FILE: buttons

//! The view bar's buttons (decisions 0287): a profile keeps them per screen, `ViewButton
//! <screen>.<slot>` (a bare slot is the line's screen), and a button holds any object by kind and
//! GUID. A tap on a held button is written out as that object's own line before rights, Oops and
//! the journal (`View 3`, `Sequence 4` = Select, `DataPool 2 Group 1`); a tap on an empty one only
//! says so. `Assign <Object> <n> At ViewButton <s>.<n>` puts an object on a button, whatever its
//! kind; Store on an empty button stores this screen as a new view and assigns it.
//!
//! Owns:          the button address, what a button holds, the tap's line, Assign At ViewButton,
//!                Store / Label / Delete ViewButton, the bar's rows.
//! Doesn't own:   storing and recalling views (`views`), the objects' default verbs (their rows).
//! Entry points:  tap_line, assign_intercept, cmd_viewbutton, cmd_store_viewbutton,
//!                cmd_label_viewbutton, cmd_delete_viewbutton, held, bar, address.
//! Decisions:     docs/decisions/0287-view-bar-any-object-pool-windows.

use lumacore_tree::{ButtonKind, ShowData, UserProfile, ViewButton};
use uuid::Uuid;

use super::{parse, reply, unquote};
use crate::AppState;
use crate::DispatchResult;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What a button holds, found by its GUID: the object's kind, number and name, and the DataPool
/// it lives in (None for a view, which lives on the profile).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub kind: ButtonKind,
    pub no: u32,
    pub datapool: Option<u32>,
    pub name: String,
    pub guid: Uuid,
}

/// One button of a screen's bar (the `viewbar` feed): its slot and what it holds; `held` is None
/// when the object it links is gone (the link is kept, decisions §21).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarButton {
    pub slot: u32,
    pub kind: ButtonKind,
    pub held: Option<Held>,
}

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// The tap (`Hook::Rewrite`): `ViewButton <s>.<n>` on a held button is the line of what it
/// holds, so the object's own row runs it with its rights and Oops. A view takes the flags after
/// the address (`/Screen 2`); any other word leaves the line as typed. An empty or broken button is left to `cmd_viewbutton`.
pub fn tap_line(state: &AppState, line: &str) -> Option<Result<String, String>> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let (first, rest) = words.split_first()?;
    if !crate::objects::find(first).is_some_and(|o| std::ptr::eq(o, &super::VIEW_BUTTON)) {
        return None;
    }
    let (addr, flags) = rest.split_first()?;
    let (screen, slot) = address(state, addr).ok()?;
    let held = held_on(state, screen, slot)?;
    let lum = object_line(state, &held);
    match (held.kind, flags.first()) {
        (_, None) => Some(Ok(lum)),
        (ButtonKind::View, Some(f)) if f.starts_with('/') => Some(Ok(format!("{lum} {}", flags.join(" ")))),
        _ => None,
    }
}

/// `Assign <Object> <n> [At] ViewButton <s>.<n>` (`Hook::Intercept`): any object onto a button,
/// `Assign View 0 At ViewButton <s>.<n>` clears it. The line's rights are the ones dispatch
/// checks for it (the object's Assign).
pub fn assign_intercept(state: &mut AppState, tokens: &[String], _line: &str) -> Option<DispatchResult> {
    let is_assign = matches!(
        crate::commands::registry::resolve(tokens.first()?),
        Ok(crate::commands::registry::Word::Command(c)) if c.object_verb == Some(crate::objects::Verb::Assign)
    );
    if !is_assign {
        return None;
    }
    let at = tokens
        .iter()
        .position(|t| crate::objects::find(t).is_some_and(|o| std::ptr::eq(o, &super::VIEW_BUTTON)))?;
    let args: Vec<&str> = tokens[1..].iter().map(String::as_str).collect();
    if let Err(refused) = crate::rights::check_statement(state, &tokens[0], &args) {
        return Some(DispatchResult::Output(refused.into()));
    }
    let mut object: Vec<&str> = tokens[1..at].iter().map(String::as_str).collect();
    if object.last().is_some_and(|w| w.eq_ignore_ascii_case("at")) {
        object.pop();
    }
    Some(crate::objects::out(reply(assign(state, &object, tokens.get(at + 1)))))
}

fn assign(state: &mut AppState, object: &[&str], addr: Option<&String>) -> Result<String, String> {
    let usage = "Usage: Assign <Object> <n> At ViewButton <screen>.<n> (View, Group, Sequence, Macro)";
    let (screen, slot) = address(state, addr.ok_or(usage)?)?;
    let cleared = matches!(object, [kind, "0"] if crate::objects::find(kind).is_some_and(|o| std::ptr::eq(o, &super::VIEW)));
    if cleared {
        clear(state, screen, slot)?;
        return Ok(format!("ViewButton {screen}.{slot} cleared"));
    }
    let held = object_of(state, object).map_err(|e| if e.is_empty() { usage.to_string() } else { e })?;
    let button = ViewButton { kind: held.kind, guid: held.guid };
    let profile = state.current_user_profile_mut().ok_or("No user profile")?;
    profile.content.view_buttons.entry(screen).or_default().insert(slot, button);
    Ok(format!("ViewButton {screen}.{slot} → {} \"{}\"", shown(&held), held.name))
}

/// `ViewButton <s>.<n>` that the tap didn't write out: an empty button only says so (it never
/// clears the screen), a button whose object is gone says which.
pub fn cmd_viewbutton(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let (screen, slot) = address(state, p.rest.first().ok_or("Expected a view button number")?)?;
        Err(match button(state, screen, slot) {
            Some(b) => format!("ViewButton {screen}.{slot} holds a {} that is gone — Delete ViewButton {screen}.{slot} clears it", b.kind.keyword()),
            None => format!(
                "ViewButton {screen}.{slot} is empty — Store ViewButton {screen}.{slot} stores this screen there, \
                 Assign <Object> <n> At ViewButton {screen}.{slot} puts an object on it"
            ),
        })
    }))
}

/// `Store ViewButton <s>.<n> ["name"] [/Screen "a,b"]` — into the view it holds, or (empty) into
/// the first free view, assigned to the button. A button holding another kind is refused.
pub fn cmd_store_viewbutton(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let (screen, slot) = address(state, p.rest.first().ok_or("Expected a view button number")?)?;
        let name = unquote(&p.rest[1..]);
        match held_on(state, screen, slot) {
            Some(h) if h.kind == ButtonKind::View => return super::store_view(state, h.no, name, &p.screens),
            Some(h) => {
                return Err(format!("ViewButton {screen}.{slot} holds {} — Delete ViewButton {screen}.{slot} first", shown(&h)));
            }
            None => {}
        }
        let profile = state.current_user_profile_mut().ok_or("No user profile")?;
        let no = (1..).find(|n| !profile.content.views.contains_key(n)).unwrap_or(1);
        let msg = super::store_view(state, no, name, &p.screens)?;
        let profile = state.current_user_profile_mut().ok_or("No user profile")?;
        let guid = profile.content.views[&no].meta.identity.guid;
        profile.content.view_buttons.entry(screen).or_default().insert(slot, ViewButton { kind: ButtonKind::View, guid });
        Ok(format!("{msg} → ViewButton {screen}.{slot}"))
    }))
}

/// `Label ViewButton <s>.<n> "name" | /Clear` — label the view it holds; without a name it asks.
/// Another kind is labelled with its own word.
pub fn cmd_label_viewbutton(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let (screen, slot) = address(state, p.rest.first().ok_or("Expected a view button number")?)?;
        let h = held_on(state, screen, slot).ok_or_else(|| format!("ViewButton {screen}.{slot} is empty"))?;
        if h.kind != ButtonKind::View {
            return Err(format!("ViewButton {screen}.{slot} holds {} — Label {} \"name\" labels it", shown(&h), object_line(state, &h)));
        }
        match (unquote(&p.rest[1..]), p.clear) {
            (Some(name), _) => super::label_view(state, h.no, name),
            (None, true) => super::label_view(state, h.no, format!("View {}", h.no)),
            (None, false) => super::label_ask(state, &format!("ViewButton {screen}.{slot}"), h.no),
        }
    }))
}

/// `Delete ViewButton <s>.<n>` — clear the button; what it held stays.
pub fn cmd_delete_viewbutton(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let (screen, slot) = address(state, p.rest.first().ok_or("Expected a view button number")?)?;
        clear(state, screen, slot)?;
        Ok(format!("ViewButton {screen}.{slot} cleared"))
    }))
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `<screen>.<slot>` or a bare `<slot>` on the line's screen (the profile's number of the screen
/// the line came from, else 1).
pub fn address(state: &AppState, token: &str) -> Result<(u32, u32), String> {
    let number = |t: &str| t.parse::<u32>().ok().filter(|n| *n > 0);
    let parsed = match token.split_once('.') {
        Some((s, n)) => number(s).zip(number(n)),
        None => number(token).map(|n| (line_screen(state), n)),
    };
    parsed.ok_or_else(|| "Expected a view button: <screen>.<n> or <n>".to_string())
}

/// The profile's number of the screen the line came from (the user's own), else 1.
pub fn line_screen(state: &AppState) -> u32 {
    let user = state.effective_user_name();
    state
        .current_live_screen
        .and_then(|n| {
            let store = state.root.runtime.live_screens.read().ok()?;
            store.get(n).filter(|s| s.user.eq_ignore_ascii_case(user)).map(|s| s.screen)
        })
        .unwrap_or(1)
}

fn button(state: &AppState, screen: u32, slot: u32) -> Option<ViewButton> {
    let profile = state.root.show_data.profile_for_user(state.effective_user_name())?;
    profile.content.view_buttons.get(&screen)?.get(&slot).copied()
}

fn held_on(state: &AppState, screen: u32, slot: u32) -> Option<Held> {
    let profile = state.root.show_data.profile_for_user(state.effective_user_name())?;
    held(&state.root.show_data, profile, button(state, screen, slot)?)
}

fn clear(state: &mut AppState, screen: u32, slot: u32) -> Result<(), String> {
    let profile = state.current_user_profile_mut().ok_or("No user profile")?;
    let bar = profile.content.view_buttons.get_mut(&screen);
    bar.and_then(|b| b.remove(&slot)).ok_or_else(|| format!("ViewButton {screen}.{slot} is empty"))?;
    if profile.content.view_buttons.get(&screen).is_some_and(|b| b.is_empty()) {
        profile.content.view_buttons.remove(&screen);
    }
    Ok(())
}

/// What `button` holds, found by its GUID on the profile (a view) or in the DataPools.
pub fn held(show: &ShowData, profile: &UserProfile, button: ViewButton) -> Option<Held> {
    let g = button.guid;
    let found = |no: u32, datapool: Option<u32>, meta: &lumacore_base::NodeMeta| Held {
        kind: button.kind,
        no,
        datapool,
        name: meta.name().map_or_else(|| format!("{} {no}", button.kind.keyword()), str::to_string),
        guid: g,
    };
    if button.kind == ButtonKind::View {
        return profile.content.views.iter().find(|(_, v)| v.meta.identity.guid == g).map(|(n, v)| found(*n, None, &v.meta));
    }
    show.datapools.iter().find_map(|(d, pool)| match button.kind {
        ButtonKind::Group => pool.groups.iter().find(|(_, x)| x.meta.identity.guid == g).map(|(n, x)| found(*n, Some(*d), &x.meta)),
        ButtonKind::Sequence => pool.sequences.iter().find(|(_, x)| x.meta.identity.guid == g).map(|(n, x)| found(*n, Some(*d), &x.meta)),
        ButtonKind::Macro => pool.macros.iter().find(|(_, x)| x.meta.identity.guid == g).map(|(n, x)| found(*n, Some(*d), &x.meta)),
        ButtonKind::View => None,
    })
}

/// The line a tap on `held` runs: its object, owner first when it is in another DataPool than
/// the user's (`DataPool 2 Group 1`); the object's default verb does the rest.
pub fn object_line(state: &AppState, held: &Held) -> String {
    let mine = crate::pools::user_datapool(&state.root.show_data, state.effective_user_name());
    match held.datapool {
        Some(d) if d != mine => format!("{} {d} {} {}", crate::objects::DATAPOOL.keyword, held.kind.keyword(), held.no),
        _ => format!("{} {}", held.kind.keyword(), held.no),
    }
}

/// The object an Assign names (`View 3`, `Sequence 4`, `DataPool 2 Group 1`). An empty error is
/// "not a form this takes" (the caller says the usage).
fn object_of(state: &AppState, words: &[&str]) -> Result<Held, String> {
    let (last_obj, kind) = words
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, w)| crate::objects::find(w).and_then(kind_of).map(|k| (i, k)))
        .ok_or_else(String::new)?;
    let [no] = &words[last_obj + 1..] else {
        return Err(format!("Name one {} (Assign {} <n> At ViewButton <s>.<n>)", kind.keyword(), kind.keyword()));
    };
    let no: u32 = no.parse().map_err(|_| format!("Expected a {} number", kind.keyword()))?;
    let owners = &words[..last_obj];
    let profile = state.root.show_data.profile_for_user(state.effective_user_name()).ok_or("No user profile")?;
    let datapool = match owners {
        [] => crate::pools::user_datapool(&state.root.show_data, state.effective_user_name()),
        _ if kind == ButtonKind::View => return Err(String::new()),
        _ => {
            let pool = crate::resolve::words(state, &crate::objects::DATAPOOL, &owners[1..])?;
            match pool.members.as_slice() {
                [one] if crate::objects::find(owners[0]).is_some_and(|o| std::ptr::eq(o, &crate::objects::DATAPOOL)) => one.no,
                _ => return Err(String::new()),
            }
        }
    };
    let guid = match kind {
        ButtonKind::View => profile.content.views.get(&no).map(|v| v.meta.identity.guid),
        _ => {
            let pool = state.root.show_data.datapools.get(&datapool);
            pool.and_then(|p| match kind {
                ButtonKind::Group => p.groups.get(&no).map(|x| x.meta.identity.guid),
                ButtonKind::Sequence => p.sequences.get(&no).map(|x| x.meta.identity.guid),
                ButtonKind::Macro => p.macros.get(&no).map(|x| x.meta.identity.guid),
                ButtonKind::View => None,
            })
        }
    };
    let guid = guid.ok_or_else(|| format!("{} {no} is empty", kind.keyword()))?;
    held(&state.root.show_data, profile, ViewButton { kind, guid }).ok_or_else(|| format!("{} {no} is empty", kind.keyword()))
}

/// The button kind of an object row (by the row itself, never its name).
pub fn kind_of(object: &'static crate::objects::ObjectDef) -> Option<ButtonKind> {
    [
        (&super::VIEW, ButtonKind::View),
        (&crate::commands::groups::GROUP, ButtonKind::Group),
        (&crate::objects::SEQUENCE, ButtonKind::Sequence),
        (&crate::objects::MACRO, ButtonKind::Macro),
    ]
    .into_iter()
    .find(|(o, _)| std::ptr::eq(*o, object))
    .map(|(_, k)| k)
}

/// The views a profile's buttons hold, as `screen.slot` (`List View`).
pub fn slots_of_view(profile: &UserProfile, view: Uuid) -> Vec<String> {
    profile
        .content
        .view_buttons
        .iter()
        .flat_map(|(s, bar)| bar.iter().filter(move |(_, b)| b.guid == view).map(move |(n, _)| format!("{s}.{n}")))
        .collect()
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// One screen's bar of a profile, in slot order (the `viewbar` feed and the TUI).
pub fn bar(show: &ShowData, profile: &UserProfile, screen: u32) -> Vec<BarButton> {
    let Some(buttons) = profile.content.view_buttons.get(&screen) else {
        return Vec::new();
    };
    buttons.iter().map(|(slot, b)| BarButton { slot: *slot, kind: b.kind, held: held(show, profile, *b) }).collect()
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

fn shown(h: &Held) -> String {
    match h.datapool {
        Some(d) => format!("{} {} (DataPool {d})", h.kind.keyword(), h.no),
        None => format!("{} {}", h.kind.keyword(), h.no),
    }
}

#[cfg(test)]
mod tests;
