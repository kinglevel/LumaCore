// ██╗     ██╗██████╗
// ██║     ██║██╔══██╗
// ██║     ██║██████╔╝
// ██║     ██║██╔══██╗
// ███████╗██║██████╔╝
// ╚══════╝╚═╝╚═════╝
// FILE: lib

//! lumacore-drives: this station's drives (decisions §38 "Drives and the one files element") —
//! the volumes the OS has mounted (`probe`), the library tree on a drive (`library`), the ids
//! (`ids`), the live list and the remembered table (`table`). The state keeps them (`lumacore_state::drives`),
//! the watcher thread the server's, the Drive object `lumacore-cmd-station`'s. Network drives
//! (SMB, FTP, SFTP as LumaCore's own clients) are planned here.
//!
//! Owns:          the probe, the library tree, the id rule.
//! Doesn't own:   the state, the thread, the object.
//! Entry points:  probe::probe, library::init_library, ids::next_id.
//! Decisions:     §38.

pub mod ids;
pub mod library;
pub mod probe;
pub mod table;
