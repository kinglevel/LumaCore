// ███╗   ███╗███████╗███╗   ██╗██╗   ██╗███████╗
// ████╗ ████║██╔════╝████╗  ██║██║   ██║██╔════╝
// ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║███████╗
// ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║╚════██║
// ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝███████║
// ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: menus

//! Every menu as the kit reads it (decisions §30 "12d", `GET /api/menu-types`): a menu's tabs,
//! each grid's columns with the kind, range and level of the declared property they show, the
//! slab's cells from what the grid's object really takes, the bound cells and the power toggle.
//!
//! Owns:          `menu_types_json`, `menu_json`, `column_json`, `slab_json` (`GET /api/menu-types`).
//! Doesn't own:   the menu table (`lumacore_objects::menu_defs`), the rows (`menu_rows`).
//! Entry points:  menu_types_json, menu_json, column_json, slab_json.
//! Decisions:     §30 "12d", §34 20.7, §34 21.

use lumacore_objects::menu_defs::{Col, Content, Detail, Look, MenuDef, MenuKind, Own, PartContent, Tab};
use lumacore_objects::objects::{ObjectDef, Verb};
use lumacore_objects::props::PropKind;
use lumacore_registry::command::Rights;
use serde_json::{Value, json};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

const GUEST: Rights = Rights::Guest;

/// The column a bound cell edits: `prop`, labelled `label`.
fn bound_col(prop: &'static str, label: &'static str) -> Col {
    Col {
        id: "bound", label, field: "", prop: Some(prop), own: None, width: 0, priority: 1, frozen: false,
        section: "", unit: "", per_row: false, tone: "", expand: "", detail: "", draft: false,
    }
}

fn kind_word(k: MenuKind) -> &'static str {
    match k {
        MenuKind::Setup => "setup",
        MenuKind::Page => "page",
    }
}

/// A column's kind, range and level from the object's declared property
/// (a keyed setting or a tree prop), else from the column's own kind.
/// Err = a prop the object doesn't declare (the registry test fails).
pub fn column_json(obj: Option<&ObjectDef>, c: &Col) -> Result<Value, String> {
    let mut v = json!({
        "id": c.id, "label": c.label, "field": c.field, "prop": c.prop,
        "width": c.width, "priority": c.priority, "frozen": c.frozen,
        "section": c.section, "unit": c.unit, "per_row": c.per_row,
        "tone": if c.tone.is_empty() { None } else { Some(c.tone) },
        "expand": if c.expand.is_empty() { None } else { Some(c.expand) },
        "detail": if c.detail.is_empty() { None } else { Some(c.detail) },
        "draft": c.draft,
    });
    // A column the page makes per network: a station row's NIC (text, the
    // Station object's Setup level).
    if !c.expand.is_empty() {
        let rights = obj.map_or(GUEST, |o| (o.rights)(Verb::Set, &[]));
        v["kind"] = json!("text");
        v["rights"] = json!(rights.label());
        v["readonly"] = json!(obj.is_none());
        return Ok(v);
    }
    let mut readonly = c.prop.is_none() || obj.is_none();
    let (kind, rights) = match (c.prop, obj) {
        (Some(p), Some(o)) => {
            let found = o
                .props
                .and_then(|d| d.get(p))
                .map(|s| (prop_kind(s.kind, s.words, s.named), lumacore_objects::props::rights_of(o, s), s.unit, o.props.map_or(s.home, |d| d.home_of(s))));
            match (found, c.own) {
                (Some((k, r, u, home)), _) => {
                    if c.unit.is_empty() && !u.is_empty() {
                        v["unit"] = json!(u);
                    }
                    // Where the value lives (decisions §39): the edit card's cells show it.
                    v["home"] = json!(home.word());
                    // Edited live under a claim area's draft (the Patch dialog's Live mode).
                    v["live"] = json!(o.props.is_some_and(|d| d.is_live(p)));
                    (k, r)
                }
                // An object that sets it in its own handler (User).
                (None, Some(own)) if o.supports(Verb::Set) => (own_kind(own), (o.rights)(Verb::Set, &["x", p])),
                (None, _) => return Err(format!("{} has no property {p}", o.keyword)),
            }
        }
        _ => {
            readonly = true;
            (c.own.map_or_else(|| json!({"kind": "text"}), own_kind), GUEST)
        }
    };
    for (k, val) in kind.as_object().into_iter().flatten() {
        v[k] = val.clone();
    }
    v["rights"] = json!(rights.label());
    v["readonly"] = json!(readonly);
    Ok(v)
}

fn prop_kind(k: PropKind, words: (&str, &str), named: &[&str]) -> Value {
    match k {
        PropKind::OnOff | PropKind::Switch => json!({"kind": "bool", "options": [words.0, words.1]}),
        PropKind::Int { min, max } => json!({"kind": "int", "min": min, "max": max, "options": named}),
        PropKind::Choice(o) => json!({"kind": "list", "options": o}),
        PropKind::Link { .. } | PropKind::OwnLink { .. } => json!({"kind": "list", "options": []}),
        PropKind::Text | PropKind::Colour | PropKind::Rgba | PropKind::Icon | PropKind::Declared => json!({"kind": "text"}),
    }
}

fn own_kind(o: Own) -> Value {
    match o {
        Own::Text => json!({"kind": "text"}),
        Own::Bool(a, b) => json!({"kind": "bool", "options": [a, b]}),
        Own::List(opts) => json!({"kind": "list", "options": opts}),
    }
}

/// One slab cell: its line template (None = the client does it, or it is
/// greyed with `off`), why it's greyed and the level.
fn cell(line: Option<String>, off: Option<String>, rights: Rights) -> Value {
    json!({"line": line, "off": off, "rights": rights.label()})
}

/// The left half of the function block for a tab (decisions §30 "12d": the
/// fixed order Select · New · Copy · Cut · Paste · Delete · Oops · Import ·
/// Export · Edit), from what the grid's object really takes.
pub fn slab_json(tab: &Tab) -> Value {
    let oops = cell(Some("Oops".into()), None, GUEST);
    let grid = match tab.content {
        Content::Grid(g) | Content::Drives(g) => Some(g),
        _ => None,
    };
    let Some(g) = grid else {
        let why = match tab.content {
            Content::Settings(_) => "Settings cells: tap one to change it",
            _ => "Nothing to select here",
        };
        let off = |w: &str| cell(None, Some(w.to_string()), GUEST);
        return json!({
            "select": off(why), "new": off(why), "copy": off(why), "cut": off(why),
            "paste": off(why), "delete": off(why), "oops": oops,
            "import": off("Import of settings is planned (12d part 5)"),
            "export": off("Export of settings is planned (12d part 5)"),
            "edit": off(why),
        });
    };
    let Some(obj) = g.object_decl().map(lumacore_objects::objects::def_of) else {
        let off = cell(None, Some(g.hardware.unwrap_or("These rows aren't objects").to_string()), GUEST);
        return json!({
            "select": cell(None, None, GUEST), "new": off.clone(), "copy": off.clone(), "cut": off.clone(),
            "paste": off.clone(), "delete": off.clone(), "oops": oops, "import": off.clone(),
            "export": off.clone(), "edit": off,
        });
    };
    let plural = format!("{}s", obj.keyword);
    let verb_cell = |verb: Verb, line: String| -> Value {
        let rights = (obj.rights)(verb, &[]);
        if obj.supports(verb) {
            cell(Some(line), None, rights)
        } else if obj.is_planned(verb) {
            cell(None, Some(format!("{} {} is planned", verb.label(), obj.keyword)), rights)
        } else {
            cell(None, Some(format!("{plural} can't be {}", past(verb))), rights)
        }
    };
    // Rows that come from the hardware (or the network, or Assign) can't be
    // made, copied, moved, pasted, deleted or imported here (decisions §30
    // "12d": greyed with the reason).
    let hw = |c: Value, verb: Verb| match g.hardware {
        Some(why) => cell(None, Some(why.to_string()), (obj.rights)(verb, &[])),
        None => c,
    };
    let editable = g.columns.iter().any(|c| c.prop.is_some() || !c.expand.is_empty());
    let paste = if obj.supports(Verb::Copy) || obj.supports(Verb::Move) {
        cell(None, Some("Copy or Cut first".into()), (obj.rights)(Verb::Store, &[]))
    } else {
        cell(None, Some(format!("{plural} can't be copied or moved")), (obj.rights)(Verb::Store, &[]))
    };
    // A New that isn't the object's Store (Library: `NewShow`).
    let new = match (g.store, g.new_rights) {
        (Some(line), Some(rights)) => cell(Some(line.into()), None, rights),
        _ => hw(verb_cell(Verb::Store, g.store.unwrap_or("Store {object}").into()), Verb::Store),
    };
    // On the object door the station keeps the clipboard (decisions §35 9): Copy / Cut put the
    // rows there, Paste puts them at the selected row (bare: a copy's next free numbers).
    let on_door = obj.tree.as_ref().is_some_and(|t| t.door.is_some());
    let (copy, cut, paste) = if on_door {
        (
            hw(verb_cell(Verb::Copy, "Copy {object} {rows}".into()), Verb::Copy),
            hw(verb_cell(Verb::Cut, "Cut {object} {rows}".into()), Verb::Cut),
            hw(verb_cell(Verb::Paste, "Paste {object} {at}".into()), Verb::Paste),
        )
    } else {
        (
            hw(verb_cell(Verb::Copy, "Copy {object} {rows} At {at}".into()), Verb::Copy),
            hw(verb_cell(Verb::Move, "Move {object} {rows} At {at}".into()), Verb::Move),
            hw(paste, Verb::Store),
        )
    };
    json!({
        "select": cell(None, None, GUEST),
        "new": new,
        "copy": copy,
        "cut": cut,
        "paste": paste,
        "delete": hw(verb_cell(Verb::Delete, format!("Delete {{object}} {{rows}}{}", if matches!(tab.content, Content::Drives(_)) { "{drive}" } else { "" })), Verb::Delete),
        "oops": oops,
        // A tab whose own action is an import (Patch: Import MVR…, decisions §34 "Import MVR
        // hardened" 1) takes the Import cell: no line, the page opens its dialog.
        "import": if tab.actions.iter().any(|a| a.value == "import") {
            // The line is the fallback (a bare Import Mvr asks for the file); the page opens the
            // dialog first.
            cell(Some("Import Mvr".into()), None, lumacore_objects::commands::registry::Rights::Setup)
        } else {
            hw(verb_cell(Verb::Import, "Import {object}".into()), Verb::Import)
        },
        "export": verb_cell(Verb::Export, "Export {object} {rows}".into()),
        "edit": if editable {
            cell(None, None, (obj.rights)(Verb::Set, &[]))
        } else {
            cell(None, Some("Nothing here can be changed".into()), GUEST)
        },
    })
}

fn past(v: Verb) -> &'static str {
    match v {
        Verb::Store => "made here",
        Verb::Copy => "copied",
        Verb::Move => "moved",
        Verb::Delete => "deleted",
        Verb::Import => "imported",
        Verb::Export => "exported",
        _ => "changed",
    }
}

fn tab_json(t: &Tab) -> Result<Value, String> {
    let content = match t.content {
        Content::Grid(g) | Content::Drives(g) => {
            let columns = g.columns.iter().map(|c| column_json(g.object_decl().map(lumacore_objects::objects::def_of), c)).collect::<Result<Vec<_>, _>>()?;
            let matrix = g.matrix.iter().map(|c| column_json(g.object_decl().map(lumacore_objects::objects::def_of), c)).collect::<Result<Vec<_>, _>>()?;
            json!({
                "kind": "grid",
                "drives": matches!(t.content, Content::Drives(_)),
                "grid": {
                    "key": g.key,
                    "object": g.object_decl().map(|o| o.keyword),
                    "row_object": g.row_object,
                    "id": g.id,
                    "row_key": g.row_key,
                    "feed": g.feed,
                    "rows": g.rows,
                    "cards": g.cards,
                    "new_line": if g.hardware.is_some() { None } else { g.new_line },
                    "hardware": g.hardware,
                    "left_filter": g.left_filter,
                    "locked": g.locked.map(|(field, why)| json!({"field": field, "why": why})),
                    "owner": g.owner,
                    "matrix": if matrix.is_empty() { None } else { Some(matrix) },
                    "dmx_map": g.dmx_map,
                    "detail": g.detail.map(detail_json).transpose()?,
                    "columns": columns,
                }
            })
        }
        Content::Settings(target) => json!({"kind": "settings", "settings": target}),
        Content::Custom(note) => json!({"kind": "custom", "note": note}),
        Content::Empty(note) => json!({"kind": "empty", "note": note}),
    };
    let bound = t
        .bound
        .iter()
        .map(|b| {
            let obj = lumacore_objects::objects::find(b.object).ok_or_else(|| format!("no object {}", b.object))?;
            let kind = column_json(Some(obj), &bound_col(b.prop, b.label))?;
            Ok(json!({"object": obj.keyword, "prop": b.prop, "label": b.label, "kind": kind["kind"], "options": kind.get("options"), "rights": kind["rights"]}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "name": t.name,
        "label": t.label,
        "sub": t.sub,
        "home": t.home,
        "phone": t.phone,
        "content": content,
        "slab": slab_json(t),
        "actions": t.actions.iter().map(|a| json!({
            "id": a.id, "label": a.label, "icon": a.icon, "tone": a.tone, "line": a.line,
            "rights": a.rights.label(), "needs": if a.needs.is_empty() { None } else { Some(a.needs) },
            "mine": if a.mine.is_empty() { None } else { Some(a.mine) },
            "claim": a.claim,
            "value": if a.value.is_empty() { None } else { Some(a.value) },
            "short": if a.short.is_empty() { None } else { Some(a.short) },
            "when": if a.when.is_empty() { None } else { Some(a.when) },
        })).collect::<Vec<_>>(),
        "bound": bound,
    }))
}

/// A row's detail as the kit reads it (`<luma-detail>`).
fn detail_json(d: &Detail) -> Result<Value, String> {
    let groups = d
        .groups
        .iter()
        .map(|g| {
            let parts = g
                .parts
                .iter()
                .map(|p| {
                    let content = match &p.content {
                        PartContent::Settings(section) => json!({ "kind": "settings", "section": section }),
                        PartContent::Custom(id) => json!({ "kind": "custom", "id": id }),
                        PartContent::List(rows, key, cols) => json!({
                            "kind": "list",
                            "rows": rows,
                            "row_key": key,
                            "columns": cols.iter().map(|c| column_json(None, c)).collect::<Result<Vec<_>, _>>()?,
                        }),
                    };
                    Ok(json!({
                        "name": p.name, "label": p.label, "path": p.path, "content": content,
                        "actions": p.actions.iter().map(|(l, line)| json!({ "label": l, "line": line })).collect::<Vec<_>>(),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({ "name": g.name, "label": g.label, "path": g.path, "sub": g.sub, "parts": parts }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({ "owner": d.owner, "path": d.path, "edit": d.edit, "feed": d.feed, "groups": groups }))
}

/// One menu as the kit reads it.
pub fn menu_json(m: &MenuDef) -> Result<Value, String> {
    let power = m.power.as_ref().map(|p| {
        let look = |l: &Look| json!({"word": l.word, "press": l.press, "line": l.line});
        json!({
            "label": p.label, "feed": p.feed, "path": p.path, "invert": p.invert, "off_values": p.off_values,
            "on": look(&p.on), "off": look(&p.off), "failing": look(&p.failing), "rights": p.rights.label(),
        })
    });
    Ok(json!({
        "name": m.name,
        "label": m.label,
        "icon": m.icon,
        "aliases": m.aliases,
        "kind": kind_word(m.kind),
        "home": m.home,
        "doc": m.doc,
        "rights": GUEST.label(),
        "power": power,
        "left": m.left.as_ref().map(|l| json!({
            "feed": l.feed, "rows": l.rows, "key": l.key, "sub": l.sub, "stripe": l.stripe, "empty": l.empty,
        })),
        "area": m.area.map(|a| json!({
            "trail": a.trail,
            "kinds": a.kinds.iter().map(|(label, menu)| json!({"label": label, "menu": menu})).collect::<Vec<_>>(),
            "kinds_label": a.kinds_label, "later": a.later, "feed": a.feed, "claim": a.claim, "exit": a.exit, "close": a.close,
            "bars": a.bars.iter().map(|(title, text)| json!({ "title": title, "text": text })).collect::<Vec<_>>(),
            "adds": a.adds.iter().map(|(label, line)| json!({ "label": label, "line": line })).collect::<Vec<_>>(),
            "unit": a.unit,
            "tree": a.tree.iter().map(|t| json!({
                "id": t.id, "label": t.label, "rows": t.rows, "filter": t.filter, "list": t.list,
                "nested": t.nested, "none": t.none, "deep": t.deep, "read_only": t.read_only,
            })).collect::<Vec<_>>(),
        })),
        "tabs": m.tabs.iter().map(tab_json).collect::<Result<Vec<_>, _>>()?,
    }))
}

/// `GET /api/menu-types`: every menu.
pub fn menu_types_json() -> Value {
    Value::Array(
        lumacore_registry::get()
            .menus()
            .map(|m| menu_json(m).unwrap_or_else(|e| json!({"name": m.name, "error": e})))
            .collect(),
    )
}
