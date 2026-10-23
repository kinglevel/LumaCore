//! Variable types for LumaCore runtime and show-level variables.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariableValue {
    Bool(bool),
    Integer(i64),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VariableStore {
    #[serde(default)]
    pub values: BTreeMap<String, VariableValue>,
}
