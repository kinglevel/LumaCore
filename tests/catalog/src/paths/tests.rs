// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The golden record of the object tree (decisions §31 "Tree paths are typed"): every path a
//! user can reach on the demo show and a set of tricky inputs, through cd / enter / ls / List /
//! tree / cat / stat / completion, with the scope, claim and context each lands on. The typed
//! `TreePath` work must keep it byte for byte, except where a step fixes a bug on purpose (then
//! the diff is read line by line). Rewrite it with
//! `LUMACORE_BLESS=1 cargo test -p lumacore-catalog-tests tree_paths_are_unchanged`.
//!
//! Owns:          the golden record `testdata/tree_paths.golden.txt`.
//! Doesn't own:   the typed tree paths (the kernel's tree_path), the tree shell's own tests.
//! Entry points:  tree_paths_are_unchanged.
//! Decisions:     §31, §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::commands::dispatch;
use lumacore_objects::navigation::Navigation;

/// Subtrees whose content changes from run to run or machine to machine (time, health, this
/// station's NICs and files): only where cd lands is recorded, not what is in them.
const VOLATILE: &[&str] = &[
    "Root.Runtime.Health",
    "Root.Runtime.Journal",
    "Root.Runtime.Clock",
    "Root.Runtime.Network",
    "Root.Runtime.LumaSessions",
    "Root.Runtime.Devices",
    "Root.Runtime.Users",
    "Root.Runtime.Claims",
    "Root.StationSettings",
    "Root.Library",
    "Root.Drives",
    "Root.Diagnostics",
];

/// Deeper than this the paths are fields of one object, already shown by `ls` one level up.
const MAX_DEPTH: usize = 6;
/// How many members of a pool the walk enters, and how many lines of a listing the golden keeps.
const MAX_MEMBERS: usize = 3;
const MAX_LINES: usize = 40;

const TRICKY: &[&str] = &[
    "cd patch",
    "enter patch",
    "ls patch",
    "cd showdata.patch",
    "enter showdata.patch",
    "ls showdata.patch",
    "enter root.showdata.patch",
    "cd claims",
    "enter claims",
    "ls claims",
    "cd showdata",
    "cd SHOWDATA",
    "cd show",
    "cd data",
    "cd sett",
    "cd /",
    "cd ..",
    "cd .",
    "cd ShowData/Devices",
    "cd ShowData.Devices",
    "cd Root.ShowData.Devices.Dmx.Patch.Fixtures.101",
    "cd Root.ShowData.Devices.Dmx.Patch.Fixtures.999",
    "cd Root.ShowData.DataPools.1.Groups",
    "cd Root.ShowData.DataPools.1.Groups.2",
    "cd Root.ShowData.DataPools.7",
    "cd Root.ShowData.DataPools.DataPool",
    "cd Root.ShowData.Assets",
    "cd Root.Nothing",
    "cd root",
    "cd Root",
    "ls",
    "ls /",
    "ls ..",
    "ls Root.ShowData.DataPools.1",
    "ls Root.ShowData.DataPools.1.Groups",
    "ls Root.ShowData.DataPools",
    "ls Root.ShowData.Assets",
    "ls Root.Nothing",
    "List Macro",
    "List Timecode",
    "List Preset",
    "List Group",
    "List Root",
    "List root.showdata",
    "List Root.ShowData.Devices.Dmx.Patch",
    "List Root.ShowData.Devices.Dmx.Patch.Fixtures",
    "List Root.ShowData.DataPools.1.Groups",
    "List patch",
    "tree Root.ShowData.Devices.Dmx.Patch",
    "tree ShowData.Devices.Dmx.Patch",
    "tree Root.ShowData.DataPools",
    "cat ShowData.Devices.Dmx.Patch.Fixtures.101",
    "cat Root.ShowData.DataPools.1.Groups.1",
    "stat ShowData.Devices.Dmx.Patch.Fixtures.101",
    "stat Root.ShowData.DataPools.1",
    "cat /ShowData/Devices/Dmx/Patch/Fixtures/101",
    "tree ShowData.Devices",
    "tree Root -d 2",
    "find Groups",
    "find Wash",
    "find All",
    "count Root.ShowData",
    "Select Root.ShowData.DataPools.1",
    "Deselect",
];

fn run(state: &mut AppState, line: &str) -> String {
    let out = match dispatch(state, line) {
        DispatchResult::Output(reply) => String::from(reply),
        DispatchResult::ContextSwitch(ctx) => {
            state.context = ctx;
            String::new()
        }
        DispatchResult::Ask(ask) => format!("ask: {}: {}", ask.title(), ask.view().text),
        DispatchResult::Logout => "logout".into(),
        DispatchResult::Exit => "exit".into(),
    };
    lumacore_objects::text::strip_ansi(&out)
        .trim_end()
        .to_string()
}

/// Where the shell stands after a line: path, prompt scope, claimable object, context.
fn position(state: &AppState) -> String {
    format!(
        "  @ {} · scope {} · claim {:?} · context {:?} · repl {:?}",
        state.path_string(),
        state.scope_label(),
        state.claimable_at_path(),
        state.context_from_path(),
        state.context,
    )
}

fn record(state: &mut AppState, out: &mut String, line: &str) {
    run(state, "cd /");
    state.context = lumacore_objects::ReplContext::Fixture;
    let text = run(state, line);
    let text = numbered_guids(&text);
    out.push_str(&format!("$ {line}\n"));
    // The first lines of a long listing (the rig's thousand fixtures): the golden pins the shape.
    let lines: Vec<&str> = text.lines().collect();
    for l in lines.iter().take(MAX_LINES) {
        out.push_str(&format!("  | {l}\n"));
    }
    if lines.len() > MAX_LINES {
        out.push_str(&format!("  | … (+{} lines)\n", lines.len() - MAX_LINES));
    }
    out.push_str(&position(state));
    out.push('\n');
}

/// Every GUID in a listing replaced by its order of appearance in the whole golden: the demo is
/// built per process, so its GUIDs are fresh every run (decisions §40 15).
fn numbered_guids(text: &str) -> String {
    thread_local! {
        static SEEN: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    let is_guid = |s: &str| {
        s.len() == 36
            && s.char_indices().all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            })
    };
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(|c: char| c.is_ascii_hexdigit()) {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest.find(|c: char| !(c.is_ascii_hexdigit() || c == '-')).unwrap_or(rest.len());
        let word = &rest[..end];
        // A listing's short form (`guid:361943aa`) can't be tied to a full one: blanked.
        if word.len() == 8 && out.ends_with("guid:") && word.chars().all(|c| c.is_ascii_hexdigit()) {
            out.push_str("········");
        } else if is_guid(word) {
            let n = SEEN.with(|seen| {
                let mut seen = seen.borrow_mut();
                match seen.iter().position(|g| g == word) {
                    Some(i) => i,
                    None => {
                        seen.push(word.to_string());
                        seen.len() - 1
                    }
                }
            });
            out.push_str(&format!("guid#{n}"));
        } else {
            out.push_str(word);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn volatile(path: &str) -> bool {
    VOLATILE
        .iter()
        .any(|v| path == *v || path.starts_with(&format!("{v}.")))
}

/// Every path cd can reach: the static tree, the collections objects publish, numbered
/// datapools and objects (by number), breadth first.
fn reachable(state: &AppState) -> Vec<String> {
    let root = &state.root;
    let mut found = vec!["Root".to_string()];
    let mut i = 0;
    while i < found.len() {
        let path = found[i].clone();
        i += 1;
        if path.split('.').count() > MAX_DEPTH || (volatile(&path) && path.split('.').count() > 2) {
            continue;
        }
        let mut names: Vec<String> = lumacore_objects::navigation::nav_children_at(&path);
        if !path.starts_with("Root.Drives")
            && let Some(dynamic) = lumacore_objects::vfs::dynamic_children_at(&path, root)
        {
            // A named entry (a protocol under its group) by its name, an object by its number —
            // the first few of a pool (every kind of path once, not the rig's thousand fixtures).
            let named = dynamic.iter().filter(|c| c.node_type == "Folder").map(|c| c.display_name.clone());
            let numbered = dynamic.iter().filter(|c| c.node_type != "Folder").map(|c| c.id.to_string()).take(MAX_MEMBERS);
            names.extend(named.chain(numbered));
        }
        for name in names {
            let child = format!("{path}.{name}");
            if !found.contains(&child) {
                found.push(child);
            }
        }
    }
    found
}

fn golden() -> String {
    let mut state = lumacore_objects::demo::fresh_state();
    let mut out = String::from(
        "# Generated by tree_paths_are_unchanged (tests/catalog paths/tests.rs). Do not edit.\n\n",
    );

    out.push_str("## Every reachable path: cd, then ls\n\n");
    for path in reachable(&state) {
        record(&mut state, &mut out, &format!("cd {path}"));
        if !volatile(&path) {
            record(&mut state, &mut out, &format!("ls {path}"));
        }
    }

    out.push_str("\n## Tricky inputs\n\n");
    for line in TRICKY {
        record(&mut state, &mut out, line);
    }
    // Relative steps from where the last one stood.
    run(&mut state, "cd /");
    for line in [
        "cd showdata",
        "cd devices",
        "cd patch",
        "cd ..",
        "cd ..",
        "cd DataPools",
        "cd 1",
        "cd Groups",
        "ls",
    ] {
        let text = run(&mut state, line);
        out.push_str(&format!("$ {line} (relative)\n"));
        for l in text.lines() {
            out.push_str(&format!("  | {l}\n"));
        }
        out.push_str(&position(&state));
        out.push('\n');
    }

    // This machine's name (the station is named after the host).
    let station = state.root.station_settings.station_name.clone();
    if station.is_empty() {
        out
    } else {
        out.replace(&station, "<station>")
    }
}

#[test]
fn tree_paths_are_unchanged() {
    let file =
        lumacore_objects::demo::repo_root().join("tests/catalog/testdata/tree_paths.golden.txt");
    let made = golden();
    if std::env::var_os("LUMACORE_BLESS").is_some() {
        std::fs::create_dir_all(file.parent().expect("dir")).expect("testdata dir");
        std::fs::write(&file, &made).expect("write the golden file");
        return;
    }
    let committed = std::fs::read_to_string(&file).unwrap_or_default();
    if committed != made {
        let first = committed
            .lines()
            .zip(made.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(made.lines().count()));
        panic!(
            "the tree's paths changed (first difference at line {}): `{}` → `{}`. If on purpose, rewrite with LUMACORE_BLESS=1 and read the diff.",
            first + 1,
            committed.lines().nth(first).unwrap_or(""),
            made.lines().nth(first).unwrap_or(""),
        );
    }
}
