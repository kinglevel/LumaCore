// ██╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗ ██████╗ ██████╗
// ██║████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗
// ██║██╔██╗ ██║███████╗██████╔╝█████╗  ██║        ██║   ██║   ██║██████╔╝
// ██║██║╚██╗██║╚════██║██╔═══╝ ██╔══╝  ██║        ██║   ██║   ██║██╔══██╗
// ██║██║ ╚████║███████║██║     ███████╗╚██████╗   ██║   ╚██████╔╝██║  ██║
// ╚═╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
// FILE: inspector

//! What the NAV inspector shows for one tree path (decisions 0291): the identity rows (Slot,
//! Kind, GUID, Path), the facts beside them (a fixture's Type / Mode / Location, a datapool's
//! pools), every declared property from the one property registry `EditSetting` reads — its
//! value, its kind, its `Set` line, whether it differs from its declared default and its
//! `Reset` line — and the links: what uses the object and what it points at, each with the path
//! NAV follows (red "missing" when the GUID is gone). The TUI only draws it.
//!
//! Owns:          Inspector, Row, Section, view.
//! Doesn't own:   the rows of the tree (lumacore-objects `tree_rows`), the properties (`props`,
//!                `EditSetting`), drawing and keys (lumacore-terminal's NAV).
//! Entry points:  view.
//! Decisions:     docs/decisions/0291-nav-mode-ls-rows-and-an-inspector, docs/decisions/0106-tree-paths-typed-node.

use lumacore_objects::AppState;
use lumacore_objects::tree_path::TreePath;
use lumacore_objects::tree_rows::RowKind;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What the inspector shows for one path.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Inspector {
    /// `Fixture 101 "Front Wash"`, `Groups`.
    pub title: String,
    pub path: String,
    /// The object a line names (`Fixture 101`, `DataPool 2 Group 3`); None for a node.
    pub target: Option<String>,
    pub guid: Option<uuid::Uuid>,
    /// The object's name, when it has one.
    pub name: Option<String>,
    pub rows: Vec<Row>,
}

/// Where a row sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Slot, Kind, GUID, Path.
    Identity,
    /// Facts beside the properties: a fixture's Type, a datapool's pools, a node's counts.
    Info,
    /// A declared property.
    Property,
    /// What uses the object, what it points at.
    Link,
}

/// One row of the inspector.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub section: Section,
    pub name: String,
    pub value: String,
    /// A property's kind (`on/off`, `text`, `1–9999`); empty for the rest.
    pub kind: String,
    /// The line that sets it, ready for a value (`Set Group 1 "Mode" `); None = read-only.
    pub set: Option<String>,
    /// It differs from its declared default.
    pub changed: bool,
    /// The line that puts the default back (`Reset Group 1 "Mode"`), when it changed.
    pub reset: Option<String>,
    /// A link's dot path, NAV's to follow; None when the GUID is gone or not in the tree.
    pub link: Option<String>,
    /// A link whose GUID no longer exists.
    pub missing: bool,
    /// A property's settings cell as `EditSetting` sends it (`lumacore_wire::SettingField` as
    /// JSON: input, value, its `Set` line with `{value}`, owner first, and its `Reset` line): the
    /// Tree window draws it as a `<luma-settings>` cell.
    pub field: Option<serde_json::Value>,
}

impl Row {
    fn fact(section: Section, name: &str, value: impl Into<String>) -> Self {
        Row { section, name: name.to_string(), value: value.into(), kind: String::new(), set: None, changed: false, reset: None, link: None, missing: false, field: None }
    }
}

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

/// The inspector of `dot_path` as the acting user sees it (their datapool, their rights).
pub fn view(state: &mut AppState, dot_path: &str) -> Inspector {
    let Some(path) = TreePath::parse_dot(dot_path) else {
        return disk(state, dot_path);
    };
    if let Some((object, datapool, no)) = lumacore_objects::objects::at_path(&path) {
        return object_view(state, dot_path, object, datapool, no);
    }
    node_view(state, dot_path, &path)
}

/// A declared node, or a live entry without a GUID (a line, a master's level).
fn node_view(state: &AppState, dot_path: &str, path: &TreePath) -> Inspector {
    let parent = dot_path.rsplit_once('.').map_or("", |(p, _)| p);
    let row = path.entry().and_then(|_| lumacore_objects::tree_rows::rows(state, parent).into_iter().find(|r| r.path == dot_path));
    let mut rows = Vec::new();
    let title = match &row {
        Some(r) => {
            rows.push(Row::fact(Section::Identity, "Slot", r.slot.map(|s| s.to_string()).unwrap_or_default()));
            rows.push(Row::fact(Section::Identity, "Kind", r.type_name));
            r.text()
        }
        None => {
            rows.push(Row::fact(Section::Identity, "Kind", "Node"));
            path.node().name().to_string()
        }
    };
    rows.push(Row::fact(Section::Identity, "Path", dot_path));
    if let Some(r) = &row
        && !r.name.is_empty()
    {
        rows.push(Row::fact(Section::Info, "Text", r.name.clone()));
    }
    let meta = lumacore_objects::tree_rows::meta_of(state, dot_path);
    if !meta.is_empty() {
        rows.push(Row::fact(Section::Info, "Holds", meta));
    }
    let below = lumacore_objects::tree_rows::rows(state, dot_path);
    if !below.is_empty() {
        rows.push(Row::fact(Section::Info, "Rows", below.len().to_string()));
        for r in below.iter().filter(|r| r.kind == RowKind::Node) {
            rows.push(Row { link: Some(r.path.clone()), ..Row::fact(Section::Info, &r.name, r.meta.clone()) });
        }
    }
    Inspector { title, path: dot_path.to_string(), rows, ..Inspector::default() }
}

/// An object: identity, facts, properties, links.
fn object_view(state: &mut AppState, dot_path: &str, object: &'static lumacore_objects::objects::ObjectDef, datapool: Option<u32>, no: u32) -> Inspector {
    use lumacore_objects::vfs::NodeView;
    let (guid, name) = match lumacore_objects::vfs::resolve_node(dot_path, &state.root) {
        Some(NodeView::Object { guid, name, .. }) => (guid.parse::<uuid::Uuid>().ok(), name),
        // A datapool is a folder of its pools in the tree; its identity is its own.
        _ => match state.root.show_data.datapools.get(&no).filter(|_| std::ptr::eq(object, &lumacore_objects::objects::DATAPOOL)) {
            Some(dp) => (Some(dp.meta.guid()), dp.meta.name().map(str::to_string)),
            None => (None, None),
        },
    };
    // Owner first when the object lives in another datapool than the user's (decisions §33).
    let owner = match datapool {
        Some(dp) if dp != state.selected_datapool() => format!("DataPool {dp} "),
        _ => String::new(),
    };
    let target = format!("{owner}{} {no}", object.keyword);
    let title = match &name {
        Some(n) => format!("{target} \"{n}\""),
        None => target.clone(),
    };
    let mut rows = vec![
        Row::fact(Section::Identity, "Slot", no.to_string()),
        Row::fact(Section::Identity, "Kind", object.keyword),
        Row::fact(Section::Identity, "GUID", guid.map(|g| g.to_string()).unwrap_or_default()),
        Row::fact(Section::Identity, "Path", dot_path),
    ];
    rows.extend(facts(state, object, no, dot_path));
    rows.extend(properties(state, object, datapool, no, &owner));
    if let Some(guid) = guid {
        rows.extend(links(state, guid));
    }
    Inspector { title, path: dot_path.to_string(), target: Some(target), guid, name, rows }
}

/// The facts an object shows beside its properties.
fn facts(state: &AppState, object: &'static lumacore_objects::objects::ObjectDef, no: u32, dot_path: &str) -> Vec<Row> {
    use lumacore_objects::objects;
    if std::ptr::eq(object, &objects::FIXTURE) {
        return lumacore_objects::commands::info::fixture_facts(state, no).into_iter().map(|(k, v)| Row::fact(Section::Info, k, v)).collect();
    }
    if std::ptr::eq(object, &objects::DATAPOOL) {
        // Its pools, as `ls` and `Info DataPool` show them; each one NAV can open.
        return lumacore_objects::tree_rows::node_rows(state, dot_path)
            .into_iter()
            .map(|r| Row { link: Some(r.path.clone()), ..Row::fact(Section::Info, &r.name, if r.meta.is_empty() { "—".to_string() } else { r.meta }) })
            .collect();
    }
    if std::ptr::eq(object, &lumacore_objects::commands::groups::GROUP) {
        let dp = lumacore_objects::tree_path::TreePath::parse_dot(dot_path).and_then(|p| p.datapool());
        let members = dp.and_then(|dp| state.root.show_data.datapools.get(&dp)).and_then(|d| d.groups.get(&no)).map_or(0, |g| g.members.len());
        return vec![Row::fact(Section::Info, "Fixtures", members.to_string())];
    }
    Vec::new()
}

/// Every declared property with its value, kind and lines (`EditSetting`'s cells). `owner` is
/// `DataPool <n> ` when the object lives in another datapool than the user's.
fn properties(state: &mut AppState, object: &'static lumacore_objects::objects::ObjectDef, datapool: Option<u32>, no: u32, owner: &str) -> Vec<Row> {
    let Some(props) = object.props else {
        return Vec::new();
    };
    // `EditSetting` reads another datapool's object with that datapool in place.
    let no = no.to_string();
    let before = state.datapool_override;
    if datapool.is_some() {
        state.datapool_override = datapool;
    }
    let found = lumacore_objects::commands::edit_setting::settings_of(state, &[object.keyword, &no]);
    state.datapool_override = before;
    let Ok((_, fields)) = found else {
        return Vec::new();
    };
    fields
        .into_iter()
        .filter(|f| f.readonly != lumacore_objects::commands::edit_setting::FACT)
        .map(|f| {
            let line = owner_first(&f.line, owner);
            let changed = props.get(&f.name).and_then(|p| p.reset_value()).is_some_and(|d| !same_value(&f.value, d));
            let set = f.readonly.is_empty().then(|| line.replace("{value}", ""));
            let reset = (changed && f.readonly.is_empty()).then(|| line.replacen("Set ", "Reset ", 1).replace(" {value}", ""));
            // The cell: the same field, its lines owner first, ● and ↺ from the declared default.
            let mut cell = serde_json::to_value(&f).unwrap_or(serde_json::Value::Null);
            if let Some(o) = cell.as_object_mut() {
                o.insert("line".into(), line.clone().into());
                o.insert("key".into(), f.name.clone().into());
                o.remove("section");
                o.insert("changed".into(), changed.into());
                match &reset {
                    Some(r) => o.insert("reset".into(), r.clone().into()),
                    None => o.remove("reset"),
                };
            }
            Row { section: Section::Property, kind: f.input.label().to_string(), value: f.value, name: f.name, set, changed, reset, link: None, missing: false, field: Some(cell) }
        })
        .collect()
}

/// What uses the object and what it points at, each with the path NAV follows.
fn links(state: &AppState, guid: uuid::Uuid) -> Vec<Row> {
    let walk = state.root.show_data.walk_identity();
    let object_of = |g: uuid::Uuid| walk.objects.iter().find(|o| o.guid == g);
    let label = |o: &lumacore_base::ObjectEntry| {
        let no = o.path.rsplit('.').next().unwrap_or("");
        match &o.name {
            Some(n) if !n.is_empty() => format!("{} {no} \"{n}\"", o.kind),
            _ => format!("{} {no}", o.kind),
        }
    };
    let show_path = |o: &lumacore_base::ObjectEntry| {
        let path = format!("{}.{}", lumacore_objects::tree_path::Node::ShowData.path(), o.path);
        TreePath::parse_dot(&path).is_some_and(|p| lumacore_objects::vfs::path_exists(&state.root, &p)).then_some(path)
    };
    let mut rows = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for l in walk.links.iter().filter(|l| l.target == guid) {
        let Some(from) = l.from.and_then(object_of) else { continue };
        if seen.insert(("Used by", from.guid)) {
            rows.push(Row { link: show_path(from), ..Row::fact(Section::Link, "Used by", label(from)) });
        }
    }
    for l in walk.links.iter().filter(|l| l.from == Some(guid) && !l.contains) {
        if !seen.insert(("Links to", l.target)) {
            continue;
        }
        rows.push(match object_of(l.target) {
            Some(to) => Row { link: show_path(to), ..Row::fact(Section::Link, "Links to", label(to)) },
            None => Row { missing: true, ..Row::fact(Section::Link, "Links to", format!("missing · {}", &l.target.to_string()[..8])) },
        });
    }
    rows
}

/// A file or folder on a drive.
fn disk(state: &AppState, dot_path: &str) -> Inspector {
    use lumacore_objects::vfs::NodeView;
    let mut rows = vec![Row::fact(Section::Identity, "Path", dot_path)];
    let title = dot_path.rsplit('.').next().unwrap_or(dot_path).to_string();
    if let Some(NodeView::FileInfo { size, is_dir, .. }) = lumacore_objects::vfs::resolve_node(dot_path, &state.root) {
        rows.push(Row::fact(Section::Identity, "Kind", if is_dir { "Directory" } else { "File" }));
        if !is_dir {
            rows.push(Row::fact(Section::Info, "Size", lumacore_objects::vfs::format_file_size_pub(size)));
        }
    }
    Inspector { title, path: dot_path.to_string(), rows, ..Inspector::default() }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A field's line naming its row owner first (`Set Group 3 …` → `Set DataPool 2 Group 3 …`).
fn owner_first(line: &str, owner: &str) -> String {
    match line.strip_prefix("Set ") {
        Some(rest) if !owner.is_empty() => format!("Set {owner}{rest}"),
        _ => line.to_string(),
    }
}

/// Whether a shown value is the declared default (`""` / `none` / `—` are one empty value; any
/// case).
fn same_value(shown: &str, default: &str) -> bool {
    let empty = |v: &str| v.is_empty() || v.eq_ignore_ascii_case("none") || v == "—";
    (empty(shown) && empty(default)) || shown.trim_matches('"').eq_ignore_ascii_case(default.trim_matches('"'))
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
