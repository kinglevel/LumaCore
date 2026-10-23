// ██╗    ██╗██╗███╗   ██╗██████╗  ██████╗ ██╗    ██╗    ████████╗██╗   ██╗██████╗ ███████╗
// ██║    ██║██║████╗  ██║██╔══██╗██╔═══██╗██║    ██║    ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝
// ██║ █╗ ██║██║██╔██╗ ██║██║  ██║██║   ██║██║ █╗ ██║       ██║    ╚████╔╝ ██████╔╝█████╗
// ██║███╗██║██║██║╚██╗██║██║  ██║██║   ██║██║███╗██║       ██║     ╚██╔╝  ██╔═══╝ ██╔══╝
// ╚███╔███╔╝██║██║ ╚████║██████╔╝╚██████╔╝╚███╔███╔╝       ██║      ██║   ██║     ███████╗
//  ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═════╝  ╚═════╝  ╚══╝╚══╝        ╚═╝      ╚═╝   ╚═╝     ╚══════╝
// FILE: window type
//! What a window shows and how it is built (decisions §30 "12c hardening"): its name, label,
//! picker category, icon, keywords, settings, bar actions, the bar a new window shows, the bar's
//! minimum size, its live title and chips. Pure data, declared in Rust and sent as JSON — never in
//! a show; the catalog holds every type (decisions §34 21).
//!
//! Owns:          WindowType and its builders, ActionDef, ChipDef, BarMin, the window bar keys,
//!                section
//! Doesn't own:   the table of types (lumacore-tree), what a window shows live (lumacore-tree's
//!                WindowTypeView)
//! Entry points:  WindowType::new and its builders, setting, action, resolve_settings
//! Decisions:     §30 "12c", §34 21


use serde::Serialize;
use serde_json::{Value, json};

use crate::setting::{SettingDef, SettingKind, list_ids, same_key};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The window bar on / off (a Bool setting every visible type has).
pub const WINDOW_BAR: &str = "window_bar";
/// The window bar's items in order (a List setting every visible type has).
pub const WINDOW_BAR_ITEMS: &str = "window_bar_items";
/// The section (settings tab) of the two window bar settings.
pub const WINDOW_BAR_SECTION: &str = "Window bar";

/// A command a window type offers in its bar (decisions §30 "12c"): the line
/// uses `Self` (the window the bar sits in).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActionDef {
    /// Stable id, shares the id space with the setting keys.
    pub id: String,
    pub label: String,
    /// A kit icon name (the WebUI's `web/vendor/lucide/icons.txt`).
    pub icon: String,
    pub line: String,
    /// Lit / checked while this is true (`Self.AutoScroll`, `!Self.X`,
    /// `Self.View=input`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// An auto chip: shown only while `shown_when` is true (× clear a search).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChipDef {
    pub id: String,
    /// A template with `{Self.<Property>}` reads.
    pub label: String,
    pub shown_when: String,
    pub line: String,
}

/// Below this size (the window's content, px) the bar hides and the window
/// menu lists its items (the kit decides, there is no setting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BarMin {
    pub width_px: u32,
    pub height_px: u32,
}

impl Default for BarMin {
    fn default() -> Self {
        // About four rows of 30 px, the narrowest bar that shows one item and `…`.
        Self { width_px: 240, height_px: 120 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WindowType {
    /// Who it is under `Root.Resources.WindowTypes`: a GUID derived from its name, the same on
    /// every station and every start (decisions §12 "Tags part 1b hardened" 6).
    pub meta: lumacore_base::Identity,
    pub no: u32,
    /// Stable identifier used in layouts and views, e.g. "groups".
    pub name: String,
    pub label: String,
    /// Picker tab: "Common" / "Pools" / "Sheets" / "Tools" / "Others"
    /// (the Add Window categories).
    pub category: String,
    /// Icon for UI engines: a kit icon name (decisions §30: Lucide names plus
    /// LumaCore glyphs, the WebUI's `web/vendor/lucide/icons.txt`).
    pub icon: String,
    pub description: String,
    /// Extra search words for the picker filter.
    pub keywords: Vec<String>,
    /// Not offered in the window picker (e.g. the empty-pane placeholder).
    pub hidden: bool,
    pub settings: Vec<SettingDef>,
    /// Commands its bar offers (decisions §30 "12c").
    pub actions: Vec<ActionDef>,
    /// The bar a new window shows (setting keys and action ids, in order).
    pub bar_default: Vec<String>,
    pub bar_min: BarMin,
    /// The live title beside the tab: a template with `{Self.<Property>}`
    /// reads; empty = none.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    pub chips: Vec<ChipDef>,
}

impl WindowType {
    pub fn new(
        no: u32,
        name: &str,
        label: &str,
        category: &str,
        icon: &str,
        description: &str,
    ) -> Self {
        Self {
            meta: lumacore_base::Identity::named("WindowType", name, None),
            no,
            name: name.into(),
            label: label.into(),
            category: category.into(),
            icon: icon.into(),
            description: description.into(),
            keywords: Vec::new(),
            hidden: false,
            settings: Vec::new(),
            actions: Vec::new(),
            bar_default: Vec::new(),
            bar_min: BarMin::default(),
            title: String::new(),
            chips: Vec::new(),
        }
    }

    pub fn keywords(mut self, words: &[&str]) -> Self {
        self.keywords = words.iter().map(|w| w.to_string()).collect();
        self
    }

    pub fn settings(mut self, settings: Vec<SettingDef>) -> Self {
        self.settings = settings;
        self
    }

    pub fn actions(mut self, actions: &[(&str, &str, &str, &str, Option<&str>)]) -> Self {
        self.actions = actions
            .iter()
            .map(|(id, label, icon, line, state)| ActionDef {
                id: (*id).into(),
                label: (*label).into(),
                icon: (*icon).into(),
                line: (*line).into(),
                state: state.map(str::to_string),
            })
            .collect();
        self
    }

    pub fn chips(mut self, chips: &[(&str, &str, &str, &str)]) -> Self {
        self.chips = chips
            .iter()
            .map(|(id, label, shown_when, line)| ChipDef {
                id: (*id).into(),
                label: (*label).into(),
                shown_when: (*shown_when).into(),
                line: (*line).into(),
            })
            .collect();
        self
    }

    pub fn title(mut self, template: &str) -> Self {
        self.title = template.into();
        self
    }

    pub fn bar_min(mut self, width_px: u32, height_px: u32) -> Self {
        self.bar_min = BarMin { width_px, height_px };
        self
    }

    /// The window bar: shown by default or not, and its default items. Adds
    /// the two bar settings on their own tab (after every other setting).
    pub fn bar(mut self, on: bool, items: &[&str]) -> Self {
        self.bar_default = items.iter().map(|i| (*i).to_string()).collect();
        self.settings.extend(section(
            WINDOW_BAR_SECTION,
            vec![
                SettingDef::new(WINDOW_BAR, "Window bar", SettingKind::Bool, json!(on))
                    .hint("the row of buttons below the tabs"),
                SettingDef::new(WINDOW_BAR_ITEMS, "Bar items", SettingKind::List, json!(items))
                    .hint("setting keys and actions, left to right"),
            ],
        ));
        self
    }

    /// Ids that can sit in the bar: every setting but the bar's own two, then
    /// the actions.
    pub fn bar_ids(&self) -> Vec<&str> {
        self.settings
            .iter()
            .map(|s| s.key.as_str())
            .filter(|k| *k != WINDOW_BAR && *k != WINDOW_BAR_ITEMS)
            .chain(self.actions.iter().map(|a| a.id.as_str()))
            .collect()
    }

    pub fn action(&self, id: &str) -> Option<&ActionDef> {
        self.actions.iter().find(|a| same_key(&a.id, id))
    }

    /// Coerce `value` for `def`; a bar list keeps only ids of this type
    /// (`strict`: an unknown id is an error — a typed line; else dropped — a
    /// show load).
    pub fn coerce_setting(&self, def: &SettingDef, value: &Value, strict: bool) -> Result<Value, String> {
        let v = def.coerce(value)?;
        if !matches!(def.kind, SettingKind::List) {
            return Ok(v);
        }
        let known = self.bar_ids();
        let mut out = Vec::new();
        for id in list_ids(&v) {
            match known.iter().find(|k| same_key(k, &id)) {
                Some(k) => out.push((*k).to_string()),
                None if strict => {
                    return Err(format!(
                        "{}: '{id}' isn't an item of {}. Items: {}",
                        def.key,
                        self.label,
                        known.join(", ")
                    ));
                }
                None => {}
            }
        }
        Ok(json!(out))
    }

    /// The sections in order (the settings window's tabs).
    pub fn sections(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for s in &self.settings {
            if !out.contains(&s.section.as_str()) {
                out.push(&s.section);
            }
        }
        out
    }

    /// True when `query` names this type: its name or label, case-insensitive,
    /// ignoring spaces.
    pub fn matches(&self, query: &str) -> bool {
        let q: String = query.chars().filter(|c| !c.is_whitespace()).collect();
        let label: String = self.label.chars().filter(|c| !c.is_whitespace()).collect();
        self.name.eq_ignore_ascii_case(&q) || label.eq_ignore_ascii_case(&q)
    }

    /// A setting by key, as users type it (`AutoScroll` = `auto_scroll`).
    pub fn setting(&self, key: &str) -> Option<&SettingDef> {
        self.settings.iter().find(|s| same_key(&s.key, key))
    }

    /// Defaults merged with `stored`, every value coerced to its type.
    /// Unknown keys are dropped, and so are unknown bar item ids.
    pub fn resolve_settings(
        &self,
        stored: &serde_json::Map<String, Value>,
    ) -> serde_json::Map<String, Value> {
        self.settings
            .iter()
            .map(|def| {
                let v = stored
                    .get(&def.key)
                    .and_then(|v| self.coerce_setting(def, v, false).ok())
                    .unwrap_or_else(|| def.default.clone());
                (def.key.clone(), v)
            })
            .collect()
    }

    /// Stored settings that differ from the type's defaults (what a reset
    /// would change).
    pub fn changed_keys(&self, stored: &serde_json::Map<String, Value>) -> Vec<&str> {
        let resolved = self.resolve_settings(stored);
        self.settings
            .iter()
            .filter(|d| resolved.get(&d.key) != Some(&d.default))
            .map(|d| d.key.as_str())
            .collect()
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// Put `defs` on the settings window's tab `section`.
pub fn section(section: &str, defs: Vec<SettingDef>) -> Vec<SettingDef> {
    defs.into_iter()
        .map(|mut d| {
            d.section = section.to_string();
            d
        })
        .collect()
}
