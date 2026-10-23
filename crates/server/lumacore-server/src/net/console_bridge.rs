// ██████╗ ██████╗ ██╗██████╗  ██████╗ ███████╗
// ██╔══██╗██╔══██╗██║██╔══██╗██╔════╝ ██╔════╝
// ██████╔╝██████╔╝██║██║  ██║██║  ███╗█████╗
// ██╔══██╗██╔══██╗██║██║  ██║██║   ██║██╔══╝
// ██████╔╝██║  ██║██║██████╔╝╚██████╔╝███████╗
// ╚═════╝ ╚═╝  ╚═╝╚═╝╚═════╝  ╚═════╝ ╚══════╝
// FILE: bridge

//! `ConsoleBridge` for the HTTP engine. Commands and logins run on the state
//! thread (the same path as the UDP command port); every read comes from what
//! the state thread publishes, so HTTP never waits on the command queue
//! (decisions.md §26).
//!
//! Owns:          ServerConsoleBridge, lumacore-http's ConsoleBridge on this server: commands and logins as state jobs, reads from what the state thread publishes.
//! Doesn't own:   REST, WebSockets and the feeds (lumacore-http), the state thread (state).
//! Entry points:  ServerConsoleBridge.
//! Decisions:     §26, §30 "12b2", "12d", §34 24.2.

use lumacore_tree::TreeNode;
use serde_json::{Value, json};

use crate::state::{Published, StateHandle};

mod consent;
mod files;
mod tree;
use files::drive_shows_guard;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

pub struct ServerConsoleBridge {
    state: StateHandle,
    published: Published,
    /// `Runtime.Health` (atomics; the Realtime window reads it directly).
    health_board: std::sync::Arc<lumacore_tree::runtime::HealthBoard>,
    /// The station's counters for the live feeds (decisions §30 "12d").
    revisions: tokio::sync::watch::Sender<lumacore_http::Revisions>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl ServerConsoleBridge {
    pub fn new(
        state: StateHandle,
        published: Published,
        health_board: std::sync::Arc<lumacore_tree::runtime::HealthBoard>,
    ) -> Self {
        let revisions = tokio::sync::watch::Sender::new(lumacore_http::Revisions::default());
        let tx = revisions.clone();
        published.feeds.on_change(move |r| {
            tx.send_replace(lumacore_http::Revisions {
                show: r.show,
                station: r.station,
                show_info: r.show_info,
                claims: r.claims,
                network: r.network,
                lumasessions: r.lumasessions,
                library: r.library,
                tokens: r.tokens,
                modes: r.modes,
                undo: r.undo,
                clipboard: r.clipboard,
            });
        });
        // The live `station` feed shows `busy` when an episode starts and
        // ends (merge of bugfixes, §30 busy coverage): the watchdog moves
        // the station counter, the shared build reads it again.
        let tx = revisions.clone();
        state.on_busy_change(move |_| tx.send_modify(|r| r.station += 1));
        Self {
            // HTTP and the feeds are machine senders: the bounded lane, a
            // full lane answers 503 (decisions §31).
            state: state.machine(),
            published,
            health_board,
            revisions,
        }
    }

    /// A menu view the state thread builds only while a feed watches it:
    /// the published one, else built once now (`GET /api/network`).
    fn menu_view(
        &self,
        published: impl Fn(&crate::state::StationView) -> Value,
        build: fn(&lumacore_objects::AppState) -> Value,
    ) -> Value {
        let view = published(&self.published.station.load());
        if !view.is_null() {
            return view;
        }
        // A feed build waits like any HTTP read (merge of bugfixes, §30
        // busy coverage): busy = no view yet; the state thread publishes
        // it and moves the revision when it gets there.
        self.state
            .call_within(crate::state::HTTP_WAIT, move |ctx| build(&ctx.state))
            .unwrap_or(Value::Null)
    }
}

impl lumacore_http::ConsoleBridge for ServerConsoleBridge {
    fn execute(&self, user: &str, device: Option<&str>, cmd: &str) -> Result<String, String> {
        let (user, cmd) = (user.to_string(), cmd.to_string());
        let device = device.map(str::to_string);
        let (reply, waits) = self
            .state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                lumacore_lumasession::link::collecting(ctx, |ctx| {
                    super::handler::execute_remote_command_from(ctx, &user, device.as_deref(), &cmd).0
                })
            })
            .map_err(|e| e.to_string())?;
        // On a member: the master's answers (decisions §21 "6b").
        let reply = lumacore_lumasession::link::with_replies(reply, waits, lumacore_lumasession::link::HTTP_WAIT);
        let message = lumacore_objects::text::strip_ansi(&reply.message)
            .trim()
            .to_string();
        // Command errors come back as ok=true with a render_error message
        // ("Error: …"), so treat those as failures too.
        if reply.ok && !message.starts_with("Error:") {
            Ok(message)
        } else {
            Err(message)
        }
    }

    fn login(
        &self,
        user: &str,
        password: Option<&str>,
        remote: bool,
        from: &str,
    ) -> Result<(String, String), String> {
        consent::login(&self.state, &self.published, user, password, remote, from)
    }

    fn device_lock_refusal(&self, device: &str) -> Result<Option<String>, String> {
        let device = device.to_string();
        let refused = self
            .state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                lumacore_objects::devices::device_locked(&ctx.state, &device)
                    .then(lumacore_objects::devices::locked_refusal)
            })
            .map_err(|e| e.to_string())?;
        Ok(refused.map(|m| lumacore_objects::text::strip_ansi(&m).trim().to_string()))
    }

    fn unlock(&self, user: &str, device: &str, password: Option<&str>, from: &str) -> Result<(), String> {
        use lumacore_objects::passwords;
        let (user, device, password, from) =
            (user.to_string(), device.to_string(), password.map(str::to_string), from.to_string());
        self.state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                let ok = ctx
                    .state
                    .root
                    .show_data
                    .user(&user)
                    .is_some_and(|u| passwords::check_password(u, password.as_deref()));
                if !ok {
                    let message = format!("Wrong password for {user}");
                    crate::context::sysmon(ctx, "Session", format!("unlock of a device refused from {from}: {message}"));
                    return Err(message);
                }
                if let Err(message) = lumacore_objects::devices::may_unlock(&ctx.state, &device, &user) {
                    crate::context::sysmon(ctx, "Session", format!("unlock of a device refused from {from}: {message}"));
                    return Err(message);
                }
                lumacore_objects::devices::unlock(&mut ctx.state, &device);
                crate::context::sysmon(ctx, "Session", format!("device unlocked by {user} from {from}"));
                Ok(())
            })
            .map_err(|e| e.to_string())?
    }

    fn verify_token(&self, token: &str) -> Option<String> {
        if consent::waits(&self.published, token) {
            return None; // no login until the Meow License is accepted (decisions 0290)
        }
        let station = self.published.station.load();
        let show = self.published.show.load();
        lumacore_objects::passwords::verify_token_with(
            &station.token_secret,
            &station.revoked_tokens,
            &show,
            token,
        )
    }

    fn pending_consent(&self, token: &str) -> Option<String> {
        consent::user_of(&self.published, token)
    }

    fn logout(&self, token: &str, from: &str) {
        let (token, from) = (token.to_string(), from.to_string());
        // Queued, never dropped: a busy station revokes the token when it
        // gets to it (a dropped revoke would leave the token valid): the
        // shell lane, which never refuses.
        self.state.shell().send(move |ctx| {
            if let Some(user) = lumacore_objects::passwords::revoke_token(&mut ctx.state, &token) {
                crate::context::sysmon(ctx, "Session", format!("logout '{user}' from {from} (HTTP)"));
                // Readers see the revocation at once, not at the next publish.
                ctx.publish_station_now = true;
            }
        });
    }

    fn station(&self) -> Value {
        let mut station = self.published.station.load().station.clone();
        // The state thread stuck on one piece of work for over 5 s (R1-67, Q40).
        let busy = self.state.busy_for().is_some_and(|d| d > std::time::Duration::from_secs(5));
        if let Some(map) = station.as_object_mut() {
            map.insert("busy".into(), Value::Bool(busy));
        }
        station
    }

    fn feedback_active(&self, user: &str) -> Vec<String> {
        lumacore_sysmon::feedback::active_json(user)
    }

    fn feedback_settings(&self, user: &str) -> Value {
        let show = self.published.show.load();
        lumacore_objects::show_feedback::settings_json(show.profile_for_user(user).map(|p| &p.settings.preferences))
    }

    fn ui_keys(&self, user: &str) -> Value {
        let show = self.published.show.load();
        let station = self.published.station.load();
        let profile = show.profile_for_user(user);
        let mut keys: Vec<&lumacore_tree::UiKey> = station.ui_keys.iter().collect();
        for dp in show.datapools.values() {
            for p in dp.plugins.values() {
                keys.extend(p.buttons.iter());
            }
        }
        if let Some(p) = profile {
            keys.extend(p.content.ui_keys.values());
        }
        let prefs = profile.map(|p| &p.settings.preferences);
        json!({
            "double_click_ms": prefs.map_or(lumacore_tree::ui_keys::DEFAULT_DOUBLE_CLICK_MS, |p| p.double_click_ms),
            "hold_ms": prefs.map_or(lumacore_tree::ui_keys::DEFAULT_HOLD_MS, |p| p.hold_ms),
            "meow_key": prefs.map_or("Alt", |p| p.meow_key.as_str()),
            "pool_drag": prefs.map_or("Move", |p| p.pool_drag.as_str()),
            // A held pane maximises on the shared screen too (§30 "12c2").
            "hold_follows": prefs.is_none_or(|p| p.hold_follows),
            "keys": keys,
        })
    }

    fn pool(&self, user: &str, pool: &str, datapool: Option<u32>, profile: &Value, filter: &Value) -> Option<Value> {
        let show = self.published.show.load();
        let station = self.published.station.load();
        let profile = lumacore_objects::pools::resolve_profile(&show, user, profile);
        let filter = lumacore_objects::objects::preset::pool::filter_of(&show, filter);
        lumacore_views::pools::view(&show, &station.claims, user, pool, datapool, profile, filter)
    }

    fn view_bar(&self, user: &str, screen: u32) -> Value {
        let show = self.published.show.load();
        lumacore_views::view_bar::view(&show, user, screen)
    }

    fn windows(&self, user: &str, screen: &lumacore_tree::runtime::Screen) -> Value {
        let show = self.published.show.load();
        lumacore_views::window_state::screen_view(&show, user, screen)
    }

    fn oops(&self, user: &str) -> Value {
        let show = self.published.show.load();
        let mut v = self.published.station.load().undo.get(&user.to_lowercase()).cloned().unwrap_or_else(|| serde_json::json!({ "steps": [], "undone": [] }));
        if let (Some(o), Some((no, p))) = (v.as_object_mut(), show.profile_id_for_user(user).and_then(|n| show.user_profiles.get(&n).map(|p| (n, p)))) {
            let f = &p.settings.preferences.oops;
            o.insert("profile".into(), serde_json::json!(no));
            o.insert("locked".into(), serde_json::json!(p.locked));
            o.insert("popup_dim".into(), serde_json::json!(p.settings.preferences.popup_dim));
            o.insert(
                "filters".into(),
                serde_json::json!([
                    { "name": "CreateOops", "label": "Oops", "on": f.create_oops },
                    { "name": "OopsProgrammer", "label": "Programmer", "on": f.oops_programmer },
                    { "name": "OopsSelection", "label": "Selection", "on": f.oops_selection },
                    { "name": "OopsViews", "label": "Views", "on": f.oops_views },
                ]),
            );
        }
        v
    }

    fn encoder(&self, user: &str, screen: &lumacore_tree::runtime::Screen) -> Value {
        let show = self.published.show.load();
        let plan = self.published.render.plan.load();
        let own = self.published.station.load().mode_masters.get(&user.to_lowercase()).cloned().unwrap_or(Value::Null);
        lumacore_views::encoder::view(&show, user, screen, Some(&plan), &own)
    }

    fn devices(&self, user: &str, params: &Value) -> Value {
        use lumacore_views::devices::{Layer, Params};
        let show = self.published.show.load();
        let plan = self.published.render.plan.load();
        let frame = self.published.render.frame.load();
        let flag = |k: &str, default: bool| params[k].as_bool().unwrap_or(default);
        let number = |k: &str, default: usize| params[k].as_u64().map_or(default, |n| n as usize);
        let base = Params::default();
        let p = Params {
            layer: params["layer"].as_str().map_or(Layer::Absolute, Layer::parse),
            programmer_only: flag("programmer_only", base.programmer_only),
            selected_only: flag("selected_only", base.selected_only),
            selection_first: flag("selection_first", base.selection_first),
            folders: flag("folders", base.folders),
            from: number("from", base.from),
            count: number("count", base.count),
        };
        lumacore_views::devices::view(&show, user, Some(&plan), Some(&frame), lumacore_base::clock::session_ms(), &p)
    }

    fn sequence_sheet(&self, user: &str, sequence: &Value) -> Value {
        let show = self.published.show.load();
        lumacore_views::sequence::view(&show, user, sequence)
    }

    fn patch(&self, user: &str) -> Value {
        let show = self.published.show.load();
        let draft = self.published.draft.load();
        let mut v = lumacore_views::patch::view(&show, draft.as_ref().as_ref().map(|(h, d, m, r)| (h.as_str(), d, m, r.as_ref())), user);
        // What Paste would place (the slab counts it, decisions §34 round 3, 18).
        if let (Some(o), Some(c)) = (v.as_object_mut(), self.published.station.load().clipboards.get(&user.to_lowercase())) {
            o.insert("clipboard".into(), c.clone());
        }
        v
    }

    fn fixture_types(&self, user: &str, drive: Option<&str>) -> Value {
        let show = self.published.show.load();
        let draft = self.published.draft.load();
        let devices = match draft.as_ref() {
            Some((holder, d, _, _)) if holder.eq_ignore_ascii_case(user) => d,
            _ => &show.devices,
        };
        // A drive's gdtf folder (decisions §38): its files, each row naming the drive.
        let station = self.published.station.load();
        let dir = files::gdtf_dir(&station.station, drive);
        let library = match &dir {
            Some(dir) => lumacore_objects::commands::gdtf_library::library_in(dir),
            None if drive.is_some_and(|d| !d.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL)) => Vec::new(),
            None => lumacore_objects::commands::gdtf_library::library(),
        };
        let mut out = lumacore_views::insert::list(devices, &library);
        if let (Some(d), Some(rows)) = (drive, out["library"].as_array_mut()) {
            for r in rows {
                r["drive"] = Value::String(d.to_string());
            }
        }
        out
    }

    fn fixture_type(&self, user: &str, which: &lumacore_http::endpoints::gdtf::Which) -> Option<Value> {
        let station = self.published.station.load();
        let library = match files::gdtf_dir(&station.station, which.drive.as_deref()) {
            Some(dir) => lumacore_objects::commands::gdtf_library::library_in(&dir),
            None => lumacore_objects::commands::gdtf_library::library(),
        };
        if let Some(file) = &which.file {
            let f = library.iter().find(|f| &f.file == file)?;
            return lumacore_views::insert::mode(f.fixture_type.as_ref().ok()?, which.revision, which.mode, &f.details);
        }
        let show = self.published.show.load();
        let draft = self.published.draft.load();
        let devices = match draft.as_ref() {
            Some((holder, d, _, _)) if holder.eq_ignore_ascii_case(user) => d,
            _ => &show.devices,
        };
        let ft = devices.dmx.patch.fixture_types.get(&which.no?)?;
        // The file's details, when the library holds the same file as one of its revisions.
        let details = library
            .iter()
            .find(|f| ft.revisions.iter().any(|r| r.file_hash == f.sha256))
            .map(|f| f.details.clone())
            .unwrap_or_default();
        lumacore_views::insert::mode(ft, which.revision, which.mode, &details)
    }

    fn fixture_thumbnail(&self, user: &str, which: &lumacore_http::endpoints::gdtf::Which) -> Option<(Vec<u8>, String)> {
        let found = match &which.file {
            Some(file) => lumacore_objects::commands::gdtf_library::thumbnail(Some(file), None),
            None => {
                let show = self.published.show.load();
                let draft = self.published.draft.load();
                let devices = match draft.as_ref() {
                    Some((holder, d, _, _)) if holder.eq_ignore_ascii_case(user) => d,
                    _ => &show.devices,
                };
                let ft = devices.dmx.patch.fixture_types.get(&which.no?)?;
                let rev = if which.revision == 0 { ft.revisions.last()? } else { ft.revisions.get(which.revision - 1)? };
                lumacore_objects::commands::gdtf_library::thumbnail(None, Some(&rev.file_hash))
            }
        };
        found.map(|(bytes, mime)| (bytes, mime.to_string()))
    }

    fn save_gdtf(&self, user: &str, name: &str, bytes: Vec<u8>) -> Result<String, String> {
        // Patching is Setup's (decisions §19): the same level Import FixtureType asks.
        let show = self.published.show.load();
        let rights = show.user(user).map(|u| u.rights).unwrap_or_default();
        if rights < lumacore_tree::UserRights::Setup {
            return Err(lumacore_objects::errors::render_code(format!("Uploading a fixture file needs Setup — you are {}.", rights.label()), "E-RIGHTS-001"));
        }
        let saved = lumacore_objects::commands::gdtf_library::save_upload(name, &bytes)?;
        // On a member the master needs it too (§34 round 3: every station holds what the show
        // is built from): sent before the Store / Import line the page runs next, on the same
        // stream; the type then rides in the show to every station.
        let (user, file) = (user.to_string(), saved.clone());
        self.state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                log::info!("{user} uploaded library/gdtf/{file}");
                if let Some(link) = &ctx.state.link {
                    use base64::Engine;
                    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let _ = link.tx.send(lumacore_objects::session_link::LinkOut::Upload { name: file, data, gdtf: true });
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(saved)
    }

    fn files(&self, drive: &str, folder: &str, kind: &str) -> Value {
        files::files(&self.published.station.load().station, drive, folder, kind)
    }

    fn save_upload(&self, user: &str, drive: &str, folder: &str, name: &str, bytes: Vec<u8>) -> Result<String, String> {
        // The local library's gdtf folder keeps its store and the master's copy (save_gdtf).
        if drive.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL) && folder.eq_ignore_ascii_case("gdtf") {
            let show = self.published.show.load();
            let rights = show.user(user).map(|u| u.rights).unwrap_or_default();
            if rights < lumacore_tree::UserRights::Setup {
                return Err(lumacore_objects::errors::render_code(format!("Uploading a file needs Setup — you are {}.", rights.label()), "E-RIGHTS-001"));
            }
            return self.save_gdtf(user, name, bytes);
        }
        let rights = self.published.show.load().user(user).map(|u| u.rights).unwrap_or_default();
        let id = drive.to_string();
        let root = move || self.state.call_within(crate::state::HTTP_WAIT, move |ctx| ctx.state.ensure_library(&id)).map_err(|e| e.to_string());
        files::save_upload(rights, root, user, drive, folder, name, &bytes)
    }

    fn tree_window(&self, user: &str, params: &Value) -> Value {
        tree::tree(&self.state, user, params)
    }

    fn inspector(&self, user: &str, path: &str) -> Value {
        tree::inspector(&self.state, user, path)
    }

    fn tags(&self, tag: &Value, deep: bool) -> Value {
        let show = self.published.show.load();
        lumacore_views::tags::view(&show, tag, deep)
    }

    fn resolve_datapool(&self, user: &str, datapool: &Value) -> Option<u32> {
        let show = self.published.show.load();
        lumacore_objects::pools::resolve_datapool(&show, user, datapool)
    }

    fn render_stats(&self) -> Option<std::sync::Arc<lumacore_tree::runtime::RenderStats>> {
        Some(self.health_board.render.clone())
    }

    fn pool_object(&self, user: &str, pool: &str, datapool: Option<u32>, no: u32) -> Option<Value> {
        let show = self.published.show.load();
        let station = self.published.station.load();
        lumacore_views::pools::detail(&show, &station.claims, user, pool, datapool, no)
    }

    fn attribute_board(&self, user: &str, profile: &Value) -> Option<Value> {
        let show = self.published.show.load();
        let target = lumacore_objects::attribute_board::BoardTarget::parse(&show, user, profile);
        lumacore_views::attribute_board::view(&show, user, target)
    }

    fn detail_view(&self, kind: &str, no: u32) -> Option<Value> {
        let show = self.published.show.load();
        lumacore_views::detail::view(&show, kind, no)
    }

    fn sysmon_history(&self) -> Vec<String> {
        lumacore_sysmon::history_json()
    }

    /// A config without saved screens still has screen 1 (the default layout).
    fn media(&self, sha256: &str) -> Option<(Vec<u8>, String)> {
        let (bytes, ext) = lumacore_luma::media::stored_any(sha256)?;
        Some((bytes, lumacore_tree::media::mime_of(&ext).to_string()))
    }

    fn save_media(&self, user: &str, name: &str, bytes: Vec<u8>) -> Result<String, String> {
        let saved = lumacore_luma::media::save_upload(name, &bytes)?;
        // Imports run on the master (decisions §30 "12a2"): a member also sends
        // the file there, ahead of the Import line the client runs next. If
        // the master is lost meanwhile, the Import fails and is run again by
        // hand (the file stays here).
        let (user, file) = (user.to_string(), saved.clone());
        self.state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                log::info!("{user} uploaded {}/{file}", lumacore_luma::media::LIBRARY);
                if let Some(link) = &ctx.state.link {
                    use base64::Engine;
                    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let _ = link.tx.send(lumacore_objects::session_link::LinkOut::Upload { name: file, data, gdtf: false });
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(saved)
    }

    fn look(&self, user: &str) -> Value {
        let show = self.published.show.load();
        show.profile_for_user(user)
            .map(lumacore_objects::commands::media::look_of)
            .unwrap_or(Value::Null)
    }

    fn screen_configs(&self, user: &str) -> Value {
        use lumacore_objects::commands::screens;
        let show = self.published.show.load();
        let configs: Vec<Value> = show
            .profile_for_user(user)
            .map(|p| {
                p.content.screen_configs
                    .iter()
                    .map(|(no, c)| {
                        let mut screens: Vec<u32> = c.screens.keys().copied().collect();
                        if screens.is_empty() {
                            screens.push(1);
                        }
                        json!({ "no": no, "name": c.meta.identity.name, "screens": screens })
                    })
                    .collect()
            })
            .unwrap_or_default();
        json!({ "default": screens::default_config(&show, user), "configs": configs })
    }

    fn screen_layout(
        &self,
        user: &str,
        config: Option<u32>,
        screen: u32,
    ) -> (u32, Option<lumacore_http::ScreenLayout>) {
        use lumacore_objects::commands::screens;
        let show = self.published.show.load();
        let config = config.unwrap_or_else(|| screens::default_config(&show, user));
        (config, screens::saved(&show, user, config, screen))
    }

    fn shows(&self) -> Value {
        self.shows_for(None, None)
    }

    fn shows_for(&self, history: Option<&str>, drive: Option<&str>) -> Value {
        let view = self.published.station.load();
        // A drive's shows (decisions §38 5): the same view, on that drive's shows folder.
        let _dir = drive_shows_guard(&view.station, drive);
        if drive.is_some_and(|d| !d.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL)) && _dir.is_none() {
            return json!({ "shows": [], "error": format!("Drive \"{}\" is unknown or out", drive.unwrap_or("")) });
        }
        lumacore_luma::backups::library_view(&view.show_info, history, &view.backup)
    }

    fn show_parts(&self, show: &str, version: Option<&str>, recovery: Option<&str>, drive: Option<&str>) -> Value {
        // A show on a drive (decisions §38 5): its shows folder for the read.
        let view = self.published.station.load();
        let _dir = drive_shows_guard(&view.station, drive);
        if drive.is_some_and(|d| !d.eq_ignore_ascii_case(lumacore_objects::drives::LOCAL)) && _dir.is_none() {
            return json!({ "error": format!("Drive \"{}\" is unknown or out", drive.unwrap_or("")) });
        }
        lumacore_cmd_showfile::parts_view(show, version, recovery)
    }

    fn user_views(&self, user: &str) -> Value {
        let show = self.published.show.load();
        let profile = show.profile_for_user(user);
        let Some(profile) = profile else {
            return json!({ "views": [], "buttons": {} });
        };
        let views: Vec<Value> = profile
            .content.views
            .iter()
            .map(|(no, v)| {
                json!({
                    "no": no,
                    "name": v.meta.identity.name,
                    "screens": v.screens.keys().collect::<Vec<_>>(),
                    "windows": v.screens.values().map(|c| c.windows.len()).sum::<usize>(),
                })
            })
            .collect();
        json!({ "views": views, "buttons": profile.content.view_buttons })
    }

    fn http_allows(&self, local: std::net::IpAddr) -> bool {
        self.published.services.load().http_at(local)
    }

    fn network(&self) -> Value {
        self.menu_view(|v| v.network.clone(), lumacore_views::menu_rows::network)
    }

    fn lumasessions(&self) -> Value {
        self.menu_view(|v| v.lumasessions.clone(), lumacore_views::menu_rows::lumasessions)
    }

    fn menu_types(&self) -> Value {
        static TYPES: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        TYPES.get_or_init(lumacore_views::menus::menu_types_json).clone()
    }

    fn object_menus(&self) -> Value {
        static MENUS: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
        MENUS.get_or_init(lumacore_views::context::object_menus_json).clone()
    }

    fn tool_types(&self) -> Value {
        lumacore_objects::tools::tool_types_json()
    }

    fn revisions(&self) -> tokio::sync::watch::Receiver<lumacore_http::Revisions> {
        self.revisions.subscribe()
    }

    fn feed_demand(&self, feed: &str, on: bool) {
        use std::sync::atomic::Ordering;
        let counter = match feed {
            "network" => &self.published.feeds.network,
            "lumasessions" => &self.published.feeds.lumasessions,
            _ => return,
        };
        if on {
            counter.fetch_add(1, Ordering::Relaxed);
            // Built at the next turn of the state thread, not in 250 ms.
            self.state.send(|ctx| ctx.publish_station_now = true);
        } else {
            let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
        }
    }

    fn save_screenshot(&self, user: &str, id: u64, screen: u32, png: Vec<u8>) -> Result<String, String> {
        use lumacore_objects::commands::screenshot;
        // Asked for, and the uploader's own screen? (on the state thread)
        let uploader = user.to_string();
        let (asked_by, owner, screen_no, station) = self
            .state
            .call_within(crate::state::HTTP_WAIT, move |ctx| {
                let store = ctx.state.root.runtime.live_screens.clone();
                let mut store = store.write().map_err(|_| "screens unavailable".to_string())?;
                let live = store.get(screen).cloned().ok_or(format!("no screen {screen}"))?;
                if !live.user.eq_ignore_ascii_case(&uploader) {
                    return Err(format!("screen {screen} isn't {uploader}'s"));
                }
                let asked_by = store
                    .capture_arrived(id, screen)
                    .ok_or(format!("screenshot {id} of screen {screen} wasn't asked for"))?;
                Ok((asked_by, live.user, live.screen, ctx.state.root.station_settings.station_name.clone()))
            })
            .map_err(|e| e.to_string())??;
        // The file is written here, on the HTTP worker, never on the state thread.
        let path = screenshot::write(&screenshot::folder(&station), &owner, screen_no, &png)?;
        let shown = path.display().to_string();
        let note = shown.clone();
        self.state.send(move |_| screenshot::saved(&asked_by, &note));
        Ok(shown)
    }

    fn health(&self) -> Value {
        serde_json::to_value(self.health_board.view()).unwrap_or(Value::Null)
    }

    fn dmx_universe(&self, universe: u32) -> Option<lumacore_http::DmxUniverse> {
        use lumacore_render::Winner;
        let hub = &self.published.render;
        let frame = hub.frame.load();
        if !self
            .published
            .show
            .load()
            .devices
            .dmx
            .patch
            .universes
            .contains_key(&universe)
        {
            return None;
        }
        let values = frame.get(universe).copied().unwrap_or([0; 512]);
        let ours = frame.ours(universe).copied().unwrap_or(values);
        let won = frame.winners(universe);
        let code = |w: &Winner| match w {
            Winner::Unpatched => 0u8,
            Winner::Default => 1,
            Winner::Ours => 2,
            Winner::Park => 3,
            Winner::Input(_) => 4,
        };
        Some(lumacore_http::DmxUniverse {
            universe,
            values: values.to_vec(),
            ours: ours.to_vec(),
            input: hub.input_universe(universe).map(|i| i.to_vec()),
            winners: won.map_or(vec![0; 512], |w: &[Winner; 512]| w.iter().map(code).collect()),
            lines: won.map_or(vec![0; 512], |w: &[Winner; 512]| {
                w.iter()
                    .map(|w: &Winner| match w {
                        Winner::Input(line) => *line,
                        _ => 0,
                    })
                    .collect()
            }),
        })
    }

    fn dmx_patch(&self, universe: u32) -> Vec<lumacore_http::DmxPatchSpan> {
        let show = self.published.show.load();
        let Some(guid) = show.universe_guid(universe) else {
            return Vec::new();
        };
        let mut spans = Vec::new();
        for fixture in show.devices.dmx.patch.fixtures.values() {
            // 0 = a fixture without a number (decisions §40 7); a folder has no address anyway.
            let id = fixture.mvr.fid.unwrap_or(0);
            let cfs = lumacore_render::merge::channels(&show, fixture);
            for brk in fixture.mvr.breaks.iter().filter(|b| b.universe_id == guid) {
                let mut attributes = vec![String::new(); usize::from(brk.channel_count)];
                for cf in cfs.iter().filter(|cf| !cf.is_virtual) {
                    for b in 0..usize::from(cf.byte_count).max(1) {
                        if let Some(slot) = attributes.get_mut(usize::from(cf.dmx_offset) + b) {
                            *slot = match b {
                                0 => cf.attribute_name.clone(),
                                1 => format!("{} fine", cf.attribute_name),
                                _ => format!("{} ultra", cf.attribute_name),
                            };
                        }
                    }
                }
                spans.push(lumacore_http::DmxPatchSpan {
                    fixture: id,
                    name: fixture.name().unwrap_or("").to_string(),
                    start: brk.start_address,
                    attributes,
                });
            }
        }
        spans.sort_by_key(|s| s.start);
        spans
    }

    fn dmx_selected_universe(&self, user: &str) -> Option<u32> {
        let show = self.published.show.load();
        let first = *show.programmer_of(user)?.selected_fixtures.first()?;
        let fixture = show
            .devices
            .dmx
            .patch
            .fixtures
            .values()
            .find(|f| f.meta.guid() == first)?;
        let universe = fixture.primary_break()?.universe_id;
        show.devices
            .dmx
            .patch
            .universes
            .iter()
            .find(|(_, u)| u.meta.guid() == universe)
            .map(|(&id, _)| id)
    }
}

