//  █████╗ ██████╗ ████████╗███╗   ██╗███████╗████████╗
// ██╔══██╗██╔══██╗╚══██╔══╝████╗  ██║██╔════╝╚══██╔══╝
// ███████║██████╔╝   ██║   ██╔██╗ ██║█████╗     ██║
// ██╔══██║██╔══██╗   ██║   ██║╚██╗██║██╔══╝     ██║
// ██║  ██║██║  ██║   ██║   ██║ ╚████║███████╗   ██║
// ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═══╝╚══════╝   ╚═╝
// FILE: artnet

//! The `ArtNet` object — see the crate's README.md (decisions §34 "34f7c hardened"): Art-Net
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

/// What `Store ArtNet` and `Assign` take: the line's declared settings (decisions §34 "34f7c hardened" 7).
const FLAGS: &[lumacore_objects::commands::flags::FlagDef] = &[
    F::NETWORK_FLAG.on(&[Verb::Store]),
    F::MODE.on(&[Verb::Store]),
    F::LOCAL_UNIVERSE.on(&[Verb::Store]),
    F::AMOUNT.on(&[Verb::Store]),
    lumacore_objects::commands::flags::FlagDef::new("PortAddress", "The Art-Net universe LocalUniverse is on the wire; the rest follow on")
        .value("universe")
        .on(&[Verb::Store]),
    lumacore_objects::objects::ROUTE_UNIVERSE,
];

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

lumacore_cmd_protocols::dmx_line_verbs!(run, "ArtNet");

lumacore_cmd_protocols::line_object!(
    dmx ARTNET, "ArtNet", "PortAddress", crate::ARTNET,
    door: door,
    store: StoreArtNet,
    flags: FLAGS,
    doc: {
        syntax: "Store ArtNet [<n>] [\"name\"] /Network <name> [/Mode Output|Input] [/LocalUniverse <u> /Amount <n> /PortAddress <net.sub.uni>] | Assign Universe <u> Thru <v> At ArtNet <n> [/Universe <a>] | Assign ArtNet <n> At Universe <u> | Set ArtNet <n> \"PortAddress\"|\"Rate\"|… <value>",
        description: "An Art-Net 4 line: one DMX Protocols row on its network — Mode Output sends its run of local universes (LocalUniverse and the Amount after it) as Port-Addresses from PortAddress on (typed 5 or 0.0.5), to the nodes ArtPoll finds (or unicast to Target) at its own Rate, never above a node's refresh rate; Mode Input brings them into the local ones, at its InputPriority. List ArtNet shows each line's run and live state; Info ArtNet the protocol and the nodes heard.",
        examples: ["Store ArtNet 1 \"Truss\" /Network \"Rig\" /LocalUniverse 33 /Amount 4 /PortAddress 0.2.0", "Assign Universe 33 Thru 36 At ArtNet 1 /Universe 0.2.0", "Assign ArtNet 2 At Universe 50 /Universe 0.0.5", "Set ArtNet 1 \"Sync\" On", "Unassign ArtNet 1"],
    },
    extra_verbs: [(Verb::Assign, lumacore_objects::commands::typed::typed::<run::Assign>),
        (Verb::Unassign, lumacore_objects::commands::typed::typed::<run::Unassign>),],
);

/// This crate's rows in the catalog: the `ArtNet` object and its Import Show hook, after its
/// protocol's row and the shared line words (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_artnet::register(r);
    lumacore_cmd_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
}

/// The object; `register` adds it to the catalog.
pub static OBJECTS: &[&lumacore_objects::objects::ObjectDef] = &[&ARTNET];

/// Import Show "x" ArtNet 1: its GUID in the other show.
pub static HOOKS: &[lumacore_objects::commands::registry::Hook] = &[lumacore_objects::commands::registry::Hook::ImportByGuid(&ARTNET, |s, _, id| {
    s.protocols.line("ArtNet", id.parse().ok()?).map(|x| x.meta.identity.guid.to_string())
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
