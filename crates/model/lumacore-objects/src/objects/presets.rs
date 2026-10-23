//! TimingPreset and CurvePreset — DataPool objects (decisions §30 "12f", "Empty pool objects by
//! Store"): `Store TimingPreset 2 ["name"]` creates an empty one, Label names it, Delete removes
//! it, List / Info read it; they live at `Root.ShowData.DataPools.<n>.TimingPresets |
//! CurvePresets`. What they do (a timing preset on a cue, a curve on a fade) and their pool
//! windows come later.

use std::collections::BTreeMap;

use super::{Doc, ObjectDef, TreeDef, Undo, Verb, json_view, out};
use crate::AppState;
use crate::DispatchResult;
use crate::commands::help::HelpCategory;
use crate::commands::registry::{AliasDepth, CommandFamily};
use crate::errors::render_code;
use crate::text::render_success;
use crate::vfs::{DynChild, NodeView};
use lumacore_tree::{CurvePreset, Datapool, Root, TimingPreset};

/// What the two preset kinds share.
trait PoolPreset: serde::Serialize + serde::de::DeserializeOwned + Sized + 'static {
    const WORD: &'static str;
    const NODE_TYPE: &'static str;
    /// Its object (for the resolver and the registry).
    fn object() -> &'static ObjectDef;
    fn guid(&self) -> uuid::Uuid;
    fn empty(name: String) -> Self;
    fn name_mut(&mut self) -> &mut Option<String>;
    fn name(&self) -> &str;
    fn of(dp: &Datapool) -> &BTreeMap<u32, Self>;
    fn of_mut(dp: &mut Datapool) -> &mut BTreeMap<u32, Self>;
}

impl PoolPreset for TimingPreset {
    const WORD: &'static str = "TimingPreset";
    const NODE_TYPE: &'static str = "TimingPreset";
    fn object() -> &'static ObjectDef {
        &TIMING_PRESET
    }
    fn guid(&self) -> uuid::Uuid {
        self.meta.identity.guid
    }
    fn empty(name: String) -> Self {
        Self { meta: lumacore_base::NodeMeta::new(Some(name).filter(|n| !n.is_empty())), fade_in: 0.0, fade_out: 0.0, delay: 0.0 }
    }
    fn name_mut(&mut self) -> &mut Option<String> {
        &mut self.meta.identity.name
    }
    fn name(&self) -> &str {
        self.meta.name().unwrap_or_default()
    }
    fn of(dp: &Datapool) -> &BTreeMap<u32, Self> {
        &dp.timing_presets
    }
    fn of_mut(dp: &mut Datapool) -> &mut BTreeMap<u32, Self> {
        &mut dp.timing_presets
    }
}

impl PoolPreset for CurvePreset {
    const WORD: &'static str = "CurvePreset";
    const NODE_TYPE: &'static str = "CurvePreset";
    fn object() -> &'static ObjectDef {
        &CURVE_PRESET
    }
    fn guid(&self) -> uuid::Uuid {
        self.meta.identity.guid
    }
    fn empty(name: String) -> Self {
        Self { meta: lumacore_base::NodeMeta::new(Some(name).filter(|n| !n.is_empty())), curve_type: lumacore_tree::FadePath::Linear.word().to_string() }
    }
    fn name_mut(&mut self) -> &mut Option<String> {
        &mut self.meta.identity.name
    }
    fn name(&self) -> &str {
        self.meta.name().unwrap_or_default()
    }
    fn of(dp: &Datapool) -> &BTreeMap<u32, Self> {
        &dp.curve_presets
    }
    fn of_mut(dp: &mut Datapool) -> &mut BTreeMap<u32, Self> {
        &mut dp.curve_presets
    }
}

/// The words of a line that aren't options (`/NoConfirmation` …).
fn plain<'a>(args: &[&'a str]) -> Vec<&'a str> {
    args.iter().copied().filter(|a| !a.starts_with('/')).collect()
}

fn in_pool<P: PoolPreset>(state: &AppState) -> Option<&BTreeMap<u32, P>> {
    state.root.show_data.datapools.get(&state.selected_datapool()).map(P::of)
}

/// Store into preset `no` of your DataPool: created empty if missing; a name labels it.
fn store_at<P: PoolPreset>(state: &mut AppState, no: u32, name: Option<String>) -> String {
    if no == 0 {
        return render_code(format!("{} numbers start at 1.", P::WORD), "E-CMD-008");
    }
    let dp = state.selected_datapool();
    let Some(pool) = state.root.show_data.datapools.get_mut(&dp) else {
        return render_code(format!("DataPool {dp} doesn't exist."), "E-CMD-007");
    };
    let map = P::of_mut(pool);
    let shown = |n: &str| if n.is_empty() { String::new() } else { format!(" \"{n}\"") };
    match (map.get_mut(&no), name) {
        (None, name) => {
            let p = P::empty(name.unwrap_or_default());
            let msg = format!("{} {no}{} created — empty.", P::WORD, shown(p.name()));
            map.insert(no, p);
            render_success(&msg)
        }
        (Some(p), Some(n)) => {
            *p.name_mut() = Some(n).filter(|n| !n.is_empty());
            render_success(&format!("{} {no} labeled{}.", P::WORD, shown(p.name())))
        }
        (Some(_), None) => render_code(format!("{} {no} exists — nothing to store.", P::WORD), "E-CMD-009"),
    }
}

/// `List <Preset>s` / `Info <Preset> <n>`.
fn list<P: PoolPreset>(state: &mut AppState, args: &[&str], node: crate::tree_path::Node) -> DispatchResult {
    let words = plain(args);
    if let [no] = words.as_slice() {
        let Some(p) = no.parse::<u32>().ok().and_then(|n| in_pool::<P>(state)?.get(&n)) else {
            return out(render_code(format!("{} {no} not found.", P::WORD), "E-CMD-007"));
        };
        return out(serde_json::to_string_pretty(p).unwrap_or_default());
    }
    out(crate::commands::list::tree(state, node, args))
}

fn children<P: PoolPreset>(map: &BTreeMap<u32, P>) -> Vec<DynChild> {
    map.iter()
        .map(|(&id, p)| DynChild {
            display_name: if p.name().is_empty() { id.to_string() } else { format!("{id} \"{}\"", p.name()) },
            id,
            node_type: P::NODE_TYPE,
            is_leaf: true,
            child_count: 0,
            name: Some(p.name().to_string()).filter(|n| !n.is_empty()),
            guid: Some(p.guid()),
        })
        .collect()
}
fn first_dp(root: &Root) -> Option<u32> {
    root.show_data.datapools.keys().next().copied()
}
fn timing_in(root: &Root, dp: u32) -> Vec<DynChild> {
    root.show_data.datapools.get(&dp).map_or_else(Vec::new, |d| children(&d.timing_presets))
}
fn timing_one(root: &Root, dp: u32, id: u32) -> Option<NodeView> {
    root.show_data.datapools.get(&dp)?.timing_presets.get(&id).map(|p| json_view("TimingPreset", id, p))
}
fn curve_in(root: &Root, dp: u32) -> Vec<DynChild> {
    root.show_data.datapools.get(&dp).map_or_else(Vec::new, |d| children(&d.curve_presets))
}
fn curve_one(root: &Root, dp: u32, id: u32) -> Option<NodeView> {
    root.show_data.datapools.get(&dp)?.curve_presets.get(&id).map(|p| json_view("CurvePreset", id, p))
}

/// A preset's JSON written by the registry (`"Name"`): the one in its DataPool.
fn apply<P: PoolPreset>(root: &mut Root, dp: Option<u32>, id: u32, edit: &mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>) -> Result<(), String> {
    let dp = dp.or_else(|| first_dp(root)).ok_or("no DataPool")?;
    let p = root.show_data.datapools.get_mut(&dp).and_then(|d| P::of_mut(d).get_mut(&id)).ok_or_else(|| format!("no {} {id}", P::WORD))?;
    crate::props::edit_json(p, edit)
}

static TIMING_PROPS: crate::props::Props =
    crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(apply::<TimingPreset>), ..crate::props::Props::BASE };
static CURVE_PROPS: crate::props::Props =
    crate::props::Props { meta: crate::props::MetaLevel::Node, apply: Some(apply::<CurvePreset>), ..crate::props::Props::BASE };

/// A typed Store / Delete of one preset kind (decisions §33 3d): the slot rule and the resolver.
fn store_typed<P: PoolPreset>(state: &mut AppState, target: Option<&lumacore_syntax::ast::Target>, name: Option<String>) -> String {
    use crate::resolve::Slot;
    let (slot, datapool) = match crate::resolve::slot(state, target, P::object()) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let pool = datapool.or(state.datapool_override);
    let before = std::mem::replace(&mut state.datapool_override, pool);
    let next = |s: &AppState| crate::pools::door::next_free(&in_pool::<P>(s).map(|m| m.keys().copied().collect::<Vec<_>>()).unwrap_or_default());
    let text = match slot {
        Slot::Number(n) => store_at::<P>(state, n, name),
        Slot::NewName(n) => store_at::<P>(state, next(state), Some(n)),
        Slot::Next => store_at::<P>(state, next(state), name),
        Slot::Key(k) => render_code(format!("'{k}' isn't a {} number.", P::WORD), "E-CMD-008"),
    };
    state.datapool_override = before;
    text
}

macro_rules! preset_verbs {
    ($store:ident, $ty:ty, $word:literal) => {
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "Store", object = $word)]
        pub struct $store {
            #[lum(target)]
            pub target: Option<lumacore_syntax::ast::Target>,
            #[lum(value)]
            pub name: Option<String>,
        }
        impl crate::commands::typed::Command for $store {
            fn run(self, state: &mut AppState) -> DispatchResult {
                out(store_typed::<$ty>(state, self.target.as_ref(), self.name))
            }
        }
    };
}

preset_verbs!(StoreTimingPreset, TimingPreset, "TimingPreset");
preset_verbs!(StoreCurvePreset, CurvePreset, "CurvePreset");

// The object door (decisions §35): Copy / Cut / Paste / Move / Insert / Compact / Delete.
crate::door_verbs!(timing_door, "TimingPreset", super::TIMING_PRESET);
crate::door_verbs!(curve_door, "CurvePreset", super::CURVE_PRESET);

fn door_map<P: PoolPreset>(state: &mut AppState, dp: Option<u32>) -> Option<&mut BTreeMap<u32, P>> {
    let dp = dp?;
    Some(P::of_mut(state.root.show_data.datapools.entry(dp).or_insert_with(|| Datapool::new(Some("Default".into())))))
}

fn door_map_ref<P: PoolPreset>(state: &AppState, dp: Option<u32>) -> Option<&BTreeMap<u32, P>> {
    state.root.show_data.datapools.get(&dp?).map(P::of)
}

/// A preset kind on the door: nothing links into a preset's own fields, nothing to merge.
macro_rules! preset_door {
    ($static:ident, $ty:ty, $word:literal) => {
        pub static $static: crate::pools::door::BTreePool<$ty> = crate::pools::door::BTreePool {
            object: $word,
            in_datapool: true,

            max: crate::pools::MAX_NO,
            map: door_map::<$ty>,
            map_ref: door_map_ref::<$ty>,
            name: |p| p.meta.name().map(str::to_string),
            identify: |p, guid| p.meta.identity.guid = guid,
            rules: crate::pools::door::PoolRules { can_delete: |_, _| Ok(()), deleted: |_, _| {}, changed: |_| {}, merge: None, orders: &[] },
        };
    };
}

preset_door!(TIMING_DOOR, TimingPreset, "TimingPreset");
preset_door!(CURVE_DOOR, CurvePreset, "CurvePreset");

macro_rules! preset_object {
    ($static:ident, $ty:ty, $word:literal, $aliases:expr, $node:ident, $in:ident, $one:ident, $props:ident, $store:expr, $door:ident, [$($verb:expr),* $(,)?], $ask:expr, $desc:literal, $examples:expr) => {
        pub static $static: ObjectDef = ObjectDef {
            decl: crate::objects::ObjectDecl {
                keyword: $word,
                lum: crate::objects::LumShape::ids(1),
                default_verb: crate::objects::Verb::Info,
                flags: &[crate::commands::flags::OVERWRITE.on(&[Verb::Copy, Verb::Paste]), crate::commands::flags::BY.on(&[Verb::Compact])],
                aliases: $aliases,
                doc: Doc {
                    display: $word,
                    category: HelpCategory::DataPool,
                    also_applies_to: &[],
                    syntax: concat!("Store ", $word, " <n> [\"name\"] | Label ", $word, " <n> \"name\" | Delete ", $word, " <n> | ", $word, " <n>"),
                    description: $desc,
                    examples: $examples,
                    related: &["sequence", "cue", "store"],
                    alias_depth: AliasDepth::Base,
                    hidden: false,
                },
                family: Some(CommandFamily::Playback),
                call_undo: Undo::ReadOnly,
                call_class: crate::commands::registry::CommandClass::Commit,
                scope: crate::objects::on_master,
                reach: crate::objects::show_reach,
                rights: crate::rights::content,
                planned: &[],
            },
            pool: None,
            verbs: &[
                (Verb::List, |s, a| list::<$ty>(s, a, crate::tree_path::Node::$node)),
                (Verb::Info, |s, a| list::<$ty>(s, a, crate::tree_path::Node::$node)),
                (Verb::Store, $store),
                $($verb),*
            ],
            delete_ask: Some($ask),
            complete: None,
            props: Some(&$props),
            tree: Some(TreeDef {
                door: Some(&$door),
                datapool: Some(super::DatapoolTree { children: $in, object: $one }),
                name: None,
                node: crate::tree_path::Node::$node,
                children: |root| first_dp(root).map_or_else(Vec::new, |dp| $in(root, dp)),
                object: |root, id| $one(root, first_dp(root)?, id),
            }),
        };
    };
}

preset_object!(TIMING_PRESET, TimingPreset, "TimingPreset", &["timingpresets"], TimingPresets, timing_in, timing_one, TIMING_PROPS,
    crate::commands::typed::typed::<StoreTimingPreset>, TIMING_DOOR, [
    (Verb::Delete, crate::commands::typed::typed::<timing_door::Delete>),
    (Verb::Copy, crate::commands::typed::typed::<timing_door::Copy>),
    (Verb::Cut, crate::commands::typed::typed::<timing_door::Cut>),
    (Verb::Paste, crate::commands::typed::typed::<timing_door::Paste>),
    (Verb::Move, crate::commands::typed::typed::<timing_door::Move>),
    (Verb::Insert, crate::commands::typed::typed::<timing_door::Insert>),
    (Verb::Compact, crate::commands::typed::typed::<timing_door::Compact>),
    ], |s, a| crate::commands::door::delete_ask(s, &TIMING_PRESET, a),
    "A timing preset of the DataPool (Fade In / Out, Delay). Store creates an empty one; using it on a cue comes later.",
    &["Store TimingPreset 1 \"Slow\"", "Label TimingPreset 1 \"Slow\"", "Delete TimingPreset 1"]);
preset_object!(CURVE_PRESET, CurvePreset, "CurvePreset", &["curvepresets"], CurvePresets, curve_in, curve_one, CURVE_PROPS,
    crate::commands::typed::typed::<StoreCurvePreset>, CURVE_DOOR, [
    (Verb::Delete, crate::commands::typed::typed::<curve_door::Delete>),
    (Verb::Copy, crate::commands::typed::typed::<curve_door::Copy>),
    (Verb::Cut, crate::commands::typed::typed::<curve_door::Cut>),
    (Verb::Paste, crate::commands::typed::typed::<curve_door::Paste>),
    (Verb::Move, crate::commands::typed::typed::<curve_door::Move>),
    (Verb::Insert, crate::commands::typed::typed::<curve_door::Insert>),
    (Verb::Compact, crate::commands::typed::typed::<curve_door::Compact>),
    ], |s, a| crate::commands::door::delete_ask(s, &CURVE_PRESET, a),
    "A curve preset of the DataPool (a fade's shape). Store creates an empty one (Linear); using it comes later.",
    &["Store CurvePreset 1 \"Soft\"", "Label CurvePreset 1 \"Soft\"", "Delete CurvePreset 1"]);

#[cfg(test)]
mod tests {
    use crate::AppState;

    fn run(state: &mut AppState, line: &str) -> String {
        match crate::commands::dispatch(state, line) {
            crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
            crate::DispatchResult::Ask(a) => format!("ASK {}", a.title()),
            _ => String::new(),
        }
    }

    fn demo() -> AppState {
        crate::demo::fresh_state()
    }

    /// `Store Sequence n` (decisions §30 "Empty pool objects by Store"): empty init, a name labels
    /// it, with programmer values the next cue (the last + 1); an existing one with an empty
    /// programmer stays.
    #[test]
    fn store_sequence_creates_it_and_stores_the_next_cue() {
        let mut s = demo();
        let seq = |s: &AppState, n: u32| s.root.show_data.datapools[&1].sequences.get(&n).cloned();
        let r = run(&mut s, "Store Sequence 7 \"Main\"");
        assert!(!r.contains("E-"), "{r}");
        let q = seq(&s, 7).expect("created");
        assert!(q.cues.is_empty());
        assert_eq!(q.meta.identity.name.as_deref(), Some("Main"));
        assert!(run(&mut s, "Store Sequence 7").contains("E-CMD-009"), "an empty programmer stores nothing");
        run(&mut s, "Fixture 101 At 50");
        run(&mut s, "Store Sequence 7");
        run(&mut s, "Fixture 101 At 70");
        run(&mut s, "Store Sequence 7");
        assert_eq!(seq(&s, 7).unwrap().cues.keys().copied().collect::<Vec<_>>(), [1000, 2000]);
        run(&mut s, "Store Sequence 7 Cue 2.5");
        run(&mut s, "Fixture 101 At 20");
        run(&mut s, "Store Sequence 7");
        assert!(seq(&s, 7).unwrap().cues.contains_key(&3500), "the last cue + 1");
        // Unnamed: no name, shown by its number.
        run(&mut s, "Store Sequence 8");
        assert_eq!(seq(&s, 8).unwrap().meta.identity.name, None);
        assert_eq!(seq(&s, 8).unwrap().cues.len(), 1, "values in the programmer: Cue 1");
    }

    /// Store Cue (decisions §30): bare `Store` / `Store <n>` = Store Cue; an empty
    /// programmer makes an empty cue; the next cue is the last + 1; over an existing cue the ask
    /// and its flags; none selected = a new sequence after the highest, selected; a bare
    /// `Store Sequence` = after the highest; OffCue stays last.
    #[test]
    fn store_cue_works_like_a_console() {
        let mut s = demo();
        let seqs = |s: &AppState| s.root.show_data.datapools[&1].sequences.keys().copied().collect::<Vec<_>>();
        let n = seqs(&s).last().copied().unwrap_or(0) + 1;
        let r = run(&mut s, "Store 1");
        assert!(!r.contains("E-"), "{r}");
        assert!(r.contains("created and selected"), "{r}");
        assert_eq!(s.selected_sequence(), Some((1, n)));
        let seq = |s: &AppState| s.root.show_data.datapools[&1].sequences[&n].clone();
        assert_eq!(seq(&s).cues[&1000].values().count(), 0, "an empty cue");
        run(&mut s, "Store");
        run(&mut s, "Fixture 101 At 50");
        let r = run(&mut s, "Store Cue");
        assert!(!r.contains("E-"), "{r}");
        assert_eq!(seq(&s).cues.keys().copied().collect::<Vec<_>>(), [1000, 2000, 3000]);
        assert_eq!(seq(&s).cues[&3000].values().count(), 1);
        // Over an existing cue: the ask with all five; the text names the flags.
        let ask_of = |s: &mut AppState, line: &str| match crate::commands::dispatch(s, line) {
            crate::DispatchResult::Ask(a) => a,
            _ => panic!("{line}: no ask"),
        };
        // An empty cue just takes the values; one holding values asks.
        let r = run(&mut s, "Store 1");
        assert!(r.contains("Stored Cue 1") || r.contains("Merged into Cue 1"), "{r}");
        let ask = ask_of(&mut s, "Store 1");
        assert!(ask.refusal().contains("/CreateSecondCue"), "{}", ask.refusal());
        assert_eq!(ask.line_of("Create Second Cue"), Some(format!("Store Sequence {n} Cue 1 /CreateSecondCue")));
        assert!(ask.line_of("Remove").is_some());
        run(&mut s, &format!("Store Sequence {n} Cue 1 /Merge"));
        assert_eq!(seq(&s).cues[&1000].values().count(), 1, "merging the same value keeps one");
        let r = run(&mut s, "Store 1 /CreateSecondCue");
        assert!(!r.contains("E-"), "{r}");
        assert!(seq(&s).cues.contains_key(&1100), "1 → 1.1");
        run(&mut s, "Store 1 /Remove");
        assert_eq!(seq(&s).cues[&1000].values().count(), 0);
        // An empty programmer over an existing cue: Merge / Overwrite; Overwrite empties it.
        run(&mut s, "ResetAll");
        assert!(s.current_programmer().is_none_or(|p| p.values.values().all(|v| !v.active)), "cleared");
        let ask = ask_of(&mut s, "Store 3");
        assert!(ask.line_of("Remove").is_none() && ask.line_of("Overwrite").is_some());
        run(&mut s, "Store 3 /Overwrite");
        assert_eq!(seq(&s).cues[&3000].values().count(), 0);
        // OffCue is last; nothing before CueZero.
        assert!(run(&mut s, "Store Cue \"OffCue\"").contains("E-CMD-010"));
        assert!(run(&mut s, "Store Cue -1").contains("E-"));
        // A bare Store Sequence: after the highest.
        let r = run(&mut s, "Store Sequence");
        assert!(!r.contains("E-"), "{r}");
        assert!(seqs(&s).contains(&(n + 1)));
    }

    /// `Store Group n` with nothing selected creates an empty group; an existing one is never
    /// emptied by it.
    #[test]
    fn store_group_without_a_selection_creates_an_empty_group() {
        let mut s = demo();
        let r = run(&mut s, "Store Group 400 \"Later\"");
        assert!(!r.contains("E-"), "{r}");
        let g = &s.user_groups()[&400];
        assert!(g.members.is_empty());
        assert_eq!(g.meta.identity.name.as_deref(), Some("Later"));
        run(&mut s, "Fixture 101 Thru 102");
        run(&mut s, "Store Group 400 /Merge");
        run(&mut s, "ResetAll");
        let r = run(&mut s, "Store Group 400");
        assert!(r.contains("E-CMD-009"), "{r}");
        assert_eq!(s.user_groups()[&400].members.len(), 2, "never wiped");
    }

    /// TimingPreset and CurvePreset: Store (empty), a name, Label, the tree, owner first, Delete.
    #[test]
    fn timing_and_curve_presets_store_label_and_delete() {
        let mut s = demo();
        assert!(!run(&mut s, "Store TimingPreset 2 \"Slow\"").contains("E-"));
        assert!(!run(&mut s, "Store CurvePreset 1").contains("E-"));
        let dp = &s.root.show_data.datapools[&1];
        assert_eq!(dp.timing_presets[&2].meta.name(), Some("Slow"));
        assert_eq!(dp.curve_presets[&1].curve_type, "Linear");
        assert!(run(&mut s, "Store TimingPreset 2").contains("E-CMD-009"));
        run(&mut s, "Label CurvePreset 1 \"Soft\"");
        assert_eq!(s.root.show_data.datapools[&1].curve_presets[&1].meta.name(), Some("Soft"));
        assert!(run(&mut s, "List TimingPreset").contains("Slow"));
        assert!(run(&mut s, "List Root.ShowData.DataPools.1.CurvePresets").contains("Soft"));
        // Another DataPool's, owner first.
        s.root.show_data.datapools.insert(2, lumacore_tree::Datapool::new(Some("Second".into())));
        s.root.show_data.after_load();
        assert!(!run(&mut s, "Store DataPool 2 TimingPreset 5").contains("E-"));
        assert!(s.root.show_data.datapools[&2].timing_presets.contains_key(&5));
        assert!(!s.root.show_data.datapools[&1].timing_presets.contains_key(&5));
        assert!(run(&mut s, "Delete TimingPreset 2").starts_with("ASK"));
        run(&mut s, "Delete TimingPreset 2 /NoConfirmation");
        assert!(s.root.show_data.datapools[&1].timing_presets.is_empty());
    }
}
