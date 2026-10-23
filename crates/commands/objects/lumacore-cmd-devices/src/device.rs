// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝
// FILE: device

//! This device: bare `Device` shows its state; `Set Device "ScreenConfig" Follow|<n>` picks the
//! screen config it shows (station-local, never an Oops step).
//!
//! Owns:          the Device object: this device's state and its ScreenConfig setting.
//! Doesn't own:   the device table (lumacore-state).
//! Entry points:  DEVICE.
//! Decisions:     §30 "12a3", "12c2".

use lumacore_objects::commands::help::HelpCategory;
use lumacore_objects::commands::registry::{AliasDepth, CommandClass};
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::{Doc, ObjectDef, Undo, Verb, out};
use lumacore_objects::{AppState, DispatchResult};

// *************************************************************************************************
// Device: this device's settings
// *************************************************************************************************

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Set Device "ScreenConfig" Follow|<n>`: the device the line came from.
/// `Set Device "ScreenConfig" Follow|<n>` (the keyed property's write; `Follow` = the user's).
fn device_screen_config(state: &mut AppState, value: &str) -> Result<String, String> {
    let Some(id) = state.current_device.clone() else {
        return Err(render_code(
            "Device settings are per device (an app or a browser tab); this client isn't one.",
            "E-CMD-010",
        ));
    };
    let value = value.trim_matches('"').trim().to_string();
    let config = if value.eq_ignore_ascii_case("follow") {
        None
    } else {
        let Some(no) = value.parse::<u32>().ok().filter(|n| *n > 0) else {
            return Err(render_code(
                format!("'{value}' isn't a screen config (Follow or a number)."),
                "E-CMD-008",
            ));
        };
        let known = state
            .root
            .show_data
            .profile_for_user(state.effective_user_name())
            .map(|p| {
                p.content.screen_configs.contains_key(&no) || p.content.screen_configs.is_empty()
            });
        if known == Some(false) {
            return Err(render_code(
                format!("Your profile has no ScreenConfig {no}."),
                "E-CMD-007",
            ));
        }
        Some(no)
    };
    state.devices.entry(id).or_default().screen_config = config;
    Ok(match config {
        None => "Device ScreenConfig: Follow user".to_string(),
        Some(n) => format!("Device ScreenConfig {n}"),
    })
}

static DEVICE_PROPS: [lumacore_objects::props::Property; 1] = [lumacore_objects::props::Property {
    name: "ScreenConfig",
    kind: lumacore_objects::props::PropKind::Int { min: 1, max: 9999 },
    named: &["Follow"],
    rights: lumacore_objects::commands::registry::Rights::Guest,
    home: lumacore_objects::props::Home::Station,
    section: "Device",
    doc: "which of the user's screen configs this device shows (Follow = the user's)",
    default: Some("Follow"),
    access: lumacore_objects::props::Access::Keyed {
        get: |s, k| {
            s.devices
                .get(k)
                .and_then(|d| d.screen_config)
                .map_or_else(|| "Follow".to_string(), |n| n.to_string())
        },
        set: |s, _, v| device_screen_config(s, v),
    },
    ..lumacore_objects::props::Property::BASE
}];

/// This device (`Set Device "ScreenConfig" 2`): station-local, never an Oops step.
static DEVICE_SETTINGS: lumacore_objects::props::Props = lumacore_objects::props::Props {
    lists: &[&DEVICE_PROPS],
    keyed: Some(lumacore_objects::props::Keyed {
        rows: |s| s.current_device.clone().into_iter().collect(),
        row: |s, w| {
            s.current_device
                .clone()
                .filter(|d| d.eq_ignore_ascii_case(w.trim_matches('"')))
        },
    }),
    implicit: Some(|s| s.current_device.clone().unwrap_or_default()),
    label: Some(|_, _| "Device".to_string()),
    ..lumacore_objects::props::Props::BASE
};

/// Bare `Device`: this device's state.
fn device_call(state: &mut AppState, _args: &[&str]) -> DispatchResult {
    let Some(id) = state.current_device.clone() else {
        return out(render_code(
            "This client isn't a device (an app or a browser tab).",
            "E-CMD-010",
        ));
    };
    let d = state.devices.get(&id).cloned().unwrap_or_default();
    out(format!(
        "Device {id}\n  Locked         {}\n  NormalTheme    {}\n  ScreenConfig   {}\n",
        if d.locked { "yes" } else { "no" },
        if d.normal_theme { "on" } else { "off" },
        d.screen_config
            .map_or_else(|| "Follow user".to_string(), |n| n.to_string()),
    ))
}

pub static DEVICE: ObjectDef = ObjectDef {
    decl: lumacore_objects::objects::ObjectDecl {
        keyword: "Device",
        lum: lumacore_objects::objects::LumShape::ids(1),
        default_verb: lumacore_objects::objects::Verb::Info,
        flags: &[],
        aliases: &[],
        doc: Doc {
            display: "Device",
            category: HelpCategory::System,
            also_applies_to: &[],
            syntax: "Device | Set Device \"ScreenConfig\" Follow|<n> | Reset Device",
            description: "This device (every window of an app, or one browser tab; decisions §30 \"12a3\"). \
                          ScreenConfig: which of the user's screen configs it shows — Follow user (default) \
                          or a fixed number, e.g. a tablet on config 2. Station-local, never replicated, \
                          not an Oops step.",
            examples: &[
                "Set Device \"ScreenConfig\" 2       -- this tablet always shows config 2",
                "Set Device \"ScreenConfig\" Follow  -- back to the user's",
            ],
            related: &["desklock", "normaltheme", "screenconfig"],
            alias_depth: AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: CommandClass::Commit,
        scope: lumacore_objects::devices::on_station,
        reach: lumacore_objects::devices::device_reach,
        rights: lumacore_objects::rights::anyone,
        planned: &[],
    },
    pool: None,
    verbs: &[(Verb::Info, device_call)],
    delete_ask: None,
    complete: None,
    props: Some(&DEVICE_SETTINGS),
    tree: None,
};
