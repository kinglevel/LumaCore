//! Screen configurations — the live screens of a workspace, kept in the show
//! (on the user profile, with any number of screens).
//!
//! Every screen is one screen `(config, screen)` of its user's profile.
//! Changes to a screen are copied into
//! `UserProfile.screen_configs[config].screens[screen]` by the server loop
//! (`sync`), so save → shutdown → reopen puts every window back; opening a
//! screen or loading a show lays the screens out from there (`saved`,
//! `restore_all`). Which screens a machine has open, and where, is client-local.
//!
//! ```text
//! ScreenConfig                         list this user's screen configs
//! ScreenConfig 2                       switch this client to config 2
//! Store ScreenConfig 2 ["name"]        copy this client's config into 2 (/Overwrite to replace)
//! Label ScreenConfig 2 "Workstation"   rename
//! Delete ScreenConfig 2
//! Store Screen [n]                     add a screen to this client's config
//! Delete Screen n                      remove a screen from it
//! ```

use std::collections::BTreeMap;

use lumacore_tree::runtime::{LayoutNode, Window, default_layout};
use lumacore_tree::{ScreenConfig, ScreenContent};

use crate::AppState;
use crate::errors::render_code;
use crate::text::render_success;
use lumacore_wire::UiEvent;

fn config_name(no: u32) -> String {
    if no == 1 {
        "Default".to_string()
    } else {
        format!("Config {no}")
    }
}

/// Copy changed screens into their users' screen configs. Marks the
/// show unsaved when a saved layout actually changed. Each copy goes through
/// the change path as the screen's user, so it replicates (on a member it
/// goes to the master — decisions §21 "6b") and is a UI Oops step, a Main
/// action (layout; decisions §29 "11c"). The web client saves a drag or a
/// resize once, on release, so each is one step; switching the active tab
/// isn't a step (`only_tabs_moved`).
pub fn sync(state: &mut AppState) {
    let changed = match state.root.runtime.live_screens.write() {
        Ok(mut store) => store.take_changed(),
        Err(_) => return,
    };
    if changed.is_empty() {
        return;
    }
    let sessions: Vec<_> = match state.root.runtime.live_screens.read() {
        Ok(store) => changed
            .iter()
            .filter_map(|no| store.get(*no).cloned())
            .collect(),
        Err(_) => return,
    };
    let acting = state.acting_user.clone();
    for vs in sessions {
        if state.begin_acting_as_existing_user(&vs.user).is_none() {
            continue;
        }
        let undo = if only_tabs_moved(state, &vs) {
            crate::commands::registry::Undo::Untracked
        } else {
            crate::commands::registry::main(lumacore_tree::OopsCategory::UI)
        };
        crate::commands::record_change(
            state,
            &format!("Layout of Screen {} (ScreenConfig {})", vs.screen, vs.config),
            undo,
            |state| write_screen(state, &vs),
        );
    }
    state.acting_user = acting;
}

/// The live screen differs from its saved layout only in which tab each
/// pane shows (not an Oops step).
fn only_tabs_moved(state: &AppState, vs: &lumacore_tree::runtime::Screen) -> bool {
    let Some((root, windows)) = saved(&state.root.show_data, &vs.user, vs.config, vs.screen) else {
        return false;
    };
    serde_json::to_value(&windows).ok() == serde_json::to_value(&vs.windows).ok()
        && serde_json::to_value(root.without_active()).ok()
            == serde_json::to_value(vs.root.without_active()).ok()
}

fn write_screen(state: &mut AppState, vs: &lumacore_tree::runtime::Screen) {
    let Some(profile) = state.root.show_data.profile_for_user_mut(&vs.user) else {
        return;
    };
    let config = profile
        .content.screen_configs
        .entry(vs.config)
        .or_insert_with(|| ScreenConfig {
            meta: lumacore_base::NodeMeta::new(Some(config_name(vs.config))),
            screens: BTreeMap::new(),
        });
    let content = ScreenContent {
        // The screen keeps its identity while its layout changes.
        meta: config.screens.get(&vs.screen).map(|s| s.meta.clone()).unwrap_or_default(),
        root: vs.root.clone(),
        windows: vs.windows.clone(),
        // The bar setting is the saved screen's own; the live layout never carries it.
        encoder_bar: config.screens.get(&vs.screen).map(|s| s.encoder_bar.clone()).unwrap_or_default(),
    };
    let same = config
        .screens
        .get(&vs.screen)
        .is_some_and(|old| serde_json::to_value(old).ok() == serde_json::to_value(&content).ok());
    if !same {
        config.screens.insert(vs.screen, content);
        state.show_dirty = true;
    }
}

/// Each live screen's saved layout (by live screen number), as JSON: what
/// [`follow_profiles`] compares against after a profile op.
pub fn saved_layouts(state: &AppState) -> std::collections::HashMap<u32, Option<serde_json::Value>> {
    let Ok(store) = state.root.runtime.live_screens.read() else {
        return Default::default();
    };
    store
        .all()
        .map(|s| {
            let saved = saved(&state.root.show_data, &s.user, s.config, s.screen)
                .and_then(|(root, windows)| serde_json::to_value((&root, &windows)).ok());
            (s.no, saved)
        })
        .collect()
}

/// On a member, after the master changed a profile: a live screen follows
/// its saved layout only when that saved layout changed (`before` =
/// [`saved_layouts`] before the op) — a shared profile shares its look
/// (Q33), but an unrelated profile op never drops this screen's own
/// unsynced edits (R1-58).
pub fn follow_profiles(state: &mut AppState, before: &std::collections::HashMap<u32, Option<serde_json::Value>>) {
    let sessions: Vec<(u32, String, u32, u32)> = match state.root.runtime.live_screens.read() {
        Ok(store) => store.all().map(|s| (s.no, s.user.clone(), s.config, s.screen)).collect(),
        Err(_) => return,
    };
    let changed: Vec<_> = sessions
        .into_iter()
        .filter_map(|(no, user, config, screen)| {
            let now = saved(&state.root.show_data, &user, config, screen)?;
            let json = serde_json::to_value((&now.0, &now.1)).ok();
            (before.get(&no).cloned().flatten() != json).then_some((no, now))
        })
        .collect();
    if let Ok(mut store) = state.root.runtime.live_screens.write() {
        for (no, (root, windows)) in changed {
            store.restore(no, root, windows);
        }
    }
}

/// The saved layout of `user`'s screen, if the show has one.
pub fn saved(
    show: &lumacore_tree::ShowData,
    user: &str,
    config: u32,
    screen: u32,
) -> Option<(LayoutNode, BTreeMap<u32, Window>)> {
    let content = show
        .profile_for_user(user)?
        .content.screen_configs
        .get(&config)?
        .screens
        .get(&screen)?;
    Some((content.root.clone(), content.windows.clone()))
}

/// The config a user starts with.
pub fn default_config(show: &lumacore_tree::ShowData, user: &str) -> u32 {
    let Some(u) = show.user(user) else {
        return 1;
    };
    show.profile_of(u)
        .and_then(|(_, p)| {
            p.content.screen_configs
                .iter()
                .find(|(_, c)| Some(c.meta.identity.guid) == u.default_screen_config)
                .or_else(|| p.content.screen_configs.iter().next())
                .map(|(&n, _)| n)
        })
        .unwrap_or(1)
}

/// After a show load: lay every live screen out from the loaded show
/// (its saved screen, or the default layout).
pub fn restore_all(state: &mut AppState) {
    let sessions: Vec<(u32, String, u32, u32)> = match state.root.runtime.live_screens.read() {
        Ok(store) => store
            .all()
            .map(|s| (s.no, s.user.clone(), s.config, s.screen))
            .collect(),
        Err(_) => return,
    };
    let layouts: Vec<_> = sessions
        .into_iter()
        .map(|(no, user, config, screen)| {
            (
                no,
                saved(&state.root.show_data, &user, config, screen).unwrap_or_else(default_layout),
            )
        })
        .collect();
    if let Ok(mut store) = state.root.runtime.live_screens.write() {
        for (no, (root, windows)) in layouts {
            store.restore(no, root, windows);
        }
    }
}

// ── Commands ─────────────────────────────────────────────────────────────────

fn reply(result: Result<String, String>) -> String {
    result.map_or_else(|e| render_code(&e, "E-CMD-010"), |m| render_success(&m))
}

fn number(token: Option<&&str>, what: &str) -> Result<u32, String> {
    token
        .and_then(|t| t.parse::<u32>().ok())
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("Expected a {what} number"))
}

fn unquote(parts: &[&str]) -> Option<String> {
    let s = parts.join(" ").trim().trim_matches('"').trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// (user, config) of the screen the command came from.
fn this_config(state: &AppState) -> (String, u32) {
    let user = state.effective_user_name().to_string();
    let config = state
        .current_live_screen
        .and_then(|no| {
            state
                .root
                .runtime
                .live_screens
                .read()
                .ok()?
                .get(no)
                .map(|s| s.config)
        })
        .unwrap_or_else(|| default_config(&state.root.show_data, &user));
    (user, config)
}

/// The user's screen configs (`ScreenConfig`, `List ScreenConfig`).
pub fn list_screenconfigs(state: &AppState) -> String {
    let (user, current) = this_config(state);
    let Some(profile) = state.root.show_data.profile_for_user(&user) else {
        return render_code("No user profile", "E-CMD-011");
    };
    if profile.content.screen_configs.is_empty() {
        return render_success("No screen configs yet — screens are saved as you use them.");
    }
    let mut out = String::from("Screen configs:\n");
    for (no, c) in &profile.content.screen_configs {
        let mark = if *no == current {
            "  ◀ this client"
        } else {
            ""
        };
        out.push_str(&format!(
            "  {no:>3}  {:<20} {} screen(s){mark}\n",
            c.meta.name().unwrap_or_default(),
            c.screens.len().max(1)
        ));
    }
    out
}

/// `ScreenConfig` lists, `ScreenConfig n` switches this client.
pub fn cmd_screenconfig(state: &mut AppState, args: &[&str]) -> String {
    sync(state);
    let (user, _) = this_config(state);
    if args.is_empty() {
        return list_screenconfigs(state);
    }
    reply((|| {
        let no = number(args.first(), "screen config")?;
        let exists = state
            .root
            .show_data
            .profile_for_user(&user)
            .is_some_and(|p| p.content.screen_configs.contains_key(&no));
        if !exists && no != 1 {
            return Err(format!(
                "No screen config {no} — Store ScreenConfig {no} creates it"
            ));
        }
        if state.current_live_screen.is_none() {
            return Err("ScreenConfig switches a UI client; this session has no screens".into());
        }
        state.ui_requests.push(UiEvent::ScreenConfig { no });
        Ok(format!("Screen config {no}"))
    })())
}

/// `Store ScreenConfig n ["name"] [/Overwrite]` — args after `screenconfig`.
pub fn cmd_store_screenconfig(state: &mut AppState, args: &[&str]) -> String {
    sync(state);
    reply((|| {
        let overwrite = crate::commands::flags::has(args, &crate::commands::flags::OVERWRITE);
        let args: Vec<&str> = args
            .iter()
            .copied()
            .filter(|a| !a.starts_with('/'))
            .collect();
        let no = number(args.first(), "screen config")?;
        let name = unquote(&args[1..]);
        let (user, current) = this_config(state);
        let profile = state
            .root
            .show_data
            .profile_for_user_mut(&user)
            .ok_or("No user profile")?;
        if profile.content.screen_configs.contains_key(&no) && !overwrite {
            return Err(format!(
                "Screen config {no} exists. Use /Overwrite to replace it"
            ));
        }
        let screens = profile
            .content.screen_configs
            .get(&current)
            .map(|c| c.screens.clone())
            .unwrap_or_default();
        // A copy is a new object: new GUIDs.
        let screens: BTreeMap<u32, ScreenContent> = screens
            .into_iter()
            .map(|(n, s)| {
                (
                    n,
                    ScreenContent {
                        meta: Default::default(),
                        ..s
                    },
                )
            })
            .collect();
        let count = screens.len();
        let name = name.unwrap_or_else(|| config_name(no));
        profile.content.screen_configs.insert(
            no,
            ScreenConfig {
                meta: lumacore_base::NodeMeta::new(Some(name.clone())),
                screens,
            },
        );
        state.show_dirty = true;
        Ok(format!(
            "Screen config {no} \"{name}\" stored ({count} screen(s) from config {current})"
        ))
    })())
}

/// `Delete ScreenConfig n` — not the one this client uses.
pub fn cmd_delete_screenconfig(state: &mut AppState, args: &[&str]) -> String {
    reply((|| {
        let no = number(args.first(), "screen config")?;
        let (user, current) = this_config(state);
        if no == current {
            return Err(format!("Screen config {no} is in use here — switch first"));
        }
        let profile = state
            .root
            .show_data
            .profile_for_user_mut(&user)
            .ok_or("No user profile")?;
        let config = profile
            .content.screen_configs
            .remove(&no)
            .ok_or(format!("No screen config {no}"))?;
        state.show_dirty = true;
        Ok(format!("Deleted screen config {no} \"{}\"", config.meta.name().unwrap_or_default()))
    })())
}

/// `Store Screen [n]` — add a screen (default layout) to this client's config.
pub fn cmd_store_screen(state: &mut AppState, args: &[&str]) -> String {
    sync(state);
    reply((|| {
        let (user, current) = this_config(state);
        let profile = state
            .root
            .show_data
            .profile_for_user_mut(&user)
            .ok_or("No user profile")?;
        let config = profile
            .content.screen_configs
            .entry(current)
            .or_insert_with(|| ScreenConfig {
                meta: lumacore_base::NodeMeta::new(Some(config_name(current))),
                screens: BTreeMap::new(),
            });
        let no = match args.first() {
            Some(t) => number(Some(t), "screen")?,
            None => (1..).find(|n| !config.screens.contains_key(n)).unwrap_or(1),
        };
        if config.screens.contains_key(&no) {
            return Err(format!("Screen {no} exists in config {current}"));
        }
        let (root, windows) = default_layout();
        config.screens.insert(
            no,
            ScreenContent {
                meta: Default::default(),
                root,
                windows,
                encoder_bar: Default::default(),
            },
        );
        state.show_dirty = true;
        Ok(format!(
            "Screen {no} added to config {current} ({} screens)",
            config.screens.len()
        ))
    })())
}

/// `Delete Screen n` — remove a screen from this client's config.
pub fn cmd_delete_screen(state: &mut AppState, args: &[&str]) -> String {
    reply((|| {
        let no = number(args.first(), "screen")?;
        let (user, current) = this_config(state);
        let profile = state
            .root
            .show_data
            .profile_for_user_mut(&user)
            .ok_or("No user profile")?;
        let config = profile
            .content.screen_configs
            .get_mut(&current)
            .ok_or(format!("No screen config {current}"))?;
        if config.screens.len() <= 1 {
            return Err("A screen config keeps at least one screen".into());
        }
        config
            .screens
            .remove(&no)
            .ok_or(format!("No screen {no} in config {current}"))?;
        state.show_dirty = true;
        Ok(format!("Screen {no} removed from config {current}"))
    })())
}

// ── Objects: ScreenConfig and Screen (see terminal/objects) ─────────────────

use crate::objects::{ObjectDef, Verb, filter, out};
use crate::DispatchResult;

pub fn cmd_set_screen(state: &mut AppState, args: &[&str]) -> String {
    use lumacore_tree::runtime::{EncoderLayer, EncoderMode};
    let usage = || {
        render_code(
            "Usage: Set Screen [n] EncoderBar On|Off|Toggle|\"layout\" | EncoderMode Attributes|CueTimingVertical|CueTimingHorizontal|Masters|Effects | EncoderGroup <n>|Next|Previous | EncoderPage <n>|Next|Previous | EncoderLayer Value|Fade|Delay|Next",
            "E-CMD-006",
        )
    };
    let user = state.effective_user_name().to_string();
    let Some(current) = state.current_live_screen else {
        return render_code("Set Screen needs a screen: run it from a screen of the UI.", "E-CMD-010");
    };
    let (numbered, rest) = match args.first().and_then(|a| a.parse::<u32>().ok()) {
        Some(n) => (Some(n), &args[1..]),
        None => (None, args),
    };
    let (config, own_screen) = {
        let store = state.root.runtime.live_screens.read().expect("screens lock");
        match store.get(current) {
            Some(s) => (s.config, s.screen),
            None => return render_code("This screen is gone.", "E-CMD-010"),
        }
    };
    let screen_no = numbered.unwrap_or(own_screen);
    let [prop, value @ ..] = rest else { return usage() };
    let value: Vec<&str> = value.to_vec();
    let one = value.first().copied().unwrap_or("");
    // EncoderBar is On / Off / Toggle, EncoderLayout the layout (both on the screen config).
    if prop.eq_ignore_ascii_case("encoderbar") || prop.eq_ignore_ascii_case("encoderlayout") {
        // Saved on the profile's screen config.
        let Some(profile) = state.current_user_profile_mut() else {
            return render_code("No user profile.", "E-CMD-011");
        };
        let layouts: Vec<(u32, uuid::Uuid, String)> =
            profile.content.attributes.encoder_bars.iter().map(|(n, b)| (*n, b.meta.identity.guid, b.meta.name().unwrap_or_default().to_string())).collect();
        let Some(content) = profile.content.screen_configs.get_mut(&config).and_then(|c| c.screens.get_mut(&screen_no)) else {
            return render_code(format!("Screen {screen_no} isn't in your screen config {config}."), "E-CMD-007");
        };
        let shown = content.encoder_bar.shown(screen_no);
        let reply = match one.trim_matches('"') {
            w if w.eq_ignore_ascii_case("on") => {
                content.encoder_bar.on = Some(true);
                format!("Screen {screen_no}: encoder bar on")
            }
            w if w.eq_ignore_ascii_case("off") => {
                content.encoder_bar.on = Some(false);
                format!("Screen {screen_no}: encoder bar off")
            }
            w if w.eq_ignore_ascii_case("toggle") => {
                content.encoder_bar.on = Some(!shown);
                format!("Screen {screen_no}: encoder bar {}", if shown { "off" } else { "on" })
            }
            "" => return usage(),
            w => {
                let found = layouts
                    .iter()
                    .find(|(id, _, name)| id.to_string() == w || name.eq_ignore_ascii_case(w));
                let Some((_, guid, name)) = found else {
                    return render_code(format!("No encoder bar layout \"{w}\" (Edit EncoderBar lists yours)."), "E-CMD-007");
                };
                content.encoder_bar.layout = Some(*guid);
                content.encoder_bar.on = Some(true);
                format!("Screen {screen_no}: encoder bar \"{name}\"")
            }
        };
        state.show_dirty = true;
        return render_success(&reply);
    }
    // What the live screen's bar shows (runtime).
    let target = if numbered.is_some() {
        let store = state.root.runtime.live_screens.read().expect("screens lock");
        match store.find_screen(&user, config, screen_no) {
            Some(n) => n,
            None => return render_code(format!("Screen {screen_no} isn't open."), "E-CMD-010"),
        }
    } else {
        current
    };
    // `Next` / `Previous` step (a `+` would read as a row range: `Set Screen 1 + 2 …`).
    let step = |now: u32| -> Result<u32, String> {
        match value.as_slice() {
            [w] if w.eq_ignore_ascii_case("next") => Ok(now.saturating_add(1)),
            [w] if w.eq_ignore_ascii_case("previous") => Ok(now.saturating_sub(1).max(1)),
            [n] => n
                .parse::<u32>()
                .ok()
                .filter(|n| *n >= 1)
                .ok_or_else(|| render_code(format!("'{n}' is not a number (1 or more), Next or Previous."), "E-CMD-008")),
            _ => Err(usage()),
        }
    };
    let now = {
        let store = state.root.runtime.live_screens.read().expect("screens lock");
        store.get(target).map(|s| s.encoder.clone()).unwrap_or_default()
    };
    let mut next = now.clone();
    let reply = match prop.to_ascii_lowercase().as_str() {
        "encodermode" => match EncoderMode::parse(one) {
            Some(m) => {
                next.mode = m;
                // Cue timing opens on the sequence's current cue (group 0 = follow it).
                next.group = if matches!(m, EncoderMode::CueTimingVertical | EncoderMode::CueTimingHorizontal) { 0 } else { 1 };
                next.page = 1;
                format!("encoder mode {}", m.word())
            }
            None => {
                let words: Vec<&str> = EncoderMode::ALL.iter().map(|m| m.word()).collect();
                return render_code(format!("'{one}': a mode is {}.", words.join(", ")), "E-CMD-008");
            }
        },
        "encodergroup" => match step(now.group.max(1)) {
            Ok(n) => {
                next.group = n;
                next.page = 1;
                format!("encoder group {n}")
            }
            Err(e) => return e,
        },
        "encoderpage" => match step(now.page) {
            Ok(n) => {
                next.page = n;
                format!("encoder page {n}")
            }
            Err(e) => return e,
        },
        "encoderlayer" => {
            let layer = if one.eq_ignore_ascii_case("next") {
                let i = EncoderLayer::ALL.iter().position(|l| *l == now.layer).unwrap_or(0);
                Some(EncoderLayer::ALL[(i + 1) % EncoderLayer::ALL.len()])
            } else {
                EncoderLayer::parse(one)
            };
            match layer {
                Some(l) => {
                    next.layer = l;
                    format!("encoder layer {}", l.word())
                }
                None => return render_code(format!("'{one}': a layer is Value, Fade, Delay or Next."), "E-CMD-008"),
            }
        }
        _ => return usage(),
    };
    let set = state.root.runtime.live_screens.write().expect("screens lock").set_encoder(target, |e| *e = next);
    match set {
        Ok(_) => render_success(&format!("Screen {screen_no}: {reply}")),
        Err(e) => render_code(e, "E-CMD-010"),
    }
}

fn screen_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_store_screen(state, args))
}
fn screen_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(cmd_delete_screen(state, args))
}
fn screen_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let (user, config) = this_config(state);
    let nos = state
        .root
        .show_data
        .profile_for_user(&user)
        .and_then(|p| p.content.screen_configs.get(&config))
        .map(|c| c.screens.keys().map(u32::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    filter(nos, prefix)
}

// ── A screen's properties (decisions §33 58, 64) ─────────────────────────────

/// This client's screen config and the screen it shows.
fn own_screen(state: &AppState) -> Option<(u32, u32)> {
    let store = state.root.runtime.live_screens.read().ok()?;
    let s = store.get(state.current_live_screen?)?;
    Some((s.config, s.screen))
}

fn screen_encoder(state: &AppState, key: &str) -> lumacore_tree::runtime::EncoderState {
    let store = state.root.runtime.live_screens.read().expect("screens lock");
    let (config, _) = own_screen(state).unwrap_or_default();
    let no = key.parse().unwrap_or(0);
    store.find_screen(state.effective_user_name(), config, no).and_then(|n| store.get(n)).map(|s| s.encoder.clone()).unwrap_or_default()
}

fn screen_content(state: &AppState, key: &str) -> Option<lumacore_tree::ScreenContent> {
    let (config, _) = own_screen(state)?;
    let profile = state.root.show_data.profile_for_user(state.effective_user_name())?;
    profile.content.screen_configs.get(&config)?.screens.get(&key.parse().ok()?).cloned()
}

macro_rules! screen_prop {
    ($name:literal, $kind:expr, $named:expr, $home:expr, $doc:literal, $get:expr) => {
        crate::props::Property {
            name: $name,
            kind: $kind,
            named: $named,
            rights: crate::commands::registry::Rights::Guest,
            home: $home,
            section: "Encoders",
            doc: $doc,
            access: crate::props::Access::Keyed {
                get: $get,
                set: |s, k, v| crate::props::from_reply(cmd_set_screen(s, &[k, $name, v])),
            },
            ..crate::props::Property::BASE
        }
    };
}

static SCREEN_PROPS: [crate::props::Property; 6] = [
    screen_prop!("EncoderBar", crate::props::PropKind::Switch, &[], crate::props::Home::Show, "the encoder bar on this screen (your profile's screen config)", |s, k| {
        let n = k.parse().unwrap_or(0);
        if screen_content(s, k).is_some_and(|c| c.encoder_bar.shown(n)) { "On" } else { "Off" }.to_string()
    }),
    screen_prop!("EncoderLayout", crate::props::PropKind::Text, &[], crate::props::Home::Show, "which of your encoder bar layouts it shows (a name or number)", |s, k| {
        let guid = screen_content(s, k).and_then(|c| c.encoder_bar.layout);
        s.root.show_data.profile_for_user(s.effective_user_name())
            .and_then(|p| p.content.attributes.encoder_bars.values().find(|b| Some(b.meta.identity.guid) == guid).map(|b| b.meta.name().unwrap_or_default().to_string()))
            .unwrap_or_default()
    }),
    screen_prop!("EncoderMode", crate::props::PropKind::Choice(lumacore_tree::runtime::EncoderMode::WORDS), &[], crate::props::Home::Station, "what the encoders drive", |s, k| screen_encoder(s, k).mode.word().to_string()),
    screen_prop!("EncoderGroup", crate::props::PropKind::Int { min: 0, max: 999 }, &["Next", "Previous"], crate::props::Home::Station, "the group the encoders show (Next / Previous step)", |s, k| screen_encoder(s, k).group.to_string()),
    screen_prop!("EncoderPage", crate::props::PropKind::Int { min: 1, max: 999 }, &["Next", "Previous"], crate::props::Home::Station, "the page of the group (Next / Previous step)", |s, k| screen_encoder(s, k).page.to_string()),
    screen_prop!("EncoderLayer", crate::props::PropKind::Choice(lumacore_tree::runtime::EncoderLayer::WORDS), &["Next"], crate::props::Home::Station, "Value, Fade or Delay (Next steps)", |s, k| screen_encoder(s, k).layer.word().to_string()),
];

/// Your screens in this client's screen config; a bare `Screen` is the one the line comes from.
static SCREEN_SETTINGS: crate::props::Props = crate::props::Props {
    lists: &[&SCREEN_PROPS],
    keyed: Some(crate::props::Keyed {
        rows: |s| {
            let Some((config, _)) = own_screen(s) else { return Vec::new() };
            s.root.show_data.profile_for_user(s.effective_user_name())
                .and_then(|p| p.content.screen_configs.get(&config))
                .map(|c| c.screens.keys().map(u32::to_string).collect())
                .unwrap_or_default()
        },
        row: |s, w| {
            let n: u32 = w.trim_matches('"').parse().ok()?;
            screen_content(s, &n.to_string()).map(|_| n.to_string())
        },
    }),
    implicit: Some(|s| own_screen(s).map(|(_, n)| n.to_string()).unwrap_or_default()),
    label: Some(|_, k| format!("Screen {k}")),
    ..crate::props::Props::BASE
};

pub static SCREEN: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Screen",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[],
        aliases: &["screens"],
        doc: crate::commands::registry::Doc::new(
            "Screen",
            "A screen of this client's screen config",
        ),
        family: None,
        call_undo: crate::commands::registry::Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_station,
        reach: crate::objects::show_reach,
        rights: crate::rights::screen,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::Store, screen_store), (Verb::Delete, screen_delete)],
    delete_ask: None,
    complete: Some(screen_complete),
    props: Some(&SCREEN_SETTINGS),
    tree: None,
};

#[cfg(test)]
mod follow_tests {
    use super::*;

    fn live_label(state: &AppState, no: u32) -> Option<String> {
        state.root.runtime.live_screens.read().unwrap().get(no).unwrap().windows.get(&2)?.label.clone()
    }

    /// R1-58: an unrelated profile op never drops a live screen's own edits;
    /// a change of its saved layout (a shared profile, Q33) is followed.
    #[test]
    fn a_live_screen_follows_only_a_changed_saved_layout() {
        let mut state = AppState::new();
        let no = state.root.runtime.live_screens.write().unwrap().join_or_create("Admin", None);
        sync(&mut state);
        let before = saved_layouts(&state);
        state
            .root
            .runtime
            .live_screens
            .write()
            .unwrap()
            .update_window(no, 2, |w| {
                w.label = Some("Local".into());
                Ok(())
            })
            .unwrap();
        follow_profiles(&mut state, &before);
        assert_eq!(live_label(&state, no).as_deref(), Some("Local"), "the screen's own edit stays");

        let (config, screen) = {
            let store = state.root.runtime.live_screens.read().unwrap();
            let s = store.get(no).unwrap();
            (s.config, s.screen)
        };
        let profile = state.root.show_data.profile_for_user_mut("Admin").unwrap();
        let saved = profile.content.screen_configs.get_mut(&config).unwrap().screens.get_mut(&screen).unwrap();
        saved.windows.get_mut(&2).unwrap().label = Some("Shared".into());
        follow_profiles(&mut state, &before);
        assert_eq!(live_label(&state, no).as_deref(), Some("Shared"), "the shared layout changed: followed");
    }
}
