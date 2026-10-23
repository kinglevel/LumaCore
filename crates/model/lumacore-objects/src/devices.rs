// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝
// FILE: devices

//! The device toggles' shape (decisions §30 "12a3", §34 25.17): `On` / `Off` / `Toggle <Word>`
//! as one exported macro (`device_object!`), the switch, the page event and the scopes, and the
//! window toggles built from it — `Sidebars` (F12), `ViewBar` (Meow+F10), `FullScreen`
//! (Meow+F12), each acting on the window the line came from (a `Page` UI event). DeskLock,
//! NormalTheme, Device and ScreenConfig are `lumacore-cmd-devices`'; the device table is the
//! state's (`device_table`, re-exported here).
//!
//! Owns:          the toggle shape (device_object!, Switch, page, the scopes) and the window toggles.
//! Doesn't own:   DeskLock / NormalTheme / Device / ScreenConfig (lumacore-cmd-devices), the device table (state).
//! Entry points:  device_object!, Switch, switch_of, page, on_station, on_client, device_reach, SIDEBARS, VIEW_BAR, FULL_SCREEN.
//! Decisions:     §30 "12a3", "12b2", §34 25.17.

use crate::errors::render_code;
use crate::objects::{ObjectDef, Scope, Verb};
use crate::AppState;

pub use crate::device_table::*;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Clone, Copy)]
pub enum Switch {
    On,
    Off,
    Toggle,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub fn switch_of(verb: Verb) -> Switch {
    match verb {
        Verb::On => Switch::On,
        Verb::Off => Switch::Off,
        _ => Switch::Toggle,
    }
}

/// `Sidebars` / `ViewBar` / `FullScreen`: the window the line came from.
pub fn page(state: &mut AppState, what: &str, how: Switch) -> String {
    if !state.ui_client {
        return render_code(format!("{what} acts on a window of the UI."), "E-CMD-010");
    }
    state.ui_requests.push(lumacore_wire::UiEvent::Page {
        what: what.to_ascii_lowercase(),
        on: match how {
            Switch::On => Some(true),
            Switch::Off => Some(false),
            Switch::Toggle => None,
        },
    });
    String::new()
}

pub fn device_reach(_: Verb) -> crate::commands::registry::Reach {
    crate::commands::registry::Reach::Show
}

pub fn on_station(_: Verb, _: &[&str]) -> Scope {
    Scope::Station
}

pub fn on_client(_: Verb, _: &[&str]) -> Scope {
    Scope::Client
}

/// A device toggle (`On` / `Off` / `Toggle <Word>`): DeskLock and NormalTheme (lumacore-cmd-devices)
/// and the window toggles here.
#[macro_export]
macro_rules! device_object {
    ($static:ident, $kw:literal, $scope:expr, $rights:expr, $desc:literal, $syntax:literal, $examples:expr, $run:expr) => {
        pub static $static: ObjectDef = ObjectDef {
            decl: $crate::objects::ObjectDecl {
                keyword: $kw,
                lum: $crate::objects::LumShape::ONE,
                default_verb: $crate::objects::Verb::Toggle,
                flags: &[],
                aliases: &[],
                doc: $crate::commands::registry::Doc {
                    display: $kw,
                    category: $crate::commands::help::HelpCategory::System,
                    also_applies_to: &[],
                    syntax: $syntax,
                    description: $desc,
                    examples: $examples,
                    related: &["desklock", "normaltheme", "sidebars", "viewbar", "fullscreen"],
                    alias_depth: $crate::commands::registry::AliasDepth::Base,
                    hidden: false,
                },
                family: Some($crate::commands::registry::CommandFamily::Action),
                call_undo: $crate::commands::registry::Undo::ReadOnly,
                call_class: $crate::commands::registry::CommandClass::Commit,
                scope: $scope,
                reach: $crate::devices::device_reach,
                rights: $rights,
                planned: &[],
            },
            pool: None,
            verbs: &[
                ($crate::objects::Verb::On, |s, a| $crate::objects::out(($run)(s, $crate::devices::switch_of($crate::objects::Verb::On), a))),
                ($crate::objects::Verb::Off, |s, a| $crate::objects::out(($run)(s, $crate::devices::switch_of($crate::objects::Verb::Off), a))),
                ($crate::objects::Verb::Toggle, |s, a| $crate::objects::out(($run)(s, $crate::devices::switch_of($crate::objects::Verb::Toggle), a))),
            ],
            delete_ask: None,
            complete: None,
            props: None,
            tree: None,
        };
    };
}

device_object!(
    SIDEBARS,
    "Sidebars",
    on_client,
    crate::rights::anyone,
    "Full screen of the windows (F12): hide or show every bar of this window — the sidebars, the view bar, the top bar and the command line bar (decisions §30 \"12b2\"). Typing while hidden floats the command line in.",
    "Sidebars | On Sidebars | Off Sidebars",
    &["Sidebars    -- toggle (F12)"],
    |s, how, _| page(s, "Sidebars", how)
);

device_object!(
    VIEW_BAR,
    "ViewBar",
    on_client,
    crate::rights::anyone,
    "Hide or show the view bar of this window (Meow+F10).",
    "ViewBar | On ViewBar | Off ViewBar",
    &["ViewBar    -- toggle (Meow+F10)"],
    |s, how, _| page(s, "ViewBar", how)
);

device_object!(
    FULL_SCREEN,
    "FullScreen",
    on_client,
    crate::rights::anyone,
    "Native full screen of this window on its display (Meow+F12): macOS native full screen, \
     Windows / Linux their real full screen, a browser its full screen.",
    "FullScreen | On FullScreen | Off FullScreen",
    &["FullScreen    -- toggle (Meow+F12)"],
    |s, how, _| page(s, "FullScreen", how)
);
