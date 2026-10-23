//! `Set <target> "<Property>" <value>` — one typed shape over the one property registry
//! (decisions §33 7, 14, 16, 17, 57, 58).
//!
//! The target is resolved by the one resolver (`crate::resolve`): ids, ranges, names, owners,
//! `Root.` tree paths; a bare owner keyword is your own / this one (the object's declared
//! `Props::implicit`: your profile, this station, the show). The property is the object's
//! declared [`Property`], named quoted (decision 7). The value is parsed once by the property's
//! kind: On / Off / Yes / No (Toggle where declared, decision 14), a relative number
//! (`+ 10`, `* 2`), a `Thru` spread over the rows, a declared word, quoted text. Every row is
//! set inside the one line — one Oops step; rows refused are named with their codes.
//!
//! Objects whose properties aren't declared yet keep their own `Set` handlers until §33 3c2
//! moves them; [`route`] returns None for those lines.

use lumacore_syntax::ast::{Item, Target, Value};

use crate::commands::set_rows;
use crate::errors::render_code;
use crate::objects::ObjectDef;
use crate::props::{self, Access, PropKind, Property, Props};
use crate::resolve::{self, Resolved};
use crate::vfs::NodeView;
use crate::{AppState, DispatchResult, Reply};

/// `Set <target> "<Property>" <value>`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Set")]
pub struct SetProperty {
    #[lum(target)]
    pub target: Target,
    #[lum(property)]
    pub property: String,
    #[lum(value)]
    pub value: Value,
}

/// One row a line sets.
#[derive(Debug, Clone)]
pub struct Row {
    /// What the declaration reads and writes it by: a keyed row's key, a tree member's number.
    pub key: String,
    pub no: u32,
    pub datapool: Option<u32>,
    /// How replies name it (`Output 3`, `Interface en0`).
    pub label: String,
}

/// A `Set` line for an object with declared properties (`args` after `Set`); None = the line
/// isn't one (an object whose `Set` handler still parses its own words, §33 3c2).
pub fn route(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let words: Vec<&str> = args.iter().copied().filter(|w| !w.starts_with('/')).collect();
    let Some(first) = words.first() else {
        return DispatchResult::Output(Reply::error(
            "Set what? Set <object> [<n>] \"<Property>\" <value> — EditSetting <object> shows them.",
            "E-CMD-006",
        ));
    };
    let Some((obj, named)) = named_object(state, args) else {
        // A target-less form (`Set ClearBuffer On`, decisions §33 16): Set names its target.
        let profile = crate::objects::USER_PROFILE.props.and_then(|p| p.get(first));
        let msg = match profile {
            Some(p) => format!("Set names its target: Set UserProfile \"{}\" {}", p.name, words[1..].join(" ")),
            None => format!("Set what? Name the object first: Set <object> [<n>] \"<Property>\" <value> ('{first}' is no object)."),
        };
        return DispatchResult::Output(Reply::error(msg, "E-CMD-006"));
    };
    let Some(props) = obj.props else {
        return DispatchResult::Output(Reply::error(format!("{} has no properties to Set.", obj.keyword), "E-CMD-015"));
    };
    if named.is_none() {
        let at = args.iter().rposition(|w| crate::objects::find(w).is_some_and(|o| std::ptr::eq(o, obj))).unwrap_or(0);
        return DispatchResult::Output(no_property(state, obj, props, &args[at..]));
    }
    let line = statement_line("Set", args);
    let bound = match crate::commands::typed::bound::<SetProperty>(&line) {
        Ok(b) => b,
        Err(reply) => return DispatchResult::Output(reply),
    };
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    bound.run_on(state, obj, props, confirmed)
}

/// The statement back as a line for the parser; an empty word was `""` (the old tokenizer drops
/// the quotes of an empty text).
fn statement_line(verb: &str, args: &[&str]) -> String {
    std::iter::once(verb).chain(args.iter().map(|a| if a.is_empty() { "\"\"" } else { a })).collect::<Vec<_>>().join(" ")
}

/// The object a `Set` line sets a property of — the last object word before the property (a
/// child after its owner: `Station "Tower" Network "Rig"`), or a `Root.` tree path's — and the
/// property it names, if it names one of that object's.
pub(crate) fn named_object(state: &AppState, args: &[&str]) -> Option<(&'static ObjectDef, Option<&'static Property>)> {
    let mut obj: Option<&'static ObjectDef> = None;
    for w in args.iter().filter(|w| !w.starts_with('/')) {
        let quoted = w.starts_with('"');
        if let Some(p) = obj.and_then(|o| o.props).and_then(|p| p.get(w)) {
            return Some((obj?, Some(p)));
        }
        if quoted {
            continue;
        }
        // A tree path (`Root.ShowData.Protocols.Outputs.1`): the collection it names.
        if w.contains('.') && !w.starts_with(|c: char| c.is_ascii_digit()) {
            let t = resolve::target_of(w).ok()?;
            obj = Some(resolve::resolve_one(state, &t, crate::objects::default_object(state)).ok()?.object);
        } else if let Some(o) = crate::objects::find(w) {
            obj = Some(o);
        }
    }
    obj.map(|o| (o, None))
}

/// The refusal for a `Set` line naming none of its object's properties (None = its object has
/// no declared properties); the object is the one `Set` would set (a child after its owner).
pub fn no_property_of(state: &AppState, args: &[&str]) -> Option<Reply> {
    let (obj, _) = named_object(state, args)?;
    let props = obj.props?;
    let at = args.iter().rposition(|w| crate::objects::find(w).is_some_and(|o| std::ptr::eq(o, obj))).unwrap_or(0);
    Some(no_property(state, obj, props, &args[at..]))
}

/// A `Set` line (`args` after `Set`) that names none of the object's properties: what it has.
pub fn no_property(state: &AppState, obj: &ObjectDef, props: &Props, args: &[&str]) -> Reply {
    let writable: Vec<String> = props.all().map(|p| format!("\"{}\" ({})", p.name, kind_text(p))).collect();
    let word = args.iter().filter(|w| !w.starts_with('/')).rev().nth(1).filter(|_| args.len() > 2);
    // A field the object has but doesn't declare is read-only (derived or the system's).
    let stored = word.is_some_and(|w| {
        let first = args.get(1).and_then(|n| n.parse::<u32>().ok());
        obj.tree.as_ref().zip(first).is_some_and(|(t, n)| {
            let dp = t.datapool.as_ref().map(|_| state.selected_datapool());
            matches!(t.object_in(&state.root, dp, n), Some(NodeView::Object { json, .. }) if props::json_has_key(&json, w.trim_matches('"')))
        })
    });
    if let (Some(hint), Some(w)) = (props.hint, word) {
        return Reply::error(format!("{} has no property {w}. {hint}", obj.keyword), "E-SET-001");
    }
    match word {
        Some(w) if stored => Reply::error(
            format!("{} {w} is read-only (derived or owned by the system). Writable: {}", obj.keyword, writable.join(", ")),
            "E-SET-002",
        ),
        Some(w) => Reply::error(format!("{} has no property {w}. Writable: {}", obj.keyword, writable.join(", ")), "E-SET-001"),
        None => Reply::error(format!("Set {} needs a quoted property and a value. Writable: {}", obj.keyword, writable.join(", ")), "E-SET-001"),
    }
}

/// What a property takes, as a terminal shows it.
pub fn kind_text(p: &Property) -> String {
    let base = match p.kind {
        PropKind::OnOff | PropKind::Switch if p.words != ("On", "Off") => format!("{}/{}", p.words.0, p.words.1),
        k => props::kind_label(k),
    };
    if p.named.is_empty() { base } else { format!("{base} or {}", p.named.join("/")) }
}

impl SetProperty {
    fn run_on(self, state: &mut AppState, obj: &'static ObjectDef, props: &'static Props, confirmed: bool) -> DispatchResult {
        let Some(prop) = props.get(&self.property) else {
            let writable: Vec<String> = props.all().map(|p| format!("\"{}\" ({})", p.name, kind_text(p))).collect();
            return DispatchResult::Output(Reply::error(
                format!("{} has no property \"{}\". Writable: {}", obj.keyword, self.property, writable.join(", ")),
                "E-SET-001",
            ));
        };
        // A bare object keyword without ids is the value word unless the property links an
        // object (decision 64: `Set Self "View" Input`, `Set Fixture 1 "MasterReact" Group`).
        let keyword = match &self.value {
            Value::Target(t) if !matches!(prop.kind, PropKind::Link { .. }) => match t.items.as_slice() {
                [(None, Item::Path(segments))] => match segments.as_slice() {
                    [only] if only.ids.is_none() => Some(crate::lum::vocabulary().word(only.object.node).name.clone()),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
        let raw = match keyword.map_or_else(|| value_text(&self.value), Ok) {
            Ok(r) => r,
            Err(e) => return DispatchResult::Output(e.into()),
        };
        // `Default` is a fixture value; a property goes back with Reset (decisions §33 61).
        if matches!(self.value, Value::Word(_))
            && raw.eq_ignore_ascii_case("default")
            && !matches!(prop.kind, PropKind::Choice(words) if words.iter().any(|w| w.eq_ignore_ascii_case("default")))
        {
            return DispatchResult::Output(Reply::error(
                format!("\"{}\" goes back to its default with Reset: Reset {} … \"{}\"", prop.name, obj.keyword, prop.name),
                "E-CMD-008",
            ));
        }
        // Owners that scope their children, down the path (a station's wiring row, a group's
        // member, a sequence's cue and the cue's feature group).
        if let [(None, Item::Path(segments))] = self.target.items.as_slice()
            && segments.len() > 1
            && segments[..segments.len() - 1].iter().any(|s| scopes(s).is_some())
        {
            // A field lives in the object's own row — except where the row is the owner's (a
            // profile's Theme: `view`).
            if matches!(prop.access, Access::Field(_)) && props.view.is_none() {
                let owner = segments.iter().find_map(scopes).map_or("", |(o, _)| o.keyword);
                return DispatchResult::Output(Reply::error(
                    format!("Set {owner} … {} takes what the {} sets for it: \"{}\" is the {}'s own (Set {} … \"{}\").", obj.keyword, owner.to_lowercase(), prop.name, obj.keyword, obj.keyword, prop.name),
                    "E-SET-004",
                ));
            }
            return in_scopes(state, segments, obj, props, prop, &self.value, &raw, confirmed);
        }
        let resolved = match resolve::resolve_one(state, &self.target, crate::objects::default_object(state)) {
            Ok(r) => r,
            Err(e) => return DispatchResult::Output(e.into()),
        };
        if !std::ptr::eq(resolved.object, obj) {
            return DispatchResult::Output(Reply::error(
                format!("\"{}\" is a property of {}, not of {}.", prop.name, obj.keyword, resolved.object.keyword),
                "E-SET-001",
            ));
        }
        set_resolved(state, obj, props, &resolved, prop, &self.value, &raw, confirmed)
    }
}

/// A path segment's object, when it scopes the segments after it (`Props::within`).
fn scopes(segment: &lumacore_syntax::ast::Segment) -> Option<(&'static ObjectDef, props::Within)> {
    let obj = crate::objects::find(&crate::lum::vocabulary().word(segment.object.node).name)?;
    Some((obj, obj.props?.within?))
}

/// Set down an owner-first path: each scoping owner's rows in turn, the rest within them.
#[allow(clippy::too_many_arguments)]
fn in_scopes(
    state: &mut AppState,
    segments: &[lumacore_syntax::ast::Segment],
    obj: &'static ObjectDef,
    props: &'static Props,
    prop: &'static Property,
    value: &Value,
    raw: &str,
    confirmed: bool,
) -> DispatchResult {
    // A DataPool owner in front: the rest runs in that DataPool (`Set DataPool 2 Sequence 3 Cue 2
    // "Fade" 1`), as `datapool_override` does for any verb.
    if let [first, rest @ ..] = segments
        && !rest.is_empty()
        && crate::objects::find(&crate::lum::vocabulary().word(first.object.node).name)
            .is_some_and(|o| std::ptr::eq(o, &crate::objects::DATAPOOL))
        && rest[..rest.len() - 1].iter().any(|s| scopes(s).is_some())
    {
        let pool = Target { items: vec![(None, Item::Path(vec![first.clone()]))] };
        let pools = match resolve::resolve_one(state, &pool, &crate::objects::DATAPOOL) {
            Ok(r) => r,
            Err(e) => return DispatchResult::Output(e.into()),
        };
        let [one] = pools.members.as_slice() else {
            return DispatchResult::Output(Reply::error("Name one DataPool as the owner.", "E-CMD-008"));
        };
        let before = state.datapool_override.replace(one.no);
        let result = in_scopes(state, rest, obj, props, prop, value, raw, confirmed);
        state.datapool_override = before;
        return result;
    }
    let Some(((owner_obj, within), [first, rest @ ..])) = segments.first().and_then(scopes).zip(Some(segments)).filter(|_| segments.len() > 1) else {
        // No more scoping owners: the resolver takes the rest (`DataPool 2 Group 3`, `Cue 3`).
        let target = Target { items: vec![(None, Item::Path(segments.to_vec()))] };
        let resolved = match resolve::resolve_one(state, &target, obj) {
            Ok(r) => r,
            Err(e) => return DispatchResult::Output(e.into()),
        };
        return set_resolved(state, obj, props, &resolved, prop, value, raw, confirmed);
    };
    let owner_target = Target { items: vec![(None, Item::Path(vec![first.clone()]))] };
    let owners = match resolve::resolve_one(state, &owner_target, owner_obj) {
        Ok(r) => r,
        Err(e) => return DispatchResult::Output(e.into()),
    };
    let keys: Vec<String> = match owner_obj.props.and_then(|p| p.implicit) {
        Some(implicit) if !owners.named => vec![implicit(state)],
        // Keyed rows by their key, tree members by their number (as `rows_of`).
        _ => {
            let keyed = owner_obj.tree.is_none() && owner_obj.props.is_some_and(|p| p.keyed.is_some());
            owners.members.iter().map(|m| if keyed { m.name.clone().unwrap_or_default() } else { m.no.to_string() }).collect()
        }
    };
    let mut outs = Vec::new();
    for key in keys {
        let _scope = match within(state, &key) {
            Ok(g) => g,
            Err(e) => return DispatchResult::Output(e.into()),
        };
        match in_scopes(state, rest, obj, props, prop, value, raw, confirmed) {
            DispatchResult::Output(o) => outs.push(o),
            other => return other,
        }
    }
    DispatchResult::Output(Reply::joined(outs))
}

/// The value as the declarations take it:/// The value as the declarations take it: a number, `+ 10` (relative), `21 Thru 30` (a
/// spread, split per row), a declared word, quoted text without its quotes.
fn value_text(v: &Value) -> Result<String, String> {
    let vocab = crate::lum::vocabulary();
    Ok(match v {
        Value::Number(n) => n.node.clone(),
        Value::Relative { op, number } => format!("{} {}", op.node, number.node),
        Value::Spread { from, to } => format!("{} Thru {}", from.node, to.node),
        Value::Word(w) => vocab.word(w.node).name.clone(),
        Value::Text(t) => t.node.clone(),
        // A property that holds a command line takes it in backticks (decisions §33 91).
        Value::Line(l) => l.node.clone(),
        Value::Read(_) => return Err(render_code("A Self read as a Set value isn't built yet (planned).", "E-CMD-003")),
        Value::Target(_) => return Err(render_code("Set takes a value here, not an object.", lumacore_syntax::codes::VALUE_WORD)),
    })
}

/// The rows a resolved target names: its members, or the object's implicit row when no ids
/// were written (`Set Station "Name" …` = this station, `Set UserProfile "…"` = your own).
pub fn rows_of(state: &AppState, obj: &ObjectDef, props: &Props, r: &Resolved, prop: &Property) -> Result<Vec<Row>, String> {
    // A keyed property's row is named as the object declares (`Interface en0`, `Station
    // "Tower" Network 1 "Rig"`); a tree member by its keyword and number.
    let label = |key: &str| match (props.label, prop.access) {
        (Some(l), Access::Keyed { .. }) => l(state, key),
        _ => format!("{} {key}", obj.keyword),
    };
    if !r.named {
        let Some(implicit) = props.implicit else {
            return Err(render_code(
                format!("Set {} which? Name it: Set {} 1 \"{}\" …", obj.keyword, obj.keyword, prop.name),
                "E-CMD-006",
            ));
        };
        let key = implicit(state);
        if key.is_empty() {
            return Err(render_code(format!("You have no {} here.", obj.keyword), "E-CMD-007"));
        }
        let no = key.parse().unwrap_or(0);
        return Ok(vec![Row { label: label(&key), key, no, datapool: r.datapool }]);
    }
    Ok(r.members
        .iter()
        .map(|m| {
            // Keyed rows (no tree) by their key, a Path object's (a tag: its number moves) by its
            // GUID, tree members by their number.
            let key = if obj.tree.is_none() && props.keyed.is_some() {
                m.name.clone().unwrap_or_default()
            } else if reads_a_path(obj) {
                m.guid.map(|g| g.to_string()).unwrap_or_default()
            } else {
                m.no.to_string()
            };
            Row { label: label(&key), key, no: m.no, datapool: r.datapool }
        })
        .collect())
}

/// An object whose ids are paths (`Tag 1.2`, decisions §12 "Tags hardened"): its rows are its
/// members' GUIDs, never a number that a move changes.
pub fn reads_a_path(obj: &ObjectDef) -> bool {
    matches!(obj.lum.reading, lumacore_registry::object::IdReading::Path)
}

#[allow(clippy::too_many_arguments)]
fn set_resolved(
    state: &mut AppState,
    obj: &'static ObjectDef,
    props: &'static Props,
    resolved: &Resolved,
    prop: &'static Property,
    value: &Value,
    raw: &str,
    confirmed: bool,
) -> DispatchResult {
    let rows = match rows_of(state, obj, props, resolved, prop) {
        Ok(r) => r,
        Err(e) => return DispatchResult::Output(e.into()),
    };
    if !confirmed
        && let Some(ask) = props.ask
    {
        let keys: Vec<String> = rows.iter().map(|r| r.key.clone()).collect();
        if let Some(asked) = ask(state, &keys, resolved.named, prop, raw) {
            return asked;
        }
    }
    if let [row] = rows.as_slice() {
        let value = match value {
            Value::Spread { from, .. } => from.node.clone(),
            _ => raw.to_string(),
        };
        return DispatchResult::Output(match write(state, obj, props, row, prop, &value) {
            Ok(msg) => crate::text::render_success(&msg).into(),
            Err(e) => e.into(),
        });
    }
    let values = match value {
        Value::Spread { from, to } => set_rows::spread(&format!("{} Thru {}", from.node, to.node), rows.len()).unwrap_or_default(),
        _ => vec![raw.to_string(); rows.len()],
    };
    let keys: Vec<String> = rows.iter().map(|r| r.key.clone()).collect();
    let labels: Vec<String> = rows.iter().map(|r| r.label.clone()).collect();
    set_rows::run_each(state, &keys, &labels, prop.name, &values, raw, |state, key, value| {
        let row = rows.iter().find(|r| r.key == key).expect("a named row");
        DispatchResult::Output(match write(state, obj, props, row, prop, value) {
            Ok(msg) => crate::text::render_success(&msg).into(),
            Err(e) => e.into(),
        })
    })
}

/// The current value of a row's property as text (`On` / `Off` for a stored bool).
pub fn current(state: &AppState, obj: &ObjectDef, row: &Row, prop: &Property) -> Option<String> {
    match prop.access {
        Access::Keyed { get, .. } => Some(get(state, &row.key)),
        Access::Named { get, .. } => Some(get(state, &row.key, prop.name)),
        Access::Field(field) => {
            let json = match (&obj.tree, obj.props.and_then(|p| p.view)) {
                (Some(tree), _) => match tree.object_in(&state.root, row.datapool, row.no) {
                    Some(NodeView::Object { json, .. }) => json,
                    _ => return None,
                },
                (None, Some(view)) => view(&state.root, row.no)?,
                (None, None) => return None,
            };
            let v = field.split('.').try_fold(&json, |v, k| v.get(k))?;
            Some(match v {
                serde_json::Value::Bool(b) => if *b { prop.words.0 } else { prop.words.1 }.to_string(),
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            })
        }
    }
}

/// `raw` checked against the property: the text its write takes (a named word as declared,
/// on / off as its `words`, a relative number worked out against the current value).
pub fn checked(state: &mut AppState, obj: &ObjectDef, row: &Row, prop: &Property, raw: &str) -> Result<String, String> {
    let raw = raw.trim().trim_matches('"');
    let what = || format!("{} \"{}\"", row.label, prop.name);
    if let Some(n) = prop.named.iter().find(|n| n.eq_ignore_ascii_case(raw)) {
        return Ok((*n).to_string());
    }
    match prop.kind {
        PropKind::OnOff | PropKind::Switch => {
            let on = match props::on_off(raw) {
                Some(b) => b,
                None if matches!(prop.kind, PropKind::Switch) && raw.eq_ignore_ascii_case("toggle") => {
                    !current(state, obj, row, prop).is_some_and(|v| props::on_off(&v) == Some(true))
                }
                None => {
                    let words = if matches!(prop.kind, PropKind::Switch) { "On, Off, Yes, No or Toggle" } else { "On, Off, Yes or No" };
                    return Err(render_code(format!("{} takes {words} — not '{raw}'.", what()), lumacore_syntax::codes::VALUE_WORD));
                }
            };
            Ok(if on { prop.words.0 } else { prop.words.1 }.to_string())
        }
        PropKind::Int { .. } => {
            let now = current(state, obj, row, prop)
                .and_then(|v| v.trim_end_matches(char::is_alphabetic).trim().parse::<i64>().ok())
                .unwrap_or(0);
            let raw = match props::relative(raw, now) {
                Some(next) => next.map_err(|e| render_code(format!("{}: {e}", what()), "E-CMD-008"))?.to_string(),
                None => raw.to_string(),
            };
            props::parse(prop.kind, &raw, &mut state.root)
                .map(|v| v.to_string())
                .map_err(|e| invalid(&what(), e, prop))
        }
        // The object's type checks it (a window setting: Toggle, Now, relative, links).
        PropKind::Declared => Ok(raw.to_string()),
        // A choice is written as declared; the rest is checked by the write (a link resolves its
        // name there, a colour becomes its bytes).
        PropKind::Choice(_) => props::parse(prop.kind, raw, &mut state.root)
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            })
            .map_err(|e| invalid(&what(), e, prop)),
        _ => Ok(raw.to_string()),
    }
}

fn invalid(what: &str, e: String, prop: &Property) -> String {
    let named = if prop.named.is_empty() { String::new() } else { format!(" or {}", prop.named.join("/")) };
    render_code(format!("{what}: {e}{named} ({}).", kind_text(prop)), "E-SET-003")
}

/// Set one property of one row: the object's own write, else read-only / may / level, the
/// value check and the write. Ok = the success text; Err = a coded refusal.
pub fn write(state: &mut AppState, obj: &ObjectDef, props: &Props, row: &Row, prop: &'static Property, raw: &str) -> Result<String, String> {
    if prop.name == "Name" {
        crate::resolve::check_name(crate::tokens::unquote(raw))?;
    }
    if let Some(own) = props.own_write
        && let Some(done) = own(state, &row.key, prop, raw)
    {
        return done;
    }
    if let Some(why) = props.readonly {
        return Err(render_code(format!("{} \"{}\" is read-only: {why}.", row.label, prop.name), "E-CMD-010"));
    }
    if let Some(may) = props.may {
        may(state, row.no).map_err(|e| crate::errors::reply(e, "E-SET-002"))?;
    }
    // Devices change in the draft (decisions §34 "34g4 patch hardened" 9, round 3 10): with no
    // draft open, only the live columns reach the rig — the meta level (name, note, paint, tags
    // …) and another owner's scoped rows (a group's grid) aren't patch data; another's draft
    // refuses in dispatch.
    let own_data = props.lists.iter().any(|l| l.iter().any(|p| std::ptr::eq(p, prop)));
    if state.patch_draft.is_none()
        && own_data
        && obj.tree.as_ref().is_some_and(|t| t.node.is_under(crate::tree_path::Node::Devices))
        && !props.is_live(prop.name)
    {
        return Err(render_code(
            format!("{} \"{}\" changes in the Devices draft: Claim Root.ShowData.Devices first.", row.label, prop.name),
            "E-CMD-010",
        ));
    }
    crate::rights::require(state, props::rights_of(obj, prop), &format!("Set {} \"{}\"", obj.keyword, prop.name))?;
    let value = checked(state, obj, row, prop, raw)?;
    match prop.access {
        Access::Named { set, .. } => {
            let msg = set(state, &row.key, prop.name, &value)?;
            (props.after)(state);
            Ok(msg)
        }
        Access::Keyed { set, .. } => {
            let msg = set(state, &row.key, &value)?;
            (props.after)(state);
            Ok(msg)
        }
        Access::Field(field) => {
            let apply = props.apply.ok_or_else(|| render_code(format!("{} can't be written.", obj.keyword), "E-SET-002"))?;
            let json = props::parse(prop.kind, &value, &mut state.root).map_err(|e| invalid(&format!("{} \"{}\"", row.label, prop.name), e, prop))?;
            let shown = match prop.kind {
                PropKind::Link { .. } | PropKind::OwnLink { .. } if json.is_null() => "none".to_string(),
                PropKind::Link { .. } | PropKind::OwnLink { .. } => value.clone(),
                _ if json.is_null() => "none".to_string(),
                kind => props::shown(kind, &json).unwrap_or_else(|| json.to_string()),
            };
            // An empty text clears an optional one (`Label Fixture 3 ""`: no name); a text the
            // object must have keeps the empty string.
            let empty_text = matches!(prop.kind, PropKind::Text) && value.is_empty();
            let put = |state: &mut AppState, json: &serde_json::Value| {
                apply(&mut state.root, row.datapool, row.no, &mut |j| {
                    let slot = props::field_mut(j, field).ok_or_else(|| format!("no field {field}"))?;
                    *slot = json.clone();
                    Ok(())
                })
            };
            let done = if empty_text { put(state, &serde_json::Value::Null).or_else(|_| put(state, &json)) } else { put(state, &json) };
            // A refusal with its own code keeps it (E-OUTPUT-003 …).
            done.map_err(|e| crate::errors::reply(format!("{} {}: {e}", row.label, prop.name), "E-SET-003"))?;
            let shown = if empty_text { "none".to_string() } else { shown };
            state.show_dirty = true;
            Ok(format!("{} {} = {shown}", row.label, prop.name))
        }
    }
}

/// `Reset <target> "<Property>"` — one property back to its declared default (decisions §33 61).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Reset")]
pub struct ResetProperty {
    #[lum(target)]
    pub target: Target,
    #[lum(property)]
    pub property: String,
}

/// `Reset <target>` — every property of the rows back to its declared default.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Reset")]
pub struct ResetAll {
    #[lum(target)]
    pub target: Target,
}

/// A `Reset` line for an object with declared properties (`args` after `Reset`); None = the
/// object resets its own way (`Reset Self`, `Reset Window 1.4`) or isn't one.
pub fn reset_route(state: &mut AppState, args: &[&str]) -> Option<DispatchResult> {
    let (obj, named) = named_object(state, args)?;
    let props = obj.props?;
    let line = statement_line("Reset", args);
    let (target, only) = match named {
        Some(_) => match crate::commands::typed::bound::<ResetProperty>(&line) {
            Ok(b) => (b.target, Some(b.property)),
            Err(reply) => return Some(DispatchResult::Output(reply)),
        },
        // An object that resets its own way keeps it until it moves onto the registry (§33 3c2).
        None if obj.handler(crate::objects::Verb::Reset).is_some() => return None,
        None => match crate::commands::typed::bound::<ResetAll>(&line) {
            Ok(b) => (b.target, None),
            Err(reply) => return Some(DispatchResult::Output(reply)),
        },
    };
    // Down an owner-first path of scoping owners (`Reset Sequence 1 Cue 2 FeatureGroup "Dimmer"
    // "Fade"` = the cue's own timing again): one property at a time, like Set.
    if let [(None, Item::Path(segments))] = target.items.as_slice()
        && segments.len() > 1
        && segments[..segments.len() - 1].iter().any(|s| scopes(s).is_some())
    {
        let list: Vec<&'static Property> = match &only {
            Some(name) => props.get(name).into_iter().collect(),
            None => props.all().filter(|p| p.reset_value().is_some()).collect(),
        };
        if list.is_empty() {
            return Some(DispatchResult::Output(no_property(state, obj, props, args)));
        }
        let mut outs = Vec::new();
        for p in list {
            let Some(value) = p.reset_value() else { continue };
            let v = Value::Text(lumacore_syntax::ast::Spanned::new(value.to_string(), lumacore_syntax::Span { start: 0, end: 0 }));
            match in_scopes(state, segments, obj, props, p, &v, value, true) {
                DispatchResult::Output(o) => outs.push(o),
                other => return Some(other),
            }
        }
        return Some(DispatchResult::Output(Reply::joined(outs)));
    }
    let resolved = match resolve::resolve_one(state, &target, crate::objects::default_object(state)) {
        Ok(r) => r,
        Err(e) => return Some(DispatchResult::Output(e.into())),
    };
    let list: Vec<&'static Property> = match &only {
        Some(name) => match props.get(name) {
            Some(p) => vec![p],
            None => return Some(DispatchResult::Output(no_property(state, obj, props, args))),
        },
        None => props.all().filter(|p| p.reset_value().is_some()).collect(),
    };
    let first = list.first().copied()?;
    let rows = match rows_of(state, obj, props, &resolved, first) {
        Ok(r) => r,
        Err(e) => return Some(DispatchResult::Output(e.into())),
    };
    let mut outs = Vec::new();
    for row in &rows {
        for p in &list {
            let Some(value) = p.reset_value() else {
                outs.push(Reply::error(format!("{} \"{}\" has no default to go back to.", row.label, p.name), "E-CMD-010"));
                continue;
            };
            outs.push(match write(state, obj, props, row, p, value) {
                Ok(msg) => crate::text::render_success(&msg).into(),
                Err(e) => e.into(),
            });
        }
    }
    Some(DispatchResult::Output(Reply::joined(outs)))
}

/// The level a `Set` line's property needs on `obj` (the rights gate reads it): the property
/// the line names, quoted or not, wherever it stands.
pub fn prop_rights(obj: &ObjectDef, args: &[&str]) -> Option<crate::commands::registry::Rights> {
    let props = obj.props?;
    args.iter().filter(|a| !a.starts_with('/')).find_map(|a| props.get(a)).map(|p| props::rights_of(obj, p))
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
