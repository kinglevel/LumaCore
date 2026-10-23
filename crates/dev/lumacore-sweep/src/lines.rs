// ██╗     ██╗███╗   ██╗███████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: lines

//! Lines to run: the exhaustive plan (every word × keyword × flag × property) and random lines
//! grown word by word from the completion engine. Everything comes from Vocab.
//!
//! Owns:          the exhaustive plan of cases (Case, its id) and the random line generator (Rng, random_line).
//! Doesn't own:   what words exist (vocab.rs), running and checking a line (world.rs).
//! Entry points:  exhaustive, random_line, Case, Rng.
//! Decisions:     §25 "Step 10 hardening", §34 20.10.

use std::collections::BTreeSet;

use crate::vocab::{self, Vocab};
use crate::world::Expect;
use lumacore_objects::AppState;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One planned line.
#[derive(Debug, Clone)]
pub struct Case {
    pub line: String,
    pub expect: Option<Expect>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Case {
    fn plain(line: String) -> Self {
        Self { line, expect: None }
    }

    /// Stable id of the case (the plan's cursor is the set of done ids).
    pub fn id(&self) -> u64 {
        fnv(&self.line)
    }
}

/// FNV-1a: stable across builds and platforms (unlike `DefaultHasher`).
pub fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3))
}

/// The exhaustive plan, in a fixed order, each line once.
pub fn exhaustive(vocab: &Vocab, base: &AppState) -> Vec<Case> {
    let mut cases = Vec::new();

    // Every name alone (verbs and their aliases).
    for cmd in &vocab.commands {
        for name in cmd.names() {
            cases.push(Case::plain(name.to_string()));
        }
    }

    // Commands with what they take.
    for cmd in &vocab.commands {
        match cmd.object_verb {
            Some(verb) => {
                for obj in &vocab.objects {
                    if !vocab::swept((obj.reach)(verb)) {
                        continue;
                    }
                    let mut flags = vocab::Vocab::command_flags(cmd);
                    flags.extend(vocab::Vocab::object_flags(obj, verb));
                    cases.push(Case::plain(format!("{} {}", cmd.verb, obj.keyword)));
                    for id in vocab.ids_of(obj.keyword) {
                        let line = format!("{} {} {id}", cmd.verb, obj.keyword);
                        for flag in &flags {
                            cases.push(Case::plain(format!("{line} {flag}")));
                        }
                        cases.push(Case::plain(line));
                    }
                }
            }
            None => {
                for arg in vocab::args_for(cmd.args, vocab, base) {
                    cases.push(Case::plain(format!("{} {arg}", cmd.verb)));
                }
                for flag in vocab::Vocab::command_flags(cmd) {
                    cases.push(Case::plain(format!("{} {flag}", cmd.verb)));
                }
            }
        }
    }

    // Every object called, alone and with ids, values, ranges and flags.
    for obj in &vocab.objects {
        if !vocab::swept((obj.reach)(obj.default_verb)) {
            continue;
        }
        for name in std::iter::once(obj.keyword).chain(obj.aliases.iter().copied()) {
            cases.push(Case::plain(name.to_string()));
        }
        let flags = vocab::Vocab::object_flags(obj, obj.default_verb);
        for id in vocab.ids_of(obj.keyword) {
            let line = format!("{} {id}", obj.keyword);
            for tail in ["", " At 50", " At Full", " Thru", " + 2", " - 1"] {
                cases.push(Case::plain(format!("{line}{tail}")));
            }
            for flag in &flags {
                cases.push(Case::plain(format!("{line} {flag}")));
            }
        }
    }

    // Every writable property with valid, edge and wrong values.
    for target in &vocab.props {
        for id in &target.ids {
            for raw in vocab::values_of(target.prop.kind) {
                cases.push(Case {
                    line: format!("Set {} {id} \"{}\" {raw}", target.object, target.prop.name),
                    expect: Some(Expect::ReadBack {
                        object: target.object,
                        id: *id,
                        prop: target.prop,
                        raw,
                    }),
                });
            }
        }
    }
    // Every other field is read-only.
    for (object, id, field) in &vocab.read_only {
        cases.push(Case {
            line: format!("Set {object} {id} {field} 1"),
            expect: Some(Expect::ReadOnly {
                object,
                id: *id,
                field: field.clone(),
            }),
        });
    }

    // What the docs show.
    for ex in &vocab.examples {
        cases.push(Case::plain(ex.clone()));
    }

    let mut seen = BTreeSet::new();
    cases.retain(|c| vocab::swept_line(&c.line) && seen.insert(c.line.clone()));
    cases
}

/// A small deterministic random source (splitmix64): a seed replays a run.
#[derive(Debug, Clone)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        (!items.is_empty()).then(|| &items[self.below(items.len())])
    }

    pub fn chance(&mut self, percent: u64) -> bool {
        self.next_u64() % 100 < percent
    }
}

/// Words that aren't names: numbers, ranges, values, quoted names.
const LITERALS: &[&str] = &[
    "1", "2", "3", "101", "0", "999", "-1", "1.5", "Thru", "At", "+", "-", "Full", "Zero", "50", "\"sweep\"",
    "\"\"", ";", "Please", "If", "1.001", "2.1", "On", "Off", "Yes", "No",
];

/// A random line: each next word is what completion offers there (most of
/// the time), else a literal, a flag or any keyword. Show and Drive lines
/// (the tree shell stays out).
pub fn random_line(rng: &mut Rng, state: &AppState, vocab: &Vocab) -> String {
    for _ in 0..20 {
        let mut line = String::new();
        let words = 1 + rng.below(7);
        for i in 0..words {
            let probe = if line.is_empty() { String::new() } else { format!("{line} ") };
            let (_, offered) = lumacore_suggest::complete_input(&probe, probe.chars().count(), state);
            let word = if i == 0 || (!offered.is_empty() && rng.chance(70)) {
                rng.pick(&offered).cloned()
            } else {
                match rng.below(4) {
                    0 => rng.pick(&vocab.flags).cloned(),
                    1 => rng.pick(&vocab.objects).map(|o| o.keyword.to_string()),
                    _ => rng.pick(LITERALS).map(|s| s.to_string()),
                }
            };
            let Some(word) = word else { break };
            let word = word.trim().to_string();
            if word.is_empty() {
                continue;
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(&word);
        }
        if !line.is_empty() && vocab::swept_line(&line) {
            return line;
        }
    }
    "Clear".to_string()
}
