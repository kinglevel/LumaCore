// ████████╗██████╗ ███████╗███████╗
// ╚══██╔══╝██╔══██╗██╔════╝██╔════╝
//    ██║   ██████╔╝█████╗  █████╗
//    ██║   ██╔══██╗██╔══╝  ██╔══╝
//    ██║   ██║  ██║███████╗███████╗
//    ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝
// FILE: tree

//! A dialog's tree in its answer's line (decisions 0293, the Update ask): `{tree}` becomes the
//! statements its ticked and unticked nodes add (`DialogTree::statements`), with the toggles and
//! selectors as answered; the line's own filling runs after.
//!
//! Owns:          Ask::tree, fill.
//! Doesn't own:   the tree's shape (lumacore-wire `DialogTree`), the rest of a line (`Ask::line_for`).
//! Entry points:  fill.
//! Decisions:     docs/decisions/0293-presets-link-into-cues-update-recast-extract.

use lumacore_wire::DialogAnswer;

use super::Ask;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Ask {
    /// A tree to tick: `{tree}` in a line becomes its statements ([`fill`]).
    pub fn tree(mut self, tree: lumacore_wire::DialogTree) -> Self {
        self.view.tree = Some(tree);
        self
    }
}

/// `line` with `{tree}` replaced by the tree's statements for `answer` (joined by `; `).
pub(super) fn fill(ask: &Ask, answer: &DialogAnswer, line: String) -> String {
    let Some(tree) = &ask.view.tree else { return line };
    let toggles = |id: &str| answer.toggles.get(id).copied().or_else(|| ask.view.toggles.iter().find(|t| t.name == id).map(|t| t.on));
    let chosen = |name: &str| {
        let s = ask.view.selectors.iter().find(|s| s.name == name)?;
        let value = answer.selectors.get(name).unwrap_or(&s.value);
        s.choices.iter().position(|c| c == value).and_then(|i| s.texts.get(i)).cloned()
    };
    line.replace("{tree}", &tree.statements(toggles, chosen).join("; "))
}
