//! A Lum line (decisions §33): parse, the cursor at every offset, highlight and print never
//! panic, and a clean line prints to one spelling that parses back the same.
#![no_main]

use lumacore_syntax::{FenceDef, ObjectDef, Origin, VerbDef, Vocabulary, at_cursor, highlight, parse, print};
use std::sync::OnceLock;

fn vocab() -> &'static Vocabulary {
    static V: OnceLock<Vocabulary> = OnceLock::new();
    V.get_or_init(|| {
        let mut v = Vocabulary::new();
        for verb in ["Store", "At", "Set", "Go", "Go-", "Copy", "Delete", "On", "Off", "Full"] {
            v.verb(verb, VerbDef::default());
        }
        v.verb("Flash", VerbDef { hold: true }).helper("At").value("Full").value("On").value("Off").property("Name");
        let obj = |parts, children: &[&str]| ObjectDef { counted: true, id_parts: parts, children: children.iter().map(|c| c.to_string()).collect() };
        v.object("Fixture", obj(1, &[])).object("Group", obj(1, &[])).object("Cue", obj(2, &[]));
        v.object("Sequence", obj(1, &["Cue"])).object("DataPool", obj(1, &["Group", "Sequence"]));
        v.object("Blind", ObjectDef::default());
        v.fence("cd", FenceDef::default()).fence("ls", FenceDef { flags: vec![("r".into(), false), ("n".into(), true)] });
        v.option("Merge", false).option("Type", true);
        v
    })
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else { return };
    let v = vocab();
    for origin in [Origin::Typed, Origin::Stored, Origin::Code] {
        let line = parse(src, v, origin);
        let _ = highlight(&line, v);
        for i in (0..=src.len()).filter(|&i| src.is_char_boundary(i)) {
            let _ = at_cursor(&line, i);
        }
        if line.is_clean() {
            let once = print(&line, v);
            let again = parse(&once, v, origin);
            assert!(again.is_clean(), "{src:?} printed as {once:?}: {:?}", again.diags);
            assert_eq!(print(&again, v), once);
        }
    }
});
