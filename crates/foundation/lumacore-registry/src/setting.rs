// ███████╗███████╗████████╗████████╗██╗███╗   ██╗ ██████╗
// ██╔════╝██╔════╝╚══██╔══╝╚══██╔══╝██║████╗  ██║██╔════╝
// ███████╗█████╗     ██║      ██║   ██║██╔██╗ ██║██║  ███╗
// ╚════██║██╔══╝     ██║      ██║   ██║██║╚██╗██║██║   ██║
// ███████║███████╗   ██║      ██║   ██║██║ ╚████║╚██████╔╝
// ╚══════╝╚══════╝   ╚═╝      ╚═╝   ╚═╝╚═╝  ╚═══╝ ╚═════╝
// FILE: setting
//! A window type's typed setting (decisions §30 "12b2", "12c"): its key, label, kind (Bool, a
//! number with named values, a choice, text, a Link, a colour, a list), default, hint, settings
//! tab and `Now` source — and the key rule users type (`AutoScroll` = `auto_scroll`). Pure data:
//! a window type's rows name them (decisions §34 21).
//!
//! Owns:          SettingDef, SettingKind, NowSource, FOLLOW_SELECTED, same_key, list_ids
//! Doesn't own:   the window types (window_type), what a setting shows live (lumacore-tree)
//! Entry points:  SettingDef::new and its builders, same_key, list_ids
//! Decisions:     §30 "12b2", "12c", §34 21

use serde::Serialize;
use serde_json::{Value, json};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What `Now` means for a setting that takes it (`Set Self "Since" Now`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NowSource {
    /// The station's wall clock, ms since the Unix epoch (a System Monitor
    /// line's `time_ms`): a mark that survives a restart, unlike a `seq`.
    WallMs,
}

/// Value type and constraints of one setting.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SettingKind {
    Bool,
    Int {
        min: i64,
        max: i64,
        /// Numbers with a name, shown and typed by it (`0` = `Fit`).
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        named: Vec<(i64, String)>,
    },
    /// One of a fixed set of values; `options` are `(value, label)` pairs.
    Enum {
        options: Vec<(Value, String)>,
    },
    /// Free text (a filter, a search), at most `max_len` characters.
    Text {
        max_len: usize,
    },
    /// A link to an object of the show (decisions §30 "12b2"): the station
    /// lists the pick list's items and resolves a typed number or name.
    /// `by_guid`: stored as the object's GUID (strict identity), else as its
    /// number (a DMX universe is an address). `follow`: a first item that
    /// follows the user's selection, stored as `"selected"`.
    Link {
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        follow: Option<String>,
        by_guid: bool,
    },
    /// A colour `#rrggbb`; empty = the theme's colour.
    Colour,
    /// An ordered list of ids (`WindowBarItems`), typed `"a,b,c"`; the
    /// window type checks them against its bar ids.
    List,
}

/// The stored value of a link that follows the user's selection.
pub const FOLLOW_SELECTED: &str = "selected";

#[derive(Debug, Clone, Serialize)]
pub struct SettingDef {
    pub key: String,
    pub label: String,
    #[serde(flatten)]
    pub kind: SettingKind,
    pub default: Value,
    /// Short explanation, e.g. what 0 means.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// The tab it sits on in the settings window (decisions §30 "12b2": tabs
    /// from the sections; one section = no tab strip).
    pub section: String,
    /// It takes `Now` (resolved by the station when the line runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub now: Option<NowSource>,
}

impl SettingDef {
    pub fn new(key: &str, label: &str, kind: SettingKind, default: Value) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind,
            default,
            hint: None,
            section: String::new(),
            now: None,
        }
    }

    pub fn takes_now(mut self, now: NowSource) -> Self {
        self.now = Some(now);
        self
    }

    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// The named numbers of an `Int` (`0` = `Fit`).
    pub fn named(mut self, names: &[(i64, &str)]) -> Self {
        if let SettingKind::Int { named, .. } = &mut self.kind {
            *named = names.iter().map(|(n, l)| (*n, (*l).to_string())).collect();
        }
        self
    }

    /// A value as shown: an option's or a named number's label, a link's
    /// stored text; `selected` as the follow label.
    pub fn label_of(&self, value: &Value) -> String {
        match &self.kind {
            SettingKind::Enum { options } => options
                .iter()
                .find(|(v, _)| v == value)
                .map(|(_, l)| l.clone())
                .unwrap_or_default(),
            SettingKind::Int { named, .. } => value
                .as_i64()
                .and_then(|n| named.iter().find(|(v, _)| *v == n).map(|(_, l)| l.clone()))
                .unwrap_or_else(|| value.to_string()),
            SettingKind::Bool => if value.as_bool() == Some(true) { "Yes" } else { "No" }.into(),
            SettingKind::Link { follow: Some(f), .. } if value.as_str() == Some(FOLLOW_SELECTED) => f.clone(),
            SettingKind::List => list_ids(value).join(","),
            _ => match value {
                Value::String(s) => s.clone(),
                Value::Null => String::new(),
                other => other.to_string(),
            },
        }
    }

    /// Coerce a value into this setting's type and range.
    /// Accepts strings too, so command-line input ("8", "on") works.
    pub fn coerce(&self, value: &Value) -> Result<Value, String> {
        match &self.kind {
            SettingKind::Bool => match value {
                Value::Bool(b) => Ok(json!(b)),
                Value::String(s) => match s.to_lowercase().as_str() {
                    "on" | "true" | "yes" | "1" => Ok(json!(true)),
                    "off" | "false" | "no" | "0" => Ok(json!(false)),
                    _ => Err(format!("{}: expected on/off", self.key)),
                },
                _ => Err(format!("{}: expected on/off", self.key)),
            },
            SettingKind::Int { min, max, named } => {
                let n = match value {
                    Value::Number(n) => n.as_f64().map(|f| f.round() as i64),
                    Value::String(s) => s.trim().parse::<i64>().ok().or_else(|| {
                        named
                            .iter()
                            .find(|(_, l)| l.eq_ignore_ascii_case(s.trim()))
                            .map(|(n, _)| *n)
                    }),
                    _ => None,
                }
                .ok_or_else(|| {
                    let names: Vec<&str> = named.iter().map(|(_, l)| l.as_str()).collect();
                    if names.is_empty() {
                        format!("{}: expected a number", self.key)
                    } else {
                        format!("{}: expected a number or {}", self.key, names.join(", "))
                    }
                })?;
                if named.iter().any(|(v, _)| *v == n) {
                    return Ok(json!(n));
                }
                Ok(json!(n.clamp(*min, *max)))
            }
            SettingKind::Link { follow, by_guid, .. } => {
                let text = match value {
                    Value::String(s) => s.trim().to_string(),
                    Value::Number(n) => n.to_string(),
                    _ => String::new(),
                };
                if follow.as_ref().is_some_and(|f| {
                    f.eq_ignore_ascii_case(&text) || text.eq_ignore_ascii_case(FOLLOW_SELECTED)
                }) {
                    return Ok(json!(FOLLOW_SELECTED));
                }
                if *by_guid {
                    uuid::Uuid::parse_str(&text)
                        .map(|g| json!(g.to_string()))
                        .map_err(|_| format!("{}: expected a {}", self.key, self.label))
                } else {
                    text.parse::<i64>()
                        .ok()
                        .filter(|n| *n >= 1)
                        .map(|n| json!(n))
                        .ok_or_else(|| format!("{}: expected a {} number", self.key, self.label))
                }
            }
            SettingKind::Colour => {
                let text = value.as_str().unwrap_or_default().trim().to_ascii_lowercase();
                if matches!(text.as_str(), "" | "none" | "theme") {
                    return Ok(json!(""));
                }
                // The one hex reading (lumacore_base::paint); `Theme` is this setting's own word.
                match lumacore_base::paint::hex_rgb(&text) {
                    Some(rgb) => Ok(json!(lumacore_base::paint::hex_of(rgb))),
                    None => Err(format!("{}: expected a colour #rrggbb (or Theme)", self.key)),
                }
            }
            SettingKind::List => match value {
                Value::Array(_) | Value::String(_) | Value::Null => Ok(json!(list_ids(value))),
                _ => Err(format!("{}: expected a list of ids \"a,b,c\"", self.key)),
            },
            SettingKind::Text { max_len } => match value {
                Value::String(s) => Ok(json!(s.trim().chars().take(*max_len).collect::<String>())),
                Value::Number(n) => Ok(json!(n.to_string())),
                Value::Null => Ok(json!("")),
                _ => Err(format!("{}: expected text", self.key)),
            },
            SettingKind::Enum { options } => options
                .iter()
                .find(|(v, label)| {
                    v == value
                        || value
                            .as_str()
                            .is_some_and(|s| s.eq_ignore_ascii_case(label) || v.as_str().is_some_and(|v| v.eq_ignore_ascii_case(s)))
                })
                .map(|(v, _)| v.clone())
                .ok_or_else(|| {
                    let labels: Vec<&str> = options.iter().map(|(_, l)| l.as_str()).collect();
                    format!("{}: expected one of {}", self.key, labels.join(", "))
                }),
        }
    }
}

/// The ids of a list value: an array of strings, or `"a, b,c"`; trimmed,
/// empty ones and repeats dropped (the first stays).
pub fn list_ids(value: &Value) -> Vec<String> {
    let raw: Vec<String> = match value {
        Value::Array(items) => items.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
        Value::String(s) => s.trim().trim_matches('"').split(',').map(str::to_string).collect(),
        _ => Vec::new(),
    };
    let mut out: Vec<String> = Vec::new();
    for id in raw {
        let id = id.trim().to_string();
        if !id.is_empty() && !out.iter().any(|o| same_key(o, &id)) {
            out.push(id);
        }
    }
    out
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// A setting key or property name compared the way users type it: any
/// case, `_` ignored (`AutoScroll` = `auto_scroll`).
pub fn same_key(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.chars().filter(|c| *c != '_').flat_map(char::to_lowercase).collect::<String>();
    norm(a) == norm(b)
}
