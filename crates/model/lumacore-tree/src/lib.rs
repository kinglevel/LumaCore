//! lumacore-tree: Object tree data model for LumaCore, following known industry standards
//!
//! Provides the foundational types used by all other LumaCore crates:
//! - TreeNode trait and NodeMeta for identity and addressing
//! - Root, ShowData, StationSettings for hierarchical show structure
//! - FixtureType, Fixture for GDTF fixture modeling
//! - Universe (routes to protocol lines) for DMX universe management
//! - Line: protocol output/input lines (sACN, Art-Net, OSC)
//! - Programmer, UserProfile for selection state
//! - TreeEvent for reactive observation

pub mod attribute_resolver;
pub mod datapool;
pub mod dmx_slots;
pub use lumacore_base::events;
pub use lumacore_base::id_pool;
pub mod identity;
pub use lumacore_base::keybindings;
pub mod media;
pub mod merge;
pub use lumacore_base::node;
pub mod output_ops;
pub mod programmer;
pub use lumacore_runtime as runtime;
pub use lumacore_tags as tags;
/// The fixture data (MVR part, metadata) for the crates above the tree (decisions §34 25.10).
pub use lumacore_fixtures as fixtures;
pub mod tree;
pub mod ui_keys;
pub use lumacore_base::undo;
pub mod universe_ops;
pub mod bookmarks;
pub mod usage;
pub mod window_types;

// Re-export all public types
pub use datapool::{
    Datapool, DatapoolPriorityOrder, DistributePreset, DynamicState, FilterItem, Group, GroupMember, GridCell, GroupMasterMode, GridCursorMove,
    LayoutItem, LoopRegion, MacroExecute, MacroItem, MacroLine, MacroWait, PluginItem, Preset,
    PresetEntry, PresetMode, PriorityItem, SignatureEntry, TempoEntry, TimeBase,
    TimecodeClip, TimecodeEvent, TimecodeItem, TimecodeMarker, TimecodeRegion, TimecodeTrack,
    TrackEvents, WorldItem,
};
pub use events::{TreeEvent, TreeObserver};
pub use id_pool::IdPool;
pub use media::{Asset, AssetRoot, AssetsData, LockScreen, MediaFile, Theme, ThemeAdjust, ThemePalette};
pub use keybindings::{
    BindingSource, KeyBinding, KeyBindingMap, debug_keybindings, normalize_key, resolve_bindings,
    resolve_bindings_with_source, vanilla_keybindings,
};
pub use lumacore_base::error_code::{self, Coded, ErrorInfo};
pub use lumacore_base::replication::{PendingReplicationOp, ReplicationJournal, ReplicationOp};
pub use lumacore_base::value::{Absolute, Value, next_stamp};
pub use node::{IconRef, NodeMeta, TreeNode};
pub use output_ops::{LineError, LineOps};
pub use programmer::{MvrImportPreset, MvrLost, MvrMatch, MvrMissing, MvrRotation, MvrTake};
pub use programmer::{OopsConfirmation, OopsSettings, Programmer, ProgrammerValues, SelectionState};
pub use programmer::{
    ClearState, ProgValue, ScreenConfig, ScreenContent, Selected, StoredView, USER_COLORS, USER_COLOR_NAMES, DEFAULT_PROFILE, User,
    ProfileContent, ProfileSettings, SuggestionsMode, UserPermissions, UserPreferences, UserProfile, UserRights, Verbosity, WindowPreset,
    WindowPresets, ProfileUi, GridState, GridSort, ViewButton, ButtonKind,
};
pub use bookmarks::Bookmarks;
pub use usage::{Usage, UsageEntry};
pub use runtime::{
    NodeRuntime, Claim, ClaimAsk, Clipboard, CommandsDone, DiscoveryPeerRuntime, EngineRuntime, JoinedSessionRuntime,
    LineRuntime, LumaSessionsRuntime, MemberRuntime, Modes, NetworkConnectionRuntime, NetworkHealthRuntime, NetworkInterfaceRuntime,
    NetworkListenerRuntime, NetworkRuntime, RuntimeState, RuntimeUser,
    SessionClientRuntime, SessionStationRuntime, SourceRuntime, StationPriority, StationStatus,
    UserMode,
    WatchTarget,
};
pub use lumacore_runtime::{MasterMode, ModeMaster};
pub use tree::{
    MergeChoice,
    BuildInfo, CueObject, CuePart, CueTrigger, GroupTiming, KnownStation, SessionData, CueValue, CurvePreset, FadePath, DevicesData, DiagnosticRoot,
    Effect, EffectEngineConfig, EffectEnginesData, EffectGenerator, EffectGraphNode,
    EffectParameter, ExecutorObject, DmxDevices, DevicesMetadata,
    MibPreference, NicServices, ParkData, ParkedChannel, ParkedValue, PatchData, PlaybackData,
    CueZeroMode, CUE_ZERO, Echo, EchoPlaybacks, GrandMasters, LivePlaybacks, MasterLevel, Masters, PlaybackPriority, PlaybackStates, ProtocolEndpointConfig, now_ms, ProtocolNode, ProtocolsData, ResourceEntry,
    ResourceRoot, RestartMode, Root, SequenceAssignment, SequenceObject, SequenceSettings,
    FORMAT_VERSION, SequenceState, ShowData, ShowSettings, Spatial3DData,
    SpatialPatchObject, SpatialSceneObject, StationSettings, StationWiring, TimingPreset, UIEngineState,
    UIEnginesRoot, UIWorkspaceState, VariableStore, VariableValue, cue_key, cue_label, DriveKind, DriveMemory};
pub use ui_keys::{Gesture, UiKey, UiKeyOwner, builtin_ui_keys};
pub use undo::{
    DEFAULT_MAX_BYTES, DEFAULT_MAX_FRAMES, LocalSnapshot, LocalSubtree, OopsCategory, OopsKind, ShowSubtree, SubtreeSnapshot, UndoFrame,
    UndoStack, UndoStep, UserUndoStep,
};
pub use universe_ops::{UniverseError, UniverseOps};
pub use window_types::{SettingDef, SettingKind, WindowType, WindowTypeView, WindowView};

/// This crate's rows in the catalog, after the crates it builds on (decisions §34 21).
pub fn register(r: &mut lumacore_registry::Registry) {
    lumacore_protocols::register(r);
    lumacore_tags::register(r);
    if !r.enter(env!("CARGO_PKG_NAME")) {
        return;
    }
    r.errors(output_ops::OUTPUT_ERRORS);
    r.errors(universe_ops::UNIVERSE_ERRORS);
    r.window_types(window_types::builtin_window_types().into_values());
    r.keys(ui_keys::core_keys());
}

/// This crate's tests read the catalog: it is installed before the first one (decisions §34 21.8).
#[cfg(test)]
#[ctor::ctor]
fn test_catalog() {
    lumacore_registry::install_with(|r| {
        crate::register(r);
        lumacore_sacn::register(r);
    });
}
