//! The attribute resolver (decisions §30 "12f-a"): where an attribute the
//! show has never seen goes, decided from data — the shipped standard table
//! (`data/standard_attributes.json`, the 345 GDTF standard names) and the
//! templates' ordered rules (`data/attribute_templates.json`). Seeding copies
//! a template's groups and rules into the show; from then on the show's own
//! rules (`ShowData.attribute_rules`, editable, learned) decide.
//!
//! Steps, first that answers wins, each recorded with its confidence:
//! the layer knows it · a **learned** rule (someone's own choice) · **exact** (a standard name) · **family** (a family
//! rule) · **parent** (its main attribute's group) · **context** (its
//! activation group's members) · **hint** (a keyword rule, or the file's
//! Feature through the template) · **guess** (the fallback group). A file
//! string never creates a group.

use lumacore_fixtures::attribute_def::{
    ActivationGroup, AttributeDefinition, AttributeRule, EncoderBarLayout, EncoderButton, FeatureGroup,
    NaturalReadout, PhysicalUnit, PlaceHow, PlacedAttribute, Placement, ProfileAttributes, RuleKind,
};
use lumacore_fixtures::fixture_type::FileAttribute;
use crate::tree::ShowData;
use lumacore_base::{Annotation, Guid, IconRef, Identity, NodeMeta};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use uuid::Uuid;

/// The template a new show starts from.
pub const DEFAULT_TEMPLATE: &str = "LumaCore FX";

/// One attribute of the standard table.
#[derive(Debug, Clone, Deserialize)]
pub struct StdAttribute {
    pub name: String,
    pub pretty: String,
    /// `Group.Feature` in the standard grouping.
    pub feature: String,
    #[serde(default)]
    pub special: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub main: Option<String>,
    #[serde(default)]
    pub activation: Option<String>,
    #[serde(default)]
    pub readout: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StdActivation {
    name: String,
}

#[derive(Debug, Deserialize)]
struct StdFile {
    activation_groups: Vec<StdActivation>,
    attributes: Vec<StdAttribute>,
}

/// One group of a template.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateGroup {
    pub name: String,
    pub color: String,
    pub icon: String,
    pub features: Vec<String>,
}

/// A grouping template: its groups and where each rule class goes.
#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    pub name: String,
    pub groups: Vec<TemplateGroup>,
    /// Rule class → `Group.Feature`.
    pub classes: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct TemplatesFile {
    rules: Vec<(String, String, String)>,
    std_class: BTreeMap<String, String>,
    fallback: String,
    refine: BTreeMap<String, Vec<String>>,
    templates: Vec<Template>,
}

/// The shipped data, parsed once.
pub struct Standard {
    by_name: BTreeMap<String, StdAttribute>,
    activation_groups: Vec<String>,
    rules: Vec<(String, String, RuleKind)>,
    std_class: BTreeMap<String, String>,
    fallback: String,
    refine: BTreeMap<String, Vec<String>>,
    templates: Vec<Template>,
}

/// The standard table and the templates (embedded at build time).
pub fn standard() -> &'static Standard {
    static S: OnceLock<Standard> = OnceLock::new();
    S.get_or_init(|| {
        let f: StdFile = serde_json::from_str(include_str!("../data/standard_attributes.json"))
            .expect("data/standard_attributes.json parses");
        let t: TemplatesFile = serde_json::from_str(include_str!("../data/attribute_templates.json"))
            .expect("data/attribute_templates.json parses");
        Standard {
            by_name: f.attributes.into_iter().map(|a| (a.name.to_lowercase(), a)).collect(),
            activation_groups: f.activation_groups.into_iter().map(|a| a.name).collect(),
            rules: t
                .rules
                .into_iter()
                .map(|(p, c, k)| (p, c, if k == "keyword" { RuleKind::Keyword } else { RuleKind::Family }))
                .collect(),
            std_class: t.std_class,
            fallback: t.fallback,
            refine: t.refine,
            templates: t.templates,
        }
    })
}

impl Standard {
    /// A standard attribute by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&StdAttribute> {
        self.by_name.get(&name.to_lowercase())
    }
    pub fn len(&self) -> usize {
        self.by_name.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
    }
    pub fn template(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.name.eq_ignore_ascii_case(name))
    }
    pub fn template_names(&self) -> Vec<&str> {
        self.templates.iter().map(|t| t.name.as_str()).collect()
    }
}

/// Whether `name` matches `pattern`: `*` = any characters, `{n}` = one or
/// more digits, everything else literal; case-insensitive, whole name.
pub fn matches(pattern: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match p.first() {
            None => n.is_empty(),
            Some('*') => (0..=n.len()).any(|i| go(&p[1..], &n[i..])),
            Some('{') if p.len() >= 3 && p[1] == 'n' && p[2] == '}' => {
                let digits = n.iter().take_while(|c| c.is_ascii_digit()).count();
                (1..=digits).any(|i| go(&p[3..], &n[i..]))
            }
            Some(c) => n.first().is_some_and(|x| x.eq_ignore_ascii_case(c)) && go(&p[1..], &n[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    go(&p, &n)
}

/// A template group's Paint (decisions object-paint-palette-free-theme-links): the theme slot of
/// its name when LumaCore has one (each viewer's own colour), else the template's colour.
fn group_paint(name: &str, hex: &str) -> Option<lumacore_base::Paint> {
    if crate::media::is_builtin_slot(name) {
        return Some(lumacore_base::Paint::Theme(name.to_string()));
    }
    lumacore_base::paint::hex_rgb(hex).map(lumacore_base::Paint::Rgb)
}

fn unit_of(name: Option<&str>) -> PhysicalUnit {
    match name.unwrap_or("") {
        "Percent" => PhysicalUnit::Percent,
        "Length" => PhysicalUnit::Length,
        "Mass" => PhysicalUnit::Mass,
        "Time" => PhysicalUnit::Time,
        "Temperature" => PhysicalUnit::Temperature,
        "LuminousIntensity" => PhysicalUnit::LuminousIntensity,
        "Angle" => PhysicalUnit::Angle,
        "Force" => PhysicalUnit::Force,
        "Frequency" => PhysicalUnit::Frequency,
        "Current" => PhysicalUnit::Current,
        "Voltage" => PhysicalUnit::Voltage,
        "Power" => PhysicalUnit::Power,
        "Energy" => PhysicalUnit::Energy,
        "Area" => PhysicalUnit::Area,
        "Volume" => PhysicalUnit::Volume,
        "Speed" => PhysicalUnit::Speed,
        "Acceleration" => PhysicalUnit::Acceleration,
        "AngularSpeed" => PhysicalUnit::AngularSpeed,
        "AngularAcceleration" => PhysicalUnit::AngularAcceleration,
        "WaveLength" => PhysicalUnit::WaveLength,
        "ColorComponent" => PhysicalUnit::ColorComponent,
        _ => PhysicalUnit::None,
    }
}

// ─── Layers ──────────────────────────────────────────────────────────────────

/// Where one attribute is in a layer.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub group: Uuid,
    pub feature: Option<String>,
    pub position: u32,
    pub placement: Placement,
    pub new: bool,
}

/// A grouping of attributes: the show's (cue timing reads it) or a user
/// profile's own (decisions §30 "12f-a"). The resolver and every board / line
/// work on either through this.
pub trait AttributeLayer {
    fn groups(&self) -> &BTreeMap<u32, FeatureGroup>;
    fn groups_mut(&mut self) -> &mut BTreeMap<u32, FeatureGroup>;
    fn rules(&self) -> &BTreeMap<u32, AttributeRule>;
    fn rules_mut(&mut self) -> &mut BTreeMap<u32, AttributeRule>;
    fn template(&self) -> &str;
    fn set_template(&mut self, t: String);
    /// Every attribute the layer has placed.
    fn names(&self) -> Vec<String>;
    fn place_of(&self, name: &str) -> Option<Place>;
    /// Place an attribute (the show only places attributes it defines).
    fn set_place(&mut self, name: &str, p: Place);
}

impl AttributeLayer for ShowData {
    fn groups(&self) -> &BTreeMap<u32, FeatureGroup> {
        &self.feature_groups
    }
    fn groups_mut(&mut self) -> &mut BTreeMap<u32, FeatureGroup> {
        &mut self.feature_groups
    }
    fn rules(&self) -> &BTreeMap<u32, AttributeRule> {
        &self.attribute_rules
    }
    fn rules_mut(&mut self) -> &mut BTreeMap<u32, AttributeRule> {
        &mut self.attribute_rules
    }
    fn template(&self) -> &str {
        &self.show_settings.attribute_template
    }
    fn set_template(&mut self, t: String) {
        self.show_settings.attribute_template = t;
    }
    fn names(&self) -> Vec<String> {
        self.attribute_defs.keys().cloned().collect()
    }
    fn place_of(&self, name: &str) -> Option<Place> {
        let d = self.attribute_defs.get(name)?;
        Some(Place {
            group: d.feature_group?,
            feature: d.feature.clone(),
            position: d.position,
            placement: d.placement.clone(),
            new: d.new,
        })
    }
    fn set_place(&mut self, name: &str, p: Place) {
        if let Some(d) = self.attribute_defs.get_mut(name) {
            d.feature_group = Some(p.group);
            d.feature = p.feature;
            d.position = p.position;
            d.placement = p.placement;
            d.new = p.new;
        }
    }
}

impl AttributeLayer for ProfileAttributes {
    fn groups(&self) -> &BTreeMap<u32, FeatureGroup> {
        &self.groups
    }
    fn groups_mut(&mut self) -> &mut BTreeMap<u32, FeatureGroup> {
        &mut self.groups
    }
    fn rules(&self) -> &BTreeMap<u32, AttributeRule> {
        &self.rules
    }
    fn rules_mut(&mut self) -> &mut BTreeMap<u32, AttributeRule> {
        &mut self.rules
    }
    fn template(&self) -> &str {
        &self.template
    }
    fn set_template(&mut self, t: String) {
        self.template = t;
    }
    fn names(&self) -> Vec<String> {
        self.placed.keys().cloned().collect()
    }
    fn place_of(&self, name: &str) -> Option<Place> {
        let p = self.placed.get(name)?;
        Some(Place {
            group: p.group,
            feature: p.feature.clone(),
            position: p.position,
            placement: p.placement.clone(),
            new: p.new,
        })
    }
    fn set_place(&mut self, name: &str, p: Place) {
        let entry = self.placed.entry(name.to_string()).or_insert_with(|| PlacedAttribute {
            group: p.group,
            feature: None,
            position: 0,
            placement: Placement::default(),
            new: false,
            readout: None,
            resolution: None,
        });
        entry.group = p.group;
        entry.feature = p.feature;
        entry.position = p.position;
        entry.placement = p.placement;
        entry.new = p.new;
    }
}

/// What the resolver needs to know about the show besides a layer: the
/// attributes in use (with what their files say), each attribute's main
/// attribute and activation group (from the show's definitions).
#[derive(Debug, Clone, Default)]
pub struct Ctx {
    pub used: BTreeMap<String, FileAttribute>,
    pub main: BTreeMap<String, String>,
    pub activation: BTreeMap<String, String>,
}

impl Ctx {
    pub fn of(show: &ShowData) -> Self {
        let ag: BTreeMap<Uuid, &str> =
            show.activation_groups.values().map(|a| (a.meta.identity.guid, a.name.as_str())).collect();
        Ctx {
            used: used_attributes(show),
            main: show
                .attribute_defs
                .values()
                .filter_map(|d| d.main_attribute.clone().map(|m| (d.name.clone(), m)))
                .collect(),
            activation: show
                .attribute_defs
                .values()
                .filter_map(|d| d.activation_group.and_then(|g| ag.get(&g)).map(|n| (d.name.clone(), n.to_string())))
                .collect(),
        }
    }
}

/// The next free position at the end of `group`.
pub fn next_position<L: AttributeLayer + ?Sized>(layer: &L, group: Uuid) -> u32 {
    layer
        .names()
        .iter()
        .filter_map(|n| layer.place_of(n))
        .filter(|p| p.group == group)
        .map(|p| p.position + 1)
        .max()
        .unwrap_or(1)
}

/// Copy a template's groups and rules into a layer (a group of the same name
/// keeps its GUID; learned rules stay, re-pointed by group name) and place
/// every attribute it knows again by the new rules.
pub fn seed_layer<L: AttributeLayer + ?Sized>(layer: &mut L, ctx: &Ctx, template: &str) -> Result<(), String> {
    let std = standard();
    let t = std
        .template(template)
        .ok_or_else(|| format!("no attribute template \"{template}\" (there are {})", std.template_names().join(", ")))?;
    let old_names: BTreeMap<Uuid, String> = layer.groups().values().map(|g| (g.meta.identity.guid, g.meta.name().unwrap_or_default().to_string())).collect();
    let learned: Vec<AttributeRule> = layer.rules().values().filter(|r| r.learned).cloned().collect();
    let by_name: BTreeMap<String, Uuid> = layer.groups().values().map(|g| (g.meta.name().unwrap_or_default().to_lowercase(), g.meta.identity.guid)).collect();
    *layer.groups_mut() = t
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let id = i as u32 + 1;
            (
                id,
                FeatureGroup {
                    meta: NodeMeta {
                        identity: Identity { guid: by_name.get(&g.name.to_lowercase()).copied().unwrap_or_else(Uuid::new_v4), name: Some(g.name.clone()) },
                        annotation: Annotation { paint: group_paint(&g.name, &g.color), icon: Some(IconRef::Builtin(g.icon.clone())), ..Default::default() },
                    },
                    features: g.features.clone(),
                },
            )
        })
        .collect();
    let group_of = |groups: &BTreeMap<u32, FeatureGroup>, gf: &str| -> Option<(Uuid, Option<String>)> {
        let (g, f) = gf.split_once('.').unwrap_or((gf, ""));
        let grp = groups.values().find(|x| x.meta.name().unwrap_or_default().eq_ignore_ascii_case(g))?;
        Some((grp.meta.identity.guid, (!f.is_empty()).then(|| f.to_string())))
    };
    let mut template_rules = Vec::new();
    for (pattern, class, kind) in &std.rules {
        let Some(gf) = t.classes.get(class) else { continue };
        let Some((group, feature)) = group_of(layer.groups(), gf) else { continue };
        template_rules.push(AttributeRule {
            meta: NodeMeta::default(),
            pattern: pattern.clone(),
            group,
            feature,
            kind: *kind,
            class: Some(class.clone()),
            learned: false,
        });
    }
    let (family, keyword): (Vec<_>, Vec<_>) = template_rules.into_iter().partition(|r| r.kind == RuleKind::Family);
    let mut rules = family;
    for mut r in learned {
        let name = old_names.get(&r.group).cloned().unwrap_or_default();
        if let Some(g) = layer.groups().values().find(|g| g.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name)) {
            r.group = g.meta.identity.guid;
            rules.push(r);
        }
    }
    rules.extend(keyword);
    *layer.rules_mut() = numbered(rules);
    layer.set_template(t.name.clone());
    // Place everything known again by the new rules (parents looked up in
    // the old places), then number each group's attributes in name order.
    let names = layer.names();
    let mut fresh: Vec<(String, Resolved, bool)> = Vec::new();
    for n in &names {
        let old = layer.place_of(n);
        let file = ctx.used.get(n).cloned().unwrap_or_else(|| FileAttribute {
            name: n.clone(),
            feature: old.as_ref().and_then(|o| o.placement.file_feature.clone()),
            ..Default::default()
        });
        let r = resolve_in(layer, ctx, &file);
        fresh.push((n.clone(), r, old.is_some_and(|o| o.new)));
    }
    let mut next: BTreeMap<Uuid, u32> = BTreeMap::new();
    for (n, r, new) in fresh {
        let pos = next.entry(r.group).or_insert(0);
        *pos += 1;
        layer.set_place(&n, Place { group: r.group, feature: r.feature, position: *pos, placement: r.placement, new });
    }
    Ok(())
}

fn numbered(rules: Vec<AttributeRule>) -> BTreeMap<u32, AttributeRule> {
    rules
        .into_iter()
        .enumerate()
        .map(|(i, r)| (i as u32 + 1, r))
        .collect()
}

/// Seed the show: a template's groups and rules, and the standard activation
/// groups when the show has none.
pub fn seed(show: &mut ShowData, template: &str) -> Result<(), String> {
    if show.activation_groups.is_empty() {
        show.activation_groups = standard()
            .activation_groups
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let id = i as u32 + 1;
                (id, ActivationGroup { meta: NodeMeta::with_guid(Guid::named("ActivationGroup", n).0, None), name: n.clone() })
            })
            .collect();
    }
    let ctx = Ctx::of(show);
    seed_layer(show, &ctx, template)
}

/// Where an attribute goes.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    pub group: Uuid,
    pub feature: Option<String>,
    pub placement: Placement,
    /// The rule that placed it (its GUID), when a rule did (the Rules window's Which rule?).
    pub rule: Option<Uuid>,
}

/// A rule class's group: the first rule of that class (it follows the user's
/// edits), else the layer's template's class map by group name.
fn by_class<L: AttributeLayer + ?Sized>(layer: &L, class: &str) -> Option<(Uuid, Option<String>)> {
    if let Some(r) = layer.rules().values().find(|r| r.class.as_deref() == Some(class)) {
        return Some((r.group, r.feature.clone()));
    }
    let t = standard().template(layer.template())?;
    let gf = t.classes.get(class)?;
    let (g, f) = gf.split_once('.').unwrap_or((gf.as_str(), ""));
    let grp = layer.groups().values().find(|x| x.meta.name().unwrap_or_default().eq_ignore_ascii_case(g))?;
    Some((grp.meta.identity.guid, (!f.is_empty()).then(|| f.to_string())))
}

fn first_rule<'a, L: AttributeLayer + ?Sized>(layer: &'a L, name: &str, kind: RuleKind) -> Option<&'a AttributeRule> {
    layer.rules().values().filter(|r| r.kind == kind).find(|r| matches(&r.pattern, name))
}

/// Where the show's rules put `attr`.
pub fn resolve(show: &ShowData, attr: &FileAttribute) -> Resolved {
    resolve_in(show, &Ctx::of(show), attr)
}

/// Where a layer's rules put `attr` (a known attribute keeps its place;
/// call this for unknown ones). Never fails: the last step is a guess.
pub fn resolve_in<L: AttributeLayer + ?Sized>(layer: &L, ctx: &Ctx, attr: &FileAttribute) -> Resolved {
    let std = standard();
    let file_feature = attr.feature.clone();
    let done = |group: Uuid, feature: Option<String>, how: PlaceHow, why: String, rule: Option<Uuid>| Resolved {
        group,
        feature,
        placement: Placement { how, why, file_feature: file_feature.clone() },
        rule,
    };
    // A vendor name with spaces ("Pan Invert", "Reset all") is not a GDTF
    // name: it skips the exact and family steps and goes by keyword.
    let vendor = attr.name.chars().any(char::is_whitespace) && std.get(&attr.name).is_none();
    let family = if vendor { None } else { first_rule(layer, &attr.name, RuleKind::Family) };
    // A learned rule is someone's own choice: it beats the standard.
    if let Some(r) = layer
        .rules()
        .values()
        .filter(|r| r.learned)
        .find(|r| matches(&r.pattern, &attr.name))
    {
        return done(r.group, r.feature.clone(), PlaceHow::Family, format!("learned rule {}", r.pattern), Some(r.meta.identity.guid));
    }
    // Exact: a standard name keeps its standard class; a family rule may
    // only refine it (Beam → Shutter / Effects, Control → StrobeMode).
    if let Some(s) = std.get(&attr.name)
        && let Some(class) = std.std_class.get(&s.feature) {
            let refined = family
                .and_then(|r| r.class.as_ref())
                .filter(|c| std.refine.get(class).is_some_and(|ok| ok.contains(c)));
            let class = refined.unwrap_or(class);
            if let Some((g, f)) = by_class(layer, class) {
                return done(g, f, PlaceHow::Exact, format!("standard name ({})", s.feature), None);
            }
        }
    if let Some(r) = family {
        return done(r.group, r.feature.clone(), PlaceHow::Family, format!("family {}", r.pattern), Some(r.meta.identity.guid));
    }
    // Parent: its main attribute's group.
    let main = attr
        .main_attribute
        .clone()
        .or_else(|| std.get(&attr.name).and_then(|s| s.main.clone()))
        .or_else(|| ctx.main.get(&attr.name).cloned());
    if let Some(m) = main.as_deref()
        && let Some(p) = layer.place_of(m) {
            return done(p.group, p.feature, PlaceHow::Parent, format!("its main attribute {m}"), None);
        }
    // Context: a member of its activation group the layer knows.
    let ag = attr.activation_group.clone().or_else(|| ctx.activation.get(&attr.name).cloned());
    if let Some(ag) = ag.as_deref() {
        let sibling = ctx
            .activation
            .iter()
            .filter(|(n, a)| a.eq_ignore_ascii_case(ag) && **n != attr.name)
            .find_map(|(n, _)| layer.place_of(n).map(|p| (n.clone(), p)));
        if let Some((n, p)) = sibling {
            return done(p.group, p.feature, PlaceHow::Context, format!("activation group {ag} ({n})"), None);
        }
    }
    // Hint: a keyword, then the file's Feature through the template classes.
    if let Some(r) = first_rule(layer, &attr.name, RuleKind::Keyword) {
        let word = r.pattern.trim_matches('*');
        return done(r.group, r.feature.clone(), PlaceHow::Hint, format!("keyword \"{word}\""), Some(r.meta.identity.guid));
    }
    if let Some(ff) = attr.feature.as_deref()
        && let Some((g, f)) = std.std_class.get(ff).and_then(|c| by_class(layer, c)) {
            return done(g, f, PlaceHow::Hint, format!("the file says {ff}"), None);
        }
    let (g, f) = by_class(layer, &std.fallback)
        .or_else(|| layer.groups().values().next().map(|g| (g.meta.identity.guid, None)))
        .unwrap_or((Uuid::nil(), None));
    done(g, f, PlaceHow::Guess, "nothing matched".to_string(), None)
}

/// Every attribute name the patched fixtures use, with what their files say.
pub fn used_attributes(show: &ShowData) -> BTreeMap<String, FileAttribute> {
    let mut out: BTreeMap<String, FileAttribute> = BTreeMap::new();
    let types = &show.devices.dmx.patch.fixture_types;
    for fixture in show.devices.dmx.patch.fixtures.values() {
        let Some(ft) = types.values().find(|t| t.meta.identity.guid == fixture.mvr.fixture_type) else {
            continue;
        };
        let Some((rev, mode)) = ft.mode_of(fixture.mvr.revision, fixture.mvr.mode) else {
            continue;
        };
        for cf in &mode.channel_functions {
            let name = cf.attribute_name.clone();
            if name.is_empty() || out.contains_key(&name) {
                continue;
            }
            let file = rev
                .attributes
                .iter()
                .find(|a| a.name == name)
                .cloned()
                .unwrap_or_else(|| FileAttribute { name: name.clone(), ..Default::default() });
            out.insert(name, file);
        }
    }
    out
}

/// A profile's layer built from the show's (decisions §30 "12f-a": a new
/// profile's bar = the show's feature groups): the same groups (its own
/// GUIDs), rules and template, every attribute the show defines placed as
/// the show has it, and one layout "Default" with a strip button per group.
pub fn profile_from_show(show: &ShowData) -> ProfileAttributes {
    let mut p = ProfileAttributes { template: show.show_settings.attribute_template.clone(), ..Default::default() };
    let mut map: BTreeMap<Uuid, Uuid> = BTreeMap::new();
    for (id, g) in &show.feature_groups {
        let mut mine = g.clone();
        mine.meta.identity.guid = Uuid::new_v4();
        map.insert(g.meta.identity.guid, mine.meta.identity.guid);
        p.groups.insert(*id, mine);
    }
    p.rules = show
        .attribute_rules
        .iter()
        .filter_map(|(id, r)| {
            map.get(&r.group).map(|g| {
                let mut r = r.clone();
                r.meta.identity.guid = Uuid::new_v4();
                r.group = *g;
                (*id, r)
            })
        })
        .collect();
    for d in show.attribute_defs.values() {
        if let Some(g) = d.feature_group.and_then(|g| map.get(&g)) {
            p.placed.insert(
                d.name.clone(),
                PlacedAttribute {
                    group: *g,
                    feature: d.feature.clone(),
                    position: d.position,
                    placement: d.placement.clone(),
                    new: false,
                    readout: None,
                    resolution: None,
                },
            );
        }
    }
    p.encoder_bars.insert(1, default_layout(&p, "Default"));
    p
}

/// A layout with a strip button per group, its attributes in order.
pub fn default_layout(p: &ProfileAttributes, name: &str) -> EncoderBarLayout {
    EncoderBarLayout {
        meta: NodeMeta::new(Some(name.to_string())),
        buttons: p
            .groups
            .values()
            .map(|g| {
                let mut v: Vec<(&u32, &String)> =
                    p.placed.iter().filter(|(_, a)| a.group == g.meta.identity.guid).map(|(n, a)| (&a.position, n)).collect();
                v.sort();
                EncoderButton { group: g.meta.identity.guid, slots: v.into_iter().map(|(_, n)| Some(n.clone())).collect() }
            })
            .collect(),
    }
}

/// Put an attribute at the bottom of its group's strip button on every
/// layout (a layout without that group gets a button at the end).
pub fn add_to_layouts(p: &mut ProfileAttributes, name: &str, group: Uuid) {
    for bar in p.encoder_bars.values_mut() {
        for b in bar.buttons.iter_mut() {
            b.slots.retain(|s| s.as_deref() != Some(name));
        }
        match bar.buttons.iter_mut().find(|b| b.group == group) {
            Some(b) => b.slots.push(Some(name.to_string())),
            None => bar.buttons.push(EncoderButton { group, slots: vec![Some(name.to_string())] }),
        }
    }
}

/// Give a profile the attributes the show defines that it has never placed:
/// placed by its own rules, marked new (they wait at the suggested spot for
/// the user's answer; a locked profile isn't asked). Returns them.
pub fn push_to_profile(p: &mut ProfileAttributes, ctx: &Ctx, show_names: &[String], ask: bool) -> Vec<String> {
    let mut added = Vec::new();
    for n in show_names {
        if p.placed.contains_key(n) {
            continue;
        }
        let file = ctx.used.get(n).cloned().unwrap_or_else(|| FileAttribute { name: n.clone(), ..Default::default() });
        let r = resolve_in(p, ctx, &file);
        let position = next_position(p, r.group);
        p.set_place(n, Place { group: r.group, feature: r.feature, position, placement: r.placement, new: ask });
        add_to_layouts(p, n, r.group);
        added.push(n.clone());
    }
    added
}

/// Give every attribute the patched fixtures use a definition (the ones the
/// show doesn't know are placed by the resolver and marked new), seed every
/// profile that has no layer yet from the show as it was, and push the new
/// attributes into every profile (marked new: each user is asked). Seeds the
/// default template first when the show has no groups. Returns the new names.
pub fn sync(show: &mut ShowData) -> Vec<String> {
    let used = used_attributes(show);
    if used.keys().all(|n| show.attribute_defs.contains_key(n)) {
        return Vec::new();
    }
    if show.feature_groups.is_empty() {
        let _ = seed(show, DEFAULT_TEMPLATE);
    }
    // Profiles without a layer start from the show before the new arrive.
    let unseeded: Vec<u32> =
        show.user_profiles.iter().filter(|(_, p)| !p.content.attributes.is_seeded()).map(|(id, _)| *id).collect();
    for id in unseeded {
        let fresh = profile_from_show(show);
        std::sync::Arc::make_mut(show.user_profiles.get_mut(&id).unwrap()).content.attributes = fresh;
    }
    let std = standard();
    let mut added = Vec::new();
    // Main attributes before their sub-attributes, so "parent" can answer.
    let mut todo: Vec<FileAttribute> = used.values().filter(|a| !show.attribute_defs.contains_key(&a.name)).cloned().collect();
    todo.sort_by_key(|a| (a.main_attribute.is_some() || std.get(&a.name).is_some_and(|s| s.main.is_some()), a.name.clone()));
    let mut seen = BTreeSet::new();
    for file in todo {
        if !seen.insert(file.name.clone()) {
            continue;
        }
        let ctx = Ctx::of(show);
        let r = resolve_in(show, &ctx, &file);
        let s = std.get(&file.name);
        let activation = s
            .and_then(|s| s.activation.clone())
            .or_else(|| file.activation_group.clone())
            .and_then(|n| show.activation_groups.values().find(|a| a.name.eq_ignore_ascii_case(&n)))
            .map(|a| a.meta.identity.guid);
        let position = next_position(show, r.group);
        let def = AttributeDefinition {
            meta: NodeMeta::with_guid(Guid::named("Attribute", &file.name).0, None),
            name: file.name.clone(),
            pretty: s
                .map(|s| s.pretty.clone())
                .or_else(|| (!file.pretty.is_empty()).then(|| file.pretty.clone()))
                .unwrap_or_else(|| file.name.clone()),
            physical_unit: unit_of(s.and_then(|s| s.unit.as_deref()).or(file.physical_unit.as_deref())),
            natural_readout: if s.and_then(|s| s.readout.as_deref()) == Some("Physical") {
                NaturalReadout::Physical
            } else {
                NaturalReadout::Percent
            },
            feature_group: Some(r.group),
            activation_group: activation,
            main_attribute: s.and_then(|s| s.main.clone()).or_else(|| file.main_attribute.clone()),
            encoder_resolution: Default::default(),
            intensity: 0.0,
            mib: false,
            feature: r.feature,
            special: s.and_then(|s| s.special.clone()),
            position,
            placement: r.placement,
            new: true,
        };
        show.attribute_defs.insert(file.name.clone(), def);
        added.push(file.name.clone());
    }
    // Every profile learns of them (its own rules place them).
    let ctx = Ctx::of(show);
    let ids: Vec<u32> = show.user_profiles.keys().copied().collect();
    for id in ids {
        let prof = std::sync::Arc::make_mut(show.user_profiles.get_mut(&id).unwrap());
        let ask = !prof.locked;
        push_to_profile(&mut prof.content.attributes, &ctx, &added, ask);
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded(template: &str) -> ShowData {
        let mut s = ShowData::default();
        seed(&mut s, template).unwrap();
        s
    }

    fn group_name(show: &ShowData, r: &Resolved) -> String {
        show.feature_groups.values().find(|g| g.meta.identity.guid == r.group).unwrap().meta.name().unwrap_or_default().to_string()
    }

    fn file(name: &str) -> FileAttribute {
        FileAttribute { name: name.into(), ..Default::default() }
    }

    #[test]
    fn the_shipped_data_parses() {
        let s = standard();
        assert_eq!(s.len(), 345);
        assert_eq!(s.template_names(), vec!["LumaCore FX", "GDTF standard"]);
    }

    #[test]
    fn patterns_match_whole_names_with_digits() {
        assert!(matches("Shutter{n}*", "Shutter1StrobeRandom"));
        assert!(matches("Shutter{n}*", "shutter12"));
        assert!(!matches("Shutter{n}*", "ShutterStrobe"));
        assert!(matches("*MSpeed", "PixelMSpeed"));
        assert!(!matches("Pan*", "Expand"));
        assert!(matches("*reset*", "Reset all"));
    }

    #[test]
    fn every_standard_name_lands_exactly_in_both_templates() {
        for t in ["LumaCore FX", "GDTF standard"] {
            let show = seeded(t);
            for a in standard().by_name.values() {
                let r = resolve(&show, &file(&a.name));
                assert_eq!(r.placement.how, PlaceHow::Exact, "{t}: {}", a.name);
            }
        }
    }

    #[test]
    fn lumacore_fx_folds_gobo_beam_focus_and_splits_the_shutter() {
        let show = seeded("LumaCore FX");
        let g = |n: &str| group_name(&show, &resolve(&show, &file(n)));
        assert_eq!(g("Gobo1"), "FX");
        assert_eq!(g("Zoom"), "FX");
        assert_eq!(g("Iris"), "FX");
        assert_eq!(g("Shutter1"), "Shutter");
        assert_eq!(g("Shutter1StrobeRandom"), "Shutter");
        assert_eq!(g("StrobeMode"), "Shutter");
        assert_eq!(g("Dimmer"), "Dimmer");
        assert_eq!(g("MediaContent1"), "Video");
        assert_eq!(g("ColorAdd_R"), "Color");
        assert_eq!(g("Gobo1WheelMode"), "Control");
        let gd = seeded("GDTF standard");
        let g2 = |n: &str| group_name(&gd, &resolve(&gd, &file(n)));
        assert_eq!(g2("Gobo1"), "Gobo");
        assert_eq!(g2("Shutter1"), "Beam");
        assert_eq!(g2("Zoom"), "Focus");
    }

    #[test]
    fn unknown_names_go_by_family_then_hint_then_guess() {
        let show = seeded("LumaCore FX");
        let r = resolve(&show, &file("Shutter2StrobeBurst"));
        assert_eq!((group_name(&show, &r).as_str(), r.placement.how), ("Shutter", PlaceHow::Family));
        let r = resolve(&show, &file("PixelMSpeed"));
        assert_eq!((group_name(&show, &r).as_str(), r.placement.how), ("Control", PlaceHow::Family));
        let r = resolve(&show, &FileAttribute { feature: Some("Dimmer.Dimmer".into()), ..file("Pan Invert") });
        assert_eq!((group_name(&show, &r).as_str(), r.placement.how), ("Control", PlaceHow::Hint));
        let r = resolve(&show, &file("Pixel Pattern"));
        assert_eq!((group_name(&show, &r).as_str(), r.placement.how), ("FX", PlaceHow::Hint));
        let r = resolve(&show, &FileAttribute { feature: Some("CrowdLED.CrowdLED".into()), ..file("CrowdLED") });
        assert_eq!((group_name(&show, &r).as_str(), r.placement.how), ("Control", PlaceHow::Guess));
        // A file string never makes a group.
        assert_eq!(show.feature_groups.len(), 8);
    }

    #[test]
    fn a_sub_attribute_follows_its_main_attribute() {
        let mut show = seeded("LumaCore FX");
        let gobo = show.feature_groups.values().find(|g| g.meta.name() == Some("Control")).unwrap().meta.identity.guid;
        show.attribute_defs.insert(
            "VendorWheel".into(),
            AttributeDefinition {
                meta: NodeMeta::with_guid(Guid::named("Attribute", "VendorWheel").0, None),
                name: "VendorWheel".into(),
                pretty: "VW".into(),
                physical_unit: PhysicalUnit::None,
                natural_readout: NaturalReadout::Percent,
                feature_group: Some(gobo),
                activation_group: None,
                main_attribute: None,
                encoder_resolution: Default::default(),
                intensity: 0.0,
                mib: false,
                feature: None,
                special: None,
                position: 1,
                placement: Placement::default(),
                new: false,
            },
        );
        let r = resolve(&show, &FileAttribute { main_attribute: Some("VendorWheel".into()), ..file("VendorWheelSpinny") });
        assert_eq!(r.placement.how, PlaceHow::Parent);
        assert_eq!(r.group, gobo);
    }

    #[test]
    fn seeding_again_keeps_group_guids_and_learned_rules() {
        let mut show = seeded("LumaCore FX");
        let fx = show.feature_groups.values().find(|g| g.meta.name() == Some("FX")).unwrap().meta.identity.guid;
        let n = show.attribute_rules.len() as u32 + 1;
        show.attribute_rules.insert(
            n,
            AttributeRule {
                meta: NodeMeta::default(),
                pattern: "Pixel*".into(),
                group: fx,
                feature: None,
                kind: RuleKind::Family,
                class: None,
                learned: true,
            },
        );
        seed(&mut show, "LumaCore FX").unwrap();
        assert_eq!(show.feature_groups.values().find(|g| g.meta.name() == Some("FX")).unwrap().meta.identity.guid, fx);
        let learned: Vec<_> = show.attribute_rules.values().filter(|r| r.learned).collect();
        assert_eq!(learned.len(), 1);
        // Learned rules sit before the keyword rules.
        let no_of = |want: RuleKind, learned: bool| show.attribute_rules.iter().find(|(_, r)| r.kind == want && r.learned == learned).map(|(n, _)| *n).unwrap();
        assert!(no_of(RuleKind::Family, true) < no_of(RuleKind::Keyword, false));
    }

    fn placed_in(p: &ProfileAttributes, name: &str) -> String {
        let g = p.placed[name].group;
        p.groups.values().find(|x| x.meta.identity.guid == g).unwrap().meta.name().unwrap_or_default().to_string()
    }

    #[test]
    fn a_profile_starts_as_the_shows_grouping_and_learns_new_attributes_marked_new() {
        let mut show = seeded("LumaCore FX");
        let ctx = Ctx::default();
        // The show knows Gobo1 (FX); a profile copies that, with its own GUIDs.
        show.attribute_defs.insert("Gobo1".into(), def_in(&show, "Gobo1", "FX"));
        let mut p = profile_from_show(&show);
        assert_eq!(placed_in(&p, "Gobo1"), "FX");
        assert!(p.groups.values().all(|g| !show.feature_groups.values().any(|s| s.meta.identity.guid == g.meta.identity.guid)));
        assert_eq!(p.encoder_bars.len(), 1);
        // The user keeps Gobo apart: own group, own rule.
        let gobo = FeatureGroup { meta: NodeMeta::new(Some("Gobo".into())), features: vec![] };
        let gg = gobo.meta.identity.guid;
        p.groups.insert(99, gobo);
        p.placed.get_mut("Gobo1").unwrap().group = gg;
        let n = p.rules.len() as u32 + 1;
        p.rules.insert(n, AttributeRule { meta: NodeMeta::default(), pattern: "Gobo{n}*".into(), group: gg, feature: None, kind: RuleKind::Family, class: None, learned: true });
        p.rules = numbered({ let mut v: Vec<_> = p.rules.values().cloned().collect(); let l = v.pop().unwrap(); v.insert(0, l); v });
        // Knowledge wins; a new one is placed by the profile's rules, marked new.
        let added = push_to_profile(&mut p, &ctx, &["Gobo1".into(), "Gobo2PosShake".into()], true);
        assert_eq!(added, vec!["Gobo2PosShake".to_string()]);
        assert_eq!(placed_in(&p, "Gobo1"), "Gobo");
        assert_eq!(placed_in(&p, "Gobo2PosShake"), "Gobo");
        assert!(p.placed["Gobo2PosShake"].new);
        assert_eq!(p.waiting(), vec!["Gobo2PosShake"]);
        // It lands at the bottom of its strip button on the layout.
        let bar = &p.encoder_bars[&1];
        let b = bar.buttons.iter().find(|b| b.group == gg).unwrap();
        assert_eq!(b.slots.last().unwrap().as_deref(), Some("Gobo2PosShake"));
    }

    #[test]
    fn a_locked_profile_learns_without_being_asked() {
        let show = seeded("LumaCore FX");
        let mut p = profile_from_show(&show);
        push_to_profile(&mut p, &Ctx::default(), &["Zoom".into()], false);
        assert!(!p.placed["Zoom"].new);
    }

    fn def_in(show: &ShowData, name: &str, group: &str) -> AttributeDefinition {
        let g = show.feature_groups.values().find(|x| x.meta.name() == Some(group)).unwrap().meta.identity.guid;
        AttributeDefinition {
            meta: NodeMeta::with_guid(Guid::named("Attribute", name).0, None),
            name: name.into(),
            pretty: name.into(),
            physical_unit: PhysicalUnit::None,
            natural_readout: NaturalReadout::Percent,
            feature_group: Some(g),
            activation_group: None,
            main_attribute: None,
            encoder_resolution: Default::default(),
            intensity: 0.0,
            mib: false,
            feature: None,
            special: None,
            position: 1,
            placement: Placement::default(),
            new: false,
        }
    }
}
