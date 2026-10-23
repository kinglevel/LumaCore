//! Selection resolver: converts a `SelectionExpr` AST into concrete fixture IDs.
//!
//! Uses live ShowData (fixtures + the user's datapool's groups, decisions §22)
//! to resolve names, ranges, and groups.

use lumacore_tree::programmer::Programmer;
use lumacore_tree::{Group, TreeNode};
use lumacore_fixtures::Fixture;
use std::borrow::Borrow;
use std::collections::BTreeMap;

use super::{IfPredicate, SelOp, SelectionExpr};

/// Resolve a `SelectionExpr` into fixture IDs in selection order (`6 Thru 1`
/// counts down; a fixture named twice is kept where it first appears).
///
/// - `ById(n)` — returns `[n]` if that fixture exists, else `[]`.
/// - `ByName(s)` — case-insensitive name match across all fixtures.
/// - `Range(a, b)` — all fixture IDs in the BTreeMap where `a <= id <= b`.
/// - `Add(l, r)` — union (deduplicated, sorted).
/// - `Sub(l, r)` — set difference.
/// - `Group(n)` — the members of that group (the user's datapool), in order.
/// - `GroupByName(s)` — case-insensitive name match among those groups.
/// - `All` — all fixture IDs from the map.
pub fn resolve_selection<F: Borrow<Fixture>>(
    expr: &SelectionExpr,
    fixtures: &BTreeMap<u32, F>,
    groups: &BTreeMap<u32, Group>,
) -> Vec<u32> {
    let mut seen = std::collections::HashSet::new();
    resolve_inner(expr, fixtures, groups)
        .into_iter()
        .filter(|id| seen.insert(*id))
        .collect()
}

fn resolve_inner<F: Borrow<Fixture>>(
    expr: &SelectionExpr,
    fixtures: &BTreeMap<u32, F>,
    groups: &BTreeMap<u32, Group>,
) -> Vec<u32> {
    match expr {
        SelectionExpr::ById(id) => {
            if fixtures.contains_key(id) {
                vec![*id]
            } else {
                vec![]
            }
        }

        SelectionExpr::ByName(name) => {
            let lower = name.to_lowercase();
            fixtures
                .iter()
                .filter(|(_, f)| Borrow::<Fixture>::borrow(*f).name().map(|n| n.to_lowercase() == lower).unwrap_or(false))
                .map(|(&id, _)| id)
                .collect()
        }

        SelectionExpr::Range(a, b) if a <= b => fixtures.range(*a..=*b).map(|(&id, _)| id).collect(),
        SelectionExpr::Range(a, b) => fixtures.range(*b..=*a).rev().map(|(&id, _)| id).collect(),

        SelectionExpr::Chain(first, rest) => {
            let mut ids = resolve_inner(first, fixtures, groups);
            for (op, term) in rest {
                let term_ids = resolve_inner(term, fixtures, groups);
                match op {
                    SelOp::Add => ids.extend(term_ids),
                    SelOp::Sub => {
                        let gone: std::collections::HashSet<u32> = term_ids.into_iter().collect();
                        ids.retain(|id| !gone.contains(id));
                    }
                }
            }
            ids
        }

        SelectionExpr::Group(gid) => groups
            .get(gid)
            .map(|g| guids_to_ids(&g.fixtures(), fixtures))
            .unwrap_or_default(),

        SelectionExpr::GroupByName(name) => {
            let lower = name.to_lowercase();
            find_group_by_name(&lower, groups)
                .map(|g| guids_to_ids(&g.fixtures(), fixtures))
                .unwrap_or_default()
        }

        SelectionExpr::All => fixtures.keys().copied().collect(),
    }
}

/// Convert a list of fixture GUIDs to user-facing integer IDs using the fixtures BTreeMap.
fn guids_to_ids<F: Borrow<Fixture>>(guids: &[uuid::Uuid], fixtures: &BTreeMap<u32, F>) -> Vec<u32> {
    guids
        .iter()
        .filter_map(|guid| {
            fixtures
                .iter()
                .find(|(_, f)| Borrow::<Fixture>::borrow(*f).meta.identity.guid == *guid)
                .map(|(&id, _)| id)
        })
        .collect()
}

/// Find a group by name (case-insensitive).
fn find_group_by_name<'a>(lower_name: &str, groups: &'a BTreeMap<u32, Group>) -> Option<&'a Group> {
    groups.values().find(|g| {
        g.name()
            .map(|n| n.to_lowercase() == lower_name)
            .unwrap_or(false)
    })
}

/// Apply `If` filter predicates to a list of fixture IDs.
///
/// Each predicate narrows the list (logical AND between predicates).
/// - `InGroup(name)` — keeps only fixtures that are members of any group with that name.
/// - `InGroupById(id)` — keeps only fixtures in the group with that ID.
/// - `AttributeGt/Lt/Eq(attr, pct)` — checks `programmer.active_values` for the attribute.
///   Values and the threshold are both percent (0.0–100.0): the programmer
///   holds `Value`s, never DMX bytes.
pub fn apply_if_filter<F: Borrow<Fixture>>(
    preds: &[IfPredicate],
    mut ids: Vec<u32>,
    groups: &BTreeMap<u32, Group>,
    programmer: &Programmer,
    fixtures: &BTreeMap<u32, F>,
) -> Vec<u32> {
    for pred in preds {
        ids = match pred {
            IfPredicate::InGroup(name) => {
                let lower = name.to_lowercase();
                let group = find_group_by_name(&lower, groups);
                match group {
                    None => vec![],
                    Some(g) => {
                        let members: std::collections::HashSet<uuid::Uuid> =
                            g.fixtures().into_iter().collect();
                        ids.into_iter()
                            .filter(|id| {
                                // Convert u32 ID to GUID for membership check
                                fixtures
                                    .get(id)
                                    .map(|f| members.contains(&Borrow::<Fixture>::borrow(f).meta.identity.guid))
                                    .unwrap_or(false)
                            })
                            .collect()
                    }
                }
            }

            IfPredicate::InGroupById(gid) => {
                let group = groups.get(gid);
                match group {
                    None => vec![],
                    Some(g) => {
                        let members: std::collections::HashSet<uuid::Uuid> =
                            g.fixtures().into_iter().collect();
                        ids.into_iter()
                            .filter(|id| {
                                fixtures
                                    .get(id)
                                    .map(|f| members.contains(&Borrow::<Fixture>::borrow(f).meta.identity.guid))
                                    .unwrap_or(false)
                            })
                            .collect()
                    }
                }
            }

            IfPredicate::AttributeGt(attr, threshold) => ids
                .into_iter()
                .filter(|&fid| {
                    let guid = match fixtures.get(&fid) {
                        Some(f) => Borrow::<Fixture>::borrow(f).meta.identity.guid,
                        None => return false,
                    };
                    match programmer.get_value(guid, attr) {
                        Some(v) => v.percent().is_some_and(|p| p > *threshold),
                        None => false,
                    }
                })
                .collect(),

            IfPredicate::AttributeLt(attr, threshold) => ids
                .into_iter()
                .filter(|&fid| {
                    let guid = match fixtures.get(&fid) {
                        Some(f) => Borrow::<Fixture>::borrow(f).meta.identity.guid,
                        None => return false,
                    };
                    match programmer.get_value(guid, attr) {
                        Some(v) => v.percent().is_some_and(|p| p < *threshold),
                        None => false,
                    }
                })
                .collect(),

            IfPredicate::AttributeEq(attr, threshold) => ids
                .into_iter()
                .filter(|&fid| {
                    let guid = match fixtures.get(&fid) {
                        Some(f) => Borrow::<Fixture>::borrow(f).meta.identity.guid,
                        None => return false,
                    };
                    match programmer.get_value(guid, attr) {
                        Some(v) => v.percent().is_some_and(|p| (p - threshold).abs() < 0.5),
                        None => false,
                    }
                })
                .collect(),
        };
    }

    ids
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lumacore_tree::programmer::Programmer;
    use lumacore_tree::{Datapool, Group};

    // ── Fixture helpers ──────────────────────────────────────────────────────

    fn make_fixture(name: Option<&str>) -> Fixture {
        let nil = uuid::Uuid::nil();
        Fixture::new(name.map(|s| s.to_string()), nil, nil, nil, nil, 1, 1)
    }

    fn fixtures_1_thru_10() -> BTreeMap<u32, Fixture> {
        (1..=10).map(|id| (id, make_fixture(None))).collect()
    }

    fn empty_datapools() -> BTreeMap<u32, Group> {
        BTreeMap::new()
    }

    fn make_group(
        dp: &mut Datapool,
        gid: u32,
        name: Option<&str>,
        ids: Vec<u32>,
        fixtures: &BTreeMap<u32, Fixture>,
    ) {
        let mut g = Group::new(name.map(|s| s.to_string()));
        // Convert integer IDs to GUIDs using the fixture map
        let guids: Vec<uuid::Uuid> = ids
            .iter()
            .filter_map(|&id| fixtures.get(&id).map(|f| f.meta.identity.guid))
            .collect();
        g.set_row(&guids);
        dp.groups.insert(gid, g);
    }

    // ── resolve_selection tests ──────────────────────────────────────────────

    #[test]
    fn resolve_by_id_exists() {
        let fixtures = fixtures_1_thru_10();
        assert_eq!(
            resolve_selection(&SelectionExpr::ById(1), &fixtures, &empty_datapools()),
            vec![1]
        );
    }

    #[test]
    fn resolve_by_id_missing() {
        let fixtures = fixtures_1_thru_10();
        let result = resolve_selection(&SelectionExpr::ById(99), &fixtures, &empty_datapools());
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_range() {
        let fixtures = fixtures_1_thru_10();
        assert_eq!(
            resolve_selection(&SelectionExpr::Range(1, 5), &fixtures, &empty_datapools()),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn resolve_all() {
        let fixtures = fixtures_1_thru_10();
        let ids = resolve_selection(&SelectionExpr::All, &fixtures, &empty_datapools());
        assert_eq!(ids, (1u32..=10).collect::<Vec<_>>());
    }

    #[test]
    fn resolve_add() {
        let fixtures = fixtures_1_thru_10();
        let expr = SelectionExpr::Chain(Box::new(SelectionExpr::ById(1)), vec![(SelOp::Add, SelectionExpr::ById(5))]);
        assert_eq!(
            resolve_selection(&expr, &fixtures, &empty_datapools()),
            vec![1, 5]
        );
    }

    #[test]
    fn resolve_sub() {
        let fixtures = fixtures_1_thru_10();
        let expr = SelectionExpr::Chain(Box::new(SelectionExpr::Range(1, 5)), vec![(SelOp::Sub, SelectionExpr::ById(3))]);
        assert_eq!(
            resolve_selection(&expr, &fixtures, &empty_datapools()),
            vec![1, 2, 4, 5]
        );
    }

    #[test]
    fn resolve_group_by_id() {
        let fixtures = fixtures_1_thru_10();
        let mut dp = Datapool::new(Some("Default".to_string()));
        make_group(&mut dp, 1, Some("Wash"), vec![3, 5], &fixtures);
        let datapools = dp.groups.clone();

        assert_eq!(
            resolve_selection(&SelectionExpr::Group(1), &fixtures, &datapools),
            vec![3, 5]
        );
    }

    #[test]
    fn resolve_group_by_name() {
        let fixtures = fixtures_1_thru_10();
        let mut dp = Datapool::new(Some("Default".to_string()));
        make_group(&mut dp, 1, Some("Front Wash"), vec![2, 4, 6], &fixtures);
        let datapools = dp.groups.clone();

        assert_eq!(
            resolve_selection(
                &SelectionExpr::GroupByName("front wash".to_string()),
                &fixtures,
                &datapools
            ),
            vec![2, 4, 6]
        );
    }

    #[test]
    fn resolve_by_name() {
        let mut fixtures: BTreeMap<u32, Fixture> = BTreeMap::new();
        fixtures.insert(42, make_fixture(Some("Mac Aura")));
        fixtures.insert(43, make_fixture(Some("Other")));

        assert_eq!(
            resolve_selection(
                &SelectionExpr::ByName("mac aura".to_string()),
                &fixtures,
                &empty_datapools()
            ),
            vec![42]
        );
    }

    // ── apply_if_filter tests ────────────────────────────────────────────────

    #[test]
    fn filter_in_group_keeps_members() {
        let fixtures = fixtures_1_thru_10();
        let mut dp = Datapool::new(None);
        make_group(&mut dp, 1, Some("Wash"), vec![1, 3, 5], &fixtures);
        let datapools = dp.groups.clone();

        let ids = vec![1, 2, 3, 4, 5];
        let preds = vec![IfPredicate::InGroup("Wash".to_string())];
        let prog = Programmer::new();

        assert_eq!(
            apply_if_filter(&preds, ids, &datapools, &prog, &fixtures),
            vec![1, 3, 5]
        );
    }

    #[test]
    fn filter_attribute_gt() {
        let fixtures = fixtures_1_thru_10();
        let datapools = empty_datapools();
        let mut prog = Programmer::new();
        // Use GUIDs from fixture map
        let guid_1 = fixtures.get(&1).unwrap().meta.identity.guid;
        let guid_2 = fixtures.get(&2).unwrap().meta.identity.guid;
        prog.set_value(guid_1, "Dimmer", lumacore_tree::Value::normalized(78.4)); // ~78%
        prog.set_value(guid_2, "Dimmer", lumacore_tree::Value::normalized(19.6)); // ~20%

        let ids = vec![1, 2];
        let preds = vec![IfPredicate::AttributeGt("Dimmer".to_string(), 50.0)];

        assert_eq!(
            apply_if_filter(&preds, ids, &datapools, &prog, &fixtures),
            vec![1]
        );
    }

    #[test]
    fn filter_attribute_lt() {
        let fixtures = fixtures_1_thru_10();
        let datapools = empty_datapools();
        let mut prog = Programmer::new();
        let guid_1 = fixtures.get(&1).unwrap().meta.identity.guid;
        let guid_2 = fixtures.get(&2).unwrap().meta.identity.guid;
        prog.set_value(guid_1, "Dimmer", lumacore_tree::Value::normalized(78.4)); // ~78%
        prog.set_value(guid_2, "Dimmer", lumacore_tree::Value::normalized(19.6)); // ~20%

        let ids = vec![1, 2];
        let preds = vec![IfPredicate::AttributeLt("Dimmer".to_string(), 50.0)];

        assert_eq!(
            apply_if_filter(&preds, ids, &datapools, &prog, &fixtures),
            vec![2]
        );
    }
}

/// R1-10: a selection of 200 000 `+` terms parses, resolves, prints, clones
/// and drops without a stack overflow (it used to nest one level per term).
#[cfg(test)]
#[test]
fn a_very_long_selection_never_nests() {
    let mut fixtures = BTreeMap::new();
    for n in 1..=3 {
        fixtures.insert(
            n,
            Fixture::new(None, uuid::Uuid::new_v4(), uuid::Uuid::nil(), uuid::Uuid::nil(), uuid::Uuid::nil(), 1, 1),
        );
    }
    let mut tokens = vec!["1".to_string()];
    for _ in 0..200_000 {
        tokens.push("+".into());
        tokens.push("2".into());
    }
    tokens.push("-".into());
    tokens.push("1".into());
    let parsed = super::parse_selection(&tokens).unwrap();
    let copy = parsed.expr.clone();
    assert_eq!(copy, parsed.expr);
    assert!(copy.to_string().ends_with("+ 2 - 1"));
    assert_eq!(resolve_inner(&parsed.expr, &fixtures, &BTreeMap::new()).iter().filter(|&&n| n == 1).count(), 0);
}

/// Q6: open ranges, typed order, missing ids skipped, `-` takes out.
#[cfg(test)]
#[test]
fn selection_ranges_follow_the_one_rule() {
    let mut fixtures = BTreeMap::new();
    for n in [1, 2, 3, 5, 6, 8] {
        fixtures.insert(
            n,
            Fixture::new(None, uuid::Uuid::new_v4(), uuid::Uuid::nil(), uuid::Uuid::nil(), uuid::Uuid::nil(), 1, 1),
        );
    }
    let ids = |s: &str| {
        let tokens: Vec<String> = s.split_whitespace().map(String::from).collect();
        resolve_inner(&super::parse_selection(&tokens).unwrap().expr, &fixtures, &BTreeMap::new())
    };
    assert_eq!(ids("1 Thru 6"), vec![1, 2, 3, 5, 6]);
    assert_eq!(ids("6 Thru 1"), vec![6, 5, 3, 2, 1]);
    assert_eq!(ids("5 Thru"), vec![5, 6, 8]);
    assert_eq!(ids("Thru 2"), vec![1, 2]);
    assert_eq!(ids("Thru - 2 Thru 6"), vec![1, 8]);
    assert_eq!(ids("1 Thru 4294967295"), vec![1, 2, 3, 5, 6, 8]);
}
