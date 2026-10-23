// ██████╗ ███████╗██╗   ██╗██╗ ██████╗███████╗███████╗    ███╗   ███╗███████╗████████╗ █████╗
// ██╔══██╗██╔════╝██║   ██║██║██╔════╝██╔════╝██╔════╝    ████╗ ████║██╔════╝╚══██╔══╝██╔══██╗
// ██║  ██║█████╗  ██║   ██║██║██║     █████╗  ███████╗    ██╔████╔██║█████╗     ██║   ███████║
// ██║  ██║██╔══╝  ╚██╗ ██╔╝██║██║     ██╔══╝  ╚════██║    ██║╚██╔╝██║██╔══╝     ██║   ██╔══██║
// ██████╔╝███████╗ ╚████╔╝ ██║╚██████╗███████╗███████║    ██║ ╚═╝ ██║███████╗   ██║   ██║  ██║
// ╚═════╝ ╚══════╝  ╚═══╝  ╚═╝ ╚═════╝╚══════╝╚══════╝    ╚═╝     ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝
// FILE: devices_meta

//! `Root.ShowData.Devices.Metadata.MVR` (decisions §34 "34g4 patch hardened" 7): what MVR files
//! bring besides fixtures, by UUID (each entry's GUID is the MVR uuid). The Patch and 3DSpatial
//! both use it.
//!
//! Layers and Groups are the patch tree (point 12) and can be edited. Classes, Positions,
//! MappingDefinitions and Files arrive and change only through imports, matched by UUID.
//!
//! Owns:          MvrMetadata and its entries.
//! Doesn't own:   the fixtures (Devices.Dmx.Patch), the import that fills it.
//! Entry points:  MvrMetadata, MvrLayer, MvrGroup, MvrClass, MvrPosition, MvrMappingDefinition, MvrFile.
//! Decisions:     §34 "34g4 patch hardened" 7, 12.

use std::collections::BTreeMap;

use lumacore_base::node::NodeMeta;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fixture_mvr::Matrix;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// `Devices.Metadata.MVR`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MvrMetadata {
    #[serde(default)]
    pub layers: BTreeMap<u32, MvrLayer>,
    /// MVR's GroupObjects: they nest, in a layer or in each other.
    #[serde(default)]
    pub groups: BTreeMap<u32, MvrGroup>,
    #[serde(default)]
    pub classes: BTreeMap<u32, MvrClass>,
    #[serde(default)]
    pub positions: BTreeMap<u32, MvrPosition>,
    #[serde(default)]
    pub mapping_definitions: BTreeMap<u32, MvrMappingDefinition>,
    /// One record per import.
    #[serde(default)]
    pub files: BTreeMap<u32, MvrFile>,
}

/// An MVR `<Layer>`: flat, the top of the tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrLayer {
    pub meta: NodeMeta,
    #[serde(default)]
    pub matrix: Matrix,
    /// Its place among the layers (decisions §40 6: spaced 10, 20, 30 …).
    #[serde(default)]
    pub order: u32,
}

/// An MVR `<GroupObject>`: in a layer or in another group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrGroup {
    pub meta: NodeMeta,
    /// The layer or group it is in.
    pub parent: Uuid,
    #[serde(default)]
    pub matrix: Matrix,
    /// Its place among its parent's children (decisions §40 6).
    #[serde(default)]
    pub order: u32,
}

/// An MVR `<Class>` (AUXData).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrClass {
    pub meta: NodeMeta,
}

/// An MVR `<Position>` (AUXData).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrPosition {
    pub meta: NodeMeta,
}

/// An MVR `<MappingDefinition>` (AUXData): a video source mapped onto fixtures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrMappingDefinition {
    pub meta: NodeMeta,
    #[serde(default)]
    pub size_x: u32,
    #[serde(default)]
    pub size_y: u32,
    /// `<Source>`: its linked geometry, type and value.
    #[serde(default)]
    pub source: MappingSource,
    #[serde(default)]
    pub scale_handling: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MappingSource {
    pub linked_geometry: String,
    pub kind: String,
    pub value: String,
}

/// One import of an MVR file: what came, from where, when and by whom. The file itself is in
/// `library/mvr/.store/<sha256>.mvr`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrFile {
    pub meta: NodeMeta,
    pub sha256: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub provider_version: String,
    #[serde(default)]
    pub mvr_version: String,
    /// When, ms since the epoch.
    #[serde(default)]
    pub at_ms: u64,
    /// Who imported it (a login).
    #[serde(default)]
    pub user: String,
    /// What it was imported into (a layer, group, class, tag or the patch), as written.
    #[serde(default)]
    pub scope: String,
    /// The import preset used.
    #[serde(default)]
    pub preset: String,
    /// How many of each change it made (added, updated, …).
    #[serde(default)]
    pub counts: BTreeMap<String, u32>,
    /// The file's `<UserData>`, as text.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub user_data: String,
}
