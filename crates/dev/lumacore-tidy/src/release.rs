// ██████╗ ███████╗██╗     ███████╗ █████╗ ███████╗███████╗
// ██╔══██╗██╔════╝██║     ██╔════╝██╔══██╗██╔════╝██╔════╝
// ██████╔╝█████╗  ██║     █████╗  ███████║███████╗█████╗
// ██╔══██╗██╔══╝  ██║     ██╔══╝  ██╔══██║╚════██║██╔══╝
// ██║  ██║███████╗███████╗███████╗██║  ██║███████║███████╗
// ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: release
//! Tidy rules for a public tree: no word the owner bans and no private detail (a home folder, a
//! personal mail address) in any tracked text file. The banned words live only in the
//! environment (`LUMACORE_BANNED_WORDS`, typed by whoever runs the gate), never in the tree, so
//! the rule can't carry what it guards against.
//!
//! Owns:          the release rules and their scanners
//! Doesn't own:   the list of banned words (the environment), the files (git's index)
//! Entry points:  banned_words_are_absent, no_private_details
//! Decisions:     the public release cleanup (2026-10-23)

use std::process::Command;

use regex::Regex;

use crate::files::workspace;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The variable that holds the banned words: comma-separated, case-insensitive, each a regex.
const BANNED_WORDS_VAR: &str = "LUMACORE_BANNED_WORDS";

/// The most hits a failure lists (the count says how many more).
const LISTED: usize = 200;

/// Folders of other people's code: their authors' addresses are theirs to publish.
const THIRD_PARTY: &[&str] = &["vendor/", "tests/e2e/node_modules/"];

/// File endings that look like a mail domain in a fixture's file name (`Generic@Dimmer.gdtf`).
const FILE_ENDINGS: &[&str] = &["gdtf", "mvr", "xml", "json", "png", "svg", "rs", "js", "mjs", "ts", "md", "luma", "lum", "txt", "html", "css", "zip", "glb", "3ds", "jpg", "gif"];

/// Mail addresses anyone may publish: no-reply senders and the reserved example domains.
const PUBLIC_MAIL: &str = r"(?i)^(noreply@.*|.*@example\.(com|org|net))$";

/// A home folder: `/home/<name>/`, `/Users/<name>/`, `C:\Users\<name>\` (a placeholder in angle
/// brackets is no name).
const HOME_PATH: &str = r"(?:^|[^A-Za-z0-9_.~-])(/home/[A-Za-z0-9._-]+/|/Users/[A-Za-z0-9._-]+/|[A-Za-z]:\\Users\\[A-Za-z0-9._ -]+\\)";

/// A mail address.
const MAIL: &str = r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}\b";

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Every tracked text file (git's index; a file with a NUL byte is binary and skipped), with its
/// path from the workspace root.
fn tracked_text_files() -> Vec<(String, String)> {
    let root = workspace();
    let out = Command::new("git").args(["ls-files", "-z"]).current_dir(&root).output().expect("git ls-files (the release rules need git on the path)");
    assert!(out.status.success(), "git ls-files failed: {}", String::from_utf8_lossy(&out.stderr));
    let mut files = Vec::new();
    for rel in out.stdout.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let rel = String::from_utf8_lossy(rel).to_string();
        // A file deleted in the work tree but still in the index is not there to read.
        let Ok(bytes) = std::fs::read(root.join(&rel)) else { continue };
        if bytes.contains(&0) {
            continue;
        }
        files.push((rel, String::from_utf8_lossy(&bytes).to_string()));
    }
    files
}

/// The banned words of `spec` (comma-separated), each a case-insensitive regex.
fn banned_words(spec: &str) -> Result<Vec<Regex>, String> {
    spec.split(',').map(str::trim).filter(|t| !t.is_empty()).map(|t| Regex::new(&format!("(?i){t}")).map_err(|e| format!("{BANNED_WORDS_VAR}: `{t}` is no regex: {e}"))).collect()
}

/// `file:line: what` for every line of `text` a banned word matches.
fn banned_hits(rel: &str, text: &str, words: &[Regex]) -> Vec<String> {
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        for word in words {
            if let Some(m) = word.find(line) {
                hits.push(format!("{rel}:{}: `{}`", i + 1, m.as_str()));
            }
        }
    }
    hits
}

/// `file:line: what` for every home folder and personal mail address in `text`.
fn private_hits(rel: &str, text: &str) -> Vec<String> {
    let home = Regex::new(HOME_PATH).expect("HOME_PATH");
    let mail = Regex::new(MAIL).expect("MAIL");
    let public = Regex::new(PUBLIC_MAIL).expect("PUBLIC_MAIL");
    let third_party = THIRD_PARTY.iter().any(|p| rel.starts_with(p));
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        for c in home.captures_iter(line) {
            hits.push(format!("{rel}:{}: a home folder `{}`", i + 1, &c[1]));
        }
        if third_party {
            continue;
        }
        for m in mail.find_iter(line) {
            let address = m.as_str();
            let ending = address.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
            if public.is_match(address) || FILE_ENDINGS.contains(&ending.as_str()) {
                continue;
            }
            hits.push(format!("{rel}:{}: a mail address `{address}`", i + 1));
        }
    }
    hits
}

/// The failure text: the first `LISTED` hits and how many more.
fn listing(hits: &[String]) -> String {
    let mut text = hits.iter().take(LISTED).cloned().collect::<Vec<_>>().join("\n");
    if hits.len() > LISTED {
        text.push_str(&format!("\n… and {} more", hits.len() - LISTED));
    }
    text
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

/// No tracked text file holds a word of `LUMACORE_BANNED_WORDS`. Unset or empty, the rule has
/// nothing to look for and says so.
#[test]
fn banned_words_are_absent() {
    let spec = std::env::var(BANNED_WORDS_VAR).unwrap_or_default();
    let words = banned_words(&spec).unwrap_or_else(|e| panic!("{e}"));
    if words.is_empty() {
        println!("banned_words_are_absent: skipped ({BANNED_WORDS_VAR} is not set)");
        return;
    }
    let hits: Vec<String> = tracked_text_files().iter().flat_map(|(rel, text)| banned_hits(rel, text, &words)).collect();
    assert!(hits.is_empty(), "{} banned word(s) in tracked files ({BANNED_WORDS_VAR}):\n{}", hits.len(), listing(&hits));
}

/// No tracked text file names a home folder or a personal mail address: a path is relative to
/// the repo (or `env!("LUMACORE_ROOT")`), an address is a no-reply or an example one.
#[test]
fn no_private_details() {
    let hits: Vec<String> = tracked_text_files().iter().flat_map(|(rel, text)| private_hits(rel, text)).collect();
    assert!(hits.is_empty(), "{} private detail(s) in tracked files:\n{}", hits.len(), listing(&hits));
}

#[test]
fn release_scanners_find_what_they_guard() {
    let words = banned_words("zebra, qu+x ,").unwrap();
    assert_eq!(words.len(), 2);
    assert_eq!(banned_hits("a.md", "one\nA Zebra here\nquuux", &words), ["a.md:2: `Zebra`", "a.md:3: `quuux`"]);
    assert!(banned_words("(").is_err());
    // Built at run time, so this file holds no detail the rule itself would find.
    let (home, users, at) = ("home", "Users", "@");
    let text = format!("see /{home}/alice/x and /{users}/bob/y\nC:\\{users}\\Carol\\z\n/{home}/<name>/ ~/{home}/x/\nmail alice{at}corp.io, noreply{at}github.com, x{at}example.org, Generic{at}Dimmer.gdtf");
    let hits = private_hits("b.md", &text);
    assert_eq!(hits.len(), 4, "{hits:?}");
    assert!(hits[0].contains("alice/") && hits[1].contains("bob/") && hits[2].contains("Carol") && hits[3].contains("corp.io"));
    assert_eq!(private_hits("vendor/x/Cargo.toml", &format!("authors = [\"a <a{at}b.dev>\"]")), Vec::<String>::new());
}
