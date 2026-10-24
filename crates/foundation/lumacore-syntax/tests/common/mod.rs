//! A vocabulary covering the words of lum.md's examples.
//!
//! The crate's own rule vocabulary: the station builds the real one from its registries
//! (`lumacore_objects::lum`) and runs lum.md against it too (`tests/catalog`). This one stays
//! while that test still lists pending lines (§33 part 2), so every example is checked by the
//! parser's rules today.

use lumacore_syntax::{FenceDef, ObjectDef, VerbDef, Vocabulary};

fn counted(id_parts: u8, children: &[&str]) -> ObjectDef {
    ObjectDef { counted: true, id_parts, children: children.iter().map(|c| c.to_string()).collect(), ..ObjectDef::default() }
}

fn single(children: &[&str]) -> ObjectDef {
    ObjectDef { counted: false, id_parts: 0, children: children.iter().map(|c| c.to_string()).collect(), ..ObjectDef::default() }
}

pub fn lum_vocab() -> Vocabulary {
    let mut v = Vocabulary::new();
    for verb in [
        "Assign", "At", "Bind", "Call", "Claim", "Clear", "Copy", "Delete", "Dismiss", "Full", "Go", "Go-", "Goto",
        "Import", "SaveShow", "Info", "Invite", "Label", "Join", "Leave", "List", "LoadShow", "Move", "Off", "On", "Oops", "Park", "Select",
        "SelectFixtures", "Set", "Store", "Toggle", "Unassign", "Unclaim", "Watch", "Zero", "Reset", "Sort", "Split",
        // Decided in §35 (2026-10-15), not built yet.
        "Cut", "Paste", "Insert", "Compact",
        // Decided in §34 round 3 19 (2026-10-18).
        "Clone",
        // Drives (§38): the file words that take /Drive.
        "Export",
        // Presets in cues (decisions 0293) and the Meow License (0290).
        "Update", "Recast", "Extract", "Login", "Consent", "Logout",
        // The paint picker (decisions 0296).
        "Paint",
    ] {
        v.verb(verb, VerbDef::default());
    }
    for hold in ["Top", "Flash", "Temp", "Swap"] {
        v.verb(hold, VerbDef { hold: true });
    }
    v.helper("At").helper("Fade").helper("Delay");
    for value in ["Full", "Zero", "On", "Off", "Toggle", "Yes", "No", "HTP", "LTP", "Alt", "High", "Ask", "Terminal", "Summary", "Night", "Family", "Address", "Houselights", "Channel",
        // MVR import (§34 "Import MVR hardened" 2): the preset's choices.
        "Auto", "UUID", "FID", "FixtureName", "Offer", "Take", "None", "Mark", "Ignore", "Recreate", "Parent",
        // The id types a line can set (decisions §40 7, 10) and what the pool-XML reader takes (§40 13).
        "Generic", "Media", "Fog", "Groups",
        // The palette's colour words (decisions 0294).
        "Red", "Orange", "Amber", "Yellow", "Lime", "Green", "Teal", "Cyan", "Sky", "Blue", "Violet", "Magenta", "Black", "White",
    ] {
        v.value(value);
    }
    v.property("Name");

    // MVR's ID types stand before a fixture's second id (`Fixture Houselights 4`).
    let fixture = ObjectDef { id_kinds: ["Channel", "Generic", "Houselights", "Media", "Effect"].map(String::from).to_vec(), ..counted(1, &["DMX", "Multipatch"]) };
    v.object("Fixture", fixture)
        .object("Group", counted(1, &[]))
        .object("Preset", counted(2, &[]))
        .object("Cue", counted(2, &["FeatureGroup"]))
        .object("Sequence", counted(1, &["Cue"]))
        .object("DataPool", counted(1, &["Group", "Sequence", "Preset"]))
        .object("Master", counted(2, &["Group"]))
        .object("DMX", counted(2, &[]))
        .object("Universe", counted(1, &[]))
        .object("Sacn", counted(1, &[]))
        .object("ArtNet", counted(1, &[]))
        .object("Osc", counted(1, &[]))
        .object("View", counted(1, &[]))
        .object("ViewButton", counted(2, &[]))
        .object("Window", counted(2, &[]))
        .object("UIKey", counted(1, &[]))
        .object("Executor", counted(1, &[]))
        .object("Attribute", counted(1, &[]))
        .object("FixtureType", counted(1, &[]))
        .object("Station", counted(1, &["Network", "Interface"]))
        .object("Network", counted(1, &[]))
        .object("Interface", counted(1, &[]))
        // Drives (§38, decided 2026-10-19, not built): the Drive object, Import Mvr.
        .object("Drive", counted(1, &[]))
        .object("Mvr", counted(1, &[]))
        // The pool-XML reader (decisions §40 13, not built): `Import PoolXml "file" Groups`.
        .object("PoolXml", counted(1, &[]))
        .object("User", counted(1, &["Programmer"]))
        .object("UserProfile", counted(1, &["Theme", "LockScreen", "Attribute", "FeatureGroup"]))
        .object("Theme", single(&["Slot"]))
        .object("Slot", single(&[]))
        .object("LockScreen", single(&[]))
        .object("FeatureGroup", counted(1, &[]))
        .object("Tag", counted(u8::MAX, &["Tag"]))
        .object("AttributeTemplate", counted(1, &[]))
        .object("ShowData", single(&[]))
        .object("Client", counted(1, &[]))
        .object("Shell", counted(1, &[]))
        .object("LumaSession", counted(1, &[]))
        .object("Menu", counted(1, &[]))
        .object("Alias", counted(1, &[]))
        .object("Root", counted(0, &[]))
        .object("Programmer", single(&[]))
        .object("Patch", single(&[]))
        // The MVR tree's words (§34 34g4 patch part 3c).
        .object("MvrLayer", counted(1, &[]))
        .object("MvrGroup", counted(1, &[]))
        .object("MvrClass", counted(1, &[]))
        .object("MvrPosition", counted(1, &[]))
        .object("Multipatch", counted(1, &[]))
        .object("AttributeMap", counted(1, &[]))
        .object("AttributeRule", counted(1, &[]))
        .object("Users", single(&[]))
        // Your tool, Run or Edit (§34 round 3, "RunTool / EditTool").
        .object("Tool", counted(1, &[]))
        // Identify, Probe and Find (§34 round 3): one word, an object owning what it flashes —
        // `Identify Fixture 104` runs its default verb (On), `Off Identify Fixture 104`.
        .object("Identify", single(&["Fixture", "DMX"]))
        .object("Claims", single(&[]));

    v.fence("cd", FenceDef::default())
        .fence("pwd", FenceDef::default())
        .fence("ls", FenceDef { flags: vec![("r".into(), false)] })
        .fence("cat", FenceDef::default())
        .fence("tree", FenceDef { flags: vec![("d".into(), true)] })
        .fence("find", FenceDef { flags: vec![("type".into(), true)] })
        .fence("head", FenceDef { flags: vec![("n".into(), true)] })
        .fence("diff", FenceDef { flags: vec![("all".into(), false)] })
        // Decided in §35 (2026-10-15), not built yet.
        .fence("mv", FenceDef::default())
        .fence("cp", FenceDef::default())
        .fence("rm", FenceDef::default());

    for (option, value) in [
        ("Universe", true),
        ("Count", true),
        ("Mode", true),
        ("Type", true),
        ("Tab", true),
        ("From", true),
        ("Description", true),
        ("Shortcuts", true),
        ("Every", true),
        ("Merge", false),
        ("Overwrite", false),
        ("Keep", false),
        ("NoSave", false),
        ("NoConfirmation", false),
        ("NoOops", false),
        ("Verbose", false),
        ("Debug", false),
        ("Force", false),
        ("WithSubs", false),
        ("R", false),
        ("WithChildren", false),
        ("KeepChildren", false),
        ("By", true),
        // Decided in §34 round 3 19 (2026-10-18).
        ("FixtureType", true),
        ("Revision", true),
        ("Input", false),
        ("KeepSpacing", false),
        ("Swap", false),
        ("Skip", false),
        ("Map", true),
        ("Remember", false),
        ("KeepOutput", false),
        ("NoDependencies", false),
        // Identify's probe the way a fixture would show (§34 round 3, Find).
        ("As", true),
        // Drives (§38): /Drive on every file word, /Path on Store Drive.
        ("Drive", true),
        ("Path", true),
        // MVR import (§34 "Import MVR hardened" 2, decided 2026-10-20, not built).
        ("Scope", true),
        ("Match", true),
        ("Lost", true),
        ("IgnoreRotation", false),
        ("FidOffset", true),
        ("UniverseOffset", true),
        ("X", true),
        ("Y", true),
        ("Z", true),
        ("Rotate", true),
        ("Addresses", true),
        ("Ids", true),
        ("Names", true),
        ("Position", true),
        ("Revisions", true),
        ("NewFixtures", true),
        // The review (decision 3): Apply, Cancel, a cell's Take / Offer.
        ("Apply", false),
        ("Cancel", false),
        ("Take", true),
        ("Offer", true),
        // The placement asks (decision 4).
        ("Gone", true),
        ("Group", true),
        // The id type an untyped CustomId gets (§40 9) and the pool-XML reader's pool (§40 13).
        ("IdType", true),
        ("DataPool", true),
        // Update's content and cue choices (decisions 0293).
        ("Add", false),
        ("Original", false),
        ("Active", false),
        ("Shield", false),
    ] {
        v.option(option, value);
    }
    v
}
