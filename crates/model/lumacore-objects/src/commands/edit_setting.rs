//! `EditSetting <object> [<id>]` (decisions §29 "11b
//! hardening"): an object's settings. In the web or app it opens a settings
//! pop-up (`UiEvent::EditSettings`); in a terminal it lists the settings
//! with their current values and the `Set` lines that change them. Every
//! change is an ordinary command line, so it goes through the rights gate
//! and the change path.
//!
//! The settings are what the object declares (`ObjectDef::props`, decisions §33 58):
//! `EditSetting Fixture 3`, `EditSetting Show`, `EditSetting Interface "en0"` — every cell's
//! line is `Set <row> "<Property>" {value}`.

use lumacore_wire::{SettingField, UiEvent};
use serde_json::Value;

use crate::AppState;
use crate::errors::render_code;

fn text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "On" } else { "Off" }.to_string(),
        other => other.to_string(),
    }
}

/// An object's declared properties (decisions §33 58) with their current values: the row the
/// words name (`Fixture 3`, `Interface "en0"`), or its implicit one (`Station`, `Show`, your
/// profile).
fn object_fields(state: &mut AppState, word: &str, rest: &[&str]) -> Result<(String, Vec<SettingField>), String> {
    let obj = crate::objects::find(word)
        .ok_or_else(|| render_code(format!("'{word}' is not an object."), "E-CMD-001"))?;
    let props = obj.props.ok_or_else(|| render_code(format!("{} has no settings.", obj.keyword), "E-CMD-015"))?;
    // Owner first (`EditSetting UserProfile 2 Theme`): the child's settings within that row.
    if let (Some(within), Some(i)) = (props.within, rest.iter().position(|w| obj.lum.children.iter().any(|c| crate::objects::def_of(c).props.is_some_and(|p| p.view.is_some()) && c.matches(w)))) {
        let owners = crate::resolve::words(state, obj, &rest[..i])?;
        let key = match (owners.named, props.implicit) {
            (false, Some(implicit)) => implicit(state),
            _ => match owners.members.as_slice() {
                [one] => one.no.to_string(),
                _ => return Err(render_code(format!("EditSetting {} takes one row.", obj.keyword), "E-CMD-006")),
            },
        };
        let _scope = within(state, &key)?;
        return object_fields(state, rest[i], &rest[i + 1..]);
    }
    let resolved = crate::resolve::words(state, obj, rest)?;
    let first = props.all().next().ok_or_else(|| render_code(format!("{} has no settings.", obj.keyword), "E-CMD-015"))?;
    let rows = super::set::rows_of(state, obj, props, &resolved, first)?;
    let [row] = rows.as_slice() else {
        return Err(render_code(format!("EditSetting {} takes one row.", obj.keyword), "E-CMD-006"));
    };
    let path_object = super::set::reads_a_path(obj);
    let json = match (&obj.tree, props.view) {
        // A Path object's row is its GUID: its properties are keyed, no JSON by number.
        (Some(_), _) if path_object => None,
        (Some(t), _) => match t.object_in(&state.root, row.datapool, row.no) {
            Some(crate::vfs::NodeView::Object { json, .. }) => Some(json),
            _ => None,
        },
        (None, Some(view)) => view(&state.root, row.no),
        (None, None) => None,
    };
    // How a line names the row: none for an implicit one, else its number or quoted key.
    let row_word = if !resolved.named {
        String::new()
    } else if path_object {
        // Named as typed (`Tag 1.1`, `Tag "Lx1"`): its path today.
        format!("{} ", rest.join(" "))
    } else if row.key.parse::<u32>().is_ok() {
        format!("{} ", row.key)
    } else {
        format!("\"{}\" ", row.key)
    };
    let title = match props.label {
        Some(label) if obj.tree.is_none() || path_object => label(state, &row.key),
        _ => format!("{} {}", obj.keyword, row.key),
    };
    let mut fields = fields_of(state, obj, props, row, json.as_ref(), &format!("Set {} {row_word}", obj.keyword));
    // Settings children that live in this row (a profile's Theme, its Paint, its LockScreen;
    // decisions §33 60): their cells too, owner first.
    fn children(state: &AppState, obj: &'static crate::objects::ObjectDef, row: &super::set::Row, head: &str, fields: &mut Vec<SettingField>) {
        for child in obj.lum.children.iter().map(|c| crate::objects::def_of(c)) {
            let Some(props) = child.props.filter(|p| p.view.is_some()) else { continue };
            let json = props.view.and_then(|v| v(&state.root, row.no));
            let head = format!("{head}{} ", child.keyword);
            fields.extend(fields_of(state, child, props, row, json.as_ref(), &head));
            children(state, child, row, &head, fields);
        }
    }
    children(state, obj, row, &format!("Set {} {row_word}", obj.keyword), &mut fields);
    // Every show object with a NodeMeta (decisions §12 "Tags hardened" 8): its facts on the Meta
    // tab, and the tabs other crates add (Tags).
    let guid = json.as_ref().and_then(|j| j.get("meta")).and_then(|m| m.get("guid")).and_then(Value::as_str).and_then(|g| g.parse().ok());
    if let Some(guid) = guid.or_else(|| path_object.then(|| row.key.parse().ok()).flatten()) {
        let dp = match (row.datapool, obj.tree.as_ref().and_then(|t| t.datapool.as_ref())) {
            (Some(dp), Some(_)) => format!("DataPool {dp} "),
            _ => String::new(),
        };
        let target = format!("{dp}{} {}", obj.keyword, row_word.trim_end());
        // The object's own tabs first, then Meta and the added tabs (the tabs follow the order).
        let (meta, own): (Vec<SettingField>, Vec<SettingField>) = std::mem::take(&mut fields).into_iter().partition(|f| f.section == "Meta");
        fields = own;
        fields.extend(meta);
        fields.extend(facts(state, guid));
        for tab in lumacore_registry::get().rows::<crate::objects::SettingsTab>() {
            fields.extend((tab.fields)(state, guid, target.trim_end()));
        }
    }
    Ok((title, fields))
}

/// The Meta tab's facts (decisions §12 "Tags hardened" 8): read only, from the identity walk.
fn facts(state: &AppState, guid: uuid::Uuid) -> Vec<SettingField> {
    let walk = state.root.show_data.walk_identity();
    let path = walk.objects.iter().find(|o| o.guid == guid).map(|o| format!("{}.{}", crate::tree_path::Node::ShowData.path(), o.path)).unwrap_or_default();
    let mut users: Vec<String> = walk
        .links
        .iter()
        .filter(|l| l.target == guid)
        .filter_map(|l| l.from)
        .filter_map(|f| walk.objects.iter().find(|o| o.guid == f))
        .map(|o| format!("{} {}", o.kind, o.path.rsplit('.').next().unwrap_or("")))
        .collect();
    users.dedup();
    let used = if users.is_empty() {
        "nothing".to_string()
    } else if users.len() > 4 {
        format!("{} and {} more", users[..4].join(", "), users.len() - 4)
    } else {
        users.join(", ")
    };
    let fact = |name: &str, value: String| SettingField {
        name: name.to_string(),
        key: String::new(),
        changed: false,
        reset: String::new(),
        section: "Meta".to_string(),
        input: lumacore_wire::FieldInput::Text { max_len: 0 },
        hint: String::new(),
        value,
        line: String::new(),
        pick: None,
        home: String::new(),
        unit: String::new(),
        rights: String::new(),
        readonly: "a fact of the show".to_string(),
    };
    vec![fact("GUID", guid.to_string()), fact("Path", path), fact("Used by", used)]
}

/// The cells of a row's declared properties; each line is `<head>"<Property>" {value}`.
fn fields_of(
    state: &AppState,
    obj: &'static crate::objects::ObjectDef,
    props: &crate::props::Props,
    row: &super::set::Row,
    json: Option<&Value>,
    head: &str,
) -> Vec<SettingField> {
    props
        .all()
        .map(|p| {
            let (value, pick) = match (p.access, json) {
                (crate::props::Access::Field(field), Some(json)) => {
                    let raw = field.split('.').try_fold(json, |v, k| v.get(k)).cloned().unwrap_or(Value::Null);
                    let pick = pick(state, p.kind, row.no, &raw);
                    // A link shows the name it points at, not its GUID.
                    let value = match &pick {
                        Some(pick) => pick
                            .items
                            .iter()
                            .find(|i| raw.as_str() == Some(i.guid.as_str()))
                            .map(|i| i.name.clone())
                            .or_else(|| pick.missing.clone())
                            .unwrap_or_default(),
                        None => crate::props::shown(p.kind, &raw).unwrap_or_else(|| text(&raw)),
                    };
                    (value, pick)
                }
                _ => (super::set::current(state, obj, row, p).unwrap_or_default(), None),
            };
            let section = if !p.section.is_empty() {
                p.section.to_string()
            } else if std::ptr::eq(obj, &crate::objects::USER_PROFILE) {
                super::media::profile_section(p.name).to_string()
            } else if matches!(p.access, crate::props::Access::Field(f) if f.starts_with("meta.")) {
                // The NodeMeta every show object has: the Meta tab (decisions §12 "Tags hardened" 8).
                "Meta".to_string()
            } else {
                String::new()
            };
            SettingField {
                name: p.name.to_string(),
                key: String::new(),
                changed: false,
                reset: String::new(),
                section,
                input: input(p),
                hint: if matches!(p.access, crate::props::Access::Keyed { .. }) { p.doc.to_string() } else { String::new() },
                value,
                line: format!("{head}\"{}\" {{value}}", p.name),
                pick,
                // Facts (About) live nowhere to change: no home mark.
                home: if props.readonly.is_some() { String::new() } else { props.home_of(p).word().to_string() },
                unit: p.unit.to_string(),
                rights: crate::props::rights_of(obj, p).label().to_string(),
                readonly: props.readonly.unwrap_or("").to_string(),
            }
        })
        .collect()
}

/// The kit's input for a property.
pub fn input(p: &crate::props::Property) -> lumacore_wire::FieldInput {
    use crate::props::PropKind;
    match p.kind {
        PropKind::Int { min, max } => lumacore_wire::FieldInput::Number { min, max, named: p.named.iter().map(|n| n.to_string()).collect() },
        k => crate::props::field_input(k),
    }
}

/// `EditSetting <object> [<id>]`.
pub fn cmd_edit_setting(state: &mut AppState, args: &[&str]) -> String {
    let mut about = (String::new(), String::new());
    let found = match args {
        [] => Err(render_code(
            "EditSetting what? Usage: EditSetting <object> <n>  ·  EditSetting Show",
            "E-CMD-006",
        )),
        [word, rest @ ..] if !rest.is_empty() && crate::objects::find(word).is_some_and(|o| std::ptr::eq(o, &super::users::USER)) => {
            super::users::user_fields(state, rest)
        }
        [word, rest @ ..] if crate::objects::find(word).is_some_and(|o| std::ptr::eq(o, &super::windows::WINDOW)) => {
            super::windows::window_fields(state, rest).map(|f| {
                about = (f.subtitle, f.target);
                (f.title, f.fields)
            })
        }
        // `EditSetting Self` reaching here unresolved: the owner's window.
        [word, ..] if crate::objects::find(word).is_some_and(|o| std::ptr::eq(o, &crate::self_ref::SELF)) => {
            crate::self_ref::owner_window(state)
                .and_then(|(s, n)| super::windows::window_fields(state, &[&format!("{s}.{n}")]))
                .map(|f| {
                    about = (f.subtitle, f.target);
                    (f.title, f.fields)
                })
        }
        [word, rest @ ..] => object_fields(state, word, rest),
    };
    let (title, fields) = match found {
        Ok(f) => f,
        Err(e) => return e,
    };
    let mut out = format!("Settings of {title}:\n");
    for f in &fields {
        out.push_str(&format!(
            "  {:<14} {:<24} ({})   → {}\n",
            f.name,
            if f.value.is_empty() { "—".to_string() } else { f.value.clone() },
            f.input.label(),
            f.line.replace("{value}", "<value>")
        ));
    }
    let (subtitle, target) = about;
    state.ui_requests.push(UiEvent::EditSettings { title, fields, subtitle, target });
    out.trim_end().to_string()
}

/// A link setting's pick list (decisions §30 "12b"): what it can point at,
/// whether the current GUID is gone, and Add New for a user who may store
/// one. `object` is the setting's object number (for `OwnLink`).
fn pick(state: &AppState, kind: crate::props::PropKind, object: u32, current: &Value) -> Option<lumacore_wire::Pick> {
    let (word, items, optional, add) = match kind {
        crate::props::PropKind::Link { kind, items, optional, add, .. } => (kind, items(&state.root), optional, add),
        crate::props::PropKind::OwnLink { kind, items } => (kind, items(&state.root, object), true, None),
        _ => return None,
    };
    let current = current.as_str().unwrap_or_default();
    let missing = (!current.is_empty() && !items.iter().any(|i| i.guid == current))
        .then(|| format!("missing · {}", current.get(..8).unwrap_or(current)));
    let add_no = add.map(|_| items.iter().map(|i| i.no).max().unwrap_or(0) + 1);
    // Add New only for a user the gate lets store one (decisions §30 "12d").
    let add = add
        .zip(add_no)
        .filter(|(kw, n)| crate::rights::check_statement(state, "Store", &[kw, &n.to_string()]).is_ok())
        .map(|(kw, n)| format!("Store {kw} {n}"));
    Some(lumacore_wire::Pick {
        kind: word.to_string(),
        items,
        optional,
        follow: None,
        missing,
        add_no: add.as_ref().and(add_no),
        by_guid: false,
        add,
    })
}
