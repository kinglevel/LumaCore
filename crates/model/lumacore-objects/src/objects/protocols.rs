// ██████╗ ██████╗  ██████╗ ████████╗ ██████╗  ██████╗ ██████╗ ██╗     ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔════╝██╔═══██╗██║     ██╔════╝
// ██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ███████╗
// ██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██║     ██║   ██║██║     ╚════██║
// ██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╗╚██████╔╝███████╗███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚══════╝╚══════╝
// FILE: protocols

//! The protocol words' shared pieces (decisions §34 25.17, "34f7c hardened"): the Network is
//! `lumacore-cmd-station`'s, the lines each protocol's own crate's (on `lumacore-cmd-protocols`);
//! they name these, so they stay in the kernel.
//!
//! Owns:          what the lines and the Network share below their crates: the Store options and a network by number or name.
//! Doesn't own:   the Network (lumacore-cmd-station), the lines (lumacore-cmd-protocols and each protocol's crate).
//! Entry points:  NETWORK_FLAG, USER, PORT, MODE, LOCAL_UNIVERSE, AMOUNT, network_id.
//! Decisions:     §11, §31, §34 25.17, "34f7c hardened".

use crate::commands::flags as F;
use super::Verb;
use lumacore_tree::{Root, TreeNode};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The options a line's Store takes are its declared settings (decisions §34 "34f7c hardened" 7):
/// each protocol's object lists the ones it has (`Store Sacn 1 /Network "Rig" /LocalUniverse 1
/// /Amount 32 /SacnUniverse 101`), with its own protocol-universe option beside these.
pub const NETWORK_FLAG: F::FlagDef = F::FlagDef::new("Network", "The network it is on").value("network");
pub const USER: F::FlagDef = F::FlagDef::new("User", "The user its commands run as (Data lines: OSC)").value("user");
pub const PORT: F::FlagDef = F::FlagDef::new("Port", "The UDP port it listens on (OSC)").value("port");
pub const MODE: F::FlagDef = F::FlagDef::new("Mode", "Output (local universes go out) or Input (the protocol's come in)").value("mode");
pub const LOCAL_UNIVERSE: F::FlagDef = F::FlagDef::new("LocalUniverse", "The first local universe the line carries").value("universe");
pub const AMOUNT: F::FlagDef = F::FlagDef::new("Amount", "How many local universes from LocalUniverse").value("count");

/// Store Network's options.
pub const STORE_NETWORK_FLAGS: &[F::FlagDef] = &[NETWORK_FLAG.on(&[Verb::Store])];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Network <n|"name">` → the network's id.
pub fn network_id(root: &Root, word: &str) -> Option<u32> {
    let nets = &root.show_data.protocols.networks;
    word.parse::<u32>()
        .ok()
        .filter(|id| nets.contains_key(id))
        .or_else(|| {
            let word = word.trim_matches('"');
            nets.iter()
                .find(|(_, n)| n.name().is_some_and(|n| n.eq_ignore_ascii_case(word)))
                .map(|(&id, _)| id)
        })
}
