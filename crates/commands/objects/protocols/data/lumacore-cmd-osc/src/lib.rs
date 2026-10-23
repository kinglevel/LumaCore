//  ██████╗ ███████╗ ██████╗
// ██╔═══██╗██╔════╝██╔════╝
// ██║   ██║███████╗██║
// ██║   ██║╚════██║██║
// ╚██████╔╝███████║╚██████╗
//  ╚═════╝ ╚══════╝ ╚═════╝
// FILE: osc

//! The `Osc` object — see the crate's README.md (decisions §34 "34f7c hardened"): OSC
//! lines, declared on the one set of line handlers of `lumacore-cmd-protocols`.

use lumacore_objects::objects::Verb;
use lumacore_objects::objects::protocols as F;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What `Store Osc` take: the line's declared settings (decisions §34 "34f7c hardened" 7).
const FLAGS: &[lumacore_objects::commands::flags::FlagDef] = &[
    F::NETWORK_FLAG.on(&[Verb::Store]),
    F::MODE.on(&[Verb::Store]),
    F::USER.on(&[Verb::Store]),
    F::PORT.on(&[Verb::Store]),
];

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

lumacore_cmd_protocols::line_object!(
    data OSC, "Osc", "Universe", crate::OSC,
    door: door,
    store: StoreOsc,
    flags: FLAGS,
    doc: {
        syntax: "Store Osc [<n>] [\"name\"] /Network <name> /User <name> [/Port <p>] | Set Osc <n> \"Port\"|\"User\"|\"Target\"|\"Enabled\" <value>",
        description: "An OSC input line: the commands it receives run as the line's User, with that user's rights (never Admin as a fallback), on its network and Port (default 8000); Target accepts one source only. List Osc shows each line and whether it listens; Info Osc the protocol.",
        examples: ["Store Osc 1 \"Companion\" /Network \"Rig\" /Port 8000 /User \"LD\"", "Set Osc 1 \"Port\" 9000", "Set Osc 1 \"User\" \"Busk\""],
    },
    extra_verbs: [],
);

/// This crate's rows in the catalog: the `Osc` object and its Import Show hook, after its
/// protocol's row and the shared line words (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_osc::register(r);
    lumacore_cmd_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
}

/// The object; `register` adds it to the catalog.
pub static OBJECTS: &[&lumacore_objects::objects::ObjectDef] = &[&OSC];

/// Import Show "x" Osc 1: its GUID in the other show.
pub static HOOKS: &[lumacore_objects::commands::registry::Hook] = &[lumacore_objects::commands::registry::Hook::ImportByGuid(&OSC, |s, _, id| {
    s.protocols.line("Osc", id.parse().ok()?).map(|x| x.meta.identity.guid.to_string())
})];

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// A test binary's catalog: this crate's and what it builds on (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(crate::register);
}
