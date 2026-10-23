//! The parser's shapes and refusals, one rule of decisions §33 each.

mod common;

use lumacore_syntax::ast::{Body, FenceArg, Id, Item, Range, Slot, Value};
use lumacore_syntax::{Expect, Line, Origin, at_cursor, codes, parse, print};

fn typed(src: &str) -> Line {
    parse(src, &common::lum_vocab(), Origin::Typed)
}

fn codes_of(line: &Line) -> Vec<&'static str> {
    line.diags.iter().map(|d| d.code).collect()
}

fn printed(src: &str) -> String {
    print(&typed(src), &common::lum_vocab())
}

fn lum(line: &Line) -> &lumacore_syntax::ast::Lum {
    match &line.statements[0].body {
        Body::Lum(l) => l,
        other => panic!("not a Lum statement: {other:?}"),
    }
}

#[test]
fn an_owner_first_path_nests_and_a_child_first_one_is_refused() {
    let line = typed("Goto Sequence 2 Cue 3");
    assert!(line.is_clean());
    let target = lum(&line).target.as_ref().unwrap();
    let Item::Path(segments) = &target.items[0].1 else { panic!() };
    assert_eq!(segments.len(), 2);
    assert_eq!(codes_of(&typed("Goto Cue 3 Sequence 2")), vec![codes::CHILD_FIRST]);
    assert_eq!(codes_of(&typed("Group 1 DataPool 1")), vec![codes::CHILD_FIRST]);
}

#[test]
fn another_object_after_a_path_is_a_slot() {
    let line = typed("Park Fixture 1 Attribute \"Pan\" At 30");
    assert!(line.is_clean());
    let l = lum(&line);
    assert!(matches!(l.slots[0], Slot::Value(Value::Target(_))));
    assert!(matches!(l.slots[1], Slot::Helper { .. }));
}

#[test]
fn a_one_of_a_kind_object_as_a_value_takes_a_name() {
    // decisions 0294: `named-one = OBJECT QUOTED`, a theme paint slot.
    let line = typed("Set Group 3 \"Paint\" Theme \"Position\"");
    assert!(line.is_clean(), "{:?}", line.diags);
    let l = lum(&line);
    let Some(Slot::Value(Value::Target(t))) = l.slots.last() else { panic!("{:?}", l.slots) };
    let Item::Path(segments) = &t.items[0].1 else { panic!() };
    let ranges = &segments[0].ids.as_ref().expect("the slot's name").ranges;
    assert!(matches!(&ranges[0].1, Range::One(Id::Name(n)) if n.node == "Position"));
    assert_eq!(printed("set group 3 \"Paint\" theme \"Special 1\""), "Set Group 3 \"Paint\" Theme \"Special 1\"");
    // In front of the line it stays a target and its property: `Set Theme "Base" Night`.
    let own = typed("Set Theme \"Base\" Night");
    let Item::Path(segments) = &lum(&own).target.as_ref().unwrap().items[0].1 else { panic!() };
    assert!(segments[0].ids.is_none());
}

#[test]
fn ids_follow_the_one_range_rule() {
    let line = typed("Group 1 Thru 4 + 7 - 3");
    let Item::Path(segments) = &lum(&line).target.as_ref().unwrap().items[0].1 else { panic!() };
    let ranges = &segments[0].ids.as_ref().unwrap().ranges;
    assert_eq!(ranges.len(), 3);
    assert!(matches!(ranges[0].1, Range::Thru { from: Some(_), to: Some(_), .. }));
    assert!(matches!(&typed("Fixture 3 Thru").statements[0].body, Body::Lum(_)));
    assert!(typed("Fixture Thru 4").is_clean());
    assert!(typed("Fixture 5 Thru 2").is_clean());
}

#[test]
fn a_glued_operator_in_a_list_is_refused() {
    assert_eq!(codes_of(&typed("Group 1+3")), vec![codes::GLUED_OPERATOR]);
    assert_eq!(codes_of(&typed("Group 1 Thru 5-2")), vec![codes::GLUED_OPERATOR]);
    assert_eq!(codes_of(&typed("Group 1 +3")), vec![codes::GLUED_OPERATOR]);
    assert!(typed("Group 1 + 3").is_clean());
}

#[test]
fn a_glued_sign_is_the_number_a_spaced_one_is_relative() {
    let absolute = typed("Attribute \"Pan\" At -120");
    let Slot::Helper { value: Some(Value::Number(n)), .. } = &lum(&absolute).slots[0] else { panic!() };
    assert_eq!(n.node, "-120");
    let relative = typed("At - 10");
    assert!(matches!(lum(&relative).slots[0], Slot::Value(Value::Relative { .. })));
    assert_eq!(codes_of(&typed("At -")), vec![codes::MISSING]);
}

#[test]
fn a_dot_nests_only_in_root_paths_and_self_reads() {
    assert!(typed("Select Root.ShowData.DataPools.2").is_clean());
    assert!(typed("Set Root.ShowData.Stations.\"Tower\".Networks.\"Rig\" \"Interface\" \"en1\"").is_clean());
    let read = typed("Set Group 3 \"Name\" Self.DataPool.Name");
    assert!(matches!(&lum(&read).slots[1], Slot::Value(Value::Read(p)) if p.node == ["DataPool", "Name"]));
    assert_eq!(codes_of(&typed("DataPool.2.Group.3")), vec![codes::UNEXPECTED]);
}

#[test]
fn numbers_take_their_object_s_parts() {
    assert!(typed("Cue 1.5").is_clean());
    assert!(typed("Master 2.1 At 80").is_clean());
    assert_eq!(codes_of(&typed("Fixture 1.2")), vec![codes::ID_PARTS]);
}

/// A kind the object declares stands before its ids (`Fixture Houselights 4`, decisions §34
/// round 3 19); without ids after it the word is what it is elsewhere.
#[test]
fn an_id_kind_stands_before_the_ids() {
    use lumacore_syntax::ast::Item;
    let line = typed("Fixture houselights 1 Thru 4 + Fixture 101");
    assert!(line.is_clean(), "{:?}", line.diags);
    let Some(target) = &lum(&line).target else { panic!("a target") };
    let Item::Path(first) = &target.items[0].1 else { panic!("a path") };
    assert_eq!(first[0].kind.as_ref().map(|k| k.node.as_str()), Some("Houselights"));
    assert_eq!(printed("Fixture houselights 1 Thru 4 + Fixture 101"), "Fixture Houselights 1 Thru 4 + Fixture 101");
    let Item::Path(second) = &target.items[1].1 else { panic!("a path") };
    assert!(second[0].kind.is_none());
}

#[test]
fn property_names_are_quoted() {
    assert_eq!(codes_of(&typed("Set Fixture 3 Name \"Spot SR\"")), vec![codes::BARE_PROPERTY]);
    assert!(typed("Set Fixture 3 \"Name\" \"Spot SR\"").is_clean());
}

#[test]
fn unknown_words_name_the_closest() {
    let line = typed("Store Grou 3");
    assert_eq!(codes_of(&line), vec![codes::UNKNOWN_WORD]);
    assert!(line.diags[0].message.contains("Group"), "{}", line.diags[0].message);
    assert_eq!(codes_of(&typed("Sto Group 3")), vec![codes::UNKNOWN_WORD], "no prefix guessing");
}

#[test]
fn hold_verbs_take_on_or_off() {
    assert!(typed("Flash On Sequence 1").is_clean());
    assert_eq!(codes_of(&typed("Flash Sequence 1")), vec![codes::HOLD_PHASE]);
}

#[test]
fn options_come_last_and_take_declared_values() {
    let line = typed("Store Window 5 /Type Terminal /Overwrite");
    assert!(line.is_clean());
    assert_eq!(lum(&line).options.len(), 2);
    assert_eq!(codes_of(&typed("Store Group 4 /Bogus")), vec![codes::OPTION]);
    assert_eq!(codes_of(&typed("Store Group 4 /Type")), vec![codes::MISSING]);
    assert_eq!(codes_of(&typed("Store Group 4 /Overwrite 3")), vec![codes::UNEXPECTED]);
}

#[test]
fn if_is_mas_if_and_an_early_one_prints_at_the_end() {
    assert!(typed("If Group 5").is_clean());
    assert!(typed("Delete Cue 3 If Fixture 4").is_clean());
    assert_eq!(printed("Store If Group 5 EndIf Preset 1.1"), "Store Preset 1.1 If Group 5");
    assert_eq!(codes_of(&typed("Store If Group 5 Preset 1.1")), vec![codes::MISSING]);
}

#[test]
fn print_writes_one_spelling() {
    assert_eq!(printed("store   group 3   \"Front\""), "Store Group 3 \"Front\"");
    assert_eq!(printed("go- sequence 2"), "Go- Sequence 2");
    assert_eq!(printed("fixture 1 thru 4 at full ;at 50"), "Fixture 1 Thru 4 At Full ; At 50");
    assert_eq!(printed("select root.ShowData.DataPools.2"), "Select Root.ShowData.DataPools.2");
}

#[test]
fn stored_and_code_lines_are_exact() {
    let vocab = common::lum_vocab();
    let stored = |src| parse(src, &vocab, Origin::Stored).diags.iter().map(|d| d.code).collect::<Vec<_>>();
    assert_eq!(stored("!St 3"), vec![codes::PERSONAL_IN_STORED]);
    assert_eq!(stored("3 At 50"), vec![codes::PERSONAL_IN_STORED]);
    assert!(stored("Store 2").is_empty(), "a verb's default object isn't personal");
    assert!(stored("cd ..").is_empty(), "users may write the fence in macros");
    let code = parse("cd ..", &vocab, Origin::Code);
    assert_eq!(code.diags.iter().map(|d| d.code).collect::<Vec<_>>(), vec![codes::FENCE_IN_CODE]);
}

#[test]
fn aliases_hints_and_the_fence() {
    let alias = typed("!sg 3 \"Front\"");
    assert!(matches!(&alias.statements[0].body, Body::Alias { name, rest } if name.node == "sg" && rest.node == "3 \"Front\""));
    assert!(matches!(&typed("store?").statements[0].body, Body::Hint { word } if word.node == "store"));
    let fence = typed("find \"Front\" --type Group");
    let Body::Fence { args, .. } = &fence.statements[0].body else { panic!() };
    assert!(matches!(&args[1], FenceArg::Flag { name, long: true, value: Some(v) } if name.node == "type" && v.node == "Group"));
    assert_eq!(codes_of(&typed("ls --bogus")), vec![codes::OPTION]);
}

#[test]
fn a_quoted_word_is_text_never_a_keyword() {
    let line = typed("Store \"Group\" 3");
    let Some((_, Item::Ids(ids))) = lum(&line).target.as_ref().map(|t| &t.items[0]) else { panic!() };
    assert!(matches!(&ids.ranges[0].1, Range::One(Id::Name(n)) if n.node == "Group"));
    assert_eq!(codes_of(&typed("Label Group 1 \"open")), vec![codes::UNTERMINATED]);
}

#[test]
fn the_cursor_knows_what_comes_next() {
    let line = typed("Store Gr");
    let cursor = at_cursor(&line, 8);
    assert_eq!(cursor.partial, "Gr");
    assert!(cursor.expected.contains(&Expect::Object));
    let after_verb = at_cursor(&typed("Flash "), 6);
    assert_eq!(after_verb.expected.first(), Some(&Expect::Phase), "a hold verb asks for On / Off first");
    let group = common::lum_vocab().lookup("DataPool").unwrap();
    let in_path = at_cursor(&typed("DataPool 2 "), 11);
    assert!(in_path.expected.contains(&Expect::Child(group)));
}

#[test]
fn statements_split_at_semicolons_and_empty_ones_vanish() {
    let line = typed("Store Group 3 ; ; Oops");
    assert!(line.is_clean());
    assert_eq!(line.statements.len(), 2);
}

/// Decision 91: a command line as a value is in backticks; its quotes stay plain.
#[test]
fn a_line_value_is_in_backticks() {
    let line = typed("Bind \"F5\" `Set Self \"AutoScroll\" Toggle`");
    assert!(line.is_clean(), "{:?}", line.diags);
    assert!(lum(&line).slots.iter().any(|s| matches!(s, Slot::Value(Value::Line(l)) if l.node == "Set Self \"AutoScroll\" Toggle")));
    assert_eq!(codes_of(&typed("Bind \"F5\" `List Fixture")), vec![codes::UNTERMINATED]);
}

/// Decisions 0295: lines in a row are one value, a key's press cycle; it prints back as written.
#[test]
fn lines_in_a_row_are_one_value() {
    let line = typed("Bind \"E\" `Edit` `EditSetting` /Shortcuts \"PRG\"");
    assert!(line.is_clean(), "{:?}", line.diags);
    let rows: Vec<&str> = lum(&line)
        .slots
        .iter()
        .find_map(|s| match s {
            Slot::Value(Value::Lines(ls)) => Some(ls.iter().map(|l| l.node.as_str()).collect()),
            _ => None,
        })
        .expect("one Lines value");
    assert_eq!(rows, ["Edit", "EditSetting"]);
    assert_eq!(printed("Bind \"E\" `Edit` `EditSetting`"), "Bind \"E\" `Edit` `EditSetting`");
}
