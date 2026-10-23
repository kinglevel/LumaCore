// ███╗   ██╗███████╗████████╗
// ████╗  ██║██╔════╝╚══██╔══╝
// ██╔██╗ ██║█████╗     ██║
// ██║╚██╗██║██╔══╝     ██║
// ██║ ╚████║███████╗   ██║
// ╚═╝  ╚═══╝╚══════╝   ╚═╝
// FILE: net

//! The station's leaf network I/O — see the crate's README.md (decisions §34 20.6, 22).

pub mod discovery;
pub mod mdns;
pub mod nics;
pub mod ptp;
pub mod socket;

pub use discovery::DiscoveryPeerState;
pub use nics::Nic;
