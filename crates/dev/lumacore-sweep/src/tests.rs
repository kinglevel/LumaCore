// ███████╗██╗    ██╗███████╗███████╗██████╗     ████████╗███████╗███████╗████████╗███████╗
// ██╔════╝██║    ██║██╔════╝██╔════╝██╔══██╗    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ███████╗██║ █╗ ██║█████╗  █████╗  ██████╔╝       ██║   █████╗  ███████╗   ██║   ███████╗
// ╚════██║██║███╗██║██╔══╝  ██╔══╝  ██╔═══╝        ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ███████║╚███╔███╔╝███████╗███████╗██║            ██║   ███████╗███████║   ██║   ███████║
// ╚══════╝ ╚══╝╚══╝ ╚══════╝╚══════╝╚═╝            ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: sweep tests

//! Property tests: random command sequences from the demo show through the real change path (the
//! sweep's world and generator), plus the pure rules (order, election, masking, the clock filter).
//! Fast: all of them run in seconds. A property the code breaks today is `#[ignore]`d with its
//! finding — the overnight sweep reports it too.
//!
//! Owns:          the crate's property tests (proptest, fixed seeds).
//! Doesn't own:   the overnight run (run.rs), the world and its checks (world.rs).
//! Entry points:  the #[test] fns.
//! Decisions:     §25 "Step 10 hardening", §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use std::sync::{Arc, OnceLock};

use proptest::prelude::*;

use crate::lines::{Rng, exhaustive, random_line};
use crate::vocab::Vocab;
use crate::world::{Finding, ROLES, World, round_trip};
use lumacore_objects::demo;

/// The demo show's JSON (built once per process).
fn base() -> Arc<String> {
    static BASE: OnceLock<Arc<String>> = OnceLock::new();
    BASE.get_or_init(|| {
        let state = demo::fresh_state();
        Arc::new(serde_json::to_string_pretty(&state.root.show_data).expect("ShowData serializes"))
    })
    .clone()
}

/// Property tests are ordinary `cargo test`s: they stay off the disk.
/// `Drive` lines run only in the overnight sweep's sandbox (decisions §29).
fn show_only(line: &str) -> bool {
    use lumacore_objects::commands::registry::{Reach, line_reach};
    line_reach(line) == Reach::Show
}

/// The sweep's exhaustive plan (Show lines only), built once.
fn plan() -> &'static [String] {
    static PLAN: OnceLock<Vec<String>> = OnceLock::new();
    PLAN.get_or_init(|| {
        let world = World::new(base()).expect("the demo show loads");
        let vocab = Vocab::new(&world.master);
        exhaustive(&vocab, &world.master)
            .into_iter()
            .map(|c| c.line)
            .filter(|l| show_only(l))
            .collect()
    })
}

/// Run `n` lines from `seed` as random roles — half from the exhaustive plan,
/// half grown by completion; the findings of the checks named in `checks`,
/// with the lines that led there.
fn sequence(seed: u64, n: usize, checks: &[&str]) -> (World, Vec<(Finding, Vec<String>)>) {
    let mut world = World::new(base()).expect("the demo show loads");
    let vocab = Vocab::new(&world.master);
    let mut rng = Rng(seed);
    let mut found = Vec::new();
    for _ in 0..n {
        let line = match rng.chance(50) {
            true => rng.pick(plan()).cloned().unwrap_or_default(),
            false => random_line(&mut rng, &world.master, &vocab),
        };
        if !show_only(&line) {
            continue;
        }
        let role = *rng.pick(&ROLES).unwrap_or(&"Admin");
        let (outcome, findings) = world.run(role, &line, None);
        for f in findings {
            if checks.contains(&f.check.as_str()) {
                found.push((f, world.history.clone()));
            }
        }
        if outcome.ended {
            world.reset().expect("reset");
        }
    }
    (world, found)
}

fn report(found: &[(Finding, Vec<String>)]) -> String {
    found
        .iter()
        .take(3)
        .map(|(f, history)| format!("{} — {}\n  after:\n    {}", f.key(), f.detail, history.join("\n    ")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// The command-sequence tests run fixed seeds in `cargo test` (the same lines
/// every time, so a known finding can't make them flaky); the overnight run
/// sets `PROPTEST_RNG_SEED` to explore.
fn fixed(cases: u32) -> ProptestConfig {
    let mut c = config(cases);
    if std::env::var_os("PROPTEST_RNG_SEED").is_none() {
        c.rng_seed = proptest::test_runner::RngSeed::Fixed(0x1u64.wrapping_add(25));
    }
    c
}

proptest! {
    #![proptest_config(fixed(4))]

    /// No line panics, and nothing the change path files breaks identity
    /// (no duplicate GUIDs).
    #[test]
    fn no_line_panics_or_duplicates_a_guid(seed in any::<u64>()) {
        let (world, found) = sequence(seed, 30, &["panic", "drive-escape"]);
        prop_assert!(found.is_empty(), "{}", report(&found));
        let ids = world.master.root.show_data.identity();
        prop_assert!(ids.duplicates.is_empty() && ids.nil.is_empty(), "{:?}", ids.duplicates);
    }

    /// `.luma`: save → load gives the same content and a clean identity.
    #[test]
    fn a_saved_show_loads_back_the_same(seed in any::<u64>()) {
        let (world, _) = sequence(seed, 30, &[]);
        // Exact floats (issue 09): no tolerance for a reordered save.
        prop_assert!(round_trip(&world.master).is_ok(), "{:?}", round_trip(&world.master));
    }

    /// A refused line (rights, claim) changes nothing.
    #[test]
    fn a_refusal_changes_nothing(seed in any::<u64>()) {
        let (_, found) = sequence(seed, 30, &["refused-but-changed"]);
        prop_assert!(found.is_empty(), "{}", report(&found));
    }
}

proptest! {
    #![proptest_config(fixed(4))]

    /// `.luma`: save → load → save gives the same bytes.
    #[test]
    fn a_saved_show_saves_the_same_bytes(seed in any::<u64>()) {
        let (world, _) = sequence(seed, 30, &[]);
        prop_assert!(round_trip(&world.master).is_ok(), "{:?}", round_trip(&world.master));
    }

    /// Replication: a member fed the master's ops has the master's show.
    #[test]
    fn a_member_ends_with_the_masters_show(seed in any::<u64>()) {
        let (mut world, mut found) = sequence(seed, 30, &["member", "replication"]);
        let mut heavy = Vec::new();
        world.heavy(&mut |check: &str, detail: String| heavy.push((check.to_string(), detail)));
        found.extend(heavy.into_iter().filter(|(c, _)| c == "member").map(|(check, detail)| {
            (Finding { check, word: String::new(), detail }, world.history.clone())
        }));
        prop_assert!(found.is_empty(), "{}", report(&found));
    }

    /// Undo: Oops restores the show exactly, Redo puts the change back, and a
    /// read-only line changes nothing.
    #[test]
    fn oops_restores_and_redo_repeats(seed in any::<u64>()) {
        let (_, found) = sequence(seed, 30, &["undo", "redo", "read-only-changed"]);
        prop_assert!(found.is_empty(), "{}", report(&found));
    }

    /// Identity: no line leaves a new broken link.
    #[test]
    fn no_line_breaks_a_link(seed in any::<u64>()) {
        let (_, found) = sequence(seed, 30, &["identity"]);
        prop_assert!(found.is_empty(), "{}", report(&found));
    }
}

// ── Merge (asymmetric by design: the master wins) ────────────────────────────

/// Two shows that went their own ways from one base: (base, master, mine).
fn split(seed: u64) -> (lumacore_tree::ShowData, lumacore_tree::ShowData, lumacore_tree::ShowData) {
    let world = World::new(base()).expect("the demo show loads");
    let base_show = world.master.root.show_data.clone();
    let (a, _) = sequence(seed, 12, &[]);
    let (b, _) = sequence(seed ^ 0x5eed, 12, &[]);
    (base_show, a.master.root.show_data.clone(), b.master.root.show_data.clone())
}

fn json(show: &lumacore_tree::ShowData) -> serde_json::Value {
    serde_json::to_value(show).expect("ShowData serializes")
}

proptest! {
    #![proptest_config(fixed(3))]

    /// merge(base, x, x) = x.
    #[test]
    fn merging_a_show_with_itself_changes_nothing(seed in any::<u64>()) {
        let (base, x, _) = split(seed);
        if lumacore_tree::merge::can_merge(Some(&base), &x, &x).is_ok() {
            let (merged, _) = lumacore_tree::merge::three_way(&base, &x, &x).map_err(TestCaseError::fail)?;
            prop_assert_eq!(json(&merged), json(&x));
        }
    }

    /// merge(base, base, mine) = mine (playbacks are the master's).
    #[test]
    fn an_untouched_master_takes_mine(seed in any::<u64>()) {
        let (base, _, mine) = split(seed);
        if lumacore_tree::merge::can_merge(Some(&base), &base, &mine).is_ok() {
            let (mut merged, _) = lumacore_tree::merge::three_way(&base, &base, &mine).map_err(TestCaseError::fail)?;
            merged.playbacks = mine.playbacks.clone();
            prop_assert_eq!(json(&merged), json(&mine));
        }
    }

    /// After any merge: no duplicate GUIDs, every object in its own slot.
    #[test]
    fn a_merge_keeps_guids_unique(seed in any::<u64>()) {
        let (base, master, mine) = split(seed);
        if lumacore_tree::merge::can_merge(Some(&base), &master, &mine).is_ok() {
            let (merged, _) = lumacore_tree::merge::three_way(&base, &master, &mine).map_err(TestCaseError::fail)?;
            let ids = merged.identity();
            prop_assert!(ids.duplicates.is_empty(), "{:?}", ids.duplicates);
        }
    }
}

// ── Replication order ────────────────────────────────────────────────────────

proptest! {
    /// Only seq + 1 of the same journal applies; a repeat is ignored, a gap or
    /// another journal asks for a resync.
    #[test]
    fn ops_apply_strictly_in_order(last in 0u64..1000, seq in 0u64..1002, same in any::<bool>()) {
        use lumacore_lumasession::link::{OpOrder, op_order};
        let journal = uuid::Uuid::new_v4();
        let op = lumacore_tree::ReplicationOp {
            journal: if same { journal } else { uuid::Uuid::new_v4() },
            epoch: 1,
            seq,
            station: String::new(),
            user: String::new(),
            command: String::new(),
            affected_subtrees: Vec::new(),
            post_snapshots: Vec::new(),
            undo: Vec::new(),
            runtime: None,
            authority_timestamp_ms: 0,
        };
        let want = if !same {
            OpOrder::Gap
        } else if seq <= last {
            OpOrder::Stale
        } else if seq == last + 1 {
            OpOrder::Next
        } else {
            OpOrder::Gap
        };
        prop_assert_eq!(op_order(Some((journal, last)), &op), want);
        // Applied once, the same op again is a repeat.
        if want == OpOrder::Next {
            prop_assert_eq!(op_order(Some((journal, seq)), &op), OpOrder::Stale);
        }
    }
}

// ── Election ─────────────────────────────────────────────────────────────────

fn candidate() -> impl Strategy<Value = lumacore_lumasession::election::Candidate> {
    use lumacore_tree::StationPriority as P;
    (
        prop_oneof![Just(P::VeryLow), Just(P::Low), Just(P::Normal), Just(P::High)],
        0u64..4,
        0u64..20_000,
        "[a-f0-9]{4}",
    )
        .prop_map(|(priority, applied, online_since_ms, station_id)| lumacore_lumasession::election::Candidate {
            name: format!("S{station_id}"),
            station_id,
            priority,
            applied,
            online_since_ms,
            ip: String::new(),
            stream_port: 1,
            me: false,
        })
}

proptest! {
    /// Every member computes the same winner, whatever order it lists them in.
    #[test]
    fn every_order_elects_the_same_winner(
        mut all in prop::collection::vec(candidate(), 2..6),
        rotate in 0usize..6,
    ) {
        use lumacore_lumasession::election::best;
        // Same priority and results: online time and station id decide.
        for c in &mut all {
            c.priority = lumacore_tree::StationPriority::Normal;
            c.applied = 0;
        }
        all.sort_by(|a, b| a.station_id.cmp(&b.station_id));
        all.dedup_by(|a, b| a.station_id == b.station_id);
        let first = best(&all).map(|c| c.station_id.clone());
        let n = all.len();
        all.rotate_left(rotate % n);
        all.reverse();
        let again = best(&all).map(|c| c.station_id.clone());
        // Also every pairwise order: a winner must rank above all others.
        if let Some(w) = all.iter().find(|c| Some(&c.station_id) == first.as_ref()) {
            for c in all.iter().filter(|c| c.station_id != w.station_id) {
                prop_assert!(lumacore_lumasession::election::ranks_above(w, c), "{} vs {}: {:?}", w.station_id, c.station_id, all);
            }
        }
        prop_assert_eq!(again, first);
    }

    /// The online-time rule alone (no ties within 5 s): any order, one winner.
    #[test]
    fn distinct_stations_elect_the_same_winner(
        mut all in prop::collection::vec(candidate(), 2..6),
        rotate in 0usize..6,
    ) {
        use lumacore_lumasession::election::best;
        all.sort_by(|a, b| a.station_id.cmp(&b.station_id));
        all.dedup_by(|a, b| a.station_id == b.station_id);
        // Spread the online times so no two are within the 5 s tie.
        for (i, c) in all.iter_mut().enumerate() {
            c.online_since_ms = i as u64 * 10_000 + c.online_since_ms % 1000;
        }
        let first = best(&all).map(|c| c.station_id.clone());
        let n = all.len();
        all.rotate_left(rotate % n);
        all.reverse();
        prop_assert_eq!(best(&all).map(|c| c.station_id.clone()), first);
    }
}

// ── Clock filter ─────────────────────────────────────────────────────────────

proptest! {
    /// Whatever samples come in, the frequency stays within ±500 ppm and the
    /// filter never panics.
    #[test]
    fn the_clock_filter_keeps_its_frequency_bounds(
        samples in prop::collection::vec((0u64..200_000, -5_000_000i64..5_000_000, 0u64..50_000), 1..60),
    ) {
        use lumacore_clocksync::{Filter, Sample};
        let mut filter = Filter::default();
        let mut local = 1_000_000u64;
        for (step, delta, rtt) in samples {
            local += step;
            if let Some(e) = filter.add(Sample { local, delta, rtt }, local) {
                prop_assert!(e.frequency_ppb.abs() <= 500_000, "{}", e.frequency_ppb);
            }
        }
    }
}

// ── Passwords ────────────────────────────────────────────────────────────────

proptest! {
    /// A password never survives masking, and masking twice = once.
    #[test]
    fn masking_hides_the_password_and_is_idempotent(
        user in "[A-Za-z][A-Za-z0-9]{1,8}",
        pw in "[A-Za-z0-9]{6,16}",
        template in 0usize..6,
        tail in prop_oneof![Just(""), Just("; Fixture 1 At 50"), Just(" /NoConfirmation")],
    ) {
        use lumacore_objects::passwords::mask_secrets;
        prop_assume!(!user.contains(&pw) && !pw.contains(&user));
        let line = match template {
            0 => format!("Login \"{user}\" \"{pw}\"{tail}"),
            1 => format!("login user {user} {pw}{tail}"),
            2 => format!("Store User \"{user}\" /Password \"{pw}\"{tail}"),
            3 => format!("Set User \"{user}\" \"Password\" {pw}{tail}"),
            4 => format!("Connect 10.0.0.2 User \"{user}\" \"{pw}\"{tail}"),
            _ => format!("Fixture 1 At 50; Login \"{user}\" \"{pw}\""),
        };
        let masked = mask_secrets(&line);
        prop_assert!(!masked.contains(&pw), "{line} → {masked}");
        prop_assert_eq!(mask_secrets(&masked), masked.clone());
    }
}
