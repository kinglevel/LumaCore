//! The words the parser knows, built by the station from its registries (decisions §33).
//!
//! Plain data: the station fills it once, a test or fuzz target can build its own. A word may
//! play more than one role only where §33 says so — `At` is a verb and a helper (decision 43),
//! `On` / `Full` / `Toggle` are verbs and value words (decision 14). An object is never a verb.

use std::collections::HashMap;

/// Index of a word in its [`Vocabulary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WordId(pub u32);

#[derive(Debug, Clone, Default)]
pub struct WordDef {
    /// The one name, in canonical case.
    pub name: String,
    pub verb: Option<VerbDef>,
    pub object: Option<ObjectDef>,
    /// Starts a slot: `At`, `Fade`, `Delay`, ….
    pub helper: bool,
    /// A declared value: `Full`, `HTP`, `Ask`, ….
    pub value: bool,
    /// A property name — always written quoted (decision 7); a bare one is refused (E-SYN-005).
    pub property: bool,
    /// A tree-shell word: its statement is read with the fence's rules (decision 18).
    pub fence: Option<FenceDef>,
}

#[derive(Debug, Clone, Default)]
pub struct VerbDef {
    /// Takes `On` / `Off` right after it (Top, Flash, Temp, Swap — decision 15).
    pub hold: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ObjectDef {
    /// Takes ids. An object without ids typed alone runs its default verb (decision 27).
    pub counted: bool,
    /// How many dot parts its numbers may have (`Fixture` 1, `Cue` / `Master` / `DMX` 2).
    pub id_parts: u8,
    /// The objects that may follow it in an owner-first path (`DataPool` → `Group`).
    pub children: Vec<String>,
    /// Words that may stand before its ids and say which numbering they are in (`Fixture
    /// Houselights 4`: MVR's ID types).
    pub id_kinds: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FenceDef {
    /// Its `-x` / `--name` flags and whether each takes a value (`-n 5`, `--type Group`).
    pub flags: Vec<(String, bool)>,
}

#[derive(Debug, Clone)]
pub struct OptionDef {
    /// Canonical name without the slash.
    pub name: String,
    /// Whether the next word or quoted text is its value (decision 19).
    pub takes_value: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Vocabulary {
    words: Vec<WordDef>,
    by_name: HashMap<String, WordId>,
    options: Vec<OptionDef>,
    option_by_name: HashMap<String, usize>,
}

impl Vocabulary {
    pub fn new() -> Self {
        Self::default()
    }

    /// The word named `name`, created when new. Names are case-insensitive (decision 24).
    fn entry(&mut self, name: &str) -> &mut WordDef {
        let key = name.to_lowercase();
        let id = match self.by_name.get(&key) {
            Some(&id) => id,
            None => {
                let id = WordId(self.words.len() as u32);
                self.words.push(WordDef { name: name.to_string(), ..WordDef::default() });
                self.by_name.insert(key, id);
                id
            }
        };
        &mut self.words[id.0 as usize]
    }

    pub fn verb(&mut self, name: &str, def: VerbDef) -> &mut Self {
        self.entry(name).verb = Some(def);
        self
    }

    pub fn object(&mut self, name: &str, def: ObjectDef) -> &mut Self {
        self.entry(name).object = Some(def);
        self
    }

    pub fn helper(&mut self, name: &str) -> &mut Self {
        self.entry(name).helper = true;
        self
    }

    pub fn value(&mut self, name: &str) -> &mut Self {
        self.entry(name).value = true;
        self
    }

    pub fn property(&mut self, name: &str) -> &mut Self {
        self.entry(name).property = true;
        self
    }

    pub fn fence(&mut self, name: &str, def: FenceDef) -> &mut Self {
        self.entry(name).fence = Some(def);
        self
    }

    pub fn option(&mut self, name: &str, takes_value: bool) -> &mut Self {
        let key = name.to_lowercase();
        match self.option_by_name.get(&key) {
            Some(&i) => self.options[i].takes_value = takes_value,
            None => {
                self.option_by_name.insert(key, self.options.len());
                self.options.push(OptionDef { name: name.to_string(), takes_value });
            }
        }
        self
    }

    pub fn lookup(&self, name: &str) -> Option<WordId> {
        self.by_name.get(&name.to_lowercase()).copied()
    }

    pub fn word(&self, id: WordId) -> &WordDef {
        &self.words[id.0 as usize]
    }

    pub fn words(&self) -> impl Iterator<Item = (WordId, &WordDef)> {
        self.words.iter().enumerate().map(|(i, w)| (WordId(i as u32), w))
    }

    pub fn find_option(&self, name: &str) -> Option<&OptionDef> {
        self.option_by_name.get(&name.to_lowercase()).map(|&i| &self.options[i])
    }

    pub fn options(&self) -> &[OptionDef] {
        &self.options
    }

    /// Whether `child` may follow `owner` in a path.
    pub fn is_child(&self, owner: WordId, child: WordId) -> bool {
        let child = &self.word(child).name;
        self.word(owner)
            .object
            .as_ref()
            .is_some_and(|o| o.children.iter().any(|c| c.eq_ignore_ascii_case(child)))
    }

    /// Whether `owner` is an owner of `child` (directly or further up) — `Cue 3 Sequence 2`.
    pub fn is_owner_of(&self, owner: WordId, child: WordId) -> bool {
        let mut seen = vec![owner];
        let mut i = 0;
        while let Some(&at) = seen.get(i) {
            if self.is_child(at, child) {
                return true;
            }
            if let Some(o) = &self.word(at).object {
                for c in &o.children {
                    if let Some(id) = self.lookup(c)
                        && !seen.contains(&id)
                    {
                        seen.push(id);
                    }
                }
            }
            i += 1;
        }
        false
    }

    /// The closest known names to an unknown word, for "did you mean" (at most three).
    pub fn closest(&self, word: &str) -> Vec<&str> {
        let word = word.to_lowercase();
        let mut scored: Vec<(usize, &str)> = self
            .words
            .iter()
            .map(|w| (distance(&word, &w.name.to_lowercase()), w.name.as_str()))
            .filter(|&(d, name)| d <= 2.max(name.len() / 3))
            .collect();
        scored.sort();
        scored.into_iter().take(3).map(|(_, n)| n).collect()
    }
}

/// Edit distance (Levenshtein), small and good enough for word hints.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != cb)).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_case_insensitive_and_roles_combine() {
        let mut v = Vocabulary::new();
        v.verb("At", VerbDef::default()).helper("At");
        let at = v.lookup("at").unwrap();
        assert!(v.word(at).verb.is_some() && v.word(at).helper);
        assert_eq!(v.word(at).name, "At");
    }

    #[test]
    fn owners_are_found_up_the_tree() {
        let mut v = Vocabulary::new();
        v.object("DataPool", ObjectDef { counted: true, id_parts: 1, children: vec!["Sequence".into()], ..ObjectDef::default() })
            .object("Sequence", ObjectDef { counted: true, id_parts: 1, children: vec!["Cue".into()], ..ObjectDef::default() })
            .object("Cue", ObjectDef { counted: true, id_parts: 2, children: vec![], ..ObjectDef::default() });
        let (dp, seq, cue) = (v.lookup("DataPool").unwrap(), v.lookup("Sequence").unwrap(), v.lookup("Cue").unwrap());
        assert!(v.is_child(seq, cue) && !v.is_child(dp, cue));
        assert!(v.is_owner_of(dp, cue) && !v.is_owner_of(cue, dp));
    }

    #[test]
    fn closest_names_suggest_a_typo() {
        let mut v = Vocabulary::new();
        v.object("Group", ObjectDef::default()).verb("Store", VerbDef::default());
        assert_eq!(v.closest("Grou"), vec!["Group"]);
    }
}
