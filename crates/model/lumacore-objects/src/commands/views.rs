//! Stored views on the user (a View pool + ViewButtons).
//!
//! A view is a copy of one or more screens (screens) — their layouts and
//! windows — stored in `ShowData.UserProfiles.<user>.views`. Store copies the
//! selected screens into the view, always overwriting (consoles store views with a
//! fixed `/Overwrite`); recall copies them back, with no live link:
//! - a single-screen view lands on the screen you recall it from (or `/Screen`)
//! - a multi-screen view lands on the screens it was stored from
//!
//! ```text
//! Store View 3 "Programming" [/Screen "1,2"]   store screens 1 and 2 into view 3
//! View 3 [/Screen 2]                           recall view 3
//! View                                         list views and view buttons
//! Label View 3 "Busking"                       rename without touching contents
//! Delete View 3                                delete (and clear its buttons)
//! Assign View 3 At ViewButton 1.5              put view 3 on screen 1's button 5
//! ```
//!
//! View buttons hold any object, per screen (`buttons`, decisions 0287).
//!
//! Without `/Screen`, the screen the command came from is used (the
//! screen with focus), else the user's first screen. `/Overwrite` is
//! accepted for compatibility; storing always overwrites.

use crate::commands::flags as F;
use std::collections::BTreeMap;

use lumacore_tree::{ScreenContent, StoredView};

pub mod buttons;
pub mod pool;
pub use buttons::{cmd_delete_viewbutton, cmd_label_viewbutton, cmd_store_viewbutton, cmd_viewbutton};

use crate::AppState;
use crate::errors::render_code;
use crate::text::render_success;

struct Parsed {
    /// `/Screen "1,2"` — empty when not given.
    screens: Vec<u32>,
    /// `/Clear` (Label: back to the default name).
    clear: bool,
    rest: Vec<String>,
}

/// Split out `/Screen 2`, `/Screen "1,2"`, `/Screen=1,2` (also `/s`) and
/// `/Overwrite` from the positional args.
fn parse(args: &[&str]) -> Result<Parsed, String> {
    let mut screens = Vec::new();
    if F::has(args, &F::SCREEN) {
        let value = F::value(args, &F::SCREEN)
            .ok_or("/Screen: choose at least one screen, e.g. /Screen \"1,2\"")?;
        // Empty parts are skipped (the Store View dialog's screen toggles each add "n,").
        for part in value.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let n = part.parse::<u32>().map_err(|_| format!("/Screen: '{part}' is not a screen number"))?;
            if !screens.contains(&n) {
                screens.push(n);
            }
        }
        if screens.is_empty() {
            return Err("/Screen: choose at least one screen, e.g. /Screen \"1,2\"".into());
        }
    }
    // `/Overwrite` is accepted; storing views always overwrites.
    let clear = F::has(args, &F::CLEAR);
    let rest = F::positional(args, &[&F::SCREEN]).into_iter().map(str::to_string).collect();
    Ok(Parsed { screens, clear, rest })
}

fn view_number(token: Option<&String>) -> Result<u32, String> {
    token
        .and_then(|t| t.parse::<u32>().ok())
        .filter(|n| *n > 0)
        .ok_or_else(|| "Expected a view number".to_string())
}

fn unquote(parts: &[String]) -> Option<String> {
    let name = parts.join(" ").trim().trim_matches('"').trim().to_string();
    (!name.is_empty()).then_some(name)
}

fn reply(result: Result<String, String>) -> String {
    result.map_or_else(|e| render_code(&e, "E-CMD-007"), |m| render_success(&m))
}

fn list(nos: impl IntoIterator<Item = u32>) -> String {
    nos.into_iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// The screens to act on: the given ones (each must be the user's), else the
/// screen the command came from, else the user's first screen.
pub fn target_screens(
    state: &AppState,
    user: &str,
    screens: &[u32],
) -> Result<Vec<u32>, String> {
    let store = state
        .root
        .runtime
        .live_screens
        .read()
        .map_err(|e| e.to_string())?;
    let own = |n: u32| {
        store
            .get(n)
            .is_some_and(|s| s.user.eq_ignore_ascii_case(user))
    };
    if !screens.is_empty() {
        return match screens.iter().find(|n| !own(**n)) {
            Some(n) => Err(format!("No screen {n} for {user}")),
            None => Ok(screens.to_vec()),
        };
    }
    state
        .current_live_screen
        .filter(|n| own(*n))
        .or_else(|| store.for_user(user).map(|s| s.no).next())
        .map(|n| vec![n])
        .ok_or_else(|| {
            format!("No screen for {user} — open the UI first, or use /Screen <n>")
        })
}

/// Tell the view bars (`views_changed`) that profiles changed: those with
/// these GUIDs, or every profile (None — a show load). Raised from the few
/// places a profile can change — the change path (`change::finish`), a
/// restore (Oops / Redo, replication: `oops::restore_subtrees`) and another
/// show (`AppState::replace_show_data`) — never by a command itself.
/// The profiles the state put back or changed (`AppState::profiles_touched`): their view bars
/// refresh, once.
pub fn follow_touched_profiles(state: &mut AppState) {
    let mut touched = std::mem::take(&mut state.profiles_touched);
    touched.sort();
    touched.dedup();
    if !touched.is_empty() {
        profiles_changed(state, Some(&touched));
    }
}

pub fn profiles_changed(state: &AppState, profiles: Option<&[uuid::Uuid]>) {
    let Ok(store) = state.root.runtime.live_screens.read() else {
        return;
    };
    let show = &state.root.show_data;
    for user in show.users.values() {
        if profiles.is_none_or(|p| p.contains(&user.profile)) {
            store.views_changed(&user.login);
        }
    }
}

/// The Store View dialog ("Store View Options", decisions §30 "12b"):
/// a store tap on a view button (`button`) or a view from a UI client asks
/// the name and which of the user's screens to store — one toggle per
/// screen, this one on. Please runs `Store View|ViewButton <no> "name"
/// /Overwrite /Screen "a,b,"` (storing a view always overwrites).
pub fn store_ask(state: &AppState, button: bool, no: u32) -> Result<crate::dialogs::Ask, String> {
    let user = state.effective_user_name().to_string();
    let profile = state.root.show_data.profile_for_user(&user).ok_or("No user profile")?;
    let keyword = if button { "ViewButton" } else { "View" };
    let view_no = if button {
        let screen = buttons::line_screen(state);
        profile
            .content.view_buttons
            .get(&screen)
            .and_then(|bar| bar.get(&no))
            .and_then(|b| buttons::held(&state.root.show_data, profile, *b))
            .filter(|h| h.kind == lumacore_tree::ButtonKind::View)
            .map(|h| h.no)
    } else {
        Some(no)
    };
    let existing = view_no.and_then(|n| profile.content.views.get(&n));
    let free = (1..).find(|n| !profile.content.views.contains_key(n)).unwrap_or(1);
    let name = existing.map_or_else(|| format!("View {}", view_no.unwrap_or(free)), |v| v.meta.name().unwrap_or_default().to_string());
    let screens: Vec<(u32, String)> = {
        let store = state.root.runtime.live_screens.read().map_err(|e| e.to_string())?;
        store.for_user(&user).map(|s| (s.no, s.meta.name().unwrap_or_default().to_string())).collect()
    };
    if screens.is_empty() {
        return Err(format!("No screen for {user} — open the UI first"));
    }
    let here = state.current_live_screen;
    let mut ask = crate::dialogs::Ask::new(
        format!("Store {keyword} {no}{}", if existing.is_some() { " (overwrite)" } else { "" }),
        "The view's name and the screens it stores.",
    )
    .meta("Store View Options", None)
    .input("name", "Name", name, lumacore_wire::InputKind::Text);
    let mut flags = String::new();
    for (n, label) in &screens {
        let this = here == Some(*n);
        let label = if this { format!("{label} · this screen") } else { label.clone() };
        ask = ask.toggle(&format!("s{n}"), &label, this, &format!("{n},"));
        flags.push_str(&format!("{{s{n}}}"));
    }
    Ok(ask
        .button(
            "Store",
            lumacore_wire::ButtonKind::Please,
            format!("Store {keyword} {no} \"{{name}}\" /Overwrite /Screen \"{flags}\""),
        )
        .cancel("Cancel")
        .flags(format!("the name and /Screen \"a,b\" (Store {keyword} {no} \"name\" /Screen \"1\")")))
}

/// Copy screens into view `no` (overwrites; keeps the old name unless given).
fn store_view(
    state: &mut AppState,
    no: u32,
    name: Option<String>,
    screens: &[u32],
) -> Result<String, String> {
    let user = state.effective_user_name().to_string();
    let targets = target_screens(state, &user, screens)?;
    let mut contents = BTreeMap::new();
    let mut count = 0;
    {
        let store = state
            .root
            .runtime
            .live_screens
            .read()
            .map_err(|e| e.to_string())?;
        for vs in &targets {
            let s = store.get(*vs).ok_or("screen vanished")?;
            count += s.windows.len();
            contents.insert(
                *vs,
                ScreenContent {
                    meta: Default::default(),
                    root: s.root.clone(),
                    windows: s.windows.clone(),
                    encoder_bar: Default::default(),
                },
            );
        }
    }
    let profile = state.current_user_profile_mut().ok_or("No user profile")?;
    let existing = profile.content.views.get(&no).and_then(|v| v.meta.name().map(str::to_string));
    // Storing again keeps the view's (and each screen's) identity.
    let guid = profile.content.views.get(&no).map_or_else(uuid::Uuid::new_v4, |v| v.meta.identity.guid);
    if let Some(old) = profile.content.views.get(&no) {
        for (screen, c) in contents.iter_mut() {
            if let Some(o) = old.screens.get(screen) {
                c.meta.identity.guid = o.meta.identity.guid;
            }
        }
    }
    let name = name
        .or_else(|| existing.clone())
        .unwrap_or_else(|| format!("View {no}"));
    profile.content.views.insert(
        no,
        StoredView {
            meta: lumacore_base::NodeMeta::with_guid(guid, Some(name.clone())),
            screens: contents,
        },
    );
    let verb = if existing.is_some() {
        "Updated"
    } else {
        "Stored"
    };
    let plural = if targets.len() == 1 { "" } else { "s" };
    Ok(format!(
        "{verb} View {no} \"{name}\" from screen{plural} {} ({count} windows)",
        list(targets.iter().copied())
    ))
}

/// Copy view `no` back onto its screen(s).
fn recall_view(state: &mut AppState, no: u32, screens: &[u32]) -> Result<String, String> {
    let user = state.effective_user_name().to_string();
    let view = state
        .current_user_profile_mut()
        .ok_or("No user profile")?
        .content.views
        .get(&no)
        .cloned()
        .ok_or_else(|| format!("View {no} is empty"))?;

    // Single-screen view → the target screen; multi-screen → where it was stored.
    let placements: Vec<(u32, ScreenContent)> = if view.screens.len() == 1 {
        let target = *target_screens(state, &user, screens)?
            .first()
            .ok_or("no target screen")?;
        view.screens.into_values().map(|c| (target, c)).collect()
    } else {
        view.screens.into_iter().collect()
    };

    let mut store = state
        .root
        .runtime
        .live_screens
        .write()
        .map_err(|e| e.to_string())?;
    let mut done = Vec::new();
    let mut skipped = Vec::new();
    for (vs, content) in placements {
        let mine = store
            .get(vs)
            .is_some_and(|s| s.user.eq_ignore_ascii_case(&user));
        if mine
            && store
                .replace_layout(vs, content.root, content.windows)
                .is_ok()
        {
            done.push(vs);
        } else {
            skipped.push(vs);
        }
    }
    if done.is_empty() {
        return Err(format!(
            "View {no}: none of its screens ({}) are open",
            list(skipped)
        ));
    }
    let mut msg = format!("View {no} \"{}\" → screen {}", view.meta.name().unwrap_or_default(), list(done));
    if !skipped.is_empty() {
        msg.push_str(&format!(" (screen {} not open)", list(skipped)));
    }
    Ok(msg)
}

/// `Store View <n> ["name"] [/Screen "a,b"]` — args start after `view`.
pub fn cmd_store_view(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let no = view_number(p.rest.first())?;
        store_view(state, no, unquote(&p.rest[1..]), &p.screens)
    }))
}

/// `View <n> [/Screen <s>]` recalls; bare `View` lists.
pub fn cmd_view(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        return list_views(state);
    }
    reply(parse(args).and_then(|p| {
        let no = view_number(p.rest.first())?;
        recall_view(state, no, &p.screens)
    }))
}

/// `Label View <n> "name" | /Clear` — args start after `view`. Contents are
/// untouched. Without a name it asks (`label_ask`).
pub fn cmd_label_view(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let no = view_number(p.rest.first())?;
        match (unquote(&p.rest[1..]), p.clear) {
            (Some(name), _) => label_view(state, no, name),
            (None, true) => label_view(state, no, format!("View {no}")),
            (None, false) => label_ask(state, &format!("View {no}"), no),
        }
    }))
}

/// `Label View|ViewButton <n>` without a name asks for it (the Label dialog,
/// decisions §30 "12b"): the view's name, Please labels, an empty answer
/// puts the default name back (`/Clear`). `target` is what the line named
/// (`View 3`, `ViewButton 1.4`), `view` the view it shows.
fn label_ask(state: &mut AppState, target: &str, view: u32) -> Result<String, String> {
    let current = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .and_then(|p| p.content.views.get(&view))
        .map(|v| v.meta.name().unwrap_or_default().to_string())
        .ok_or_else(|| format!("View {view} is empty"))?;
    state.asking = Some(
        crate::dialogs::Ask::new(
            format!("Label {target}"),
            format!("A new name for View {view}. Empty puts \"View {view}\" back."),
        )
        .input("name", "Name", current, lumacore_wire::InputKind::Text)
        .button("Please", lumacore_wire::ButtonKind::Please, format!("Label {target} \"{{name}}\""))
        .or_when_empty(format!("Label {target} /Clear"))
        .cancel("Cancel")
        .flags(format!("the name (Label {target} \"name\"), or /Clear")),
    );
    Err(format!("Usage: Label {target} \"name\" | /Clear"))
}

fn label_view(state: &mut AppState, no: u32, name: String) -> Result<String, String> {
    let profile = state.current_user_profile_mut().ok_or("No user profile")?;
    let view = profile
        .content.views
        .get_mut(&no)
        .ok_or_else(|| format!("View {no} is empty"))?;
    view.meta.identity.name = Some(name.clone());
    Ok(format!("View {no} labelled \"{name}\""))
}

/// `Delete View <n>` — args start after `view`.
pub fn cmd_delete_view(state: &mut AppState, args: &[&str]) -> String {
    reply(parse(args).and_then(|p| {
        let no = view_number(p.rest.first())?;
        let profile = state.current_user_profile_mut().ok_or("No user profile")?;
        let view = profile
            .content.views
            .remove(&no)
            .ok_or_else(|| format!("View {no} is empty"))?;
        for bar in profile.content.view_buttons.values_mut() {
            bar.retain(|_, b| b.guid != view.meta.identity.guid);
        }
        profile.content.view_buttons.retain(|_, bar| !bar.is_empty());
        Ok(format!("Deleted View {no} \"{}\"", view.meta.name().unwrap_or_default()))
    }))
}

/// The user's views and the view buttons they sit on (`View`, `List View`).
pub fn list_views(state: &AppState) -> String {
    let Some(profile) = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
    else {
        return render_code("No user profile", "E-CMD-011");
    };
    if profile.content.views.is_empty() {
        return render_success(
            "No stored views. Store View <n> \"name\" stores the current screen.",
        );
    }
    let mut out = String::from("Views:\n");
    for (no, v) in &profile.content.views {
        let buttons = buttons::slots_of_view(profile, v.meta.identity.guid);
        let windows: usize = v.screens.values().map(|c| c.windows.len()).sum();
        out.push_str(&format!(
            "  {no:>3}  {:<24} screen {:<6} {} windows{}\n",
            v.meta.name().unwrap_or_default(),
            list(v.screens.keys().copied()),
            windows,
            if buttons.is_empty() {
                String::new()
            } else {
                format!("  (ViewButton {})", buttons.join(","))
            }
        ));
    }
    out
}

// ── Objects: View and ViewButton (see terminal/objects) ─────────────────────

use crate::objects::{ObjectDef, Verb, filter, out};
use crate::DispatchResult;

fn view_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_view(state, args))
}
fn view_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_store_view(state, args))
}
fn view_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_delete_view(state, args))
}
fn view_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_label_view(state, args))
}
/// `Assign View <n> At ViewButton <s>.<n>` is taken before the words run (`buttons::assign_intercept`);
/// another form says so.
fn view_assign(_state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(render_code("Usage: Assign View <n> At ViewButton <screen>.<n>", "E-CMD-006"))
}
fn view_list(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    out(list_views(state))
}
fn view_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let nos = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| p.content.views.keys().map(u32::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    filter(nos, prefix)
}

/// Export / Import write and read `library/ui/` (decisions §30 "12c").
pub fn files_reach(verb: Verb) -> crate::commands::registry::Reach {
    use crate::commands::registry::Reach;
    if matches!(verb, Verb::Import | Verb::Export) { Reach::Drive } else { Reach::Show }
}

pub static VIEW: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "View",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Call,
        flags: &[F::SCREEN.on(&[Verb::Store, Verb::Call]), F::OVERWRITE.on(&[Verb::Store]), F::CLEAR.on(&[Verb::Store])],
        aliases: &["views"],
        doc: crate::commands::registry::Doc {
            display: "View",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "View [<n>] [/Screen <s>] | Export View <n> [\"file\"] | Import View \"file\" [<n>]",
            description: "Recall a stored view onto a screen (screen). A view is a copy of a \
                          screen's windows, their placement and settings, stored on your user. \
                          Recall copies it back — editing windows afterwards does not change the \
                          stored view until you store it again. Without a number, lists your views.",
            examples: &[
                "View                              -- list stored views and view buttons",
                "View 3                            -- recall view 3 onto your first screen",
                "View 3 /Screen 2                  -- recall onto screen 2",
                "Store View 3 \"Programming\"      -- store the current screen as view 3",
                "Delete View 3                     -- delete view 3",
                "Assign View 3 At ViewButton 1.5   -- put view 3 on screen 1's button 5",
                "Export View 3                     -- library/ui/views/<name>.json",
                "Import View \"Busk\"                -- as your next free view",
            ],
            related: &["store", "delete", "assign"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_station,
        reach: files_reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: Some(&pool::VIEW_POOL),
    verbs: &[
        (Verb::Call, view_call),
        (Verb::Store, view_store),
        (Verb::Delete, view_delete),
        (Verb::Label, view_label),
        (Verb::Assign, view_assign),
        (Verb::List, view_list),
        (Verb::Export, |s, a| out(super::ui_files::export_view(s, a))),
        (Verb::Import, |s, a| out(super::ui_files::import_view(s, a))),
    ],
    delete_ask: None,
    complete: Some(view_complete),
    props: Some(&VIEW_PROPS),
    tree: None,
};

/// Your profile's views by number: their Paint (decisions 0294; the Views pool and the view bar
/// wear it).
static VIEW_PROPS: crate::props::Props = crate::props::Props {
    lists: &[&VIEW_PAINT],
    keyed: Some(crate::props::Keyed {
        rows: |s| s.root.show_data.profile_for_user(s.effective_user_name()).map_or_else(Vec::new, |p| p.content.views.keys().map(u32::to_string).collect()),
        row: |s, w| {
            let n = w.trim_matches('"').parse::<u32>().ok()?;
            s.root.show_data.profile_for_user(s.effective_user_name())?.content.views.contains_key(&n).then(|| n.to_string())
        },
    }),
    label: Some(|_, k| format!("View {k}")),
    ..crate::props::Props::BASE
};
static VIEW_PAINT: [crate::props::Property; 1] = [crate::keyed_paint!(view_meta, view_meta_mut, "View")];

fn view_meta<'a>(s: &'a AppState, k: &str) -> Option<&'a lumacore_base::NodeMeta> {
    s.root.show_data.profile_for_user(s.effective_user_name())?.content.views.get(&k.parse().ok()?).map(|v| &v.meta)
}

fn view_meta_mut<'a>(s: &'a mut AppState, k: &str) -> Option<&'a mut lumacore_base::NodeMeta> {
    let user = s.effective_user_name().to_string();
    s.root.show_data.profile_for_user_mut(&user)?.content.views.get_mut(&k.parse().ok()?).map(|v| &mut v.meta)
}

fn viewbutton_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_viewbutton(state, args))
}
fn viewbutton_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_store_viewbutton(state, args))
}
fn viewbutton_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_delete_viewbutton(state, args))
}
fn viewbutton_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_label_viewbutton(state, args))
}
fn viewbutton_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let nos = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| {
            p.content.view_buttons
                .iter()
                .flat_map(|(s, bar)| bar.keys().map(move |n| format!("{s}.{n}")))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    filter(nos, prefix)
}

pub static VIEW_BUTTON: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "ViewButton",
        lum: crate::objects::LumShape::ids(2),
        default_verb: crate::objects::Verb::Call,
        flags: &[F::SCREEN.on(&[Verb::Store]), F::OVERWRITE.on(&[Verb::Store]), F::CLEAR.on(&[Verb::Store])],
        aliases: &["viewbuttons", "vb"],
        doc: crate::commands::registry::Doc {
            display: "ViewButton",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "ViewButton [<screen>.]<n> [/Screen <s>]",
            description: "A view bar button, per screen of your profile (decisions 0287): it holds any \
                          object — a view, a group, a sequence, a macro — and a tap runs that object's \
                          default verb (View 3, Select Sequence 4, the group's selection). A bare number \
                          is a button of the screen the line comes from. An empty button only says so; \
                          Store on it stores this screen as a new view there. With a pending command, \
                          tapping a view button completes it.",
            examples: &[
                "ViewButton 1.4                      -- run what screen 1's button 4 holds",
                "Assign Sequence 3 At ViewButton 1.6 -- put sequence 3 on it",
                "Store ViewButton 1.4 \"Busking\"    -- store this screen into button 4",
                "Label ViewButton 1.4 \"Prog\"       -- label the view it holds",
                "Delete ViewButton 1.4               -- clear the button (the object stays)",
            ],
            related: &["view", "store", "assign"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_station,
        reach: crate::objects::show_reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Call, viewbutton_call),
        (Verb::Store, viewbutton_store),
        (Verb::Delete, viewbutton_delete),
        (Verb::Label, viewbutton_label),
        (Verb::List, view_list),
    ],
    delete_ask: None,
    complete: Some(viewbutton_complete),
    props: None,
    tree: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_screen_lists_and_overwrite() {
        let p = parse(&["3", "\"Prog\"", "/screen", "\"1,2\"", "/overwrite"]).unwrap();
        assert_eq!(p.screens, vec![1, 2]);
        assert_eq!(p.rest, vec!["3", "\"Prog\""]);
        assert_eq!(parse(&["3", "/screen=4"]).unwrap().screens, vec![4]);
        assert!(parse(&["3", "/screen", "x"]).is_err());
        // An undeclared flag never gets here: dispatch refuses it (E-CMD-018, decisions §31).
    }
}
