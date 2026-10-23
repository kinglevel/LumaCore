// ███████╗██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗╚════██║
// ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║███████║
// ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
// FILE: errors

//! The show file's and the media store's error codes (decisions §26, §34 25.10).
//!
//! Owns:          the show file's and the media store's error codes.
//! Doesn't own:   the other catalogs (each crate's errors).
//! Entry points:  ERRORS.
//! Decisions:     §26, §34 25.10.

use lumacore_base::ErrorInfo;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Show file and media errors.
pub const ERRORS: &[ErrorInfo] = &[
    ErrorInfo {
        code: "E-SHOW-001",
        title: "Show from a newer format",
        explain: "The show file was saved by a newer LumaCore whose show format this build doesn't know. Shows upgrade only forward (never to an older format): open it with that build or newer. The message names the format version and commit that saved it.",
    },
    ErrorInfo {
        code: "E-SHOW-002",
        title: "Show from an older format",
        explain: "The show file was saved in a show format older than the oldest this build upgrades (`OLDEST_FORMAT`, decisions 0289). Newer old shows open upgraded step by step; this one needs a build that still reads its format, or re-creating.",
    },
    ErrorInfo {
        code: "E-SHOW-003",
        title: "Show objects lack a unique GUID",
        explain: "Every object in a show has its own GUID (decisions §21 \"6d\"); merges, links and claims rely on it. The file has an object without one, or two objects share one, so it was not loaded. The message names the paths.",
    },
    ErrorInfo {
        code: "E-SHOW-004",
        title: "Won't overwrite a newer show file",
        explain: "The show file you save over was written by a newer LumaCore (a newer show format). Saving it with this build would turn it into an older format that the newer build refuses (E-SHOW-002), so the show was saved as `{name}_format{N}_{time}` instead and that copy is now loaded. `SaveShow \"name\" /Overwrite` (or the Overwrite button of the pop-up) replaces the file anyway (decisions §23 \"App ↔ local server hardening\").",
    },
    ErrorInfo {
        code: "E-SHOW-005",
        title: "Show file is damaged",
        explain: "A part of the show file doesn't match the SHA-256 its `manifest.json` lists, or a part is missing (decisions §30 \"12a2\": every entry of a `.luma` is listed with its hash). The file was changed or cut off after it was saved; load an autosave or a backup. A picture that doesn't match is only left out (its object stays, without its file).",
    },
    ErrorInfo {
        code: "E-SHOW-006",
        title: "Startup show skipped after a crash",
        explain: "The last start died while it loaded the startup show (an abort, out of memory or a kill: the `.starting` marker in the station's folder was still there), so this start skipped it (R1-17). When a recovery copy of it is newer than the file, the recovery ask offers it next. Load the show by hand with LoadShow, or pick another startup show; if it kills the station again, open one of its versions (`List Version \"x\"`, `LoadShow \"x\" /Version <stamp>`).",
    },
    ErrorInfo {
        code: "E-SHOW-007",
        title: "Show names a protocol this build lacks",
        explain: "A line of the show speaks a protocol this build doesn't have (each protocol is a crate the build includes, decisions §34 22). The show is refused rather than loaded with lines that would stay silent: open it with a build that has the protocol. The message names the line and this build's protocols.",
    },
    ErrorInfo {
        code: "E-SHOW-008",
        title: "Show has a setting its protocol doesn't declare",
        explain: "A line of the show carries a setting (`Line.settings`) its protocol doesn't declare in this build — written by a build whose protocol had it (decisions §34 \"34f7b hardening\" 5). The show is refused rather than dropping the setting: open it with that build. The message names the line and the setting.",
    },
    ErrorInfo {
        code: "E-MEDIA-001",
        title: "No such file to import",
        explain: "Pictures are imported from the station's `library/media/` folder (a browser upload lands there too): name a file inside it, without `..` or an absolute path (decisions §30 \"12a2\").",
    },
    ErrorInfo {
        code: "E-MEDIA-002",
        title: "Not a picture",
        explain: "Images and icons may be PNG, JPEG, WebP, GIF or SVG, and their header must be readable (the size is taken from it).",
    },
    ErrorInfo {
        code: "E-MEDIA-003",
        title: "File too large to embed",
        explain: "A file the show embeds (image, icon, GDTF) may be 16 MB at most, so a show saves fast, joins a LumaSession quickly and fits undo (decisions §30 \"12a2\"). Large show-critical media belongs on an asset drive: the show names an `AssetRoot`, each station maps it to a folder (§8).",
    },
    ErrorInfo {
        code: "E-MEDIA-004",
        title: "Show embeds too much",
        explain: "A show may embed 256 MB in all (pictures and GDTF files, each counted once). Remove pictures you don't use, or put large media on an asset drive (`AssetRoot`, §8).",
    },
];
