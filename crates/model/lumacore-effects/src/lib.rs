//! lumacore-effects: Effect engine data model types for LumaCore.
//!
//! Provides authored programmable effect engine configurations,
//! graph nodes, generators, and parameters.

use lumacore_base::{NodeMeta, Walk, Walker};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A node in the effect engine's processing graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectGraphNode {
    pub meta: NodeMeta,
    pub node_type: String,
    /// The nodes it feeds (GUID links).
    #[serde(default)]
    pub connections: Vec<uuid::Uuid>,
}

/// A generator definition inside an effect engine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectGenerator {
    pub meta: NodeMeta,
    pub generator_type: String,
}

/// A named parameter for an effect engine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectParameter {
    pub meta: NodeMeta,
    pub value: String,
}

/// An authored programmable effect engine configuration.
///
/// Lives under Root.ShowData.EffectEngines. Distinct from basic
/// playback Effects — this is where node-based engines, generators,
/// shader-style logic, and custom effect plugins are authored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectEngineConfig {
    pub meta: NodeMeta,
    pub engine_type: String,
    #[serde(default)]
    pub enabled: bool,
    /// Processing graph nodes.
    #[serde(default)]
    pub graph: BTreeMap<u32, EffectGraphNode>,
    /// Generator definitions.
    #[serde(default)]
    pub generators: BTreeMap<u32, EffectGenerator>,
    /// Input connection points.
    #[serde(default)]
    pub inputs: BTreeMap<u32, String>,
    /// Output connection points.
    #[serde(default)]
    pub outputs: BTreeMap<u32, String>,
    /// Named parameters.
    #[serde(default)]
    pub parameters: BTreeMap<String, EffectParameter>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectEnginesData {
    #[serde(default)]
    pub engines: BTreeMap<u32, EffectEngineConfig>,
}

// ── Identity (decisions §21 "6d") ───────────────────────────────────────────

lumacore_base::meta_identified! {
    EffectGraphNode => "EffectNode",
    EffectGenerator => "EffectGenerator",
    EffectParameter => "EffectParameter",
    EffectEngineConfig => "EffectEngine",
}

impl Walk for EffectEnginesData {
    fn walk(&self, w: &mut Walker) {
        let Self { engines } = self;
        w.pool("Engines", engines);
    }
}

impl Walk for EffectEngineConfig {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            engine_type: _,
            enabled: _,
            graph,
            generators,
            inputs,
            outputs,
            parameters,
        } = self;
        meta.walk_links(w);
        w.pool("Graph", graph);
        w.pool("Generators", generators);
        w.data(inputs);
        w.data(outputs);
        w.pool("Parameters", parameters);
    }
}

impl Walk for EffectGraphNode {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            node_type: _,
            connections,
        } = self;
        meta.walk_links(w);
        w.links("Connections", "EffectNode", connections);
    }
}

impl Walk for EffectGenerator {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            generator_type: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}

impl Walk for EffectParameter {
    fn walk(&self, w: &mut Walker) {
        let Self {
            meta,
            value: _,
        } = self;
        meta.walk_links(w);
        w.data(&());
    }
}
