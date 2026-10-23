// ██╗██████╗ ███████╗███╗   ██╗████████╗██╗████████╗██╗   ██╗
// ██║██╔══██╗██╔════╝████╗  ██║╚══██╔══╝██║╚══██╔══╝╚██╗ ██╔╝
// ██║██║  ██║█████╗  ██╔██╗ ██║   ██║   ██║   ██║    ╚████╔╝
// ██║██║  ██║██╔══╝  ██║╚██╗██║   ██║   ██║   ██║     ╚██╔╝
// ██║██████╔╝███████╗██║ ╚████║   ██║   ██║   ██║      ██║
// ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝   ╚═╝      ╚═╝
// FILE: identity

//!
//! Owns:          Identified and Walk for the fixture data.
//! Doesn't own:   the walk of everything else (lumacore-tree identity.rs), the traits (lumacore-base).
//! Entry points:  the impls.
//! Decisions:     §21 "6d", §34 25.10.

use lumacore_base::{Walk, Walker, meta_identified};

use crate::attribute_def::{ActivationGroup, AttributeDefinition, AttributeRule, EncoderBarLayout, FeatureGroup};
use crate::fixture::{Fixture, FixtureBreak, LumaFixture, MultipatchCopy};
use crate::devices_meta::{MvrClass, MvrFile, MvrGroup, MvrLayer, MvrMappingDefinition, MvrMetadata, MvrPosition};
use crate::fixture_mvr::MvrFixture;
use crate::fixture_type::{FixtureType, Mode, Revision};
use crate::universe::Universe;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

meta_identified! {
    FixtureType => "FixtureType",
    Revision => "Revision",
    Mode => "Mode",
    Fixture => "Fixture",
    MultipatchCopy => "Multipatch",
    Universe => "Universe",
    AttributeDefinition => "Attribute",
    FeatureGroup => "FeatureGroup",
    ActivationGroup => "ActivationGroup",
    AttributeRule => "AttributeRule",
    EncoderBarLayout => "EncoderBar",
    MvrLayer => "MvrLayer",
    MvrGroup => "MvrGroup",
    MvrClass => "MvrClass",
    MvrPosition => "MvrPosition",
    MvrMappingDefinition => "MvrMappingDefinition",
    MvrFile => "MvrFile",
}

impl Walk for FixtureType {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            manufacturer: _,
            category,
            revisions,
            // The file's own id: data (links use the GUID).
            gdtf_fixture_type_id: _,
        } = self;
        meta.walk_links(w);
        w.data(category);
        w.list("Revisions", revisions);
    }
}

impl Walk for Revision {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            gdtf_version: _,
            file_hash: _,
            modes,
            geometries,
            wheels,
            emitters,
            filters,
            attributes,
        } = self;
        meta.walk_links(w);
        w.list("Modes", modes);
        w.data(attributes);
        // Derived from the .gdtf file; names are GDTF's own identity.
        w.data(geometries);
        w.data(wheels);
        w.data(emitters);
        w.data(filters);
    }
}

impl Walk for Mode {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            channel_functions,
            dmx_footprint: _,
            geometry: _,
            cells,
            relations,
        } = self;
        meta.walk_links(w);
        // GDTF channel functions name attributes by their standard name.
        w.data(channel_functions);
        w.data(cells);
        w.data(relations);
    }
}

impl Walk for Fixture {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, mvr, luma } = self;
        meta.walk_links(w);
        w.child("Mvr", mvr);
        let LumaFixture { master_react: _, multipatch } = luma;
        w.list("Multipatch", multipatch);
    }
}

impl Walk for MultipatchCopy {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, breaks, matrix: _ } = self;
        meta.walk_links(w);
        for b in breaks {
            let FixtureBreak { universe_id, start_address: _, channel_count: _ } = b;
            w.link("Breaks.Universe", "Universe", *universe_id);
        }
    }
}

impl Walk for MvrFixture {
    fn walk(&self, w: &mut Walker) {
        let Self {
            parent,
            matrix: _,
            fixture_type,
            revision,
            mode,
            breaks,
            multipatch,
            class,
            position,
            // 3DSpatial isn't hardened yet: its objects' GUIDs are kept as data (point 1).
            focus: _,
            function: _,
            fixture_id: _,
            fid: _,
            order: _,
            unit_number: _,
            child_position: _,
            id_type: _,
            cid: _,
            cast_shadow: _,
            dmx_invert_pan: _,
            dmx_invert_tilt: _,
            network,
            protocols,
            alignments,
            custom_commands,
            overwrites,
            connections,
            color: _,
            mappings,
            gobo,
        } = self;
        // Its place in the tree: a layer, a group or the fixture it is a child of (never a loop).
        w.contains("Parent", "MvrLayer|MvrGroup|Fixture", *parent);
        w.link("FixtureType", "FixtureType", *fixture_type);
        w.link("Revision", "Revision", *revision);
        w.link("Mode", "Mode", *mode);
        for b in breaks {
            let FixtureBreak {
                universe_id,
                start_address: _,
                channel_count: _,
            } = b;
            w.link("Breaks.Universe", "Universe", *universe_id);
        }
        w.link_opt("Multipatch", "Fixture", *multipatch);
        w.link_opt("Class", "MvrClass", *class);
        w.link_opt("Position", "MvrPosition", *position);
        w.links("Mappings", "MvrMappingDefinition", mappings.iter().map(|m| &m.link_def));
        w.data(network);
        w.data(protocols);
        w.data(alignments);
        w.data(custom_commands);
        w.data(overwrites);
        // The other end is any MVR object (3DSpatial): data until it is hardened.
        w.data(connections);
        w.data(gobo);
    }
}

impl Walk for MvrMetadata {
    fn walk(&self, w: &mut Walker) {
        let Self {
            layers,
            groups,
            classes,
            positions,
            mapping_definitions,
            files,
        } = self;
        w.pool("Layers", layers);
        w.pool("Groups", groups);
        w.pool("Classes", classes);
        w.pool("Positions", positions);
        w.pool("MappingDefinitions", mapping_definitions);
        w.pool("Files", files);
    }
}

impl Walk for MvrLayer {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, matrix: _, order: _ } = self;
        meta.walk_links(w);
    }
}

impl Walk for MvrGroup {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, parent, matrix: _, order: _ } = self;
        meta.walk_links(w);
        w.contains("Parent", "MvrLayer|MvrGroup", Some(*parent));
    }
}

impl Walk for MvrClass {
    fn walk(&self, w: &mut Walker) {
        let Self { meta } = self;
        meta.walk_links(w);
    }
}

impl Walk for MvrPosition {
    fn walk(&self, w: &mut Walker) {
        let Self { meta } = self;
        meta.walk_links(w);
    }
}

impl Walk for MvrMappingDefinition {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            size_x: _,
            size_y: _,
            source,
            scale_handling: _,
        } = self;
        meta.walk_links(w);
        w.data(source);
    }
}

impl Walk for MvrFile {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            // The file's own identity: it is kept by its hash.
            sha256: _,
            provider: _,
            provider_version: _,
            mvr_version: _,
            at_ms: _,
            user: _,
            scope: _,
            preset: _,
            counts,
            user_data: _,
        } = self;
        meta.walk_links(w);
        w.data(counts);
    }
}

impl Walk for Universe {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            merge_mode,
            occupied: _,
        } = self;
        meta.walk_links(w);
        w.data(merge_mode);
    }
}

impl Walk for AttributeDefinition {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            name: _,
            pretty: _,
            physical_unit,
            natural_readout,
            feature_group,
            activation_group,
            main_attribute: _,
            encoder_resolution,
            intensity: _,
            mib: _,
            feature: _,
            special: _,
            position: _,
            placement,
            new: _,
        } = self;
        meta.walk_links(w);
        w.data(placement);
        w.data(physical_unit);
        w.data(natural_readout);
        w.link_opt("FeatureGroup", "FeatureGroup", *feature_group);
        w.link_opt("ActivationGroup", "ActivationGroup", *activation_group);
        w.data(encoder_resolution);
    }
}

impl Walk for FeatureGroup {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            features,
        } = self;
        meta.walk_links(w);
        w.data(features);
    }
}

impl Walk for AttributeRule {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            pattern: _,
            group,
            feature: _,
            kind,
            class: _,
            learned: _,
        } = self;
        meta.walk_links(w);
        w.link("Group", "FeatureGroup", *group);
        w.data(kind);
    }
}

impl Walk for ActivationGroup {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            name: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for EncoderBarLayout {
    fn walk(&self, w: &mut Walker) {
        let Self { meta, buttons } = self;
        meta.walk_links(w);
        for b in buttons {
            w.link("Buttons.Group", "FeatureGroup", b.group);
        }
        w.data(buttons);
    }
}
