// ██████╗ ██╗   ██╗██╗     ███████╗███████╗
// ██╔══██╗██║   ██║██║     ██╔════╝██╔════╝
// ██████╔╝██║   ██║██║     █████╗  ███████╗
// ██╔══██╗██║   ██║██║     ██╔══╝  ╚════██║
// ██║  ██║╚██████╔╝███████╗███████╗███████║
// ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚══════╝╚══════╝
// FILE: rules

//! The Rules window (decisions 0294): one layer's AttributeRule list — the show's, or a profile's
//! own — in order (first match wins), each rule's pattern, the feature group it places into (in
//! that group's colour as the viewer sees it), its template family and feature, and whether it
//! is standard or learned; `which` = an attribute name the window asks about: the rule that places
//! it, the group and why (the `Info AttributeRule "name"` lookup). Every gesture is a line: the
//! rows carry them.
//!
//! Owns:          view (the `rules` feed).
//! Doesn't own:   the rules and the resolver (lumacore_tree::attribute_resolver), the words
//!                (lumacore_objects::commands::attributes).
//! Entry points:  view.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links, §30 "12f-a".

use lumacore_objects::attribute_board::{BoardTarget, owner_words};
use lumacore_tree::ShowData;
use lumacore_tree::attribute_resolver::{self, AttributeLayer, Ctx};
use lumacore_tree::fixtures::{FileAttribute, RuleKind};
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The rules of `target`'s layer for `user`: `{title, owner, rows: [{no, guid, pattern, group,
/// colour, family, feature, match, kind, hit}], which: {name, rule, group, why} | null, lines}`.
pub fn view(show: &ShowData, user: &str, target: BoardTarget, which: &str) -> Value {
    let (layer, title): (&dyn AttributeLayer, String) = match target {
        BoardTarget::Show => (show, "Show".into()),
        BoardTarget::Profile(no) => match show.user_profiles.get(&no) {
            Some(p) if p.content.attributes.is_seeded() => (&p.content.attributes, p.meta.identity.name.clone().unwrap_or_default()),
            // A profile that never regrouped follows the show's rules.
            Some(p) => (show, p.meta.identity.name.clone().unwrap_or_default()),
            None => return json!({ "missing": true, "rows": [] }),
        },
    };
    let owner = owner_words(show, user, target);
    let word = if owner.is_empty() { "AttributeRule".to_string() } else { format!("{owner} AttributeRule") };
    let paint = lumacore_objects::paint::Painter::new(show, user);
    let which = which.trim().trim_matches('"');
    let found = (!which.is_empty()).then(|| {
        let ctx = Ctx::of(show);
        attribute_resolver::resolve_in(layer, &ctx, &FileAttribute { name: which.to_string(), ..Default::default() })
    });
    let rows: Vec<Value> = layer
        .rules()
        .iter()
        .map(|(no, r)| {
            let g = layer.groups().values().find(|g| g.meta.identity.guid == r.group);
            json!({
                "no": no,
                "guid": r.meta.identity.guid,
                "pattern": r.pattern,
                "group": g.map(|g| g.meta.name().unwrap_or_default().to_string()).unwrap_or_else(|| "—".into()),
                "colour": g.map_or(Value::Null, |g| crate::attribute_board::group_colour(&paint, g)),
                "paint": g.map_or(Value::Null, |g| crate::attribute_board::group_colour(&paint, g)),
                "family": r.class.clone().unwrap_or_else(|| "—".into()),
                "feature": r.feature.clone().unwrap_or_default(),
                "match": if r.kind == RuleKind::Keyword { "keyword" } else { "family" },
                "kind": if r.learned { "learned" } else { "standard" },
                "hit": found.as_ref().is_some_and(|f| f.rule == Some(r.meta.identity.guid)),
            })
        })
        .collect();
    let which_json = found.as_ref().map(|f| {
        let rule = f.rule.and_then(|g| layer.rules().iter().find(|(_, r)| r.meta.identity.guid == g).map(|(n, _)| *n));
        let group = layer.groups().values().find(|g| g.meta.identity.guid == f.group).map(|g| g.meta.name().unwrap_or_default().to_string());
        json!({ "name": which, "rule": rule, "group": group, "why": f.placement.why, "how": f.placement.how.word() })
    });
    let learned = layer.rules().values().filter(|r| r.learned).count();
    json!({
        "title": title,
        "owner": owner,
        "count": { "standard": layer.rules().len() - learned, "learned": learned },
        "rows": rows,
        "which": which_json,
        "groups": layer.groups().values().map(|g| g.meta.name().unwrap_or_default().to_string()).collect::<Vec<_>>(),
        "lines": {
            "move": format!("Move {word} {{from}} At {{to}}"),
            "pattern": format!("Set {word} {{no}} \"Pattern\" {{value}}"),
            "group": format!("Set {word} {{no}} \"FeatureGroup\" {{value}}"),
            "delete": format!("Delete {word} {{no}}"),
            "store": format!("Store {word} {{pattern}} FeatureGroup {{group}}"),
            "info": format!("Info {word} {{name}}"),
        },
    })
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
