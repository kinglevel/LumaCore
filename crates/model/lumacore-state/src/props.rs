//! The one property registry (decisions §33 58): every object declares its properties once,
//! on its `ObjectDef` ([`ObjectDef::props`]) — name, kind, range, the level it needs, where it
//! lives (the show / this station), its section, unit and doc, and how it is read and written.
//! `Set`, `Info`, `EditSetting`, the settings cells, the menus' grid columns, suggestions, the
//! rights gate, help and the reference read only these declarations; everything not declared is
//! read-only and `Set` says why. A write is an ordinary command, so the change path records undo,
//! replication and the journal entry.
//!
//! A property is read and written in one of two ways ([`Access`]): a field of the object's own
//! JSON in the tree (written back through [`Props::apply`], so the type checks every write), or
//! a declared get / set by row (this station's settings, an interface's services, the show's
//! own settings).
//!
//! [`ObjectDef::props`]: crate::defs::object::ObjectDef::props

use serde_json::Value;

use super::AppState;
use super::defs::command::Rights;

/// What a property holds.
#[derive(Debug, Clone, Copy)]
pub enum PropKind {
    /// Text; `""` or `none` clears an optional one.
    Text,
    /// On / off: `On`, `Off`, `Yes`, `No` (decisions §33 14).
    OnOff,
    /// On / off that also takes `Toggle` (a cell's tap, a window bar's box).
    Switch,
    /// A whole number in `min..=max`.
    Int {
        min: i64,
        max: i64,
    },
    /// One of these words (any case; stored as written here).
    Choice(&'static [&'static str]),
    /// A colour `#rrggbb`; `none` clears it.
    Colour,
    /// An object's Paint (`NodeMeta.paint`, decisions 0294): `#rrggbb`, a palette word, a theme
    /// slot link `Theme "<slot>"`, or `none` = its tags', then its pool's colour. Read by the one
    /// parser, `lumacore_base::Paint::parse`.
    Paint,
    /// An object's icon (`NodeMeta.icon`): a standard icon's name; `none`
    /// clears it.
    Icon,
    /// A link to another object (decisions §21 "6d"): typed by name or
    /// number, stored as its GUID. `resolve` finds it (None = cleared).
    /// `items` lists what it can point at (the pick list, decisions §30
    /// "12b"); `optional` offers None; `add` = the keyword Add New stores.
    Link {
        kind: &'static str,
        resolve: fn(&mut lumacore_tree::Root, &str) -> Result<Option<uuid::Uuid>, String>,
        items: fn(&lumacore_tree::Root) -> Vec<lumacore_wire::PickItem>,
        optional: bool,
        add: Option<&'static str>,
    },
    /// A link into the object's own pool (a profile's Images): written as
    /// text, the object's write resolves it; `items` lists that pool.
    OwnLink {
        kind: &'static str,
        items: fn(&lumacore_tree::Root, u32) -> Vec<lumacore_wire::PickItem>,
    },
    /// Declared by the object's type and checked by its own write (a window type's setting:
    /// on / off, a number, a choice, a link, `Toggle`, `Now`, relative input).
    Declared,
}

impl PropKind {
    /// On / off (with or without Toggle).
    pub fn is_on_off(self) -> bool {
        matches!(self, PropKind::OnOff | PropKind::Switch)
    }
}

/// Where a property lives (decisions §30 "12d", §39: every cell shows its home as an icon).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Home {
    /// This station only, never in the show (station_settings.json).
    Station,
    /// Saved in the show, the same on every station.
    Show,
    /// In the show file, one user's own (a profile's preferences and look, a window's settings,
    /// a user's password and rights) — decisions §39.
    User,
}

impl Home {
    /// The word the kit reads (`SettingField.home`).
    pub fn word(self) -> &'static str {
        match self {
            Home::Station => "station",
            Home::Show => "show",
            Home::User => "user",
        }
    }
}

/// How a property is read and written.
#[derive(Debug, Clone, Copy)]
pub enum Access {
    /// A field of the object's JSON (`meta.name`), written through [`Props::apply`].
    Field(&'static str),
    /// Declared per row (a row is its key: an interface name, a network number, this station).
    Keyed {
        /// The current value of a row as text.
        get: fn(&AppState, &str) -> String,
        /// Write the checked value (a choice / named word as declared, on / off as the
        /// property's `words`, a number as digits, text unquoted). Ok = what changed;
        /// Err = a rendered, coded refusal.
        set: fn(&mut AppState, &str, &str) -> Result<String, String>,
    },
    /// Declared per row and by name — one get / set for every property of a type's list (a
    /// window's settings come from its window type).
    Named {
        get: fn(&AppState, &str, &str) -> String,
        set: fn(&mut AppState, &str, &str, &str) -> Result<String, String>,
    },
}

/// One declared property.
#[derive(Debug, Clone, Copy)]
pub struct Property {
    /// As typed (quoted, decisions §33 7) and shown (`"Name"`, `"Priority"`); matched
    /// case-insensitively.
    pub name: &'static str,
    pub kind: PropKind,
    /// Named values a number also takes (`Off` = 0, `Unlimited`).
    pub named: &'static [&'static str],
    /// How an on / off value is shown: `("On", "Off")` or `("Yes", "No")`. Every on / off
    /// property takes all four words (decisions §33 14).
    pub words: (&'static str, &'static str),
    pub rights: Rights,
    pub home: Home,
    pub section: &'static str,
    pub unit: &'static str,
    pub doc: &'static str,
    /// What `Reset` puts back (decisions §33 61), as `Set` takes it; None = the kind's own:
    /// text, colours, icons and links clear, on / off goes Off, anything else has none.
    pub default: Option<&'static str>,
    pub access: Access,
}

impl Property {
    /// The defaults a declaration fills in from (`..Property::BASE`).
    pub const BASE: Property = Property {
        name: "",
        kind: PropKind::Text,
        named: &[],
        words: ("On", "Off"),
        rights: Rights::Program,
        home: Home::Show,
        section: "",
        unit: "",
        doc: "",
        default: None,
        access: Access::Field(""),
    };

    /// The value `Reset` writes (None = it has no default).
    pub fn reset_value(&self) -> Option<&'static str> {
        self.default.or(match self.kind {
            PropKind::Text | PropKind::Colour | PropKind::Paint | PropKind::Icon | PropKind::OwnLink { .. } => Some(""),
            PropKind::Link { optional: true, .. } => Some("none"),
            PropKind::OnOff | PropKind::Switch => Some("Off"),
            _ => None,
        })
    }
}

/// Edit one object's JSON in place: `edit` changes the value, the result is
/// read back as the object's own type (so the type checks every write).
/// The datapool is the one the object lives in (None outside datapools).
pub type Apply = fn(
    &mut lumacore_tree::Root,
    Option<u32>,
    u32,
    &mut dyn FnMut(&mut Value) -> Result<(), String>,
) -> Result<(), String>;

/// Whether this user may write object `n` (`Props::may`).
pub type MayWrite = fn(&AppState, u32) -> Result<(), String>;

/// Rows that aren't members of a tree collection: this station's interfaces, the stations a
/// line may name, the show's asset drives. The one resolver reads them (decisions §33 57).
pub struct Keyed {
    /// Every row key, in order.
    pub rows: fn(&AppState) -> Vec<String>,
    /// A typed row word (a name, a number) → its key; None = no such row.
    pub row: fn(&AppState, &str) -> Option<String>,
}

/// A write an object does its own way (this station's own settings report `old → new`,
/// another station's are sent to it — decisions §30 "Remote station edits"). None = the
/// ordinary write. The arguments: the row's key, the property, the value as given.
pub type OwnWrite = fn(&mut AppState, &str, &'static Property, &str) -> Option<Result<String, String>>;

/// The ask a `Set` needs before it runs (decisions §30 "12d": dangerous actions are station
/// asks): the rows' keys, whether ids were written, the property and the value as given.
pub type AskFirst = fn(&mut AppState, &[String], bool, &'static Property, &str) -> Option<crate::DispatchResult>;

/// A line naming this object as an owner (`Set Station "Tower" Network "Rig" "Interface" "en1"`)
/// sets its child's properties within that row: the guard keeps the scope until it is dropped.
pub type Within = fn(&mut AppState, &str) -> Result<Box<dyn std::any::Any>, String>;

/// An object's properties and how its rows are found.
pub struct Props {
    /// Which meta its rows have: that level's properties come first ([`MetaLevel::props`]).
    pub meta: MetaLevel,
    /// Its own declarations, in groups.
    pub lists: &'static [&'static [Property]],
    /// Rows by key; None = the members of the object's tree collection.
    pub keyed: Option<Keyed>,
    /// Its keyed rows are its members though it also has a tree to browse (a UI key: the core
    /// keys at `Root.Resources.UIKeys`, a line names any key by its id — decisions 0295).
    pub keyed_beside_tree: bool,
    /// The row a target without ids means (this station, the show, your own profile).
    pub implicit: Option<fn(&AppState) -> String>,
    /// How a row is named in replies and titles (`Interface en0`); None = `<Keyword> <n>`.
    pub label: Option<fn(&AppState, &str) -> String>,
    /// Writes a [`Access::Field`] property's object (None = it has none).
    pub apply: Option<Apply>,
    /// Who else may write an object besides the level (a profile: its users and Admins,
    /// decisions §30 "12a2"); None = the level decides.
    pub may: Option<MayWrite>,
    /// After a keyed write (save the station file, re-plan protocols, …).
    pub after: fn(&mut AppState),
    /// Every property is read-only, with this reason (About).
    pub readonly: Option<&'static str>,
    pub own_write: Option<OwnWrite>,
    pub ask: Option<AskFirst>,
    /// Its children's properties are set within one of its rows (a station's wiring row).
    pub within: Option<Within>,
    /// Properties declared as data (a window type's settings), built once: their names, kinds
    /// and docs for the words, help and the reference; each row's own type checks them.
    pub dynamic: Option<fn() -> &'static [Property]>,
    /// Properties that exist only under an owner that scopes this object (a group member's
    /// grid spot: `Set Group 3 Fixture 101 "X" 4`): `Set` and the words find them, the object's
    /// own settings and `Reset` don't list them.
    pub scoped: &'static [&'static [Property]],
    /// A hint a line naming none of the properties gets (`Set Group 4 "Master" 50` → the At form).
    pub hint: Option<&'static str>,
    /// The JSON a keyed row's [`Access::Field`] properties live in, for an object without a tree
    /// collection (a profile's Theme: the profile's JSON, row = the profile's number).
    pub view: Option<fn(&lumacore_tree::Root, u32) -> Option<Value>>,
    /// The properties edited live while someone holds the Devices draft (the Live Patch
    /// columns, decisions §34 "34g4 patch hardened" 8): they reach the live show and the
    /// holder's draft, unless the holder changed that cell there.
    pub live: &'static [&'static str],
    /// Every property of this object lives here whatever it declares (a profile's: User, decisions
    /// §39); None = each property's own `home`.
    pub home: Option<Home>,
    /// Properties its rows name themselves (a theme's own paint slots, decisions 0294): a quoted
    /// name no declaration has is this template under that name — an [`Access::Named`] whose
    /// write says when the row has no such name.
    pub open: Option<&'static Property>,
}

impl Props {
    /// Where a property of this object lives: the object's home when it has one, else the
    /// property's.
    pub fn home_of(&self, p: &Property) -> Home {
        self.home.unwrap_or(p.home)
    }

    /// The defaults a declaration fills in from (`..Props::BASE`).
    pub const BASE: Props = Props {
        meta: MetaLevel::None,
        lists: &[],
        keyed: None,
        keyed_beside_tree: false,
        implicit: None,
        label: None,
        apply: None,
        may: None,
        after: |_| {},
        readonly: None,
        own_write: None,
        ask: None,
        within: None,
        dynamic: None,
        scoped: &[],
        hint: None,
        view: None,
        live: &[],
        home: None,
        open: None,
    };

    /// Whether `name` is edited live under the Devices draft.
    pub fn is_live(&self, name: &str) -> bool {
        self.live.iter().any(|l| l.eq_ignore_ascii_case(name))
    }

    /// Every declared property, in order.
    pub fn all(&self) -> impl Iterator<Item = &'static Property> + '_ {
        self.meta
            .props()
            .iter()
            .chain(self.lists.iter())
            .flat_map(|l| l.iter())
            .chain(self.dynamic.map(|d| d()).unwrap_or_default().iter())
    }

    /// Every property, the owner-scoped ones too (the words, help, the reference).
    pub fn every(&self) -> impl Iterator<Item = &'static Property> + '_ {
        self.all().chain(self.scoped.iter().flat_map(|l| l.iter()))
    }

    /// The property named `name` (quotes ignored, any case), owner-scoped ones too; else, for an
    /// object whose rows name their own (`open`), that template under `name`.
    pub fn get(&self, name: &str) -> Option<&'static Property> {
        let name = name.trim_matches('"');
        self.every().find(|p| p.name.eq_ignore_ascii_case(name)).or_else(|| self.open.and_then(|t| open_named(t, name)))
    }
}

/// The template `t` under `name`, made once per name (a name the data gives a property: a theme's
/// own paint slot). Only a plain name: up to 32 characters, no quotes, not empty.
fn open_named(t: &'static Property, name: &str) -> Option<&'static Property> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static MADE: OnceLock<Mutex<HashMap<(usize, String), &'static Property>>> = OnceLock::new();
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 32 || name.contains(['"', ';', '`', '/']) {
        return None;
    }
    let key = (t as *const Property as usize, name.to_ascii_lowercase());
    let mut made = MADE.get_or_init(Default::default).lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    Some(*made.entry(key).or_insert_with(|| Box::leak(Box::new(Property { name: Box::leak(name.to_string().into_boxed_str()), ..*t }))))
}

/// The level a property needs on `obj`: the object's `Set` level, raised by the property's own.
pub fn rights_of(obj: &crate::defs::object::ObjectDef, p: &Property) -> Rights {
    (obj.rights)(crate::defs::object::Verb::Set, &[]).max(p.rights)
}

/// Which meta an object's rows have (decisions §12 "Tags part 1b hardened" 5): the level's
/// properties come with it, declared once — no object lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaLevel {
    /// Its rows have no meta of their own here (a keyed row, a setting).
    None,
    /// An [`Identity`](lumacore_base::Identity): its Name.
    Identity,
    /// A [`NodeMeta`](lumacore_base::NodeMeta) (every ShowData object): Name, Note, Locked,
    /// Hidden, Paint, Icon.
    Node,
}

/// The Identity block's properties. Their level is the lowest: the object's own Set level
/// decides who may write them ([`rights_of`]).
pub const IDENTITY_PROPS: [Property; 1] = [Property {
    name: "Name",
    access: Access::Field("meta.name"),
    kind: PropKind::Text,
    rights: Rights::Guest,
    doc: "display name",
    ..Property::BASE
}];

/// The Annotation block's properties (decisions §30 "12e": NodeMeta only).
pub const ANNOTATION_PROPS: [Property; 5] = [
    Property {
        name: "Note",
        access: Access::Field("meta.note"),
        kind: PropKind::Text,
        rights: Rights::Guest,
        doc: "operator note",
        ..Property::BASE
    },
    Property {
        name: "Locked",
        access: Access::Field("meta.locked"),
        kind: PropKind::OnOff,
        rights: Rights::Guest,
        doc: "protect from edits",
        ..Property::BASE
    },
    Property {
        name: "Hidden",
        access: Access::Field("meta.hidden"),
        kind: PropKind::OnOff,
        rights: Rights::Guest,
        doc: "hide from operator views",
        ..Property::BASE
    },
    Property {
        name: "Paint",
        access: Access::Field("meta.paint"),
        kind: PropKind::Paint,
        rights: Rights::Guest,
        doc: "its colour on screen: #rrggbb, a palette colour, Theme \"<slot>\" (the viewer's theme), or none = its tags', then the pool's",
        ..Property::BASE
    },
    Property {
        name: "Icon",
        access: Access::Field("meta.icon"),
        kind: PropKind::Icon,
        rights: Rights::Guest,
        doc: "a standard icon's name",
        ..Property::BASE
    },
];

/// The Paint alone (decisions 0294), for an object that declares its own meta properties (an
/// image, a profile): the one Paint property of the Annotation.
pub const META_PAINT: [Property; 1] = [ANNOTATION_PROPS[3]];

impl MetaLevel {
    /// The properties the level brings, in order.
    pub fn props(self) -> &'static [&'static [Property]] {
        match self {
            MetaLevel::None => &[],
            MetaLevel::Identity => &[&IDENTITY_PROPS],
            MetaLevel::Node => &[&IDENTITY_PROPS, &ANNOTATION_PROPS],
        }
    }
}

/// A stored value as people read and type it: a Paint as its word (`#rrggbb`,
/// `Theme "Position"`), an icon by its name.
pub fn shown(kind: PropKind, raw: &Value) -> Option<String> {
    match (kind, raw) {
        (PropKind::Paint, Value::Null) => Some("none".to_string()),
        (PropKind::Paint, v) => serde_json::from_value::<lumacore_base::Paint>(v.clone()).ok().map(|p| p.word()),
        (PropKind::Icon, Value::Object(m)) => m.get("Builtin").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// Read `value` back as `T` after `edit` changed its JSON.
pub fn edit_json<T>(
    value: &mut T,
    edit: &mut dyn FnMut(&mut Value) -> Result<(), String>,
) -> Result<(), String>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let mut json = serde_json::to_value(&*value).map_err(|e| e.to_string())?;
    edit(&mut json)?;
    *value = serde_json::from_value(json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn field_mut<'a>(json: &'a mut Value, path: &str) -> Option<&'a mut Value> {
    path.split('.').try_fold(json, |v, key| v.get_mut(key))
}

pub fn parse(kind: PropKind, raw: &str, root: &mut lumacore_tree::Root) -> Result<Value, String> {
    let raw = raw.trim_matches('"');
    match kind {
        PropKind::Link { resolve, .. } => Ok(match resolve(root, raw)? {
            Some(g) => Value::String(g.to_string()),
            None => Value::Null,
        }),
        PropKind::Text | PropKind::Colour | PropKind::Icon | PropKind::OwnLink { .. }
            if raw.is_empty() || raw.eq_ignore_ascii_case("none") =>
        {
            Ok(Value::Null)
        }
        PropKind::OwnLink { .. } | PropKind::Declared => Ok(Value::String(raw.to_string())),
        PropKind::Colour => match lumacore_base::paint::hex_rgb(raw) {
            Some(rgb) => Ok(Value::String(lumacore_base::paint::hex_of(rgb))),
            None => Err(format!("'{raw}' is not a colour #rrggbb")),
        },
        PropKind::Paint => match lumacore_base::Paint::parse(raw)? {
            Some(p) => serde_json::to_value(p).map_err(|e| e.to_string()),
            None => Ok(Value::Null),
        },
        PropKind::Icon => {
            if raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                Ok(serde_json::json!({ "Builtin": raw.to_ascii_lowercase() }))
            } else {
                Err(format!("'{raw}' is not an icon name"))
            }
        }
        PropKind::Text => Ok(Value::String(raw.to_string())),
        PropKind::OnOff | PropKind::Switch => match on_off(raw) {
            Some(b) => Ok(Value::Bool(b)),
            None => Err(format!("'{raw}' isn't On, Off, Yes or No")),
        },
        PropKind::Int { min, max } => match raw.parse::<i64>() {
            Ok(n) if (min..=max).contains(&n) => Ok(Value::from(n)),
            _ => Err(format!("'{raw}' is not a whole number {min}–{max}")),
        },
        PropKind::Choice(options) => options
            .iter()
            .find(|o| o.eq_ignore_ascii_case(raw))
            .map(|o| Value::String((*o).to_string()))
            .ok_or_else(|| format!("'{raw}' is not one of {}", options.join(", "))),
    }
}

/// The settings window's input for a property (decisions §30 "12b2").
pub fn field_input(kind: PropKind) -> lumacore_wire::FieldInput {
    use lumacore_wire::FieldInput;
    match kind {
        PropKind::Text => FieldInput::Text { max_len: 0 },
        PropKind::OnOff | PropKind::Switch => FieldInput::Toggle,
        PropKind::Int { min, max } => FieldInput::Number { min, max, named: Vec::new() },
        PropKind::Choice(options) => FieldInput::Choice { options: options.iter().map(|o| o.to_string()).collect() },
        PropKind::Colour => FieldInput::Colour,
        PropKind::Paint => FieldInput::Paint,
        PropKind::Icon => FieldInput::Text { max_len: 0 },
        PropKind::Link { .. } | PropKind::OwnLink { .. } => FieldInput::Link,
        PropKind::Declared => FieldInput::Text { max_len: 0 },
    }
}

pub fn kind_label(kind: PropKind) -> String {
    match kind {
        PropKind::Text => "text".to_string(),
        PropKind::OnOff => "On/Off".to_string(),
        PropKind::Switch => "On/Off/Toggle".to_string(),
        PropKind::Int { min, max } => format!("{min}–{max}"),
        PropKind::Choice(options) => options.join("/"),
        PropKind::Colour => "#rrggbb".to_string(),
        PropKind::Paint => "#rrggbb, a palette colour, Theme \"<slot>\" or none".to_string(),
        PropKind::Icon => "icon name".to_string(),
        PropKind::Link { kind, .. } | PropKind::OwnLink { kind, .. } => format!("{kind} name or number"),
        PropKind::Declared => "as its type declares".to_string(),
    }
}


/// A handler's rendered reply as a keyed write's result (Ok = its plain text, Err = the coded
/// refusal as rendered) — for an area whose own write logic stays while its parsing moves onto
/// the registry (§33 3c2).
pub fn from_reply(text: String) -> Result<String, String> {
    if crate::errors::outcome_of(&text).0 { Err(text) } else { Ok(crate::text::strip_ansi(&text)) }
}

/// The one on / off reading (decisions §33 14): `On` / `Yes` and `Off` / `No`, any case.
pub fn on_off(raw: &str) -> Option<bool> {
    match raw.trim().trim_matches('"').to_ascii_lowercase().as_str() {
        "on" | "yes" => Some(true),
        "off" | "no" => Some(false),
        _ => None,
    }
}

/// `+ 3` / `- 3` / `* 2` / `/ 2` applied to `now` (None: not relative).
pub fn relative(raw: &str, now: i64) -> Option<Result<i64, String>> {
    relative_f64(raw, now as f64).map(|r| r.map(|v| v.round() as i64))
}

/// [`relative`] for decimal values (metres, degrees): the result unrounded.
pub fn relative_f64(raw: &str, now: f64) -> Option<Result<f64, String>> {
    let raw = raw.trim();
    let op = raw.chars().next().filter(|c| "+-*/".contains(*c))?;
    // `-20` is a negative value (the theme adjusters); `- 20` is relative.
    if op == '-' && !raw[1..].starts_with(char::is_whitespace) {
        return None;
    }
    let by = raw[1..].trim();
    let Some(by) = by.parse::<f64>().ok().filter(|v| v.is_finite()) else {
        return Some(Err(format!("'{raw}' isn't a number")));
    };
    Some(Ok(match op {
        '+' => now + by,
        '-' => now - by,
        '*' => now * by,
        _ if by == 0.0 => return Some(Err("can't divide by 0".to_string())),
        _ => now / by,
    }))
}

pub fn json_has_key(json: &Value, name: &str) -> bool {
    match json {
        Value::Object(map) => map
            .iter()
            .any(|(k, v)| k.replace('_', "").eq_ignore_ascii_case(name) || json_has_key(v, name)),
        _ => false,
    }
}


