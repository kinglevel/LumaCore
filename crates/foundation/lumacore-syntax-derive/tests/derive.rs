//! `#[derive(Args)]` against a small vocabulary: the signature it writes and the binding.

use lumacore_syntax::ast::{Item, Phase, Target, Value};
use lumacore_syntax::bind::{Need, Role, SlotKind};
use lumacore_syntax::{Args, ObjectDef, OnOff, Origin, VerbDef, Vocabulary, bind, codes, parse};
use lumacore_syntax_derive::Args;

fn vocab() -> Vocabulary {
    let mut v = Vocabulary::new();
    for verb in ["Set", "Redo", "Copy", "Store", "At"] {
        v.verb(verb, VerbDef::default());
    }
    v.verb("Flash", VerbDef { hold: true });
    v.helper("At").helper("Fade");
    for value in ["On", "Off", "Yes", "No", "Full"] {
        v.value(value);
    }
    v.object("Group", ObjectDef { counted: true, id_parts: 1, id_kinds: vec![], children: vec![] })
        .object("Sequence", ObjectDef { counted: true, id_parts: 1, id_kinds: vec![], children: vec![] })
        .object("UserProfile", ObjectDef { counted: true, id_parts: 1, id_kinds: vec![], children: vec![] });
    v.option("Overwrite", false).option("Universe", true).option("NoOops", false);
    v
}

fn run<A: Args>(src: &str) -> Result<A, &'static str> {
    let v = vocab();
    let line = parse(src, &v, Origin::Typed);
    assert!(line.is_clean(), "{src}: {:?}", line.diags);
    bind::<A>(&line.statements[0], &v, &["NoOops"]).map_err(|d| d.code)
}

#[derive(Args, Debug)]
#[lum(verb = "Set")]
struct SetProperty {
    #[lum(target)]
    target: Target,
    #[lum(property)]
    property: String,
    #[lum(value)]
    value: OnOff,
}

#[derive(Args, Debug)]
#[lum(verb = "Redo")]
struct Redo {
    #[lum(value)]
    steps: Option<u32>,
}

#[derive(Args, Debug)]
#[lum(verb = "Copy", object = "Group")]
struct CopyGroup {
    #[lum(target)]
    groups: Target,
    #[lum(at = "destination")]
    to: Value,
    #[lum(option = "Overwrite")]
    overwrite: bool,
    #[lum(option = "Universe")]
    universe: Option<Value>,
}

#[derive(Args, Debug)]
#[lum(verb = "Flash")]
struct Flash {
    #[lum(phase)]
    phase: Phase,
    #[lum(target)]
    target: Option<Target>,
}

#[test]
fn the_struct_is_the_signature() {
    let sig = SetProperty::SIGNATURE;
    assert_eq!(sig.verb, "Set");
    assert_eq!(sig.target, Need::Required);
    assert_eq!(sig.slots.iter().map(|s| (s.kind, s.required)).collect::<Vec<_>>(), [(SlotKind::Property, true), (SlotKind::Value, true)]);
    assert_eq!(Redo::SIGNATURE.target, Need::Never);
    assert!(!Redo::SIGNATURE.slots[0].required);
    assert_eq!(CopyGroup::SIGNATURE.helpers[0].role, Role::Destination);
    assert_eq!(CopyGroup::SIGNATURE.usage(), "Copy Group <ids> At <to> [/Overwrite] [/Universe <value>]");
    assert_eq!(SetProperty::SIGNATURE.usage(), "Set <target> \"<property>\" <value>");
}

#[test]
fn trailing_slots_bind_from_the_right() {
    // Decision 17: your own profile's ClearBuffer, even if a profile were named "ClearBuffer".
    let own = run::<SetProperty>("Set UserProfile \"ClearBuffer\" On").unwrap();
    assert_eq!(own.property, "ClearBuffer");
    assert_eq!(own.value, OnOff(true));
    let Item::Path(segments) = &own.target.items[0].1 else { panic!() };
    assert!(segments[0].ids.is_none(), "the bare owner = your own");
    let named = run::<SetProperty>("Set UserProfile \"LD\" \"ClearBuffer\" Off").unwrap();
    let Item::Path(segments) = &named.target.items[0].1 else { panic!() };
    assert!(segments[0].ids.is_some());
    assert_eq!(named.value, OnOff(false));
}

#[test]
fn bare_ids_fill_a_verb_without_a_target() {
    assert_eq!(run::<Redo>("Redo").unwrap().steps, None);
    assert_eq!(run::<Redo>("Redo 3").unwrap().steps, Some(3));
    assert_eq!(run::<Redo>("Redo 1.5").unwrap_err(), codes::VALUE_WORD);
    assert_eq!(run::<Redo>("Redo 3 4").unwrap_err(), codes::UNEXPECTED);
    assert_eq!(run::<Redo>("Redo Group 3").unwrap_err(), codes::UNEXPECTED);
}

#[test]
fn values_options_and_helpers_are_checked() {
    assert_eq!(run::<SetProperty>("Set UserProfile \"ClearBuffer\" Full").unwrap_err(), codes::VALUE_WORD, "E-SYN-014");
    assert_eq!(run::<SetProperty>("Set Group 3 \"Name\"").unwrap_err(), codes::MISSING);
    let copy = run::<CopyGroup>("Copy Group 1 At 2 /Overwrite /Universe 101 /NoOops").unwrap();
    assert!(copy.overwrite && !copy.groups.items.is_empty());
    assert!(matches!(copy.universe, Some(Value::Number(_))));
    assert!(matches!(copy.to, Value::Number(_)));
    assert!(!run::<CopyGroup>("Copy Group 1 At 2").unwrap().overwrite);
    assert_eq!(run::<CopyGroup>("Copy Group 1 At Full").unwrap_err(), codes::VALUE_WORD, "At here is where to");
    assert_eq!(run::<CopyGroup>("Copy Group 1").unwrap_err(), codes::MISSING);
    assert_eq!(run::<CopyGroup>("Copy Group 1 At 2 Fade 3").unwrap_err(), codes::UNEXPECTED);
    assert_eq!(run::<SetProperty>("Set Group 3 \"Name\" On /Overwrite").unwrap_err(), codes::OPTION);
}

#[test]
fn a_hold_verb_binds_its_phase() {
    let flash = run::<Flash>("Flash Off Sequence 1").unwrap();
    assert_eq!(flash.phase, Phase::Off);
    assert!(flash.target.is_some());
}
