// ███████╗ █████╗  ██████╗███╗   ██╗
// ██╔════╝██╔══██╗██╔════╝████╗  ██║
// ███████╗███████║██║     ██╔██╗ ██║
// ╚════██║██╔══██║██║     ██║╚██╗██║
// ███████║██║  ██║╚██████╗██║ ╚████║
// ╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═══╝
// FILE: sacn

//! The `Sacn` object — see the crate's README.md (decisions §34 "34f7c hardened"): sACN
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

/// What `Store Sacn` and `Assign` take: the line's declared settings (decisions §34 "34f7c hardened" 7).
const FLAGS: &[lumacore_objects::commands::flags::FlagDef] = &[
    F::NETWORK_FLAG.on(&[Verb::Store]),
    F::MODE.on(&[Verb::Store]),
    F::LOCAL_UNIVERSE.on(&[Verb::Store]),
    F::AMOUNT.on(&[Verb::Store]),
    lumacore_objects::commands::flags::FlagDef::new("SacnUniverse", "The sACN universe LocalUniverse is on the wire; the rest follow on")
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

lumacore_cmd_protocols::dmx_line_verbs!(run, "Sacn");

lumacore_cmd_protocols::line_object!(
    dmx SACN, "Sacn", "SacnUniverse", crate::SACN,
    door: door,
    store: StoreSacn,
    flags: FLAGS,
    doc: {
        syntax: "Store Sacn [<n>] [\"name\"] /Network <name> [/Mode Output|Input] [/LocalUniverse <u> /Amount <n> /SacnUniverse <s>] | Assign Universe <u> Thru <v> At Sacn <n> [/Universe <s>] | Assign Sacn <n> At Universe <u> | Set Sacn <n> \"Priority\"|\"SacnUniverse\"|\"Rate\"|… <value>",
        description: "An sACN (E1.31) line: one DMX Protocols row on its network — Mode Output sends its run of local universes (LocalUniverse and the Amount after it) as sACN universes from SacnUniverse on (multicast, or unicast to Target) at its own Rate; Mode Input brings those sACN universes into the local ones, at its InputPriority. Priority (0–200) is an output's. List Sacn shows each line's run and live state; Info Sacn the protocol.",
        examples: ["Store Sacn 1 \"FOH\" /Network \"Rig\" /LocalUniverse 1 /Amount 32 /SacnUniverse 101", "Assign Universe 1 Thru 4 At Sacn 1 /Universe 101", "Assign Sacn 2 At Universe 40 /Universe 9", "Set Sacn 1 \"Priority\" 150", "Set Sacn 1 \"Rate\" 44", "Unassign Sacn 1"],
    },
    extra_verbs: [(Verb::Assign, lumacore_objects::commands::typed::typed::<run::Assign>),
        (Verb::Unassign, lumacore_objects::commands::typed::typed::<run::Unassign>),],
);

/// This crate's rows in the catalog: the `Sacn` object and its Import Show hook, after its
/// protocol's row and the shared line words (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_sacn::register(r);
    lumacore_cmd_protocols::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.rows(OBJECTS);
    r.objects(OBJECTS.iter().map(|o| &o.decl));
    r.rows(HOOKS);
}

/// The object; `register` adds it to the catalog.
pub static OBJECTS: &[&lumacore_objects::objects::ObjectDef] = &[&SACN];

/// Import Show "x" Sacn 1: its GUID in the other show.
pub static HOOKS: &[lumacore_objects::commands::registry::Hook] = &[lumacore_objects::commands::registry::Hook::ImportByGuid(&SACN, |s, _, id| {
    s.protocols.line("Sacn", id.parse().ok()?).map(|x| x.meta.identity.guid.to_string())
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
