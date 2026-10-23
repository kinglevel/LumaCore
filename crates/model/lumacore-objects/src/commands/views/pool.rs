// ██████╗  ██████╗  ██████╗ ██╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║
// ██████╔╝██║   ██║██║   ██║██║
// ██╔═══╝ ██║   ██║██║   ██║██║
// ██║     ╚██████╔╝╚██████╔╝███████╗
// ╚═╝      ╚═════╝  ╚═════╝ ╚══════╝
// FILE: pool

//! The Views pool (decisions 0287): a profile's views as pool cells. Each cell draws the view's
//! windows on each screen it stores (`layout`), names its screens and the view buttons it sits
//! on; the window's Profile setting picks whose views (the user's own by default).
//!
//! Owns:          VIEW_POOL, the cells and their layout boxes.
//! Doesn't own:   storing and recalling views (`views`), the buttons (`buttons`), the pool window.
//! Entry points:  VIEW_POOL.
//! Decisions:     docs/decisions/0287-view-bar-any-object-pool-windows, §22.

use lumacore_tree::runtime::{LayoutNode, SplitDir};
use lumacore_tree::ScreenContent;
use serde_json::{Value, json};

use crate::pools::{PoolAction, PoolCell, PoolContext, PoolDef};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The Views pool window: tap = the window's action (Call recalls the view onto this screen).
pub static VIEW_POOL: PoolDef = PoolDef {
    window: "views",
    actions: &[
        PoolAction { name: "Call", command: "{obj}", release: "" },
        PoolAction { name: "Store", command: "Store {obj}", release: "" },
        PoolAction { name: "None", command: "", release: "" },
    ],
    cells: view_cells,
    detail: None,
    head: None,
};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

fn view_cells(ctx: &PoolContext) -> Vec<PoolCell> {
    let Some(profile) = ctx.profile() else { return Vec::new() };
    profile
        .content
        .views
        .iter()
        .map(|(&no, v)| {
            let mut cell = PoolCell::of(no, &v.meta, ctx);
            cell.badges = v.screens.keys().map(|s| format!("S{s}")).collect();
            cell.badges.extend(super::buttons::slots_of_view(profile, v.meta.identity.guid).into_iter().map(|b| format!("VB {b}")));
            let screens: Vec<Value> = v.screens.iter().map(|(s, c)| json!({ "screen": s, "windows": boxes(c) })).collect();
            cell.layout = Some(json!({ "screens": screens }));
            cell
        })
        .collect()
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A screen's panes as boxes in 0–1 (`x`, `y`, `w`, `h`) with the window type each one shows.
fn boxes(content: &ScreenContent) -> Vec<Value> {
    let mut out = Vec::new();
    walk(&content.root, (0.0, 0.0, 1.0, 1.0), content, &mut out);
    out
}

fn walk(node: &LayoutNode, (x, y, w, h): (f64, f64, f64, f64), content: &ScreenContent, out: &mut Vec<Value>) {
    match node {
        LayoutNode::Leaf { windows, active } => {
            let shown = windows.get(*active).or(windows.first());
            let kind = shown.and_then(|n| content.windows.get(n)).map(|w| w.kind.as_str()).unwrap_or_default();
            out.push(json!({ "x": x, "y": y, "w": w, "h": h, "kind": kind }));
        }
        LayoutNode::Split { dir, ratio, children } => {
            // The first child takes `ratio`, the others share the rest.
            let n = children.len().max(1);
            let first = ratio.clamp(0.0, 1.0);
            let rest = if n > 1 { (1.0 - first) / (n - 1) as f64 } else { 0.0 };
            let mut at = 0.0;
            for (i, child) in children.iter().enumerate() {
                let part = if i == 0 { if n == 1 { 1.0 } else { first } } else { rest };
                let cell = match dir {
                    SplitDir::H => (x + at * w, y, part * w, h),
                    SplitDir::V => (x, y + at * h, w, part * h),
                };
                walk(child, cell, content, out);
                at += part;
            }
        }
    }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_split_is_two_boxes_by_its_ratio() {
        let leaf = |n: u32| LayoutNode::Leaf { windows: vec![n], active: 0 };
        let content = ScreenContent {
            meta: Default::default(),
            root: LayoutNode::Split { dir: SplitDir::H, ratio: 0.25, children: vec![leaf(1), LayoutNode::Split { dir: SplitDir::V, ratio: 0.5, children: vec![leaf(2), leaf(3)] }] },
            windows: Default::default(),
            encoder_bar: Default::default(),
        };
        let b = boxes(&content);
        assert_eq!(b.len(), 3);
        assert_eq!((b[0]["w"].as_f64(), b[1]["x"].as_f64(), b[2]["y"].as_f64()), (Some(0.25), Some(0.25), Some(0.5)));
    }
}
