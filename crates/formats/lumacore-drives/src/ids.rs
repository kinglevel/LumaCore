// ██╗██████╗ ███████╗
// ██║██╔══██╗██╔════╝
// ██║██║  ██║███████╗
// ██║██║  ██║╚════██║
// ██║██████╔╝███████║
// ╚═╝╚═════╝ ╚══════╝
// FILE: ids

//! A drive's id (decisions §38): its kind plus an ordinal — `local-library`; letters for fixed
//! disks (`ssda`, `ssdb`, `hdda`), numbers for the rest (`usb1`, `folder1`, `smb1`) — the next
//! free one of its kind the first time a volume is seen, remembered by the station so the same
//! stick keeps its id.
//!
//! Owns:          the next free id of a kind.
//! Doesn't own:   the table that remembers them (the station settings), the live list (lumacore-state).
//! Entry points:  next_id.
//! Decisions:     §38.

use lumacore_tree::DriveKind;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The next free id of `kind`; `taken(id)` says whether an id is in use (any case).
pub fn next_id(kind: DriveKind, taken: impl Fn(&str) -> bool) -> String {
    let prefix = kind.prefix();
    if kind == DriveKind::LocalLibrary {
        return prefix.to_string();
    }
    if matches!(kind, DriveKind::Ssd | DriveKind::Hdd) {
        for letter in 'a'..='z' {
            let id = format!("{prefix}{letter}");
            if !taken(&id) {
                return id;
            }
        }
    }
    let mut n = 1;
    loop {
        let id = format!("{prefix}{n}");
        if !taken(&id) {
            return id;
        }
        n += 1;
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_disks_take_letters_the_rest_numbers() {
        assert_eq!(next_id(DriveKind::Ssd, |_| false), "ssda");
        assert_eq!(next_id(DriveKind::Ssd, |id| id == "ssda"), "ssdb");
        assert_eq!(next_id(DriveKind::Usb, |id| id == "usb1"), "usb2");
        assert_eq!(next_id(DriveKind::Folder, |_| false), "folder1");
        assert_eq!(next_id(DriveKind::LocalLibrary, |_| true), "local-library");
    }
}
