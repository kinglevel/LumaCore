//! Identity of the show's objects (decisions §21 "6d hardening"): every
//! object implements [`Identified`] and every type [`Walk`]s its fields with
//! an exhaustive destructure, so a new field doesn't compile until it is
//! classified (object pool, GUID link or plain data).
//!
//! `ShowData::identity()` walks the whole show: the GUID index (duplicates,
//! nil GUIDs), the reference check (links to missing objects, containment
//! loops) and the object paths claims and merges name.

use crate::datapool::*;
use crate::programmer::Selected;
use crate::programmer::{ProfileContent, ProfileSettings, ProgValue, Programmer, ScreenConfig, ScreenContent, StoredView, User, UserProfile, WindowPreset};
use crate::tree::*;
use crate::media::{Asset, AssetRoot, AssetsData, MediaFile};
use crate::ui_keys::UiKey;
use lumacore_base::{Guid, Identified, IdentityReport, Walk, Walker, meta_identified, meta_only_walk};
use uuid::Uuid;

meta_identified! {
    Datapool => "DataPool",
    Group => "Group",
    Preset => "Preset",
    LayoutItem => "Layout",
    FilterItem => "Filter",
    WorldItem => "World",
    MacroItem => "Macro",
    DistributePreset => "Pawtern",
    DynamicState => "DynamicState",
    PluginItem => "Plugin",
    PriorityItem => "Priority",
    UserProfile => "Profile",
    KnownStation => "Station",
    StationWiring => "StationRow",
    ProtocolEndpointConfig => "Endpoint",
    SpatialSceneObject => "SpatialObject",
    SpatialPatchObject => "SpatialPatch",
    DatapoolPriorityOrder => "PriorityOrder",
    StoredView => "View",
    ScreenConfig => "ScreenConfig",
    ScreenContent => "Screen",
    WindowPreset => "WindowPreset",
    User => "User",
    UiKey => "UiKey",
    AssetRoot => "AssetRoot",
    Asset => "Asset",
    MacroLine => "MacroLine",
}

impl KnownStation {
    /// A known station's GUID is derived from its station id, so the same
    /// station is the same object on both sides of a merge.
    pub fn guid_for(station_id: &str) -> Uuid {
        Uuid::parse_str(station_id).unwrap_or_else(|_| Guid::named("Station", station_id).0)
    }
}

impl ShowData {
    /// Walk the whole show (see the module docs).
    pub fn walk_identity(&self) -> Walker {
        let mut w = Walker::new();
        self.walk(&mut w);
        w
    }

    /// Duplicates, nil GUIDs, broken links and containment loops.
    pub fn identity(&self) -> IdentityReport {
        self.walk_identity().report()
    }
}

impl Walk for ShowData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            devices,
            datapools,
            datapool_priority_orders,
            show_settings,
            spatial_3d,
            playback,
            playbacks,
            session,
            user_profiles,
            users,
            protocols,
            effect_engines,
            variables,
            images,
            assets,
            attribute_defs,
            feature_groups,
            activation_groups,
            attribute_rules,
            stations,
            tags,
        } = self;
        w.child("Devices", devices);
        w.pool("DataPools", datapools);
        w.pool("PriorityOrders", datapool_priority_orders);
        w.data(show_settings);
        w.child("Spatial3D", spatial_3d);
        w.child("Playback", playback);
        w.child("Playbacks", playbacks);
        w.child("LumaSession", session);
        w.pool("UserProfiles", user_profiles);
        w.pool("Users", users);
        w.child("Protocols", protocols);
        w.child("EffectEngines", effect_engines);
        w.data(variables);
        w.pool("Images", images);
        w.child("Assets", assets);
        w.pool("AttributeDefinitions", attribute_defs);
        w.pool("FeatureGroups", feature_groups);
        w.pool("ActivationGroups", activation_groups);
        w.pool("AttributeRules", attribute_rules);
        w.pool("Stations", stations);
        w.pool("Tags", tags);
    }
}

impl Walk for StationWiring {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            // The station's own id (station_settings.json): data, not a show link.
            station_id: _,
            networks,
            // Interface names are the hardware's own identity.
            services: _,
        } = self;
        meta.walk_links(w);
        w.links("Networks", "Network", networks.keys());
    }
}

impl Walk for DevicesData {
    fn walk(&self, w: &mut Walker) {
        let Self { dmx, metadata } = self;
        let DmxDevices { patch } = dmx;
        w.child("Dmx.Patch", patch);
        let DevicesMetadata { mvr } = metadata;
        w.child("Metadata.MVR", mvr);
    }
}

impl Walk for PatchData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            fixture_types,
            fixtures,
            fixture_id_pool: _,
            fid_index: _,
            universes,
            universe_id_pool: _,
        } = self;
        w.pool("FixtureTypes", fixture_types);
        // A fixture's step is its number (`Fixtures.101`), never the slot (decisions §40 5); a
        // fixture without a number (a folder) steps by its GUID.
        for f in fixtures.values() {
            match f.mvr.fid {
                Some(n) => w.pool_member("Fixtures", n, f),
                None => w.pool_member("Fixtures", f.meta.identity.guid, f),
            }
        }
        w.pool("Universes", universes);
    }
}






impl Walk for Datapool {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            groups,
            presets,
            layouts,
            filters,
            worlds,
            macros,
            distribute_presets,
            dynamic_states,
            plugins,
            timecodes,
            priorities,
            sequences,
            timing_presets,
            curve_presets,
        } = self;
        meta.walk_links(w);
        w.pool("Groups", groups);
        w.pool("Sequences", sequences);
        w.pool("TimingPresets", timing_presets);
        w.pool("CurvePresets", curve_presets);
        w.pool("Presets", presets);
        w.pool("Layouts", layouts);
        w.pool("Filters", filters);
        w.pool("Worlds", worlds);
        w.pool("Macros", macros);
        w.pool("Pawterns", distribute_presets);
        w.pool("DynamicStates", dynamic_states);
        w.pool("Plugins", plugins);
        w.pool("Timecodes", timecodes);
        w.pool("Priorities", priorities);
    }
}

meta_only_walk!(LayoutItem, FilterItem, WorldItem);

impl Walk for Group {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            members,
            master: _,
            preserve_grid: _,
            move_grid_cursor: _,
            pawtern,
        } = self;
        meta.walk_links(w);
        for m in members {
            let crate::datapool::GroupMember { fixture, cell: _ } = m;
            w.link("Members.Fixture", "Fixture", *fixture);
        }
        w.link_opt("Pawtern", "Pawtern", *pawtern);
    }
}

impl Walk for Preset {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, mode, values } = self;
        meta.walk_links(w);
        w.data(mode);
        for v in values {
            let PresetEntry {
                fixture_id,
                attribute: _,
                value,
            } = v;
            w.link_opt("Values.Fixture", "Fixture", *fixture_id);
            value.walk(w);
        }
    }
}

impl Walk for MacroItem {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, lines } = self;
        meta.walk_links(w);
        w.list("Lines", lines);
    }
}

impl Walk for MacroLine {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            command: _,
            wait,
            enabled: _,
            execute,
        } = self;
        meta.walk_links(w);
        // Command text is text (it names objects the way a user types them).
        w.data(wait);
        w.data(execute);
    }
}

impl Walk for DistributePreset {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, context: _ } = self;
        meta.walk_links(w);
    }
}

impl Walk for DynamicState {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            description: _,
        } = self;
        meta.walk_links(w);
    }
}

impl Walk for PluginItem {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            plugin_id: _,
            version: _,
            description: _,
            scripts,
            buttons,
        } = self;
        meta.walk_links(w);
        w.data(scripts);
        w.list("Buttons", buttons);
    }
}

impl Walk for PriorityItem {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, level: _ } = self;
        meta.walk_links(w);
    }
}

impl Walk for DatapoolPriorityOrder {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            priority: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for Spatial3DData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            world,
            stages,
            layers,
            cameras,
            patch,
            tracking,
        } = self;
        w.pool("World", world);
        w.pool("Stages", stages);
        w.pool("Layers", layers);
        w.pool("Cameras", cameras);
        w.pool("Patch", patch);
        w.pool("Tracking", tracking);
    }
}

impl Walk for SpatialSceneObject {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            resource_ref: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for SpatialPatchObject {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            fixture,
            object_name: _,
        } = self;
        meta.walk_links(w);
        w.link("Fixture", "Fixture", *fixture);
    }
}

impl Walk for SessionData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            patch_version: _,
            stations,
        } = self;
        w.pool("Stations", stations);
    }
}

impl Walk for KnownStation {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            station_id: _,
            ip: _,
            stream_port: _,
            priority: _,
            joined_unix_ms: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for UserProfile {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, locked: _, settings, content } = self;
        let ProfileSettings {
            prog_lang: _,
            alias_depth: _,
            preferences,
            permissions,
            show_descriptions: _,
            clear_buffer: _,
            verbosity,
            theme,
            lock_screen,
            ui,
        } = settings;
        let ProfileContent {
            attributes,
            keybindings,
            ui_keys,
            views,
            view_buttons,
            screen_configs,
            window_presets,
            images,
            icons,
            user_variables,
        } = content;
        meta.walk_links(w);
        // Menu and grid names are their own identity (registry names).
        w.data(ui);
        // Each window type numbers its own presets.
        for (ty, pool) in window_presets {
            w.pool(&format!("WindowPresets.{ty}"), pool);
        }
        w.data(theme);
        w.pool("Icons", icons);
        w.pool("Images", images);
        // The lock screen's picture is one of the profile's own (§30 "12a2").
        let crate::media::LockScreen {
            image,
            show_clock: _,
            text: _,
            auto_lock_minutes: _,
        } = lock_screen;
        w.link_opt("LockScreenImage", "Image", *image);
        // Bindings name a UiKey by its id (the key's identity).
        w.data(keybindings);
        w.pool("Views", views);
        for (screen, buttons) in view_buttons {
            for (slot, b) in buttons {
                w.link(&format!("ViewButtons.{screen}.{slot}"), b.kind.keyword(), b.guid);
            }
        }
        w.pool("ScreenConfigs", screen_configs);
        w.pool("UIKeys", ui_keys);
        w.data(preferences);
        w.data(user_variables);
        w.data(permissions);
        w.data(verbosity);
        // The profile's own attribute layer (decisions §30 "12f-a").
        let lumacore_fixtures::attribute_def::ProfileAttributes { template: _, groups, rules, placed, encoder_bars } = attributes;
        w.pool("AttributeGroups", groups);
        w.pool("AttributeRules", rules);
        // Keyed by GDTF attribute name (its identity).
        for p in placed.values() {
            w.link("Placed.Group", "FeatureGroup", p.group);
        }
        w.data(placed);
        w.pool("EncoderBars", encoder_bars);
    }
}


impl Walk for StoredView {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            screens,
        } = self;
        meta.walk_links(w);
        w.pool("Screens", screens);
    }
}

impl Walk for ScreenConfig {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            screens,
        } = self;
        meta.walk_links(w);
        w.pool("Screens", screens);
    }
}

impl Walk for WindowPreset {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            kind: _,
            label: _,
            colour: _,
            scroll,
            settings,
        } = self;
        meta.walk_links(w);
        // A window's settings are data (a DataPool setting names its GUID as
        // a value the window resolves, like in a screen's windows).
        w.data(scroll);
        w.data(settings);
    }
}

impl Walk for ScreenContent {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            root,
            windows,
            encoder_bar,
        } = self;
        meta.walk_links(w);
        // Windows are the screen's content, numbered inside it.
        w.data(root);
        w.data(windows);
        let crate::programmer::EncoderBarSetting { on: _, layout } = encoder_bar;
        w.link_opt("EncoderBar", "EncoderBar", *layout);
    }
}

impl Walk for UiKey {
    fn walk(&self, w: &mut Walker) {
        // Gesture lines are command text; `state` is a tree path.
        self.meta.walk_links(w);
        w.data(self);
    }
}

impl Walk for User {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            login: _,
            password_hash: _,
            rights,
            remote_login: _,
            profile,
            color: _,
            default_screen_config,
            default_object: _,
            programmer,
            selected,
            usage,
            bookmarks,
        } = self;
        meta.walk_links(w);
        // Usage keys name GUIDs as statistics, not links: a deleted object's
        // count is only stale, never a broken reference.
        w.data(usage);
        // A bookmark is one user's place in NAV, not a link the show depends on: a deleted
        // object's bookmark shows "missing" there (decisions 0291), never a broken reference.
        w.data(bookmarks);
        w.data(rights);
        w.link("Profile", "Profile", *profile);
        w.link_opt("DefaultScreenConfig", "ScreenConfig", *default_screen_config);
        w.child("Programmer", programmer);
        let Selected {
            datapool,
            sequence,
            timecode,
        } = selected;
        w.link_opt("SelectedDataPool", "DataPool", *datapool);
        w.link_opt("SelectedSequence", "Sequence", *sequence);
        w.link_opt("SelectedTimecode", "Timecode", *timecode);
    }
}

impl Walk for Programmer {
    fn walk(&self, w: &mut Walker) {
        let Self {
            selected_fixtures,
            values,
            clear_state,
            selection_pool: _,
            walk_index: _,
            reset_echoed: _,
            grid,
            grid_cursor: _,
            value_since_select: _,
        } = self;
        w.links("Selection", "Fixture", selected_fixtures);
        w.links("Grid", "Fixture", grid.keys());
        // Sorted: a HashMap's order differs per process.
        let mut keys: Vec<&(Uuid, String)> = values.keys().collect();
        keys.sort();
        for k in keys {
            let ProgValue {
                value,
                active: _,
                stamp: _,
                timing: _,
            } = &values[k];
            w.link("Values.Fixture", "Fixture", k.0);
            value.walk(w);
        }
        w.data(clear_state);
    }
}

impl Walk for ProtocolsData {
    fn walk(&self, w: &mut Walker) {
        let Self {
            networks,
            dmx_protocols,
            data_protocols,
            bidirectional,
        } = self;
        w.pool("Networks", networks);
        for (group, nodes) in [("DMXProtocols", dmx_protocols), ("DataProtocols", data_protocols)] {
            for (name, node) in nodes {
                w.child(&format!("{group}.{name}"), node);
            }
        }
        w.pool("Bidirectional", bidirectional);
    }
}

impl Walk for crate::tree::ProtocolNode {
    fn walk(&self, w: &mut Walker) {
        let Self { settings, lines, id_pool: _ } = self;
        // Values of the protocol's declared settings, never links.
        w.data(settings);
        w.pool("Lines", lines);
    }
}



impl Walk for ProtocolEndpointConfig {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            protocol: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

/// An image or icon file: the pool kind is where it lives (show Images,
/// profile Icons / Images).
impl Identified for MediaFile {
    fn meta(&self) -> &lumacore_base::NodeMeta {
        &self.meta
    }
    fn kind(&self) -> &'static str {
        "Image"
    }
}

impl Walk for MediaFile {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            sha256: _,
            ext: _,
            size: _,
            width: _,
            height: _,
            source: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for AssetsData {
    fn walk(&self, w: &mut Walker) {
        let Self { roots, files } = self;
        w.pool("Roots", roots);
        w.pool("Files", files);
    }
}

impl Walk for AssetRoot {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, name: _ } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for Asset {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            root,
            path: _,
            sha256: _,
            size: _,
        } = self;
        meta.walk_links(w);
        w.link("Root", "AssetRoot", *root);
    }
}





