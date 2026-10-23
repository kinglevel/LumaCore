//! The object tree's one table (decisions §31 "Tree paths are typed"): every node a path can
//! name, declared once with its parent. Children are derived from the parent links, so the
//! tree can't disagree with itself; navigation, the VFS, `cd` / `ls`, List and completion read
//! it instead of comparing `"root.showdata…"` strings.
//!
//! A path ([`TreePath`]) is a list of steps: a declared node, a numbered entry (an object in a
//! collection, a datapool, a logged-in user), a named entry (a protocol, by the name its row
//! declares: `Protocols.DMXProtocols.Sacn`), or a name only the disk knows (`Root.Drives`).

use std::fmt;

/// What is below a node besides its declared children.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Only its declared children (or nothing yet).
    Static,
    /// Objects publish its entries by number (`ObjectDef.tree`, a `TreeDef` naming this node).
    Collection,
    /// Rows the station knows now (running sequences, lines, claims, …), by number; each a
    /// leaf. The VFS lists them (`vfs::dynamic_children_at`).
    Live,
    /// Entries by number, each shaped like the schema node (`DataPools.<n>` is a `DataPool`).
    Numbered(Node),
    /// The shape of one numbered entry; never listed as a child itself.
    Schema,
    /// Entries by name, each a collection shaped like the given node (`DMXProtocols.Sacn` holds
    /// lines like `Lines`): the names are rows of the catalog (decisions §34 "34f7c hardened"
    /// 5), so this table never names one. The shape node is never listed as a child itself.
    Named(Node),
    /// Files and folders on this station's disk (only `Root.Drives` reaches them, §25).
    Disk,
}

pub struct NodeDef {
    /// The name in a path (`Root.ShowData.Devices`).
    pub name: &'static str,
    pub parent: Option<Node>,
    pub kind: Kind,
}

macro_rules! nodes {
    ($($node:ident = $name:literal $(under $parent:ident)?, $kind:expr;)*) => {
        /// Every node of the object tree (one per declared path; numbered entries and disk
        /// names are steps of a [`TreePath`], not nodes).
        #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
        pub enum Node { $($node,)* }

        impl Node {
            /// Every node, in the order children are listed.
            pub const ALL: &'static [Node] = &[$(Node::$node,)*];

            pub const fn def(self) -> &'static NodeDef {
                match self {
                    $(Node::$node => &NodeDef {
                        name: $name,
                        parent: nodes!(@parent $($parent)?),
                        kind: $kind,
                    },)*
                }
            }
        }
    };
    (@parent) => { None };
    (@parent $parent:ident) => { Some(Node::$parent) };
}

use Kind::*;

nodes! {
    Root = "Root", Static;

    ShowData = "ShowData" under Root, Static;
    Runtime = "Runtime" under Root, Static;
    Resources = "Resources" under Root, Static;
    Diagnostics = "Diagnostics" under Root, Static;
    StationSettings = "StationSettings" under Root, Static;
    Library = "Library" under Root, Static;
    Drives = "Drives" under Root, Disk;
    UIEngines = "UIEngines" under Root, Collection;

    Devices = "Devices" under ShowData, Static;
    Spatial3D = "3DSpatial" under ShowData, Static;
    Playback = "Playback" under ShowData, Static;
    Playbacks = "Playbacks" under ShowData, Static;
    DataPools = "DataPools" under ShowData, Numbered(Node::DataPool);
    Protocols = "Protocols" under ShowData, Static;
    EffectEngines = "EffectEngines" under ShowData, Static;
    Variables = "Variables" under ShowData, Static;
    ShowSettings = "ShowSettings" under ShowData, Static;
    Users = "Users" under ShowData, Collection;
    UserProfiles = "UserProfiles" under ShowData, Collection;
    Images = "Images" under ShowData, Collection;
    Assets = "Assets" under ShowData, Static;
    // The tag tree (decisions §12 "Tags hardened"): its top tags; a tag's ids read as a path.
    Tags = "Tags" under ShowData, Collection;

    // A node per device kind, and what they share (decisions §34 "34g4 patch hardened" 2, 7).
    Dmx = "Dmx" under Devices, Static;
    DevicesMetadata = "Metadata" under Devices, Static;
    Patch = "Patch" under Dmx, Static;
    Fixtures = "Fixtures" under Patch, Collection;
    FixtureTypes = "FixtureTypes" under Patch, Collection;
    Universes = "Universes" under Patch, Collection;
    // MVR's metadata by UUID; the layers, groups, classes and positions are pools of their words
    // (MvrLayer, MvrGroup, MvrClass, MvrPosition — 34g4 part 3c).
    MetadataMvr = "MVR" under DevicesMetadata, Static;
    MvrLayers = "Layers" under MetadataMvr, Collection;
    MvrGroups = "Groups" under MetadataMvr, Collection;
    MvrClasses = "Classes" under MetadataMvr, Collection;
    MvrPositions = "Positions" under MetadataMvr, Collection;
    MvrMappingDefinitions = "MappingDefinitions" under MetadataMvr, Static;
    MvrFiles = "Files" under MetadataMvr, Static;

    Park = "Park" under Playback, Live;
    Executors = "Executors" under Playback, Collection;
    Effects = "Effects" under Playback, Collection;

    // The registry of what runs (decisions §32): Live is saved, Echoes never (12f part 2).
    PlaybacksLive = "Live" under Playbacks, Static;
    RunningSequences = "Sequences" under PlaybacksLive, Live;
    LiveMasters = "Masters" under PlaybacksLive, Live;

    DatapoolPriorityOrders = "DatapoolPriorityOrders" under DataPools, Static;
    DataPool = "DataPool" under DataPools, Schema;
    Presets = "Presets" under DataPool, Collection;
    Layouts = "Layouts" under DataPool, Static;
    Filters = "Filters" under DataPool, Static;
    Worlds = "Worlds" under DataPool, Static;
    Macros = "Macros" under DataPool, Static;
    DistributePresets = "DistributePresets" under DataPool, Static;
    DynamicStates = "DynamicStates" under DataPool, Static;
    Plugins = "Plugins" under DataPool, Static;
    Timecodes = "Timecodes" under DataPool, Static;
    Priorities = "Priorities" under DataPool, Static;
    Groups = "Groups" under DataPool, Collection;
    // Cue lists and their presets are DataPool objects (decisions §32, §30 "12f").
    Sequences = "Sequences" under DataPool, Collection;
    TimingPresets = "TimingPresets" under DataPool, Collection;
    CurvePresets = "CurvePresets" under DataPool, Collection;
    Tracks = "Tracks" under Timecodes, Static;
    MarkerTrack = "MarkerTrack" under Timecodes, Static;
    TempoTrack = "TempoTrack" under Timecodes, Static;
    SignatureTrack = "SignatureTrack" under Timecodes, Static;
    LoopRegion = "LoopRegion" under Timecodes, Static;
    Duration = "Duration" under Timecodes, Static;
    TimeBase = "TimeBase" under Timecodes, Static;

    Bidirectional = "Bidirectional" under Protocols, Static;
    Networks = "Networks" under Protocols, Collection;
    DmxProtocols = "DMXProtocols" under Protocols, Named(Node::DmxLines);
    DmxLines = "Lines" under DmxProtocols, Collection;
    DataProtocols = "DataProtocols" under Protocols, Named(Node::DataLines);
    DataLines = "Lines" under DataProtocols, Collection;

    EffectEngineGraph = "Graph" under EffectEngines, Static;
    EffectEngineGenerators = "Generators" under EffectEngines, Static;
    EffectEngineInputs = "Inputs" under EffectEngines, Static;
    EffectEngineOutputs = "Outputs" under EffectEngines, Static;
    EffectEngineParameters = "Parameters" under EffectEngines, Static;

    // A profile is two halves (decisions §30 "Users & Profiles follows storage"): below
    // `UserProfiles.<n>` the schema's children; each part names a field of the half.
    UserProfile = "UserProfile" under UserProfiles, Schema;
    ProfileSettings = "Settings" under UserProfile, Static;
    ProfileContent = "Content" under UserProfile, Static;
    ProfileProgLang = "ProgLang" under ProfileSettings, Static;
    ProfileAliasDepth = "AliasDepth" under ProfileSettings, Static;
    ProfilePreferences = "Preferences" under ProfileSettings, Static;
    ProfilePermissions = "Permissions" under ProfileSettings, Static;
    ProfileShowDescriptions = "ShowDescriptions" under ProfileSettings, Static;
    ProfileClearBuffer = "ClearBuffer" under ProfileSettings, Static;
    ProfileVerbosity = "Verbosity" under ProfileSettings, Static;
    ProfileTheme = "Theme" under ProfileSettings, Static;
    ProfileLockScreen = "LockScreen" under ProfileSettings, Static;
    ProfileUi = "UI" under ProfileSettings, Static;
    ProfileAttributes = "Attributes" under ProfileContent, Static;
    ProfileKeyBindings = "KeyBindings" under ProfileContent, Static;
    ProfileUiKeys = "UIKeys" under ProfileContent, Static;
    ProfileViews = "Views" under ProfileContent, Static;
    ProfileViewButtons = "ViewButtons" under ProfileContent, Static;
    ProfileScreenConfigs = "ScreenConfigs" under ProfileContent, Static;
    ProfileWindowPresets = "WindowPresets" under ProfileContent, Static;
    ProfileImages = "Images" under ProfileContent, Static;
    ProfileIcons = "Icons" under ProfileContent, Static;
    ProfileUserVariables = "UserVariables" under ProfileContent, Static;

    AssetRoots = "Roots" under Assets, Collection;
    AssetFiles = "Files" under Assets, Collection;

    Network = "Network" under Runtime, Static;
    LumaSessions = "LumaSessions" under Runtime, Static;
    NetworkEngines = "NetworkEngines" under Runtime, Static;
    LoggedIn = "Users" under Runtime, Numbered(Node::LoggedInUser);
    SessionEngines = "SessionEngines" under Runtime, Static;
    RenderingEngines = "RenderingEngines" under Runtime, Static;
    StationDevices = "Devices" under Runtime, Live;
    GlobalVariables = "GlobalVariables" under Runtime, Static;
    Journal = "Journal" under Runtime, Static;
    Health = "Health" under Runtime, Static;
    Claims = "Claims" under Runtime, Live;

    Interfaces = "Interfaces" under Network, Live;
    Lines = "Lines" under Network, Live;
    Nodes = "Nodes" under Network, Live;
    Sources = "Sources" under Network, Live;

    SessionStations = "Stations" under LumaSessions, Live;
    SessionMembers = "Members" under LumaSessions, Live;
    SessionListeners = "Listeners" under LumaSessions, Live;
    SessionConnections = "Connections" under LumaSessions, Live;

    LoggedInUser = "User" under LoggedIn, Schema;
    UserModes = "Modes" under LoggedInUser, Static;
    UserUndo = "Undo" under LoggedInUser, Static;
    UserScreens = "Screens" under LoggedInUser, Numbered(Node::UserScreen);
    UserScreen = "Screen" under UserScreens, Schema;
    ScreenWindows = "Windows" under UserScreen, Numbered(Node::ScreenWindow);
    ScreenWindow = "Window" under ScreenWindows, Schema;

    WindowTypes = "WindowTypes" under Resources, Collection;
    UIKeys = "UIKeys" under Resources, Collection;

    LibraryGdtf = "GDTF" under Library, Static;
    LibraryShows = "shows" under Library, Static;
    LibraryPresets = "presets" under Library, Static;
    LibraryMvr = "mvr" under Library, Static;
}

/// A navigation position as a dot path: `Root`, `Root.ShowData.Devices`.
pub fn dot(segments: &[String]) -> String {
    if segments.is_empty() {
        "Root".to_string()
    } else {
        format!("Root.{}", segments.join("."))
    }
}

/// What follows `Root.` in an absolute dot path (any case), if it is one.
pub fn strip_root(text: &str) -> Option<&str> {
    let (first, rest) = text.split_once('.')?;
    first
        .eq_ignore_ascii_case(Node::Root.name())
        .then_some(rest)
}

/// A dot path as the shell shows it: `Root.ShowData.Devices` → `/ShowData/Devices`.
pub fn slashed(dot: &str) -> String {
    let mut names = dot.split('.').filter(|s| !s.is_empty()).peekable();
    if names.peek().is_some_and(|s| s.eq_ignore_ascii_case("root")) {
        names.next();
    }
    format!("/{}", names.collect::<Vec<_>>().join("/"))
}

/// Whether a dot path is on this station's disk (`Root.Drives…`, any case) — the one syntactic
/// check, so reach (decisions §25) needs no table walk.
pub fn names_drive(dot: &str) -> bool {
    let mut names = dot.split('.');
    names.next().is_some_and(|s| s.eq_ignore_ascii_case("root"))
        && names
            .next()
            .is_some_and(|s| s.eq_ignore_ascii_case(Node::Drives.name()))
}

/// Names that lead to a node from anywhere (`cd patch`, `ls claims`; cd, enter and ls alike —
/// List takes exact paths only).
pub const ALIASES: &[(&str, Node)] = &[
    ("patch", Node::Patch),
    ("showdata.patch", Node::Patch),
    ("root.showdata.patch", Node::Patch),
    ("claims", Node::Claims),
];

/// The node an alias names (any case).
pub fn alias(text: &str) -> Option<Node> {
    ALIASES
        .iter()
        .find(|(a, _)| a.eq_ignore_ascii_case(text))
        .map(|&(_, n)| n)
}

impl Node {
    pub fn name(self) -> &'static str {
        self.def().name
    }

    pub fn parent(self) -> Option<Node> {
        self.def().parent
    }

    pub fn kind(self) -> Kind {
        self.def().kind
    }

    /// Its declared children, in table order (a schema node is the shape of an entry, not a
    /// child; nor is a named node's shape).
    pub fn children(self) -> impl Iterator<Item = Node> {
        Node::ALL
            .iter()
            .copied()
            .filter(move |n| n.parent() == Some(self) && n.kind() != Schema && !n.is_named_shape())
    }

    /// Its number under its parent (decisions §33 54): its place among the parent's declared
    /// children, from 1, in the table's order — no number is written anywhere else.
    pub fn no(self) -> Option<u32> {
        let parent = self.parent()?;
        parent.children().position(|c| c == self).map(|i| i as u32 + 1)
    }

    /// The declared child with number `no` (decisions §33 54: a number in a path is the child's No).
    pub fn nth_child(self, no: u32) -> Option<Node> {
        self.children().nth((no as usize).checked_sub(1)?)
    }

    /// The declared child named `name` (any case); a numbered node's schema answers to its own
    /// name too (`DataPools.DataPool`, the reference's spelling).
    pub fn child(self, name: &str) -> Option<Node> {
        Node::ALL
            .iter()
            .copied()
            .find(|n| n.parent() == Some(self) && !n.is_named_shape() && n.name().eq_ignore_ascii_case(name))
    }

    /// The shape of a named node's entries (`DmxLines` under `DMXProtocols`).
    pub fn is_named_shape(self) -> bool {
        self.parent().is_some_and(|p| p.kind() == Named(self))
    }

    /// Its schema path (`Root.ShowData.DataPools.DataPool.Groups`).
    pub fn path(self) -> String {
        let mut names = vec![self.name()];
        let mut at = self;
        while let Some(p) = at.parent() {
            names.push(p.name());
            at = p;
        }
        names.reverse();
        names.join(".")
    }

    /// `ancestor` itself or below it.
    pub fn is_under(self, ancestor: Node) -> bool {
        std::iter::successors(Some(self), |n| n.parent()).any(|n| n == ancestor)
    }

    /// The schema of a collection's entries, when they have declared children
    /// (`UserProfiles.<n>` is a `UserProfile`).
    pub fn entry_schema(self) -> Option<Node> {
        (self.kind() == Collection)
            .then(|| Node::ALL.iter().copied().find(|n| n.parent() == Some(self) && n.kind() == Schema))
            .flatten()
    }

    /// The numbered node this schema shapes the entries of.
    fn numbered_by(self) -> Option<Node> {
        (self.kind() == Schema).then(|| self.parent()).flatten()
    }
}

/// One step of a path below `Root`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Step {
    Node(Node),
    /// Entry `n` of the collection or numbered node before it.
    Entry(u32),
    /// Entry `name` of the named node before it (a protocol: `DMXProtocols.Sacn`).
    Name(String),
    /// A name only the disk knows (below `Root.Drives`).
    Disk(String),
}

/// A path in the object tree, `Root` first (implicit).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TreePath {
    steps: Vec<Step>,
}

impl TreePath {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// The node the path stands on: a numbered entry is its schema, an object in a collection
    /// stands on the collection (see [`TreePath::entry`]).
    pub fn node(&self) -> Node {
        let mut node = Node::Root;
        for step in &self.steps {
            match step {
                Step::Node(n) => node = *n,
                Step::Entry(_) => {
                    if let Numbered(schema) = node.kind() {
                        node = schema;
                    }
                }
                Step::Name(_) => {
                    if let Named(shape) = node.kind() {
                        node = shape;
                    }
                }
                Step::Disk(_) => {}
            }
        }
        node
    }

    /// The named entry the path is in (`Root.ShowData.Protocols.DMXProtocols.Sacn.…` → `Sacn`).
    pub fn named(&self) -> Option<&str> {
        self.steps.iter().find_map(|s| match s {
            Step::Name(n) => Some(n.as_str()),
            _ => None,
        })
    }

    /// The object (or live row) number when the path ends on one.
    pub fn entry(&self) -> Option<u32> {
        match self.steps.last() {
            Some(Step::Entry(n)) if matches!(self.node().kind(), Collection | Live) => Some(*n),
            _ => None,
        }
    }

    /// The datapool the path is in (`Root.ShowData.DataPools.<n>.…`).
    pub fn datapool(&self) -> Option<u32> {
        self.numbered(Node::DataPools)
    }

    /// The entry number taken below the numbered node `of`.
    pub fn numbered(&self, of: Node) -> Option<u32> {
        let at = self.steps.iter().position(|s| *s == Step::Node(of))?;
        match self.steps.get(at + 1) {
            Some(Step::Entry(n)) => Some(*n),
            _ => None,
        }
    }

    /// Parse an absolute dot path (`Root.ShowData.DataPools.2.Groups`, any case, `Root.`
    /// optional): every name must be declared, a number must follow a collection or a
    /// numbered node, only `Root.Drives` takes names the table doesn't know.
    pub fn parse_dot(text: &str) -> Option<TreePath> {
        let mut segments = text.split('.').filter(|s| !s.is_empty()).peekable();
        if segments
            .peek()
            .is_some_and(|s| s.eq_ignore_ascii_case("root"))
        {
            segments.next();
        }
        let mut path = TreePath::root();
        for seg in segments {
            path = path.join(seg)?;
        }
        Some(path)
    }

    /// One step further down by name or number, exactly.
    pub fn join(&self, seg: &str) -> Option<TreePath> {
        let node = self.node();
        let disk = self.steps.contains(&Step::Node(Node::Drives));
        let step = if disk {
            Step::Disk(seg.to_string())
        } else if self.entry().is_some() {
            // An object's fields aren't nodes: below an entry of a collection only the
            // collection's schema declares children (a profile's Settings / Content), by name or No.
            let schema = node.entry_schema()?;
            Step::Node(schema.child(seg).or_else(|| seg.parse().ok().and_then(|n| schema.nth_child(n)))?)
        } else if let Some(child) = node.child(seg) {
            Step::Node(child)
        } else if matches!(node.kind(), Named(_)) && seg.parse::<u32>().is_err() && !self.steps.last().is_some_and(|s| matches!(s, Step::Name(_))) {
            // A protocol by name: which names exist is the catalog's to say (`vfs::path_exists`).
            Step::Name(seg.to_string())
        } else if let Ok(n) = seg.parse::<u32>() {
            match node.kind() {
                Collection | Live if self.entry().is_none() => Step::Entry(n),
                Numbered(_) if !matches!(self.steps.last(), Some(Step::Entry(_))) => Step::Entry(n),
                // A declared node's children are numbered by declaration order (§33 54), a
                // numbered entry's (it stands on its schema: `DataPools.1.2`) too.
                Static | Schema => Step::Node(node.nth_child(n)?),
                _ => return None,
            }
        } else {
            return None;
        };
        let mut steps = self.steps.clone();
        steps.push(step);
        Some(TreePath { steps })
    }

    /// Its first `n` steps.
    pub fn prefix(&self, n: usize) -> TreePath {
        TreePath {
            steps: self.steps[..n.min(self.steps.len())].to_vec(),
        }
    }

    /// The path one up (`Root` stays).
    pub fn parent(&self) -> TreePath {
        let mut steps = self.steps.clone();
        steps.pop();
        TreePath { steps }
    }

    /// Its schema spelling: a numbered entry as the schema's name
    /// (`Root.ShowData.DataPools.DataPool.Groups`), what the reference prints.
    pub fn schema_string(&self) -> String {
        let mut out = String::from("Root");
        let mut node = Node::Root;
        for step in &self.steps {
            out.push('.');
            match step {
                Step::Node(n) => {
                    node = *n;
                    out.push_str(n.name());
                }
                Step::Entry(e) => match node.kind() {
                    Numbered(schema) => {
                        node = schema;
                        out.push_str(schema.name());
                    }
                    _ => out.push_str(&e.to_string()),
                },
                Step::Name(name) => {
                    if let Named(shape) = node.kind() {
                        node = shape;
                    }
                    out.push_str(name);
                }
                Step::Disk(name) => out.push_str(name),
            }
        }
        out
    }
}

impl fmt::Display for TreePath {
    /// `Root.ShowData.DataPools.2.Groups`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Root")?;
        for step in &self.steps {
            match step {
                Step::Node(n) => write!(f, ".{}", n.name())?,
                Step::Entry(e) => write!(f, ".{e}")?,
                Step::Name(name) | Step::Disk(name) => write!(f, ".{name}")?,
            }
        }
        Ok(())
    }
}

impl Node {
    /// Its path as a [`TreePath`] (through a numbered node's schema: entry 0 would be wrong,
    /// so the schema name stands as a node step).
    pub fn tree_path(self) -> TreePath {
        let mut chain = vec![self];
        let mut at = self;
        while let Some(p) = at.parent() {
            chain.push(p);
            at = p;
        }
        chain.pop(); // Root
        chain.reverse();
        TreePath {
            steps: chain.into_iter().map(Step::Node).collect(),
        }
    }

    /// The object `enter` claims here (decisions §4): `ShowData` only on itself, else the
    /// nearest node up that names one.
    pub fn claim(self) -> Option<&'static str> {
        if self == Node::ShowData {
            return Some("ShowData");
        }
        std::iter::successors(Some(self), |n| n.parent()).find_map(|n| match n {
            Node::Devices => Some("Devices"),
            Node::DataPools => Some("DataPools"),
            Node::Protocols => Some("Protocols"),
            Node::UserProfiles => Some("UserProfiles"),
            Node::ShowSettings => Some("ShowSettings"),
            Node::StationSettings => Some("StationSettings"),
            _ => None,
        })
    }

    /// What a bare object word means here (the REPL context): the nearest node up that sets
    /// one, else Fixture.
    pub fn context(self) -> crate::ReplContext {
        use crate::ReplContext as C;
        std::iter::successors(Some(self), |n| n.parent())
            .find_map(|n| match n {
                Node::Universes => Some(C::Universe),
                Node::Fixtures => Some(C::Fixture),
                Node::Patch => Some(C::Patch),
                Node::Sequences => Some(C::Sequence),
                Node::Executors => Some(C::Executor),
                Node::Effects => Some(C::Effect),
                Node::Groups => Some(C::Group),
                Node::Presets => Some(C::Preset),
                Node::Macros => Some(C::Macro),
                Node::Timecodes => Some(C::Timecode),
                Node::DataPools => Some(C::DataPool),
                Node::EffectEngines => Some(C::EffectEngine),
                Node::UserProfiles => Some(C::UserProfile),
                _ => None,
            })
            .unwrap_or(C::Fixture)
    }

    /// The numbered node above, for a schema (`DataPool` → `DataPools`).
    pub fn schema_of(self) -> Option<Node> {
        self.numbered_by()
    }
}

