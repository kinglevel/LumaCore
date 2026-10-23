//! Programmer and UserProfile types.
//!
//! The Programmer is a first-class tree object under UserProfile.
//! Tracks fixture selection and attribute values with active/inactive state.

use crate::datapool::{GridCell, GridCursorMove, GroupMember};
use crate::keybindings::KeyBindingMap;
use crate::node::{NodeMeta, TreeNode};
use crate::tree::VariableStore;
use lumacore_base::{Value, next_stamp};
use serde::de;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Serialize `values` with tuple keys as JSON string keys: `"(fixture_guid, attr_name)"`.
///
/// JSON requires object keys to be strings, but `BTreeMap<(Uuid, String), V>` has tuple keys.
/// This serializer converts each `(guid, attr)` tuple to the string `"(guid, attr)"` which
/// serde can then deserialize back via the existing `deserialize_values`.
fn serialize_values<S>(
    values: &BTreeMap<(Uuid, String), ProgValue>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for ((fixture_guid, attr), value) in values {
        let key = format!("({fixture_guid}, {attr})");
        map.serialize_entry(&key, value)?;
    }
    map.end()
}

/// Parse a string key like `"(uuid, Dimmer)"` into a `(Uuid, String)` tuple.
///
/// Handles the format produced by `serialize_values`.
fn parse_tuple_key(s: &str) -> Option<(Uuid, String)> {
    let s = s.trim();
    let inner = s.strip_prefix('(')?.strip_suffix(')')?;
    let (id_part, attr_part) = inner.split_once(',')?;
    let guid: Uuid = id_part.trim().parse().ok()?;
    let attr = attr_part.trim().to_string();
    Some((guid, attr))
}

/// Deserialize `values` from the string-keyed format `"(guid, attr)" -> ProgValue`.
fn deserialize_values<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<(Uuid, String), ProgValue>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: BTreeMap<String, ProgValue> =
        BTreeMap::deserialize(deserializer).map_err(de::Error::custom)?;
    raw.into_iter()
        .map(|(key_str, value)| {
            let key = parse_tuple_key(&key_str).ok_or_else(|| {
                de::Error::custom(format!(
                    "invalid programmer value key: '{key_str}' (expected \"(guid, attr)\")"
                ))
            })?;
            Ok((key, value))
        })
        .collect()
}

/// Output verbosity level for command responses.
///
/// - `Summary` (default): minimal output — confirmation line only.
/// - `Verbose`: extra detail (fixture names, DMX values, attribute breakdown).
/// - `Debug`: internal state before/after (programmer snapshot, stack traces).
///
/// Set per-session with `set verbosity <level>`.
/// Override per-command with `/v` (Verbose) or `/d` (Debug) flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Verbosity {
    #[default]
    Summary,
    Verbose,
    Debug,
}

impl Verbosity {
    /// Parse a verbosity level from a string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "summary" | "s" | "0" => Some(Verbosity::Summary),
            "verbose" | "v" | "1" => Some(Verbosity::Verbose),
            "debug" | "d" | "2" => Some(Verbosity::Debug),
            _ => None,
        }
    }

    /// Display name for the verbosity level.
    pub fn display(&self) -> &'static str {
        match self {
            Verbosity::Summary => "Summary",
            Verbosity::Verbose => "Verbose",
            Verbosity::Debug => "Debug",
        }
    }

    pub fn is_verbose(&self) -> bool {
        matches!(self, Verbosity::Verbose | Verbosity::Debug)
    }

    pub fn is_debug(&self) -> bool {
        matches!(self, Verbosity::Debug)
    }
}

/// Convert a percentage (0.0–100.0) to a DMX byte value (0–255).
///
/// Values outside the 0–100 range are clamped. 50% maps to 128 (rounded).
pub fn percent_to_dmx(pct: f64) -> u8 {
    let clamped = pct.clamp(0.0, 100.0);
    (clamped / 100.0 * 255.0).round() as u8
}

/// A programmer value with active/inactive state.
///
/// - **Active**: value is "hot" — will be included in Store.
/// - **Inactive**: value is still in the programmer and still outputs, but
///   is not stored (decisions §27). Can be reactivated.
///
/// `stamp` is when the value was last touched: programmers merge LTP with
/// each other and with playbacks (decisions §4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgValue {
    pub value: Value,
    /// Whether this value is active (hot) or inactive (dimmed).
    pub active: bool,
    #[serde(default)]
    pub stamp: u64,
    /// Its own Fade / Delay (decisions §30 "12f" value timing); stored into cues with it.
    #[serde(default, skip_serializing_if = "lumacore_base::value::ValueTiming::is_cue")]
    pub timing: lumacore_base::value::ValueTiming,
    /// The preset this value came from (decisions 0293), kept when a plain value replaces the
    /// linked one (`At Preset 3` then `At 85`): `Update` puts the change back there. `value.preset`
    /// is the live link; this is the home.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Uuid>,
}

impl ProgValue {
    /// Create a new active programmer value, touched now.
    pub fn new(value: Value) -> Self {
        Self {
            origin: value.preset,
            value,
            active: true,
            stamp: next_stamp(),
            timing: Default::default(),
        }
    }
}

/// Three-stage reset state machine for the programmer.
///
/// Follows industry-standard order: Selection → Active → All.
/// Each press of Reset advances the state:
/// Fresh → SelectionCleared → ActiveCleared → Fresh (programmer empty)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ClearState {
    /// Programmer is clean — no pending reset in progress.
    #[default]
    Fresh,
    /// Selection has been cleared; next Reset deactivates values.
    SelectionCleared,
    /// Active values deactivated; next Reset empties programmer.
    ActiveCleared,
}

/// A user's rights level (decisions §30 "12d hardening": one ladder, one
/// level per user, never from the profile). Ordered: each level may do
/// everything of the ones below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum UserRights {
    /// Look, log in, their own device, password, colour and look.
    Guest,
    /// Run the show and use the programmer; never store.
    Playback,
    /// Playback, and update existing presets.
    Presets,
    /// Build the show (Program; the live patch).
    #[default]
    Program,
    /// The rig, the station and the LumaSession: full patch, lines,
    /// networks, station settings, load / shut down.
    Setup,
    /// Everything: users and profiles, `/Force`, NewShow, CleanStart.
    Admin,
}

/// A user's level as the level a word needs (the catalog's `Rights`, decisions §34 21).
impl From<UserRights> for lumacore_registry::command::Rights {
    fn from(r: UserRights) -> Self {
        use UserRights as U;
        match r {
            U::Guest => Self::Guest,
            U::Playback => Self::Playback,
            U::Presets => Self::Presets,
            U::Program => Self::Program,
            U::Setup => Self::Setup,
            U::Admin => Self::Admin,
        }
    }
}

impl UserRights {
    /// Every level, lowest first.
    pub const ALL: [Self; 6] = [Self::Guest, Self::Playback, Self::Presets, Self::Program, Self::Setup, Self::Admin];
    /// The levels' names, in order (what `Set User "x" "Rights"` takes).
    pub const LABELS: &'static [&'static str] = &["Guest", "Playback", "Presets", "Program", "Setup", "Admin"];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Guest => "Guest",
            Self::Playback => "Playback",
            Self::Presets => "Presets",
            Self::Program => "Program",
            Self::Setup => "Setup",
            Self::Admin => "Admin",
        }
    }

    /// A level by its name (any case).
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_matches('"');
        Self::ALL.into_iter().find(|r| r.label().eq_ignore_ascii_case(s))
    }
}

/// The programmer: tracks currently selected fixtures and their attribute values.
///
/// Values have an active/inactive flag. Active values drive DMX output and will
/// be included in Store operations. Inactive values are visible but dormant.
///
/// All fixture references use GUIDs for stability across renumbering and import.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Programmer {
    /// GUIDs of currently selected fixtures (in selection order).
    pub selected_fixtures: Vec<Uuid>,
    /// Attribute values: (fixture_guid, attribute_name) -> ProgValue.
    /// Contains both active and inactive values. A BTreeMap so saved bytes
    /// are stable (decisions §29, finding 1).
    #[serde(
        serialize_with = "serialize_values",
        deserialize_with = "deserialize_values",
        default
    )]
    pub values: BTreeMap<(Uuid, String), ProgValue>,
    /// Current state of the four-stage reset machine.
    #[serde(default)]
    pub clear_state: ClearState,

    // ── Walk state (Next/Previous) ───────────────────────────────────────
    /// The original multi-selection pool that Next/Previous walks through.
    /// `None` when not in walk mode.
    #[serde(default, skip)]
    pub selection_pool: Option<Vec<Uuid>>,
    /// Current index into `selection_pool`.
    #[serde(default, skip)]
    pub walk_index: usize,
    /// Tracks whether "Programmer cleared" was echoed, so next reset clears buffer.
    #[serde(default, skip)]
    pub reset_echoed: bool,

    // ── Selection grid (decisions §22) ────────────────────────────────────
    /// Grid spot of each selected fixture (the selection grid).
    #[serde(default)]
    pub grid: BTreeMap<Uuid, GridCell>,
    /// Where the next selection lands in the grid.
    #[serde(default)]
    pub grid_cursor: GridCell,
    /// A value was set since the last selection: the next plain select
    /// replaces the selection instead of adding to it (the usual console convention).
    #[serde(default)]
    pub value_since_select: bool,
}

/// How a selection combines with the current one (decisions §22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectMode {
    /// Adds, unless a value was set since the last select: then replaces.
    Plain,
    /// `+ …`: always adds.
    Add,
    /// Always replaces.
    Replace,
}

impl Programmer {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when programmer has no selection, no values, and is in Fresh state.
    pub fn is_empty(&self) -> bool {
        self.selected_fixtures.is_empty()
            && self.values.is_empty()
            && self.clear_state == ClearState::Fresh
    }

    /// Select a fixture by GUID (appends if not already selected).
    pub fn select(&mut self, fixture_guid: Uuid) {
        self.apply_selection(&[fixture_guid], &[], SelectMode::Add, true, GridCursorMove::AppendX);
    }

    /// Deselect a specific fixture by GUID.
    pub fn deselect(&mut self, fixture_guid: Uuid) {
        self.deselect_all(&[fixture_guid]);
    }

    /// Deselect these fixtures (`- Fixture 3`). Returns how many left.
    pub fn deselect_all(&mut self, fixtures: &[Uuid]) -> usize {
        let before = self.selected_fixtures.len();
        self.selected_fixtures.retain(|g| !fixtures.contains(g));
        self.prune_grid();
        self.exit_walk();
        before - self.selected_fixtures.len()
    }

    /// Clear all selected fixtures and exit walk mode.
    pub fn clear_selection(&mut self) {
        self.selected_fixtures.clear();
        self.reset_grid();
        self.exit_walk();
    }

    /// Drop grid spots of fixtures no longer selected.
    pub fn prune_grid(&mut self) {
        let selected: std::collections::HashSet<Uuid> = self.selected_fixtures.iter().copied().collect();
        self.grid.retain(|g, _| selected.contains(g));
    }

    /// The selection was cleared or replaced: the grid empties and its
    /// cursor goes home.
    fn reset_grid(&mut self) {
        self.grid.clear();
        self.grid_cursor = GridCell::default();
    }

    /// Select `fixtures` (in order) by the selection rule (decisions §22):
    /// `cells` are their stored grid spots (a group's; empty = a row from
    /// the cursor). `preserve`: keep the spots' gaps and offset, else close
    /// them up. `cursor`: where the grid cursor moves afterwards (Linked
    /// already resolved). Fixtures already selected stay where they are.
    /// Returns the selection's size.
    pub fn apply_selection(
        &mut self,
        fixtures: &[Uuid],
        cells: &[GridCell],
        how: SelectMode,
        preserve: bool,
        cursor: GridCursorMove,
    ) -> usize {
        self.prune_grid();
        let replace = match how {
            SelectMode::Replace => true,
            SelectMode::Add => false,
            SelectMode::Plain => self.value_since_select,
        };
        if replace {
            self.selected_fixtures.clear();
            self.reset_grid();
        }
        self.value_since_select = false;
        self.clear_state = ClearState::Fresh;
        self.exit_walk();

        let spots: Vec<GridCell> = if cells.len() == fixtures.len() {
            if preserve {
                cells.to_vec()
            } else {
                close_up(cells)
            }
        } else {
            (0..fixtures.len())
                .map(|i| GridCell {
                    x: i as i32,
                    ..GridCell::default()
                })
                .collect()
        };
        let base = self.grid_cursor;
        let (mut max_x, mut max_y) = (None::<i32>, None::<i32>);
        // A set, not `Vec::contains`: 35k fixtures stay linear (R1-71).
        let mut selected: std::collections::HashSet<Uuid> = self.selected_fixtures.iter().copied().collect();
        for (&fixture, spot) in fixtures.iter().zip(spots) {
            if !selected.insert(fixture) {
                continue;
            }
            let cell = GridCell {
                // Saturating: `Grid 2147483647` never overflows (R1-12).
                x: base.x.saturating_add(spot.x),
                y: base.y.saturating_add(spot.y),
                z: base.z.saturating_add(spot.z),
                ..spot
            };
            max_x = Some(max_x.map_or(cell.x, |m| m.max(cell.x)));
            max_y = Some(max_y.map_or(cell.y, |m| m.max(cell.y)));
            self.selected_fixtures.push(fixture);
            self.grid.insert(fixture, cell);
        }
        match cursor {
            GridCursorMove::AppendX | GridCursorMove::Linked => {
                if let Some(x) = max_x {
                    self.grid_cursor.x = x.saturating_add(1);
                }
            }
            GridCursorMove::NewLine => {
                if let Some(y) = max_y {
                    self.grid_cursor = GridCell {
                        y: y.saturating_add(1),
                        ..GridCell::default()
                    };
                }
            }
            GridCursorMove::None => {}
        }
        self.selected_fixtures.len()
    }

    /// The selection with its grid spots, in selection order.
    pub fn selection_members(&self) -> Vec<GroupMember> {
        self.selected_fixtures
            .iter()
            .enumerate()
            .map(|(i, &fixture)| GroupMember {
                fixture,
                cell: self.grid.get(&fixture).copied().unwrap_or(GridCell {
                    x: i as i32,
                    ..GridCell::default()
                }),
            })
            .collect()
    }

    /// Check if a fixture is currently selected (by GUID).
    pub fn is_selected(&self, fixture_guid: Uuid) -> bool {
        self.selected_fixtures.contains(&fixture_guid)
    }

    /// Set an attribute value for a fixture in the programmer (always
    /// active, touched now).
    pub fn set_value(&mut self, fixture_guid: Uuid, attribute: impl Into<String>, value: Value) {
        let key = (fixture_guid, attribute.into());
        let mut pv = ProgValue::new(value);
        // A plain value over a linked one keeps where it came from (decisions 0293).
        if pv.origin.is_none() {
            pv.origin = self.values.get(&key).and_then(|old| old.origin);
        }
        self.values.insert(key, pv);
        self.value_since_select = true;
    }

    /// The programmer value of an attribute (active or inactive).
    pub fn get(&self, fixture_guid: Uuid, attribute: &str) -> Option<&ProgValue> {
        self.values.get(&(fixture_guid, attribute.to_string()))
    }

    /// Get an active attribute value from the programmer.
    /// Returns None if the value doesn't exist or is inactive.
    pub fn get_active_value(&self, fixture_guid: Uuid, attribute: &str) -> Option<&Value> {
        self.get(fixture_guid, attribute)
            .filter(|pv| pv.active)
            .map(|pv| &pv.value)
    }

    /// Get any attribute value from the programmer (active or inactive).
    pub fn get_value(&self, fixture_guid: Uuid, attribute: &str) -> Option<&Value> {
        self.get(fixture_guid, attribute).map(|pv| &pv.value)
    }

    /// Check if a value exists and is active.
    pub fn is_value_active(&self, fixture_guid: Uuid, attribute: &str) -> bool {
        self.get(fixture_guid, attribute)
            .is_some_and(|pv| pv.active)
    }

    /// Touch every value (they become the latest — LTP). Leaving Blind does
    /// this (decisions §4).
    pub fn touch_all(&mut self) {
        for pv in self.values.values_mut() {
            pv.stamp = next_stamp();
        }
    }

    /// Clear all values (active and inactive) without affecting selection.
    pub fn clear_values(&mut self) {
        self.values.clear();
    }

    /// Deactivate all active values (set active = false). Values remain visible.
    pub fn deactivate_values(&mut self) {
        for pv in self.values.values_mut() {
            pv.active = false;
        }
    }

    /// Reactivate all inactive values.
    pub fn activate_values(&mut self) {
        for pv in self.values.values_mut() {
            pv.active = true;
        }
    }

    /// Knock in (`On`): activate the values these fixtures have in the
    /// programmer. Returns how many values were activated.
    pub fn knock_in(&mut self, fixtures: &[Uuid]) -> usize {
        let mut count = 0;
        for ((guid, _), pv) in self.values.iter_mut() {
            if fixtures.contains(guid) {
                if !pv.active {
                    pv.active = true;
                    count += 1;
                }
                // Knocking in is a touch: the values become the latest (LTP).
                pv.stamp = next_stamp();
            }
        }
        count
    }

    /// Knock out (`Off`): remove these fixtures from the programmer —
    /// their values and their selection. Returns how many values were removed.
    pub fn knock_out(&mut self, fixtures: &[Uuid]) -> usize {
        let before = self.values.len();
        self.values.retain(|(guid, _), _| !fixtures.contains(guid));
        self.selected_fixtures.retain(|g| !fixtures.contains(g));
        if let Some(pool) = self.selection_pool.as_mut() {
            pool.retain(|g| !fixtures.contains(g));
        }
        before - self.values.len()
    }

    /// Check if there are any active values in the programmer.
    pub fn has_active_values(&self) -> bool {
        self.values.values().any(|pv| pv.active)
    }

    /// Three-stage reset: advances the reset state machine one step.
    ///
    /// Industry-standard order: Selection → Active → All.
    ///
    /// Returns a status message describing what was cleared.
    ///
    /// State machine:
    /// - `Fresh` (selection present) → clear selection → `SelectionCleared`
    /// - `Fresh` (no selection, active values) → deactivate → `ActiveCleared`
    /// - `SelectionCleared` (active values) → deactivate → `ActiveCleared`
    /// - `SelectionCleared` (no active values) → empty → `Fresh`
    /// - `ActiveCleared` → empty programmer → `Fresh`
    pub fn clear_layer(&mut self) -> &'static str {
        match self.clear_state {
            ClearState::Fresh => {
                if !self.selected_fixtures.is_empty() {
                    self.selected_fixtures.clear();
                    self.reset_grid();
                    self.exit_walk();
                    self.clear_state = ClearState::SelectionCleared;
                    "Selection cleared"
                } else if self.has_active_values() {
                    self.deactivate_values();
                    self.clear_state = ClearState::ActiveCleared;
                    "Values deactivated"
                } else if !self.values.is_empty() {
                    // Only inactive values remain — clear them
                    self.values.clear();
                    self.clear_state = ClearState::Fresh;
                    "Programmer cleared"
                } else {
                    "Programmer cleared"
                }
            }
            ClearState::SelectionCleared => {
                if self.has_active_values() {
                    self.deactivate_values();
                    self.clear_state = ClearState::ActiveCleared;
                    "Values deactivated"
                } else {
                    self.values.clear();
                    self.selected_fixtures.clear();
                    self.reset_grid();
                    self.clear_state = ClearState::Fresh;
                    "Programmer cleared"
                }
            }
            ClearState::ActiveCleared => {
                self.values.clear();
                self.selected_fixtures.clear();
                self.reset_grid();
                self.clear_state = ClearState::Fresh;
                "Programmer cleared"
            }
        }
    }

    /// Unconditional full clear: clears values, selection, walk state, and resets to Fresh.
    pub fn full_clear(&mut self) {
        self.values.clear();
        self.selected_fixtures.clear();
        self.reset_grid();
        self.value_since_select = false;
        self.clear_state = ClearState::Fresh;
        self.exit_walk();
    }

    /// Exit walk mode (clear pool and index).
    pub fn exit_walk(&mut self) {
        self.selection_pool = None;
        self.walk_index = 0;
    }

    /// Check if the programmer is in walk mode (Next/Previous active).
    pub fn is_walking(&self) -> bool {
        self.selection_pool.is_some()
    }
}

/// Grid spots with gaps and offset removed (PreserveGridPositions off):
/// each axis's used values become 0, 1, 2, …
fn close_up(cells: &[GridCell]) -> Vec<GridCell> {
    fn ranks(values: impl Iterator<Item = i32>) -> Vec<i32> {
        let mut v: Vec<i32> = values.collect();
        v.sort_unstable();
        v.dedup();
        v
    }
    let xs = ranks(cells.iter().map(|c| c.x));
    let ys = ranks(cells.iter().map(|c| c.y));
    let zs = ranks(cells.iter().map(|c| c.z));
    let rank = |list: &[i32], v: i32| list.binary_search(&v).unwrap_or(0) as i32;
    cells
        .iter()
        .map(|c| GridCell {
            x: rank(&xs, c.x),
            y: rank(&ys, c.y),
            z: rank(&zs, c.z),
            ..*c
        })
        .collect()
}

/// A user profile in the show.
///
/// Each user has their own programmer, settings, and personal macros/aliases.
/// A stored view: a copy of one or more screens' layouts and
/// windows, keyed by screen (screen) number.
///
/// Store copies the selected screens into the view (always overwriting);
/// recall copies them back — a single-screen view onto the screen you recall
/// it from, a multi-screen view onto the screens it was stored from. There is
/// no live link: editing windows does not change the view until stored again.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredView {
    pub meta: NodeMeta,
    pub screens: BTreeMap<u32, ScreenContent>,
}

/// What a view button holds (decisions 0287): any object, by kind and GUID. A tap runs the
/// object's default verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewButton {
    pub kind: ButtonKind,
    pub guid: Uuid,
}

/// The kinds a view button can hold: the profile's views and the datapools' objects. A new kind
/// is one variant and its keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonKind {
    View,
    Group,
    Sequence,
    Macro,
}

impl ButtonKind {
    pub const ALL: [ButtonKind; 4] = [ButtonKind::View, ButtonKind::Group, ButtonKind::Sequence, ButtonKind::Macro];

    /// The object's keyword (its registry word and link kind).
    pub fn keyword(self) -> &'static str {
        match self {
            ButtonKind::View => "View",
            ButtonKind::Group => "Group",
            ButtonKind::Sequence => "Sequence",
            ButtonKind::Macro => "Macro",
        }
    }

    pub fn of_keyword(word: &str) -> Option<ButtonKind> {
        Self::ALL.into_iter().find(|k| k.keyword().eq_ignore_ascii_case(word))
    }
}

/// A login identity (`ShowData.Users`). Several users can share one
/// `UserProfile` (how they work: views, screen configs, keys, settings);
/// rights, password, remote login and the user's own work — programmer and
/// selection — are per user (decisions §10), so a user keeps their work when
/// they change profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub meta: NodeMeta,
    /// The name a user logs in with: the user's key, required and unique (the display name is
    /// the meta's, decisions §12 "Tags part 1b hardened" 3).
    pub login: String,
    /// `pbkdf2-sha256$<iterations>$<salt hex>$<hash hex>`; None = no password.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    /// The user's rights level (decisions §30 "12d"): the only place rights
    /// come from.
    #[serde(default)]
    pub rights: UserRights,
    /// May log in from another machine (thin client / web).
    #[serde(default = "default_true")]
    pub remote_login: bool,
    /// The `UserProfile` this user works in (GUID link).
    pub profile: Uuid,
    /// The user's own colour (`#rrggbb`): their avatar ring on the lock
    /// surface (decisions §30 "12a3"). A new user takes the next of
    /// [`USER_COLORS`]; Admin the gold accent.
    pub color: String,
    /// Screen configuration a client uses the first time this user connects
    /// (clients then remember their own choice per server); None = the
    /// profile's first (GUID link).
    #[serde(default)]
    pub default_screen_config: Option<Uuid>,
    /// The user's default object (decisions §33 21, 39): the object a bare number or name names
    /// (`Group>` then `3 At 50` = `Group 3 At 50`), set by typing the keyword alone; its keyword,
    /// None = `Fixture`. Replicated, never an Oops step (the default keyword).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_object: Option<String>,
    /// This user's programmer: selection and values (saved with the show;
    /// shared by every station and client the user is logged in on).
    #[serde(default)]
    pub programmer: Programmer,
    /// The one-only selections (decisions §29, `Select`).
    #[serde(default)]
    pub selected: Selected,
    /// What this user uses (decisions §30 "12b2"): ranks their suggestions.
    /// Saved and replicated, never undone (`ShowSubtree::Usage`).
    #[serde(default)]
    pub usage: crate::usage::Usage,
    /// This user's NAV bookmarks: a letter → an object's GUID (decisions 0291). Saved and
    /// replicated, never undone (`ShowSubtree::Bookmarks`).
    #[serde(default)]
    pub bookmarks: crate::bookmarks::Bookmarks,
}

/// The two undo subtrees of a user's work (decisions §29 "11c hardening"):
/// the programmer's values ([`ProgrammerValues`]) and the selection
/// ([`SelectionState`]: the fixture selection with its grid plus the one-only
/// selections). Both destructure `Programmer` exhaustively, so a new field
/// doesn't compile until it is placed in one of them (or skipped).
#[derive(Serialize)]
pub struct ProgrammerValuesRef<'a> {
    #[serde(serialize_with = "serialize_values_ref")]
    values: &'a BTreeMap<(Uuid, String), ProgValue>,
    clear_state: &'a ClearState,
}

fn serialize_values_ref<S: serde::Serializer>(
    values: &&BTreeMap<(Uuid, String), ProgValue>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serialize_values(values, serializer)
}

/// The owned form of [`ProgrammerValuesRef`] (an undo restore).
#[derive(Deserialize)]
pub struct ProgrammerValues {
    #[serde(deserialize_with = "deserialize_values", default)]
    values: BTreeMap<(Uuid, String), ProgValue>,
    #[serde(default)]
    clear_state: ClearState,
}

#[derive(Serialize)]
pub struct SelectionRef<'a> {
    selected_fixtures: &'a Vec<Uuid>,
    grid: &'a BTreeMap<Uuid, GridCell>,
    grid_cursor: &'a GridCell,
    selected: &'a Selected,
}

/// The owned form of [`SelectionRef`] (an undo restore).
#[derive(Deserialize)]
pub struct SelectionState {
    #[serde(default)]
    selected_fixtures: Vec<Uuid>,
    #[serde(default)]
    grid: BTreeMap<Uuid, GridCell>,
    #[serde(default)]
    grid_cursor: GridCell,
    #[serde(default)]
    selected: Selected,
}

impl User {
    /// The programmer values (undo subtree `Programmer`).
    pub fn programmer_values(&self) -> ProgrammerValuesRef<'_> {
        let Programmer {
            selected_fixtures: _,
            values,
            clear_state,
            selection_pool: _,
            walk_index: _,
            reset_echoed: _,
            grid: _,
            grid_cursor: _,
            // The add/replace flag belongs to neither subtree: in either,
            // undoing one kind of step would roll the other kind back.
            value_since_select: _,
        } = &self.programmer;
        ProgrammerValuesRef {
            values,
            clear_state,
        }
    }

    pub fn set_programmer_values(&mut self, v: ProgrammerValues) {
        self.programmer.values = v.values;
        self.programmer.clear_state = v.clear_state;
    }

    /// The selection (undo subtree `Selection`).
    pub fn selection(&self) -> SelectionRef<'_> {
        let Programmer {
            selected_fixtures,
            values: _,
            clear_state: _,
            selection_pool: _,
            walk_index: _,
            reset_echoed: _,
            grid,
            grid_cursor,
            value_since_select: _,
        } = &self.programmer;
        SelectionRef {
            selected_fixtures,
            grid,
            grid_cursor,
            selected: &self.selected,
        }
    }

    /// Put a selection back; a Next/Previous walk starts again.
    pub fn set_selection(&mut self, s: SelectionState) {
        let p = &mut self.programmer;
        p.selected_fixtures = s.selected_fixtures;
        p.grid = s.grid;
        p.grid_cursor = s.grid_cursor;
        p.selection_pool = None;
        p.walk_index = 0;
        self.selected = s.selected;
    }
}

/// A user's one-only selections, apart from the fixture selection
/// (decisions §29 "11b hardening", `Select`): the datapool pool objects
/// resolve in, the sequence bare playback words act on, the timecode show.
/// GUID links; deleting the object clears it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Selected {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datapool: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timecode: Option<Uuid>,
}

impl User {
    /// A new user logging in as `login`; `no` (its number in the show) picks its colour.
    pub fn new(no: u32, login: impl Into<String>, rights: UserRights, profile: Uuid) -> Self {
        Self {
            meta: NodeMeta::default(),
            login: login.into(),
            password_hash: None,
            rights,
            remote_login: true,
            profile,
            color: String::new(),
            default_screen_config: None,
            default_object: None,
            programmer: Programmer::new(),
            selected: Selected::default(),
            usage: Default::default(),
            bookmarks: Default::default(),
        }
        .with_default_color(no)
    }

    /// The colour a new user starts with: Admin gold, else the next of
    /// [`USER_COLORS`] by its number.
    pub fn with_default_color(mut self, no: u32) -> Self {
        self.color = if self.login.eq_ignore_ascii_case("Admin") {
            "#9a7a08".into()
        } else {
            USER_COLORS[(no.max(1) as usize - 1) % USER_COLORS.len()].into()
        };
        self
    }
}

/// Twelve calm user colours that sit with the console concept (avatar rings).
pub const USER_COLORS: [&str; 12] = [
    "#758ba0", "#3f9a90", "#c27a6a", "#8a80c8", "#9aa040", "#b0705f",
    "#5a8fb0", "#c29a3a", "#7a9a6a", "#a0708f", "#6a8a9a", "#b08a5a",
];

/// The names of [`USER_COLORS`], in the same order (`Store User … /Colour
/// Teal`, decisions §30 "12c2").
pub const USER_COLOR_NAMES: [&str; 12] = [
    "Slate", "Teal", "Clay", "Lavender", "Olive", "Rust",
    "Sky", "Amber", "Sage", "Mauve", "Steel", "Sand",
];

/// The name of the locked template profile every show has (decisions §30
/// "12c2").
pub const DEFAULT_PROFILE: &str = "Default";

fn default_true() -> bool {
    true
}

/// A named set of screens (a ScreenConfiguration, but with any number of
/// screens): the live windows of each screen, saved with the show so that
/// save → shutdown → reopen puts every window back. Which screens are open on
/// a machine, and where, is client-local and not stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConfig {
    pub meta: NodeMeta,
    /// Screen number → its windows and layout.
    #[serde(default)]
    pub screens: BTreeMap<u32, ScreenContent>,
}

/// A window's look and settings kept on the profile to load into other
/// windows of the same type (decisions §30 "12c": `WindowPreset`, a
/// `StorePreferences.Views`). It holds everything of a window — settings
/// (the bar included: `window_bar`, `window_bar_items`), label, colour and
/// scroll; loading copies it in (no live link).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowPreset {
    pub meta: NodeMeta,
    /// The window type (`dmx`); a preset loads only into windows of it.
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    #[serde(default)]
    pub scroll: crate::runtime::WindowScroll,
    #[serde(default)]
    pub settings: serde_json::Map<String, serde_json::Value>,
}

/// A profile's window presets: window type → number → preset (each type
/// numbers its own, like per-window StorePreferences).
pub type WindowPresets = BTreeMap<String, BTreeMap<u32, WindowPreset>>;

/// One screen's layout inside a stored view (ScreenContent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenContent {
    pub meta: NodeMeta,
    pub root: crate::runtime::LayoutNode,
    pub windows: BTreeMap<u32, crate::runtime::Window>,
    /// Its encoder bar (decisions §30 "12f"): shown or not, which layout.
    #[serde(default, skip_serializing_if = "EncoderBarSetting::is_default")]
    pub encoder_bar: EncoderBarSetting,
}

/// A screen's encoder bar (decisions §30 "12f"): `on` None = the default (on for Screen 1), and
/// which layout of the profile's EncoderBar pool (None = its first).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EncoderBarSetting {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Uuid>,
}

impl EncoderBarSetting {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Whether screen `screen` shows its bar.
    pub fn shown(&self, screen: u32) -> bool {
        self.on.unwrap_or(screen == 1)
    }
}

impl ScreenContent {
    /// Drop the windows whose type `known` doesn't have (a type renamed or
    /// removed in this build, R1-57, Q32); their panes close. Returns the
    /// dropped (window, type).
    pub fn drop_unknown_windows(&mut self, known: &dyn Fn(&str) -> bool) -> Vec<(u32, String)> {
        let gone: Vec<(u32, String)> = self
            .windows
            .iter()
            .filter(|(_, w)| !known(&w.kind))
            .map(|(&n, w)| (n, w.kind.clone()))
            .collect();
        if gone.is_empty() {
            return gone;
        }
        let numbers: Vec<u32> = gone.iter().map(|(n, _)| *n).collect();
        self.windows.retain(|n, _| !numbers.contains(n));
        let empty = crate::runtime::LayoutNode::Leaf { windows: Vec::new(), active: 0 };
        let root = std::mem::replace(&mut self.root, empty.clone());
        self.root = root.without(&numbers).unwrap_or(empty);
        gone
    }
}

/// Accepts the current map form and skips the old `views: []` placeholder list,
/// so shows saved before views existed still load.
fn de_views<'de, D>(d: D) -> Result<BTreeMap<u32, StoredView>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(d)?;
    Ok(match value {
        serde_json::Value::Object(_) => {
            serde_json::from_value(value).map_err(serde::de::Error::custom)?
        }
        _ => BTreeMap::new(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(default)]
    pub preferred_ui_engine: String,
    /// Double-press window for UI keys, ms ("pressing Menu 2x quickly").
    #[serde(default = "default_double_click_ms")]
    pub double_click_ms: u32,
    /// Press-and-hold threshold for UI keys, ms.
    #[serde(default = "default_hold_ms")]
    pub hold_ms: u32,
    /// The held keyboard modifier that acts as **Meow**:
    /// `Shift` (default; decisions §30 "12b2"), `Alt` or
    /// `Ctrl` (decisions §17).
    #[serde(default = "default_meow_key")]
    pub meow_key: String,
    /// Notifications: `"<category>.<level>"` → `none` | `card` | `popup`
    /// (overrides of the defaults; Message Center bells). Important and
    /// critical events can't be turned off.
    #[serde(default)]
    pub notifications: BTreeMap<String, String>,
    /// Cards: `timed` (5 s) or `permanent` (until closed) — the Notification Mode.
    #[serde(default = "default_notification_mode")]
    pub notification_mode: String,
    /// Where a selection moves the grid cursor when the group says Linked
    /// (Move Grid Cursor; decisions §22).
    #[serde(default = "default_grid_cursor_move")]
    pub grid_cursor_move: GridCursorMove,
    /// Dragging a pool button: `Off` (touch screens) or `Move` (Ctrl-drag
    /// copies). Decisions §22.
    #[serde(default = "default_pool_drag")]
    pub pool_drag: String,
    /// Oops settings (UserProfile CreateOops / OopsViews /
    /// OopsProgrammer / OopsSelection / OopsConfirmation; decisions §29 "11c").
    #[serde(default)]
    pub oops: OopsSettings,
    /// The command line's suggestion list (decisions §30 "12b2"): Auto,
    /// TabOnly or Off (Users & Profiles → Command line).
    #[serde(default)]
    pub suggestions: SuggestionsMode,
    /// Holding a pane full screen on a phone or tablet maximises that window
    /// on the shared live screen too — the desk follows (decisions §30
    /// "12c2"); off = only on the device that held it.
    #[serde(default = "default_true")]
    pub hold_follows: bool,
    /// Dim the interface behind a pop-up (decisions §30 "12f").
    #[serde(default = "default_true")]
    pub popup_dim: bool,
    /// NAV's gesture log: the lines NAV put on the command line and what it copied, three
    /// lines above the command line (decisions 0291; `L` in NAV toggles it).
    #[serde(default = "default_true")]
    pub nav_log: bool,
    /// Moving a main attribute brings its sub-attributes (decisions §30 "12f-a").
    #[serde(default)]
    pub sub_attributes_follow: SubAttributesFollow,
    /// The MVR import preset (decisions §34 "Import MVR hardened" 2): the Options step's settings,
    /// each a declared `UserProfile` property of the section "MVR import".
    #[serde(default)]
    pub mvr: MvrImportPreset,
    /// The Update ask's choices as last used (decisions 0293): Content and its two switches.
    #[serde(default)]
    pub update: UpdateChoices,
    /// The paint picker's mode as last used (decisions object-paint-palette-free-theme-links):
    /// `Palette`, `Free` or `Theme`.
    #[serde(default = "default_paint_mode")]
    pub paint_mode: String,
    /// The colours this profile painted with last, newest first (`#rrggbb`, at most
    /// [`PAINT_RECENT_MAX`]); the station keeps it as Paint lines run.
    #[serde(default)]
    pub paint_recent: Vec<String>,
}

/// How many recent colours a profile keeps for the paint picker.
pub const PAINT_RECENT_MAX: usize = 8;

fn default_paint_mode() -> String {
    "Palette".to_string()
}

/// What the Update ask starts with (decisions 0293): the Content (`OnlyExisting` → `/Merge`,
/// `AddNew` → `/Add`, `Overwrite` → `/Overwrite`), Recast (add new attributes to the cues that follow) and Shield (protect the
/// next cues). The last update's choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateChoices {
    #[serde(default = "default_update_content")]
    pub content: String,
    #[serde(default = "default_true")]
    pub recast: bool,
    #[serde(default = "default_true")]
    pub shield: bool,
}

fn default_update_content() -> String {
    "OnlyExisting".to_string()
}

impl Default for UpdateChoices {
    fn default() -> Self {
        Self { content: default_update_content(), recast: true, shield: true }
    }
}

/// How an MVR import matches, what it takes and what it offers (decisions §34 "Import MVR
/// hardened" 2); the line's options override it for one import.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MvrImportPreset {
    #[serde(default)]
    pub match_key: MvrMatch,
    #[serde(default)]
    pub lost: MvrLost,
    #[serde(default)]
    pub rotation: MvrRotation,
    #[serde(default)]
    pub missing: MvrMissing,
    /// The id type an untyped CustomId gets (decisions §40 9, 10): Generic by default.
    #[serde(default = "generic_id_type")]
    pub id_type: lumacore_fixtures::IdType,
    #[serde(default = "offer")]
    pub addresses: MvrTake,
    #[serde(default = "offer")]
    pub ids: MvrTake,
    #[serde(default = "offer")]
    pub names: MvrTake,
    #[serde(default = "offer")]
    pub position: MvrTake,
    #[serde(default = "offer")]
    pub revisions: MvrTake,
    #[serde(default = "take")]
    pub new_fixtures: MvrTake,
    #[serde(default)]
    pub fid_offset: i64,
    #[serde(default)]
    pub universe_offset: i64,
    /// The rig offset, mm (MVR's unit).
    #[serde(default)]
    pub x_mm: i64,
    #[serde(default)]
    pub y_mm: i64,
    #[serde(default)]
    pub z_mm: i64,
    /// The rig rotation about Z, degrees.
    #[serde(default)]
    pub rotate_deg: i64,
}

fn offer() -> MvrTake {
    MvrTake::Offer
}
fn take() -> MvrTake {
    MvrTake::Take
}

fn generic_id_type() -> lumacore_fixtures::IdType {
    lumacore_fixtures::IdType::Generic
}

impl Default for MvrImportPreset {
    fn default() -> Self {
        Self {
            match_key: MvrMatch::Auto,
            lost: MvrLost::Mark,
            rotation: MvrRotation::Take,
            missing: MvrMissing::Ask,
            id_type: lumacore_fixtures::IdType::Generic,
            addresses: MvrTake::Offer,
            ids: MvrTake::Offer,
            names: MvrTake::Offer,
            position: MvrTake::Offer,
            revisions: MvrTake::Offer,
            new_fixtures: MvrTake::Take,
            fid_offset: 0,
            universe_offset: 0,
            x_mm: 0,
            y_mm: 0,
            z_mm: 0,
            rotate_deg: 0,
        }
    }
}

/// The match key: Auto = UUID, then FID, then the fixture's name.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvrMatch {
    #[default]
    Auto,
    #[serde(rename = "UUID")]
    Uuid,
    #[serde(rename = "FID")]
    Fid,
    FixtureName,
}

/// A fixture in scope the file no longer has.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvrLost {
    #[default]
    Mark,
    Delete,
}

/// The file's rotation taken with its position, or ignored.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvrRotation {
    #[default]
    Take,
    Ignore,
}

/// A new fixture whose MVR group the show no longer has (decision 4).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvrMissing {
    #[default]
    Ask,
    Recreate,
    Parent,
    None,
}

/// What the file may change in a field group: nothing, taken (ticked in the review), offered
/// (shown, unticked).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MvrTake {
    None,
    Take,
    #[default]
    Offer,
}

impl MvrTake {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Take => "Take",
            Self::Offer => "Offer",
        }
    }
}

/// `SubAttributesFollow Ask | Always | Never` (default Ask).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubAttributesFollow {
    #[default]
    Ask,
    Always,
    Never,
}

/// When the command line's suggestion list opens (decisions §30 "12b2").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionsMode {
    /// After a short pause while typing (and on Tab).
    #[default]
    Auto,
    /// Only on Tab.
    TabOnly,
    /// Never: Tab completes in place.
    Off,
}

impl SuggestionsMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::TabOnly => "TabOnly",
            Self::Off => "Off",
        }
    }

    /// The mode by its label (any case; `Tab` for TabOnly).
    pub fn parse(word: &str) -> Option<Self> {
        let w = word.to_ascii_lowercase();
        [Self::Auto, Self::TabOnly, Self::Off]
            .into_iter()
            .find(|m| m.label().eq_ignore_ascii_case(&w))
            .or_else(|| (w == "tab").then_some(Self::TabOnly))
    }
}

/// Which steps a profile files and when Oops asks first (the User Settings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OopsSettings {
    /// Off: no steps at all (the other three don't matter then).
    #[serde(default = "default_true")]
    pub create_oops: bool,
    /// UI steps (windows, views, screens, watches).
    #[serde(default = "default_true")]
    pub oops_views: bool,
    #[serde(default = "default_true")]
    pub oops_programmer: bool,
    #[serde(default = "default_true")]
    pub oops_selection: bool,
    #[serde(default)]
    pub confirmation: OopsConfirmation,
}

impl Default for OopsSettings {
    fn default() -> Self {
        Self {
            create_oops: true,
            oops_views: true,
            oops_programmer: true,
            oops_selection: true,
            confirmation: OopsConfirmation::Never,
        }
    }
}

/// When Oops asks before undoing (Oops Confirmation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OopsConfirmation {
    /// Every Oops asks.
    Always,
    /// Only Main actions (create, delete, move, insert, file, layout) ask.
    Main,
    #[default]
    Never,
}

impl OopsConfirmation {
    pub fn label(self) -> &'static str {
        match self {
            OopsConfirmation::Always => "Always",
            OopsConfirmation::Main => "Main",
            OopsConfirmation::Never => "Never",
        }
    }

    pub fn parse(word: &str) -> Option<Self> {
        [Self::Always, Self::Main, Self::Never]
            .into_iter()
            .find(|c| c.label().eq_ignore_ascii_case(word))
    }
}

fn default_grid_cursor_move() -> GridCursorMove {
    GridCursorMove::AppendX
}

fn default_pool_drag() -> String {
    "Move".into()
}

fn default_notification_mode() -> String {
    "timed".into()
}

fn default_double_click_ms() -> u32 {
    crate::ui_keys::DEFAULT_DOUBLE_CLICK_MS
}

fn default_meow_key() -> String {
    "Shift".to_string()
}

fn default_hold_ms() -> u32 {
    crate::ui_keys::DEFAULT_HOLD_MS
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_ui_engine: "Shell".to_string(),
            double_click_ms: default_double_click_ms(),
            hold_ms: default_hold_ms(),
            meow_key: default_meow_key(),
            grid_cursor_move: default_grid_cursor_move(),
            pool_drag: default_pool_drag(),
            oops: OopsSettings::default(),
            suggestions: SuggestionsMode::default(),
            hold_follows: true,
            popup_dim: true,
            nav_log: true,
            sub_attributes_follow: SubAttributesFollow::Ask,
            notifications: BTreeMap::new(),
            notification_mode: default_notification_mode(),
            mvr: MvrImportPreset::default(),
            update: UpdateChoices::default(),
            paint_mode: default_paint_mode(),
            paint_recent: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    #[serde(default)]
    pub can_edit_show: bool,
    #[serde(default)]
    pub can_edit_patch: bool,
    #[serde(default)]
    pub can_trigger_playback: bool,
    #[serde(default)]
    pub can_configure_engines: bool,
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            can_edit_show: true,
            can_edit_patch: true,
            can_trigger_playback: true,
            can_configure_engines: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {

    pub meta: NodeMeta,
    /// The template profile "Default" (decisions §30 "12c2"): every
    /// show has one, new profiles start as a copy of it, and it
    /// can't be edited, renamed or deleted, nor held by a user. Rights live
    /// on the User only (a profile is views, screens, keys and settings).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
    /// How the profile works and looks (decisions §30 "Users & Profiles
    /// follows storage"): `ShowData.UserProfiles.<n>.Settings`.
    #[serde(default)]
    pub settings: ProfileSettings,
    /// What the profile holds: attributes and encoder bars, keys, views and
    /// screens, window presets, images, icons (`….Content`).
    #[serde(default)]
    pub content: ProfileContent,
}

/// A profile's settings (`ShowData.UserProfiles.<n>.Settings`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    /// The active programming language/dialect (e.g., "Lum").
    pub prog_lang: String,
    /// Alias layer depth: "Base", "ProgLang", or "Full".
    pub alias_depth: String,
    /// User preferences for UI and command handling.
    #[serde(default)]
    pub preferences: UserPreferences,
    /// Permission capability snapshot stored with the profile.
    #[serde(default)]
    pub permissions: UserPermissions,
    /// Whether to show descriptions in help and command output.
    /// Default: true for Programmer/Guest, false for Admin.
    #[serde(default = "default_show_descriptions")]
    pub show_descriptions: bool,
    /// Whether to clear the terminal buffer before each command output.
    /// When enabled, every command result is shown on a fresh screen.
    #[serde(default)]
    pub clear_buffer: bool,
    /// Default output verbosity for command responses.
    ///
    /// Override per-command with `/v` (Verbose) or `/d` (Debug) flags.
    /// Change default with `set verbosity summary|verbose|debug`.
    #[serde(default)]
    pub verbosity: Verbosity,
    /// The profile's look (decisions §30 "12a2"): it travels with the
    /// profile, so an imported profile looks the same in another show.
    #[serde(default)]
    pub theme: crate::media::Theme,
    #[serde(default)]
    pub lock_screen: crate::media::LockScreen,
    /// The profile's menu and grid state (decisions §30 "12d"): the last tab
    /// of each menu and each grid's columns. Set by command lines (`Menu …
    /// /Tab x`, `Set GridColumns …`), never an Oops step.
    #[serde(default, skip_serializing_if = "ProfileUi::is_empty")]
    pub ui: ProfileUi,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            prog_lang: "Lum".to_string(),
            alias_depth: "Base".to_string(),
            preferences: UserPreferences::default(),
            permissions: UserPermissions::default(),
            show_descriptions: true,
            clear_buffer: false,
            verbosity: Verbosity::default(),
            theme: Default::default(),
            lock_screen: Default::default(),
            ui: ProfileUi::default(),
        }
    }
}

/// What a profile holds (`ShowData.UserProfiles.<n>.Content`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileContent {
    /// The profile's own attribute layer (decisions §30 "12f-a"): groups,
    /// rules, every attribute it placed, encoder bar layouts.
    #[serde(default)]
    pub attributes: lumacore_fixtures::attribute_def::ProfileAttributes,
    /// User-custom key bindings. Override vanilla defaults from Root.
    /// Keyed by normalized key name (e.g., "Ctrl-R", "F5").
    #[serde(default)]
    pub keybindings: KeyBindingMap,
    /// The profile's own UI keys (`user.*`): everyone on the profile sees,
    /// uses and edits them. Saved with the show.
    #[serde(default)]
    pub ui_keys: BTreeMap<u32, crate::ui_keys::UiKey>,
    /// Stored views (`Store View n`), recalled onto a screen with `View n`.
    #[serde(default, deserialize_with = "de_views")]
    pub views: BTreeMap<u32, StoredView>,
    /// View bar buttons per screen (decisions 0287): the profile's screen number → slot → the
    /// object the button holds (`ViewButton <screen>.<slot>`).
    #[serde(default)]
    pub view_buttons: BTreeMap<u32, BTreeMap<u32, ViewButton>>,
    /// Screen configurations: the live screens of this workspace, kept in sync
    /// with the screens and saved with the show.
    #[serde(default)]
    pub screen_configs: BTreeMap<u32, ScreenConfig>,
    /// Window presets (decisions §30 "12c"): type → number → preset. They
    /// travel with the profile (`Export UserProfile`).
    #[serde(default)]
    pub window_presets: WindowPresets,
    #[serde(default)]
    pub images: BTreeMap<u32, crate::media::MediaFile>,
    /// The profile's own icons (icon sets later) and pictures (the lock
    /// screen background), for its own UI only.
    #[serde(default)]
    pub icons: BTreeMap<u32, crate::media::MediaFile>,
    /// Variables that travel with this user profile.
    #[serde(default)]
    pub user_variables: VariableStore,
}

/// A profile's menu and grid state (decisions §30 "12d").
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileUi {
    /// Menu name → the tab it last showed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub menu_tabs: BTreeMap<String, String>,
    /// Grid key (`dmx.lines`) → its columns as this profile set them.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub grids: BTreeMap<String, GridState>,
}

impl ProfileUi {
    pub fn is_empty(&self) -> bool {
        self.menu_tabs.is_empty() && self.grids.is_empty()
    }
}

/// One grid's columns on a profile (`Set GridColumns "dmx.lines" …`).
/// Empty / None fields are the factory's (the registry's columns).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridState {
    /// Visible column ids in order; empty = the factory's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    /// Column id → width in px.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub widths: BTreeMap<String, u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<GridSort>,
    /// `Condensed`, `Full` or `Extended`; None = Full.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set: Option<String>,
    /// The new-object line; None = shown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_line: Option<bool>,
    /// Spare DMX slots after each fixture when the grid's DMX actions pack a selection (the
    /// patch's Offset cell, decisions §34 round 3); None = packed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl GridState {
    pub fn is_factory(&self) -> bool {
        *self == Self::default()
    }
}

/// A grid's sort: the column and `asc` / `desc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridSort {
    pub col: String,
    pub dir: String,
}

fn default_show_descriptions() -> bool {
    true
}


impl UserProfile {
    /// Drop windows of a type this build doesn't know (R1-57, Q32), on a
    /// show load and a profile import alike; one note per window.
    pub fn drop_unknown_windows(&mut self) -> Vec<String> {
        let types = crate::window_types::builtin_window_types();
        let known = |kind: &str| types.values().any(|t| t.name.eq_ignore_ascii_case(kind));
        let mut notes = Vec::new();
        let who = self.meta.identity.name.clone().unwrap_or_default();
        for (no, config) in self.content.screen_configs.iter_mut() {
            for (screen, content) in &mut config.screens {
                for (n, kind) in content.drop_unknown_windows(&known) {
                    notes.push(format!("UserProfile \"{who}\" ScreenConfig {no} screen {screen}: window {n} of unknown type '{kind}' dropped"));
                }
            }
        }
        for (no, view) in self.content.views.iter_mut() {
            for (screen, content) in &mut view.screens {
                for (n, kind) in content.drop_unknown_windows(&known) {
                    notes.push(format!("UserProfile \"{who}\" View {no} screen {screen}: window {n} of unknown type '{kind}' dropped"));
                }
            }
        }
        notes
    }

    /// Bind `key` to a command line (decisions §17): the line goes into a
    /// user UiKey `user.<key>` (made or updated here) and the binding names
    /// that key. Returns the UiKey id.
    pub fn bind_command(&mut self, key: &str, command: &str, description: &str) -> String {
        let key = crate::keybindings::normalize_key(key);
        let id = format!("user.{}", key.to_lowercase());
        match self
            .content.ui_keys
            .values_mut()
            .find(|k| k.id.eq_ignore_ascii_case(&id))
        {
            Some(existing) => existing.press = command.to_string(),
            None => {
                let no = self.content.ui_keys.keys().next_back().map_or(1, |n| n + 1);
                let label = if description.is_empty() {
                    &key
                } else {
                    description
                };
                let mut ui_key = crate::ui_keys::UiKey::user(&id, label, command);
                // A user key belongs to this profile: two profiles binding the
                // same key never share a GUID (decisions §29, finding 3).
                ui_key.meta.identity.guid = lumacore_base::Guid::named(
                    "UiKey",
                    &format!("{}:{id}", self.meta.identity.guid),
                )
                .0;
                self.content.ui_keys.insert(no, ui_key);
            }
        }
        self.bind_ui_key(&key, &id, description);
        id
    }

    /// Bind `key` to an existing UiKey id.
    pub fn bind_ui_key(&mut self, key: &str, ui_key: &str, description: &str) {
        let key = crate::keybindings::normalize_key(key);
        self.content.keybindings.insert(
            key.clone(),
            crate::keybindings::KeyBinding {
                key,
                ui_key: ui_key.to_string(),
                description: description.to_string(),
            },
        );
    }

    pub fn new(name: Option<String>) -> Self {
        Self {
            meta: NodeMeta::new(name),
            locked: false,
            settings: ProfileSettings::default(),
            content: ProfileContent::default(),
        }
    }

    /// The profile Admins start with (hints without descriptions).
    pub fn new_admin(name: Option<String>) -> Self {
        let mut profile = Self::new(name);
        profile.settings.show_descriptions = false;
        profile
    }

    /// The locked template profile "Default" (decisions §30 "12c2").
    pub fn new_default() -> Self {
        let mut profile = Self::new(Some(DEFAULT_PROFILE.to_string()));
        profile.locked = true;
        profile
    }

    /// A copy with new GUIDs for itself and everything in it (strict
    /// identity, decisions §21 "6d"); links inside the profile (view
    /// buttons, the lock screen picture, window presets) follow the copies.
    pub fn copy_renewed(&self, name: Option<String>) -> Self {
        let mut w = lumacore_base::Walker::new();
        w.object("Profile", self);
        let mut json = serde_json::to_string(self).expect("a profile serializes");
        for o in &w.objects {
            if o.guid.is_nil() {
                continue;
            }
            json = json.replace(&o.guid.to_string(), &Uuid::new_v4().to_string());
        }
        let mut copy: Self = serde_json::from_str(&json).expect("the copy deserializes");
        copy.meta.identity.name = name;
        copy.locked = false;
        copy
    }
}

impl TreeNode for UserProfile {
    fn guid(&self) -> Uuid {
        self.meta.guid()
    }
    fn name(&self) -> Option<&str> {
        self.meta.name()
    }
    fn node_type(&self) -> &'static str {
        "UserProfile"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: generate a deterministic test UUID from a simple integer seed.
    fn test_guid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    // ── percent_to_dmx tests ──────────────────────────────────────────────

    #[test]
    fn percent_to_dmx_zero() {
        assert_eq!(percent_to_dmx(0.0), 0);
    }

    #[test]
    fn percent_to_dmx_fifty() {
        // 50% of 255 = 127.5, rounds to 128
        let v = percent_to_dmx(50.0);
        assert!(v == 127 || v == 128, "expected 127 or 128, got {v}");
    }

    #[test]
    fn percent_to_dmx_full() {
        assert_eq!(percent_to_dmx(100.0), 255);
    }

    #[test]
    fn percent_to_dmx_clamp_low() {
        assert_eq!(percent_to_dmx(-5.0), 0);
    }

    #[test]
    fn percent_to_dmx_clamp_high() {
        assert_eq!(percent_to_dmx(200.0), 255);
    }

    // ── ClearState / clear_layer tests (console-style order) ────────────────

    #[test]
    fn clear_layer_fresh_with_selection_clears_selection_first() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.select(g1);
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        let msg = p.clear_layer();
        assert_eq!(msg, "Selection cleared");
        assert_eq!(p.clear_state, ClearState::SelectionCleared);
        assert!(p.selected_fixtures.is_empty());
        // Values should still be there and active
        assert!(p.has_active_values());
    }

    #[test]
    fn clear_layer_selection_cleared_deactivates_values() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.select(g1);
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        p.clear_layer(); // Fresh -> SelectionCleared
        let msg = p.clear_layer(); // SelectionCleared -> ActiveCleared
        assert_eq!(msg, "Values deactivated");
        assert_eq!(p.clear_state, ClearState::ActiveCleared);
        // Values still exist but are inactive
        assert!(!p.values.is_empty());
        assert!(!p.has_active_values());
        assert_eq!(
            p.get_value(g1, "Dimmer").and_then(Value::percent),
            Some(78.4)
        ); // still readable
        assert_eq!(p.get_active_value(g1, "Dimmer"), None); // not active
    }

    #[test]
    fn clear_layer_active_cleared_empties_programmer() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.select(g1);
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        p.clear_layer(); // Fresh -> SelectionCleared
        p.clear_layer(); // SelectionCleared -> ActiveCleared
        let msg = p.clear_layer(); // ActiveCleared -> Fresh (empty)
        assert_eq!(msg, "Programmer cleared");
        assert_eq!(p.clear_state, ClearState::Fresh);
        assert!(p.values.is_empty());
        assert!(p.selected_fixtures.is_empty());
    }

    #[test]
    fn clear_layer_fresh_no_selection_skips_to_deactivate() {
        let g1 = test_guid(1);
        // No selection but active values: skip selection step
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        let msg = p.clear_layer();
        assert_eq!(msg, "Values deactivated");
        assert_eq!(p.clear_state, ClearState::ActiveCleared);
    }

    #[test]
    fn clear_layer_selection_cleared_no_active_values_empties() {
        let g1 = test_guid(1);
        // SelectionCleared but no active values: skip to empty
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        p.deactivate_values(); // make them inactive
        p.clear_state = ClearState::SelectionCleared;
        let msg = p.clear_layer();
        assert_eq!(msg, "Programmer cleared");
        assert_eq!(p.clear_state, ClearState::Fresh);
        assert!(p.values.is_empty());
    }

    #[test]
    fn clear_layer_fresh_only_inactive_values_clears_them() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        p.deactivate_values();
        let msg = p.clear_layer();
        assert_eq!(msg, "Programmer cleared");
        assert!(p.values.is_empty());
    }

    #[test]
    fn full_clear_resets_everything() {
        let g1 = test_guid(1);
        let g2 = test_guid(2);
        let mut p = Programmer::new();
        p.select(g1);
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        p.set_value(g2, "Pan", Value::normalized(39.2));
        p.clear_layer();
        assert_ne!(p.clear_state, ClearState::Fresh);
        p.full_clear();
        assert_eq!(p.clear_state, ClearState::Fresh);
        assert!(p.values.is_empty());
        assert!(p.selected_fixtures.is_empty());
    }

    #[test]
    fn deactivate_and_reactivate_values() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        assert!(p.has_active_values());
        assert_eq!(
            p.get_active_value(g1, "Dimmer").and_then(Value::percent),
            Some(78.4)
        );

        p.deactivate_values();
        assert!(!p.has_active_values());
        assert_eq!(p.get_active_value(g1, "Dimmer"), None);
        assert_eq!(
            p.get_value(g1, "Dimmer").and_then(Value::percent),
            Some(78.4)
        ); // still there

        p.activate_values();
        assert!(p.has_active_values());
        assert_eq!(
            p.get_active_value(g1, "Dimmer").and_then(Value::percent),
            Some(78.4)
        );
    }

    #[test]
    fn set_value_always_creates_active() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(39.2));
        p.deactivate_values();
        // Setting again should create new active value
        p.set_value(g1, "Dimmer", Value::normalized(78.4));
        assert!(p.is_value_active(g1, "Dimmer"));
        assert_eq!(
            p.get_active_value(g1, "Dimmer").and_then(Value::percent),
            Some(78.4)
        );
    }

    // ── Existing tests ────────────────────────────────────────────────────

    #[test]
    fn programmer_select_deselect() {
        let g1 = test_guid(1);
        let g2 = test_guid(2);
        let mut p = Programmer::new();
        p.select(g1);
        p.select(g2);
        p.select(g1); // duplicate — no-op
        assert_eq!(p.selected_fixtures, vec![g1, g2]);
        p.deselect(g1);
        assert_eq!(p.selected_fixtures, vec![g2]);
    }

    #[test]
    fn programmer_clear_selection() {
        let g1 = test_guid(1);
        let g2 = test_guid(2);
        let mut p = Programmer::new();
        p.select(g1);
        p.select(g2);
        p.clear_selection();
        assert!(p.selected_fixtures.is_empty());
    }

    #[test]
    fn programmer_is_selected() {
        let g5 = test_guid(5);
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.select(g5);
        assert!(p.is_selected(g5));
        assert!(!p.is_selected(g1));
    }

    #[test]
    fn programmer_values() {
        let g1 = test_guid(1);
        let mut p = Programmer::new();
        p.set_value(g1, "Dimmer", Value::normalized(100.0));
        assert_eq!(
            p.get_value(g1, "Dimmer").and_then(Value::percent),
            Some(100.0)
        );
        assert_eq!(
            p.get_active_value(g1, "Dimmer").and_then(Value::percent),
            Some(100.0)
        );
        assert_eq!(p.get_value(g1, "Pan"), None);
        p.clear_values();
        assert_eq!(p.get_value(g1, "Dimmer"), None);
    }

    #[test]
    fn user_profile_defaults() {
        let profile = UserProfile::new(Some("Admin".to_string()));
        assert_eq!(profile.settings.prog_lang, "Lum");
        assert_eq!(profile.settings.alias_depth, "Base");
        assert_eq!(profile.name(), Some("Admin"));
        assert_eq!(profile.node_type(), "UserProfile");
    }
}
