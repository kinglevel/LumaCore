//! The one-only selections (decisions §29 "11b hardening", `Select`:
//! "selects objects as default objects"). Apart from the fixture selection
//! (`Fixture 3`, groups, `apply_selection`) each user has one selected
//! DataPool, Sequence and Timecode, kept on `ShowData.Users` by GUID:
//!
//! - `Select DataPool 2` — pool objects (`Group 3`, `Preset 4.1`) resolve there;
//! - `Select Sequence 3` — bare `Go`, `Off Sequence`, `Goto Cue n`,
//!   `Store Cue n` act on it (nothing selected: E-PLAY-003, never Sequence 1);
//! - `Select Timecode 1`;
//! - by tree path: `Select Root.ShowData.DataPools.2`,
//!   `Select Root.ShowData.Playback.Sequences.3`.
//!
//! `Deselect [DataPool | Sequence | Timecode]` clears one (bare: all).
//! `cd` never selects; deleting the object clears its selection
//! (`AppState::clear_dangling_selections`). Other kinds are refused
//! (E-CMD-014): they have no one-only slot.

use crate::AppState;
use crate::errors::render_code;
use crate::objects;
use crate::text::render_success;

/// The kinds `Select` takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    DataPool,
    Sequence,
    Timecode,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::DataPool => "DataPool",
            Kind::Sequence => "Sequence",
            Kind::Timecode => "Timecode",
        }
    }

    fn of_word(word: &str) -> Option<Kind> {
        let obj = objects::find(word)?;
        if std::ptr::eq(obj, &objects::DATAPOOL) {
            Some(Kind::DataPool)
        } else if std::ptr::eq(obj, &objects::SEQUENCE) {
            Some(Kind::Sequence)
        } else if std::ptr::eq(obj, &objects::TIMECODE) {
            Some(Kind::Timecode)
        } else {
            None
        }
    }
}

fn takes() -> String {
    render_code(
        "Select takes DataPool, Sequence or Timecode (one of each per user). \
         Fixtures are selected with SelectFixtures: `SelectFixtures Group 3` or just `Group 3`.",
        "E-CMD-014",
    )
}

/// The object `kind` `id` (number or name): its number and GUID. A number
/// names the object with that number first; a name only when no number
/// matches (R1-39: `Select Sequence 3` never picks one *named* "3").
fn find(state: &AppState, kind: Kind, id: &str) -> Option<(u32, uuid::Uuid)> {
    let id = id.trim_matches('"');
    let show = &state.root.show_data;
    let all: Vec<(u32, Option<&str>, uuid::Uuid)> = match kind {
        Kind::DataPool => show.datapools.iter().map(|(n, d)| (*n, d.meta.identity.name.as_deref(), d.meta.identity.guid)).collect(),
        // In the user's selected DataPool (decisions §30 "12f").
        Kind::Sequence => show
            .datapools
            .get(&state.selected_datapool())
            .map(|d| d.sequences.iter().map(|(n, s)| (*n, s.meta.identity.name.as_deref(), s.meta.identity.guid)).collect())
            .unwrap_or_default(),
        Kind::Timecode => show
            .datapools
            .get(&state.selected_datapool())?
            .timecodes
            .iter()
            .map(|(n, t)| (*n, t.meta.identity.name.as_deref(), t.meta.identity.guid))
            .collect(),
    };
    let by_number = id.parse::<u32>().ok().and_then(|no| all.iter().find(|(n, ..)| *n == no));
    by_number
        .or_else(|| all.iter().find(|(_, name, _)| name.is_some_and(|n| n.eq_ignore_ascii_case(id))))
        .map(|(n, _, g)| (*n, *g))
}

/// `Root.ShowData.DataPools.2` / `Root.ShowData.Playback.Sequences.3`.
fn from_path(state: &AppState, path: &str) -> Option<(Kind, String)> {
    use crate::tree_path::{Node, Step, TreePath};
    let segments = crate::vfs::resolve_path(&state.navigation_path, path);
    let path = TreePath::parse_dot(&crate::vfs::dot_path_from_segments(&segments))?;
    let Some(Step::Entry(id)) = path.steps().last() else {
        return None;
    };
    let kind = match path.node() {
        Node::DataPool => Kind::DataPool,
        Node::Sequences => Kind::Sequence,
        _ => return None,
    };
    Some((kind, id.to_string()))
}

fn slot(sel: &mut lumacore_tree::Selected, kind: Kind) -> &mut Option<uuid::Uuid> {
    match kind {
        Kind::DataPool => &mut sel.datapool,
        Kind::Sequence => &mut sel.sequence,
        Kind::Timecode => &mut sel.timecode,
    }
}

/// The pick list of `kind`'s objects, the current one preselected.
fn ask_which(state: &mut AppState, kind: Kind) {
    let show = &state.root.show_data;
    let item = |no: u32, name: &str, guid: String| lumacore_wire::PickItem { no, name: name.to_string(), note: String::new(), guid };
    let items: Vec<lumacore_wire::PickItem> = match kind {
        Kind::DataPool => show.datapools.iter().map(|(n, d)| item(*n, d.meta.identity.name.as_deref().unwrap_or(""), d.meta.identity.guid.to_string())).collect(),
        Kind::Sequence => show
            .datapools
            .get(&state.selected_datapool())
            .map(|d| d.sequences.iter().map(|(n, s)| item(*n, s.meta.identity.name.as_deref().unwrap_or(""), s.meta.identity.guid.to_string())).collect())
            .unwrap_or_default(),
        Kind::Timecode => show
            .datapools
            .get(&state.selected_datapool())
            .map(|dp| dp.timecodes.iter().map(|(n, t)| item(*n, t.meta.identity.name.as_deref().unwrap_or(""), t.meta.identity.guid.to_string())).collect())
            .unwrap_or_default(),
    };
    if items.is_empty() {
        return;
    }
    let current = state
        .root
        .show_data
        .user(state.effective_user_name())
        .and_then(|u| match kind {
            Kind::DataPool => u.selected.datapool,
            Kind::Sequence => u.selected.sequence,
            Kind::Timecode => u.selected.timecode,
        })
        .and_then(|g| items.iter().find(|i| i.guid == g.to_string()))
        .or(items.first())
        .map(|i| i.no.to_string())
        .unwrap_or_default();
    let label = kind.label();
    let pick = lumacore_wire::Pick { kind: label.to_string(), items, ..Default::default() };
    state.asking = Some(
        crate::dialogs::Ask::new(format!("Select {label}"), format!("Which {label} do bare words act on?"))
            .input("which", label, current, lumacore_wire::InputKind::Link { pick })
            .button("Select", lumacore_wire::ButtonKind::Primary, format!("Select {label} {{which}}"))
            .cancel("Cancel")
            .flags(format!("a number: Select {label} <n>")),
    );
}

/// `Select DataPool|Sequence|Timecode <n | "name">` or `Select <tree path>`.
pub fn cmd_select(state: &mut AppState, args: &[&str]) -> String {
    let Some(first) = args.first() else {
        return render_code(
            "Select what? Usage: Select DataPool|Sequence|Timecode <n>  ·  Select <tree path>",
            "E-CMD-006",
        );
    };
    let (kind, id) = if let Some(kind) = Kind::of_word(first) {
        let Some(id) = args.get(1) else {
            // A bare `Select DataPool` asks with the pick list (decisions §30
            // "12c": the pick lists left from 12b, one ask path).
            ask_which(state, kind);
            return render_code(format!("Select {} which? Usage: Select {} <n>", kind.label(), kind.label()), "E-CMD-006");
        };
        (kind, id.to_string())
    } else if first.contains('.')
        && let Some(found) = from_path(state, first)
    {
        found
    } else {
        return takes();
    };
    let Some((no, guid)) = find(state, kind, &id) else {
        let code = if kind == Kind::Sequence { "E-PLAY-001" } else { "E-CMD-007" };
        return render_code(format!("{} {id} doesn't exist.", kind.label()), code);
    };
    let user = state.effective_user_name().to_string();
    let Some(u) = state.root.show_data.user_mut(&user) else {
        return render_code("No user to select for.", "E-CMD-011");
    };
    *slot(&mut u.selected, kind) = Some(guid);
    render_success(&format!("{} {no} selected", kind.label()))
}

/// `Deselect [DataPool|Sequence|Timecode]`: clear one selection (bare: all).
pub fn cmd_deselect(state: &mut AppState, args: &[&str]) -> String {
    let kinds = match args.first() {
        None => vec![Kind::DataPool, Kind::Sequence, Kind::Timecode],
        Some(word) => match Kind::of_word(word) {
            Some(kind) => vec![kind],
            None => return takes(),
        },
    };
    let user = state.effective_user_name().to_string();
    let Some(u) = state.root.show_data.user_mut(&user) else {
        return render_code("No user.", "E-CMD-011");
    };
    for kind in &kinds {
        *slot(&mut u.selected, *kind) = None;
    }
    let names: Vec<&str> = kinds.iter().map(|k| k.label()).collect();
    render_success(&format!("Deselected {}", names.join(", ")))
}
