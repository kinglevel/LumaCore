// ██╗     ██╗███████╗████████╗
// ██║     ██║██╔════╝╚══██╔══╝
// ██║     ██║███████╗   ██║
// ██║     ██║╚════██║   ██║
// ███████╗██║███████║   ██║
// ╚══════╝╚═╝╚══════╝   ╚═╝
// FILE: list

//! `List Mvr` — the open review as a table (number, status, name, type, address, group, ticked);
//! `Info Mvr` — its summary and the settings it was built with. Without a review both say how to
//! open one.
//!
//! Owns:          ListMvr, InfoMvr.
//! Doesn't own:   the review (the draft), its rule.
//! Entry points:  ListMvr, InfoMvr.
//! Decisions:     §34 "Import MVR hardened" 3.

use lumacore_syntax::ast::Target;

use crate::errors::render_code;
use crate::mvr_review::{ImportReview, RowStatus};
use crate::objects::out;
use crate::{AppState, DispatchResult};
//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS


/// `List Mvr`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "List", object = "Mvr")]
pub struct ListMvr {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for ListMvr {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(match review(state) {
            Some(r) => table(r),
            None => none(),
        })
    }
}

/// `Info Mvr`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "Mvr")]
pub struct InfoMvr {
    #[lum(target)]
    pub target: Option<Target>,
}

impl crate::commands::typed::Command for InfoMvr {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(match review(state) {
            Some(r) => info(r),
            None => none(),
        })
    }
}
// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS


fn review(state: &AppState) -> Option<&ImportReview> {
    state.patch_draft.as_ref().and_then(|d| d.review.as_ref())
}

fn none() -> String {
    render_code("No MVR review open: Claim Root.ShowData.Devices, then Import Mvr \"file\".", "E-CMD-009")
}

/// The rows as text, one line each.
fn table(r: &ImportReview) -> String {
    let mut lines = vec![format!("MVR review {}", r.summary())];
    lines.push(format!("{:>6}  {:<8} {:<24} {:<30} {:<9} {:<20} {}", "No", "Status", "Name", "Type · Mode", "Address", "MVR group", "Ticked"));
    for row in &r.rows {
        let cell = |f: &str| row.cells.iter().find(|c| c.field == f);
        let shown = |f: &str| cell(f).map(|c| if c.differs { format!("{}{}", c.file, c.live.as_ref().map(|l| format!(" (was {l})")).unwrap_or_default()) } else { c.file.clone() }).unwrap_or_default();
        let name = if row.status == RowStatus::Lost { row.name.clone() } else { shown("name") };
        let typ = if row.status == RowStatus::Lost { String::new() } else { shown("type") };
        let addr = if row.status == RowStatus::Lost { String::new() } else { shown("address") };
        let no = if row.no == 0 { if row.id.is_empty() { "—".to_string() } else { row.id.clone() } } else { row.no.to_string() };
        let mut status = row.status.label().to_string();
        if !row.match_key.is_empty() {
            status = format!("{status} ({})", row.match_key);
        }
        lines.push(format!("{:>6}  {:<8} {:<24} {:<30} {:<9} {:<20} {}{}", no, status, fit(&name, 24), fit(&typ, 30), addr, fit(&row.group_name, 20), if row.taken { "✓" } else { "·" }, row.clash.as_ref().map(|c| format!("  ✕ {c}")).unwrap_or_default()));
    }
    lines.join("\n")
}

fn info(r: &ImportReview) -> String {
    let c = r.counts();
    let s = &r.settings;
    [
        format!("MVR review of \"{}\"{}{}", r.file, r.drive.as_ref().map(|d| format!(" on {d}")).unwrap_or_default(), r.scope.as_ref().map(|t| format!(" · scope {t}")).unwrap_or_default()),
        format!("Rows: {} same, {} changed, {} new, {} lost, {} outside the scope, {} clash · {} ticked", c.same, c.changed, c.new, c.lost, c.outside, c.clash, c.taken),
        format!("Match {} · Lost {} · Rotation {} · Missing group {}", s.match_key, s.lost, s.rotation, s.missing),
        format!(
            "Addresses {} · Ids {} · Names {} · Position {} · Revisions {} · New fixtures {}",
            s.addresses.label(), s.ids.label(), s.names.label(), s.position.label(), s.revisions.label(), s.new_fixtures.label()
        ),
        format!("FID offset {} · universe offset {} · rig offset {} {} {} mm · rotate {}°", s.fid_offset, s.universe_offset, s.offset_mm[0], s.offset_mm[1], s.offset_mm[2], s.rotate_deg),
        "Import Mvr /Apply writes the ticked rows (part 4b); Import Mvr /Cancel drops the review.".to_string(),
    ]
    .join("\n")
}

fn fit(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n <= width { s.to_string() } else { format!("{}…", s.chars().take(width - 1).collect::<String>()) }
}
