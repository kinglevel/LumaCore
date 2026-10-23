//! The keyed settings (decisions §30 "12d hardening": "Typed props") — properties of objects
//! that aren't tree collections, declared in the one property registry (decisions §33 58,
//! `crate::props`): this station (`Set Station "Name" "FOH"`), its interfaces' services
//! (`Set Interface "en0" + "en1" "Shell" No`), the station's NIC for each show network
//! (`Set Network "Rig" "Interface" "en1"`), the show's own settings (`Set Show "Autosave" 5`),
//! the asset drives' folders and About. `Set` (`commands::set`), `EditSetting`, the rights
//! gate, the menus' grid columns and the reference read these declarations — there is no
//! other parser for them. A station's own settings, another station's (sent to it), its
//! wiring rows and the asks live here.

use crate::AppState;
use crate::commands::registry::Rights;
use crate::errors::render_code;
use crate::props::{Access, Home, Keyed, PropKind, Property, Props};

pub fn on(v: &str) -> bool {
    crate::props::on_off(v) == Some(true)
}

pub fn word(s: &Property, b: bool) -> String {
    if b { s.words.0 } else { s.words.1 }.to_string()
}

/// After a station-local setting: kept in the station's file, the protocol plan rebuilt.
pub fn station_after(state: &mut AppState) {
    state.local_settings.store_from(&state.root.station_settings);
    state.protocols_dirty = true;
}

/// A station setting's two words.
pub const STATION_WORDS: (&str, &str) = ("On", "Off");

// ── The show's settings ──────────────────────────────────────────────────────

fn show_after(state: &mut AppState) {
    state.show_dirty = true;
}

// tidy: not-keywords (the merge decision's words, as Set takes them)
const MERGE_CHOICES: &[&str] = &["Merge", "UseMaster", "UseMine", "Cancel"];

pub static SHOW_SETTINGS: Props = Props {
    lists: &[&[
        Property {
            name: "Description",
            kind: PropKind::Text,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Program,
            section: "Show",
            unit: "",
            doc: "what the show is (shown in the show lists)",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.show_data.show_settings.description.clone(),
                set: |s, _, v| {
                    s.root.show_data.show_settings.description = v.to_string();
                    Ok(format!("Show description: \"{v}\""))
                },
            },
            home: Home::Show,
        },
        Property {
            name: "Autosave",
            kind: PropKind::Int { min: 0, max: 1440 },
            named: &["Off"],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Show",
            unit: "min",
            doc: "minutes between recovery copies of a changed show (each station writes its own; the show file changes only when someone saves); Off = 0",
            default: None,
            access: Access::Keyed {
                get: |s, _| match s.root.show_data.show_settings.autosave_minutes {
                    0 => "Off".to_string(),
                    m => m.to_string(),
            },
            set: |s, _, v| {
                let minutes = if v == "Off" { 0 } else { v.parse::<u32>().unwrap_or(0) };
                s.root.show_data.show_settings.autosave_minutes = minutes;
                Ok(if minutes == 0 { "Autosave off".to_string() } else { format!("Autosave every {minutes} min") })
                },
            },
            home: Home::Show,
        },
        Property {
            name: "MergeDefault",
            kind: PropKind::Choice(MERGE_CHOICES),
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Show",
            unit: "",
            doc: "what a rejoining station does when nobody answers the merge prompt",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.show_data.show_settings.merge_default.word().replace(' ', ""),
                set: |s, _, v| {
                    let choice = lumacore_tree::MergeChoice::parse(v)
                        .ok_or_else(|| render_code(format!("'{v}' is not one of {}.", MERGE_CHOICES.join(", ")), "E-SET-003"))?;
                    s.root.show_data.show_settings.merge_default = choice;
                    Ok(format!("Merge default: {}", choice.word()))
                },
            },
            home: Home::Show,
        },
        Property {
            name: "MergeTimeout",
            kind: PropKind::Int { min: 0, max: 3600 },
            named: &["Unlimited"],
            words: STATION_WORDS,
            rights: Rights::Setup,
            section: "Show",
            unit: "s",
            doc: "seconds before the merge prompt answers itself with the default (0 = no prompt)",
            default: None,
            access: Access::Keyed {
                get: |s, _| s.root.show_data.show_settings.merge_timeout.map_or("Unlimited".to_string(), |t| t.to_string()),
                set: |s, _, v| {
                    let t = if v == "Unlimited" { None } else { Some(v.parse::<u32>().unwrap_or(0)) };
                    s.root.show_data.show_settings.merge_timeout = t;
                    Ok(match t {
                        None => "Merge prompt waits for the operator (Unlimited)".to_string(),
                        Some(0) => "Merge prompt off: the default applies at once".to_string(),
                        Some(s) => format!("Merge prompt answers itself after {s} s"),
                    })
                },
            },
            home: Home::Show,
        },
    ]],
    keyed: Some(Keyed { rows: |_| vec!["Show".to_string()], row: |_, w| w.trim_matches('"').eq_ignore_ascii_case("show").then(|| "Show".to_string()) }),
    implicit: Some(|_| "Show".to_string()),
    label: Some(|_, _| "Show".to_string()),
    after: show_after,
    ..Props::BASE
};

// ── Asset drives: this station's folder of each of the show's roots ─────────

fn asset_name(state: &AppState, key: &str) -> String {
    key.parse::<u32>().ok().and_then(|n| state.root.show_data.assets.roots.get(&n)).map_or_else(String::new, |r| r.name.clone())
}

/// `Set AssetRoot "Video" "Folder" "/Volumes/Media"` (decisions §8): the show
/// names the drive, each station maps it to a folder of its own.
pub static ASSET_ROOT_SETTINGS: Props = Props {
    lists: &[&[Property {
        name: "Folder",
        kind: PropKind::Text,
        named: &["None"],
        words: STATION_WORDS,
        rights: Rights::Setup,
        section: "Drive",
        unit: "",
        doc: "this station's folder or drive for the asset root (None = no files here)",
        default: None,
        access: Access::Keyed {
                    get: |s, k| s.root.station_settings.asset_roots.get(&asset_name(s, k).to_lowercase()).cloned().unwrap_or_else(|| "None".into()),
            set: |s, k, v| {
                let name = asset_name(s, k);
                let roots = &mut s.root.station_settings.asset_roots;
                Ok(if v.is_empty() || v == "None" {
                    roots.remove(&name.to_lowercase());
                    format!("AssetRoot \"{name}\" unmapped on this station")
                } else {
                    let there = if lumacore_luma::drive::metadata(v).is_ok() { "" } else { " (not there right now)" };
                    roots.insert(name.to_lowercase(), v.to_string());
                    format!("AssetRoot \"{name}\" → {v}{there} on this station")
                })
            },
        },
        home: Home::Station,
    }]],
    label: Some(|s, k| format!("AssetRoot {k} \"{}\"", asset_name(s, k))),
    after: station_after,
    ..Props::BASE
};

// ── About this station (read only) ──────────────────────────────────────────

macro_rules! about {
    ($name:literal, $doc:literal, $get:expr) => {
        Property {
            name: $name,
            kind: PropKind::Text,
            named: &[],
            words: STATION_WORDS,
            rights: Rights::Guest,
            section: "About",
            unit: "",
            doc: $doc,
            default: None,
            access: Access::Keyed {
                get: $get,
                set: |_, _, _| Err(render_code("About is read-only.", "E-CMD-010")),
            },
            home: Home::Station,
        }
    };
}

/// `EditSetting Server`: the build, protocols and paths (Settings › About).
pub static ABOUT_SETTINGS: Props = Props {
    lists: &[&[
        about!("Build", "the version, profile, platform and commit this build was made from", |s, _| s
            .root
            .diagnostics
            .build_info
            .build_id
            .clone()),
        about!("ClientProtocol", "what a client's hello must carry (lumacore-wire)", |_, _| lumacore_wire::CLIENT_PROTOCOL.to_string()),
        about!("SessionProtocol", "what stations of one LumaSession must share", |_, _| lumacore_wire::SESSION_PROTOCOL.to_string()),
        about!("ShowFormat", "the show format this build writes and reads", |_, _| lumacore_tree::FORMAT_VERSION.to_string()),
        about!("Station", "this station's name and id", |s, _| format!("{} · {}", s.root.station_settings.station_name, s.local_settings.station_id)),
        about!("ShowFolder", "where shows are saved", |_, _| "library/shows".to_string()),
        about!("StationFolder", "where this station keeps its own files (station_settings.json)", |_, _| lumacore_luma::station_config::dir()),
        about!("Uptime", "since this server started", |_, _| {
            let secs = lumacore_sysmon::SERVER_START.get().map_or(0, |t| t.elapsed().as_secs());
            format!("{} h {} min", secs / 3600, secs / 60 % 60)
        }),
    ]],
    keyed: Some(Keyed { rows: |_| vec!["Server".to_string()], row: |_, w| w.trim_matches('"').eq_ignore_ascii_case("server").then(|| "Server".to_string()) }),
    implicit: Some(|_| "Server".to_string()),
    label: Some(|_, _| "Server".to_string()),
    after: |_| {},
    readonly: Some("facts of this build and station"),
    ..Props::BASE
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_merge_choices_parse() {
        for c in MERGE_CHOICES {
            assert!(lumacore_tree::MergeChoice::parse(c).is_some());
        }
    }
}
