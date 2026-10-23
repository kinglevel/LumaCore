//! Editor tools (decisions §30 "12e hardening" … "12e round 5 questions"):
//! The Selection Grid tools as one registry. Each tool is one command line,
//! `Grid <owner> "<Tool>" ["<Arg>"]` (e.g. `Grid "Linearize" "LeftToRight"`,
//! owner first: `Grid Group 3 …`, `Grid DataPool 2 Group 3 …`) — one
//! statement, one Oops step, computed here on the station. Tool names are
//! quoted values, not keywords.
//!
//! The registry is sent as JSON (`GET /api/tool-types`); the page draws the
//! grouped toolbar (desk) or the low sheet (phone) from it, with each tool's
//! icon, its group and — for a tool that asks — the question (its choices
//! with their icons, or a number). A planned tool is listed and refuses
//! (`isn't built yet (planned)`).

use lumacore_tree::GroupMember;
use serde_json::{Value, json};

use crate::AppState;
use crate::commands::registry::Status;
use crate::errors::render_code;
use crate::text::render_success;

/// The tools' rows are the catalog's (decisions §34 21); `run_of` binds each built one to what it does.
pub use lumacore_registry::{ToolAsk, ToolDecl};

/// What a tool does to a group's members (order and grid spots), given the asked value.
pub type ToolRun = fn(&mut [GroupMember], &[u32], &str) -> Result<(), String>;

/// The toolbar's groups, in order.
pub(crate) const GROUPS: &[&str] = &["Order", "Gaps", "Align", "Transform", "Scale", "Arrange", "Pawterns"];

/// Every tool, in toolbar order — this crate's rows in the catalog.
pub(crate) static TOOLS: &[ToolDecl] = &[
    ToolDecl {
        id: "Linearize",
        label: "Linearize",
        icon: "list-ordered",
        group: "Order",
        ask: Some(ToolAsk::Choice {
            label: "Order by",
            options: &[("Number", "hash"), ("LeftToRight", "arrow-right"), ("TopToBottom", "arrow-down")],
        }),
        status: Status::Ready,
    },
    ToolDecl { id: "Reverse", label: "Reverse", icon: "arrow-down-up", group: "Order", ask: None, status: Status::Planned },
    ToolDecl { id: "Shuffle", label: "Shuffle", icon: "shuffle", group: "Order", ask: None, status: Status::Planned },
    ToolDecl { id: "RemoveGaps", label: "Remove gaps", icon: "fold-horizontal", group: "Gaps", ask: None, status: Status::Ready },
    ToolDecl { id: "RemoveOffset", label: "Remove offset", icon: "move-up-left", group: "Gaps", ask: None, status: Status::Ready },
    ToolDecl {
        id: "Align",
        label: "Align",
        icon: "align-horizontal-justify-start",
        group: "Align",
        ask: Some(ToolAsk::Choice {
            label: "Align rows to",
            options: &[
                ("Left", "align-horizontal-justify-start"),
                ("Center", "align-horizontal-justify-center"),
                ("Right", "align-horizontal-justify-end"),
            ],
        }),
        status: Status::Ready,
    },
    ToolDecl { id: "Distribute", label: "Distribute", icon: "align-horizontal-space-around", group: "Align", ask: None, status: Status::Planned },
    ToolDecl { id: "Transpose", label: "Transpose", icon: "move-diagonal", group: "Transform", ask: None, status: Status::Ready },
    ToolDecl {
        id: "Rotate",
        label: "Rotate",
        icon: "rotate-cw",
        group: "Transform",
        ask: Some(ToolAsk::Choice { label: "Direction", options: &[("Right", "rotate-cw"), ("Left", "rotate-ccw")] }),
        status: Status::Ready,
    },
    ToolDecl {
        id: "Flip",
        label: "Flip",
        icon: "flip-horizontal-2",
        group: "Transform",
        ask: Some(ToolAsk::Choice { label: "Axis", options: &[("X", "flip-horizontal-2"), ("Y", "flip-vertical-2")] }),
        status: Status::Ready,
    },
    ToolDecl {
        id: "Multiply",
        label: "Multiply",
        icon: "maximize-2",
        group: "Scale",
        ask: Some(ToolAsk::Number { label: "Factor", min: 2, max: 16, default: 2 }),
        status: Status::Ready,
    },
    ToolDecl {
        id: "Divide",
        label: "Divide",
        icon: "minimize-2",
        group: "Scale",
        ask: Some(ToolAsk::Number { label: "Divisor", min: 2, max: 16, default: 2 }),
        status: Status::Ready,
    },
    ToolDecl { id: "MakeSymmetrical", label: "Symmetrical", icon: "columns-2", group: "Scale", ask: None, status: Status::Planned },
    ToolDecl { id: "Line", label: "Line", icon: "minus", group: "Arrange", ask: None, status: Status::Planned },
    ToolDecl { id: "Circle", label: "Circle", icon: "circle-dashed", group: "Arrange", ask: None, status: Status::Planned },
    ToolDecl { id: "Grid", label: "Grid", icon: "grid-3x3", group: "Arrange", ask: None, status: Status::Planned },
    ToolDecl { id: "UsePawternPositions", label: "Pawtern pos.", icon: "grid-2x2-check", group: "Pawterns", ask: None, status: Status::Planned },
];

/// What a built tool does, by its id — the binding of a catalog row to its code (decisions §34 21).
pub fn run_of(id: &str) -> Option<ToolRun> {
    match id {
        "Linearize" => Some(linearize),
        "RemoveGaps" => Some(remove_gaps),
        "RemoveOffset" => Some(remove_offset),
        "Align" => Some(align),
        "Transpose" => Some(transpose),
        "Rotate" => Some(rotate),
        "Flip" => Some(flip),
        "Multiply" => Some(multiply),
        "Divide" => Some(divide),
        _ => None,
    }
}

/// A tool by its name (any case).
pub fn find(name: &str) -> Option<&'static ToolDecl> {
    lumacore_registry::get().tool(name)
}

/// `GET /api/tool-types`: the groups and every tool.
pub fn tool_types_json() -> Value {
    let tools: Vec<Value> = lumacore_registry::get()
        .tools()
        .map(|t| {
            let ask = match t.ask {
                None => Value::Null,
                Some(ToolAsk::Choice { label, options }) => json!({
                    "kind": "choice",
                    "label": label,
                    "options": options.iter().map(|(v, i)| json!({ "value": v, "icon": i })).collect::<Vec<_>>(),
                }),
                Some(ToolAsk::Number { label, min, max, default }) => {
                    json!({ "kind": "number", "label": label, "min": min, "max": max, "default": default })
                }
            };
            json!({
                "id": t.id,
                "label": t.label,
                "icon": t.icon,
                "group": t.group,
                "ask": ask,
                "planned": t.status == Status::Planned,
            })
        })
        .collect();
    json!({ "groups": lumacore_registry::get().tool_groups(), "tools": tools })
}

/// `Grid [DataPool <d>] Group <n> "<Tool>" ["<Arg>"]`: run a tool on a
/// group's members. None when `args` don't name a group (the grid cursor).
pub fn run_line(state: &mut AppState, args: &[&str]) -> Option<String> {
    let group_word = |w: &&str| crate::commands::groups::GROUP.matches(w);
    let (dp, rest) = match args {
        [first, d, rest @ ..] if crate::objects::find(first).is_some_and(|o| o.keyword == "DataPool") => {
            let Some(d) = d.trim_matches('"').parse::<u32>().ok().filter(|d| state.root.show_data.datapools.contains_key(d)) else {
                return Some(render_code(format!("No DataPool {d}."), "E-CMD-007"));
            };
            (Some(d), rest)
        }
        _ => (None, args),
    };
    let [g, id, tool_args @ ..] = rest else {
        return dp.map(|_| render_code(USAGE, "E-CMD-006"));
    };
    if !group_word(g) {
        return dp.map(|_| render_code(USAGE, "E-CMD-006"));
    }
    let Ok(id) = id.trim_matches('"').parse::<u32>() else {
        return Some(render_code(format!("'{id}' is not a group number."), "E-CMD-008"));
    };
    let Some(name) = tool_args.first().map(|w| crate::commands::tokens::unquote(w)) else {
        return Some(format!("Tools: {}", lumacore_registry::get().tools().map(|t| t.id).collect::<Vec<_>>().join(", ")));
    };
    let Some(tool) = find(name) else {
        return Some(render_code(format!("No tool '{name}'. Tools: {}", ready_names()), "E-CMD-008"));
    };
    let Some(run) = run_of(tool.id).filter(|_| tool.status == Status::Ready) else {
        return Some(render_code(format!("Grid \"{}\" isn't built yet (planned).", tool.id), "E-CMD-003"));
    };
    let arg = match (tool.ask, tool_args.get(1).map(|w| crate::commands::tokens::unquote(w))) {
        (None, _) => String::new(),
        (Some(ToolAsk::Choice { options, .. }), None) => options[0].0.to_string(),
        (Some(ToolAsk::Choice { options, label }), Some(v)) => match options.iter().find(|(o, _)| o.eq_ignore_ascii_case(v)) {
            Some((o, _)) => o.to_string(),
            None => {
                let names: Vec<&str> = options.iter().map(|(o, _)| *o).collect();
                return Some(render_code(format!("{label}: one of {}, not '{v}'.", names.join(", ")), "E-CMD-008"));
            }
        },
        (Some(ToolAsk::Number { default, .. }), None) => default.to_string(),
        (Some(ToolAsk::Number { min, max, label, .. }), Some(v)) => match v.parse::<i64>() {
            Ok(n) if (min..=max).contains(&n) => n.to_string(),
            _ => return Some(render_code(format!("{label}: a whole number {min}–{max}, not '{v}'."), "E-CMD-008")),
        },
    };
    let dp = dp.unwrap_or_else(|| state.selected_datapool());
    let numbers: Vec<u32> = {
        let Some(group) = state.root.show_data.datapools.get(&dp).and_then(|d| d.groups.get(&id)) else {
            return Some(render_code(format!("Group {id} doesn't exist."), "E-CMD-007"));
        };
        let guids: Vec<_> = group.members.iter().map(|m| m.fixture).collect();
        guids
            .iter()
            .map(|g| state.root.show_data.fixture_ids_for_guids(std::slice::from_ref(g)).first().copied().unwrap_or(u32::MAX))
            .collect()
    };
    let group = state
        .root
        .show_data
        .datapools
        .get_mut(&dp)
        .and_then(|d| d.groups.get_mut(&id))
        .expect("checked above");
    if group.members.is_empty() {
        return Some(render_code(format!("Group {id} has no members."), "E-CMD-009"));
    }
    if let Err(e) = run(&mut group.members, &numbers, &arg) {
        return Some(render_code(format!("Grid \"{}\": {e}", tool.id), "E-CMD-010"));
    }
    state.show_dirty = true;
    let shown = if arg.is_empty() { String::new() } else { format!(" \"{arg}\"") };
    Some(render_success(&format!("Group {id}: {}{shown}", tool.label)))
}

const USAGE: &str = "Usage: Grid [DataPool <d>] Group <n> \"<Tool>\" [\"<Arg>\"]";

fn ready_names() -> String {
    lumacore_registry::get().tools().filter(|t| t.status == Status::Ready).map(|t| t.id).collect::<Vec<_>>().join(", ")
}

// ── The tools ────────────────────────────────────────────────────────────────

/// Order by fixture number, left to right (rows top down), or top to bottom
/// (columns left to right) — Linearize.
fn linearize(m: &mut [GroupMember], numbers: &[u32], by: &str) -> Result<(), String> {
    let mut pairs: Vec<(GroupMember, u32)> = m.iter().cloned().zip(numbers.iter().copied()).collect();
    match by {
        "Number" => pairs.sort_by_key(|(_, n)| *n),
        "LeftToRight" => pairs.sort_by_key(|(g, _)| (g.cell.z, g.cell.y, g.cell.x)),
        _ => pairs.sort_by_key(|(g, _)| (g.cell.z, g.cell.x, g.cell.y)),
    }
    for (slot, (g, _)) in m.iter_mut().zip(pairs) {
        *slot = g;
    }
    Ok(())
}

/// Close empty columns and rows (Remove Gaps).
fn remove_gaps(m: &mut [GroupMember], _: &[u32], _: &str) -> Result<(), String> {
    let mut xs: Vec<i32> = m.iter().map(|g| g.cell.x).collect();
    let mut ys: Vec<i32> = m.iter().map(|g| g.cell.y).collect();
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();
    for g in m.iter_mut() {
        g.cell.x = xs.binary_search(&g.cell.x).unwrap_or(0) as i32;
        g.cell.y = ys.binary_search(&g.cell.y).unwrap_or(0) as i32;
    }
    Ok(())
}

/// Move the whole layout to the top left corner (Remove Offset).
fn remove_offset(m: &mut [GroupMember], _: &[u32], _: &str) -> Result<(), String> {
    let (x0, y0) = (m.iter().map(|g| g.cell.x).min().unwrap_or(0), m.iter().map(|g| g.cell.y).min().unwrap_or(0));
    for g in m.iter_mut() {
        g.cell.x -= x0;
        g.cell.y -= y0;
    }
    Ok(())
}

/// Each row moves to the left edge, the centre or the right edge of the
/// widest row; a row keeps its own spacing.
fn align(m: &mut [GroupMember], _: &[u32], to: &str) -> Result<(), String> {
    use std::collections::BTreeMap;
    let mut rows: BTreeMap<(i32, i32), (i32, i32)> = BTreeMap::new();
    for g in m.iter() {
        let e = rows.entry((g.cell.z, g.cell.y)).or_insert((g.cell.x, g.cell.x));
        e.0 = e.0.min(g.cell.x);
        e.1 = e.1.max(g.cell.x);
    }
    let left = rows.values().map(|r| r.0).min().unwrap_or(0);
    let right = rows.values().map(|r| r.1).max().unwrap_or(0);
    for g in m.iter_mut() {
        let (lo, hi) = rows[&(g.cell.z, g.cell.y)];
        let shift = match to {
            "Left" => left - lo,
            "Right" => right - hi,
            _ => (left + right - lo - hi) / 2,
        };
        g.cell.x += shift;
    }
    Ok(())
}

/// Swap columns and rows.
fn transpose(m: &mut [GroupMember], _: &[u32], _: &str) -> Result<(), String> {
    for g in m.iter_mut() {
        std::mem::swap(&mut g.cell.x, &mut g.cell.y);
    }
    Ok(())
}

/// A quarter turn of the layout, right (clockwise) or left.
fn rotate(m: &mut [GroupMember], _: &[u32], dir: &str) -> Result<(), String> {
    let max_x = m.iter().map(|g| g.cell.x).max().unwrap_or(0);
    let max_y = m.iter().map(|g| g.cell.y).max().unwrap_or(0);
    for g in m.iter_mut() {
        let (x, y) = (g.cell.x, g.cell.y);
        (g.cell.x, g.cell.y) = if dir == "Left" { (y, max_x - x) } else { (max_y - y, x) };
    }
    Ok(())
}

/// Mirror the layout on X (left ↔ right) or Y (top ↔ bottom).
fn flip(m: &mut [GroupMember], _: &[u32], axis: &str) -> Result<(), String> {
    let (lo, hi) = if axis == "Y" {
        (m.iter().map(|g| g.cell.y).min().unwrap_or(0), m.iter().map(|g| g.cell.y).max().unwrap_or(0))
    } else {
        (m.iter().map(|g| g.cell.x).min().unwrap_or(0), m.iter().map(|g| g.cell.x).max().unwrap_or(0))
    };
    for g in m.iter_mut() {
        if axis == "Y" {
            g.cell.y = lo + hi - g.cell.y;
        } else {
            g.cell.x = lo + hi - g.cell.x;
        }
    }
    Ok(())
}

/// Spread the layout: every spot times n (Multiply).
fn multiply(m: &mut [GroupMember], _: &[u32], n: &str) -> Result<(), String> {
    let n: i32 = n.parse().map_err(|_| "a factor")?;
    for g in m.iter_mut() {
        g.cell.x = g.cell.x.saturating_mul(n);
        g.cell.y = g.cell.y.saturating_mul(n);
    }
    Ok(())
}

/// Pull the layout together: every spot divided by n (Divide).
fn divide(m: &mut [GroupMember], _: &[u32], n: &str) -> Result<(), String> {
    let n: i32 = n.parse().map_err(|_| "a divisor")?;
    for g in m.iter_mut() {
        g.cell.x = g.cell.x.div_euclid(n);
        g.cell.y = g.cell.y.div_euclid(n);
    }
    Ok(())
}
