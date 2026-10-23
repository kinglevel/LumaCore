//! GDTF file importer — derives a LumaCore FixtureType from a .gdtf file.
//!
//! GDTF native (decisions §29 "11b hardening"): the `.gdtf` file is the
//! source of truth and travels in the show; the FixtureType is a view derived
//! from it here — deterministically, so a show can re-derive it on load
//! ([`refresh_revision`]). Import is **warn only**: anything readable is
//! imported, spec problems become warnings (`Info FixtureType`).
//!
//! What is derived, per mode (walking from `DMXMode@Geometry`):
//! - **Cells**: every `GeometryReference` below the root whose template
//!   carries DMX channels is a cell; its channels are instanced at
//!   `Break@DMXOffset + Offset − 1` (a template channel's `Overwrite` break
//!   takes the reference's last break). Without references, sibling
//!   geometries carrying the same attributes are cells (plain nested
//!   geometries). Cells are ordered by DMX offset; positions are composed
//!   from the geometry matrices. Root-geometry channels are shared.
//! - Every channel: attribute, geometry, break, offset/bytes, default,
//!   physical range, `Highlight`, `Snap`, every ChannelFunction with its DMX
//!   and physical range, wheel/emitter/filter and channel sets.
//! - Relations (`Multiply` = master dimmer), the geometry tree, wheels,
//!   emitters and filters.

use crate::errors::{GdtfError, GdtfWarning};
use crate::resolver::VirtualChannelResolver;
use crate::validator::validate_din_spec;
use gdtf::GdtfFile;
use gdtf::dmx_mode::{DmxBreak, DmxChannel, DmxMode};
use gdtf::fixture_type::FixtureType as GdtfFixtureType;
use gdtf::geometry::{AnyGeometry, Geometry};
use gdtf::values::{DmxValue, Matrix};
use lumacore_base::NodeMeta;
use lumacore_fixtures::{AttributeType, ChannelFunction, FixtureCategory, FixtureType, Mode, Revision};
use lumacore_fixtures::fixture_type::{
    Cell, ChannelSet, Cie, Emitter, FileAttribute, Filter, GdtfFunction, GeometryNode, Relation, Wheel, WheelSlot,
};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

/// Import a .gdtf file: the derived `FixtureType` (one revision whose
/// `file_hash` is the file's SHA-256) and the spec warnings.
///
/// # Errors
/// - `GdtfError::Io` — file could not be opened
/// - `GdtfError::InvalidGdtf` — not a GDTF archive, no description.xml, no fixture type
pub fn import_gdtf(path: &Path) -> Result<(FixtureType, Vec<GdtfWarning>), GdtfError> {
    let bytes = std::fs::read(path).map_err(GdtfError::Io)?;
    import_gdtf_bytes(&bytes).map_err(|e| match e {
        GdtfError::InvalidGdtf(msg) => {
            GdtfError::InvalidGdtf(format!("Failed to parse GDTF file '{}': {msg}", path.display()))
        }
        other => other,
    })
}

/// The SHA-256 of a .gdtf file (hex): its identity in the show container.
pub fn file_hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Import a .gdtf file from its bytes.
pub fn import_gdtf_bytes(bytes: &[u8]) -> Result<(FixtureType, Vec<GdtfWarning>), GdtfError> {
    let gdtf_file = GdtfFile::new(Cursor::new(bytes.to_vec()))
        .map_err(|e| GdtfError::InvalidGdtf(e.to_string()))?;
    let mut warnings = validate_din_spec(&gdtf_file.description);
    let gdtf_ft = gdtf_file.description.fixture_types.first().ok_or_else(|| {
        GdtfError::InvalidGdtf("GDTF file contains no fixture types".to_string())
    })?;
    let gdtf_version = format!(
        "{}.{}",
        gdtf_file.description.data_version.major, gdtf_file.description.data_version.minor
    );
    let name = gdtf_ft.name.as_ref().map(|n| n.as_ref().to_string());
    let mut fixture_type = FixtureType::new(
        gdtf_ft.manufacturer.clone(),
        name,
        parse_fixture_category(&gdtf_ft.long_name),
    );
    // The file's own id (decisions §30 "12a2"): a nil id is no id.
    let id = uuid::Uuid::from_bytes(*gdtf_ft.fixture_type_id.as_bytes());
    fixture_type.gdtf_fixture_type_id = (!id.is_nil()).then_some(id);
    let revision = build_revision(gdtf_ft, gdtf_version, file_hash(bytes), &mut warnings);
    fixture_type.revisions.push(revision);
    Ok((fixture_type, warnings))
}

/// Derive `rev` again from its file, keeping the GUIDs and numbers of the
/// revision and its modes (matched by position) — the links into it stay.
pub fn refresh_revision(rev: &mut Revision, bytes: &[u8]) -> Result<Vec<GdtfWarning>, GdtfError> {
    let (ft, warnings) = import_gdtf_bytes(bytes)?;
    let Some(mut fresh) = ft.revisions.into_iter().next() else {
        return Ok(warnings);
    };
    fresh.meta = rev.meta.clone();
    for (new, old) in fresh.modes.iter_mut().zip(&rev.modes) {
        new.meta = old.meta.clone();
    }
    *rev = fresh;
    Ok(warnings)
}

fn build_revision(
    gdtf_ft: &GdtfFixtureType,
    gdtf_version: String,
    file_hash: String,
    warnings: &mut Vec<GdtfWarning>,
) -> Revision {
    let revision_name = gdtf_ft.name.as_ref().map(|n| n.as_ref().to_string());
    let mut revision = Revision {
        meta: NodeMeta::new(revision_name),
        gdtf_version,
        file_hash,
        modes: Vec::new(),
        geometries: gdtf_ft.geometries.iter().map(geometry_node).collect(),
        wheels: gdtf_ft.wheels.iter().map(wheel).collect(),
        emitters: gdtf_ft
            .physical_descriptions
            .emitters
            .iter()
            .map(emitter)
            .collect(),
        filters: gdtf_ft
            .physical_descriptions
            .filters
            .iter()
            .map(|f| Filter {
                name: name_of(f.name.as_ref()),
                color: cie(&f.color),
            })
            .collect(),
        attributes: gdtf_ft
            .attribute_definitions
            .attributes
            .iter()
            .filter_map(|a| {
                let name = a.name.as_ref()?.as_ref().to_string();
                Some(FileAttribute {
                    name,
                    pretty: a.pretty.clone(),
                    feature: a.feature.as_ref().map(|n| n.to_string()),
                    main_attribute: a.main_attribute.as_ref().map(|n| n.to_string()),
                    activation_group: a.activation_group.as_ref().map(|n| n.to_string()),
                    physical_unit: Some(format!("{:?}", a.physical_unit)),
                })
            })
            .collect(),
    };
    for gdtf_mode in &gdtf_ft.dmx_modes {
        let mode_name = gdtf_mode.name.as_ref().map(|n| n.as_ref().to_string());
        let mode = build_mode(gdtf_ft, gdtf_mode, mode_name, warnings);
        revision.modes.push(mode);
    }
    revision
}

fn name_of(n: Option<&gdtf::values::Name>) -> String {
    n.map(|n| n.as_ref().to_string()).unwrap_or_default()
}

fn cie(c: &gdtf::values::ColorCie) -> Cie {
    Cie {
        x: c.x,
        y: c.y,
        big_y: c.z,
    }
}

fn wheel(w: &gdtf::wheel::Wheel) -> Wheel {
    use gdtf::wheel::WheelSlotOptic;
    Wheel {
        name: name_of(w.name.as_ref()),
        slots: w
            .slots
            .iter()
            .map(|s| {
                let (color, filter) = match &s.optic {
                    WheelSlotOptic::Color(c) => (Some(cie(c)), None),
                    WheelSlotOptic::Filter(node) => (None, Some(node.to_string())),
                };
                WheelSlot {
                    name: name_of(s.name.as_ref()),
                    color,
                    filter,
                    media: s.media_name.clone(),
                }
            })
            .collect(),
    }
}

fn emitter(e: &gdtf::physical_descriptions::Emitter) -> Emitter {
    use gdtf::physical_descriptions::EmitterOptic;
    let (color, dominant_wavelength) = match &e.optic {
        EmitterOptic::Color {
            color,
            dominant_wave_length,
        } => (Some(cie(color)), *dominant_wave_length),
        #[allow(unreachable_patterns)]
        _ => (None, None),
    };
    Emitter {
        name: name_of(e.name.as_ref()),
        color,
        dominant_wavelength,
    }
}

/// The GDTF element name of a geometry.
fn geometry_kind(g: &Geometry) -> &'static str {
    match g {
        Geometry::Generic(_) => "Geometry",
        Geometry::Axis(_) => "Axis",
        Geometry::FilterBeam(_) => "FilterBeam",
        Geometry::FilterColor(_) => "FilterColor",
        Geometry::FilterGobo(_) => "FilterGobo",
        Geometry::FilterShaper(_) => "FilterShaper",
        Geometry::Beam(_) => "Beam",
        Geometry::MediaServerLayer(_) => "MediaServerLayer",
        Geometry::MediaServerCamera(_) => "MediaServerCamera",
        Geometry::MediaServerMaster(_) => "MediaServerMaster",
        Geometry::Display(_) => "Display",
        Geometry::Reference(_) => "GeometryReference",
        Geometry::Laser(_) => "Laser",
        Geometry::WiringObject(_) => "WiringObject",
        Geometry::Inventory(_) => "Inventory",
        Geometry::Structure(_) => "Structure",
        Geometry::Support(_) => "Support",
        Geometry::Magnet(_) => "Magnet",
    }
}

/// A geometry's position matrix (row-major 4×4). The `gdtf` crate keeps it
/// private and writes it as `{a,b,c,d}{…}{…}{…}`.
fn matrix(g: &Geometry) -> [f64; 16] {
    let m: Option<&Matrix> = match g {
        Geometry::Generic(x) => Some(&x.position),
        Geometry::Axis(x) => Some(&x.position),
        Geometry::FilterBeam(x) => Some(&x.position),
        Geometry::FilterColor(x) => Some(&x.position),
        Geometry::FilterGobo(x) => Some(&x.position),
        Geometry::FilterShaper(x) => Some(&x.position),
        Geometry::Reference(x) => Some(&x.position),
        _ => None,
    };
    let mut out = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];
    if let Some(text) = m.and_then(|m| serde_json::to_value(m).ok())
        && let Some(text) = text.as_str()
    {
        let nums: Vec<f64> = text
            .split(['{', '}', ','])
            .filter_map(|t| t.trim().parse().ok())
            .collect();
        if nums.len() == 16 {
            out.copy_from_slice(&nums);
        }
    }
    out
}

fn multiply(a: &[f64; 16], b: &[f64; 16]) -> [f64; 16] {
    let mut out = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            out[r * 4 + c] = (0..4).map(|k| a[r * 4 + k] * b[k * 4 + c]).sum();
        }
    }
    out
}

fn geometry_node(g: &Geometry) -> GeometryNode {
    let (reference, breaks) = match g {
        Geometry::Reference(r) => (
            r.geometry.as_ref().map(|n| n.as_ref().to_string()),
            r.breaks
                .iter()
                .map(|b| (b.dmx_break, b.dmx_offset.absolute()))
                .collect(),
        ),
        _ => (None, Vec::new()),
    };
    GeometryNode {
        name: name_of(g.name()),
        kind: geometry_kind(g).to_string(),
        position: matrix(g).to_vec(),
        reference,
        breaks,
        children: g.children().iter().map(geometry_node).collect(),
    }
}

/// A GeometryReference below a mode's root, with its composed position.
struct Reference {
    name: String,
    template: String,
    breaks: Vec<(u8, u32)>,
    position: [f64; 3],
}

fn collect_references(g: &Geometry, parent: &[f64; 16], out: &mut Vec<Reference>) {
    let here = multiply(parent, &matrix(g));
    if let Geometry::Reference(r) = g
        && let Some(template) = &r.geometry
    {
        out.push(Reference {
            name: name_of(r.name.as_ref()),
            template: template.as_ref().to_string(),
            breaks: r
                .breaks
                .iter()
                .map(|b| (b.dmx_break, b.dmx_offset.absolute()))
                .collect(),
            position: [here[3], here[7], here[11]],
        });
    }
    for child in g.children() {
        collect_references(child, &here, out);
    }
}

/// The position of a named geometry below `g` (composed), if it is there.
fn find_position(g: &Geometry, parent: &[f64; 16], name: &str) -> Option<[f64; 3]> {
    let here = multiply(parent, &matrix(g));
    if g.name().is_some_and(|n| n.as_ref() == name) {
        return Some([here[3], here[7], here[11]]);
    }
    g.children().iter().find_map(|c| find_position(c, &here, name))
}

/// A DMX value as a fraction of its own resolution.
fn fraction(v: DmxValue) -> f64 {
    let bits = u32::from(v.bytes().get()) * 8;
    let max = 2f64.powi(bits as i32) - 1.0;
    (v.value() as f64 / max).clamp(0.0, 1.0)
}

/// A cell while it is found: (template, first DMX offset, position).
type CellAt = (String, u16, [f64; 3]);

const IDENTITY: [f64; 16] = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];

fn build_mode(
    gdtf_ft: &GdtfFixtureType,
    gdtf_mode: &DmxMode,
    mode_name: Option<String>,
    warnings: &mut Vec<GdtfWarning>,
) -> Mode {
    let label = mode_name.clone().unwrap_or_else(|| "<unnamed>".to_string());
    let root_name = gdtf_mode.geometry.as_ref().map(|n| n.as_ref().to_string()).unwrap_or_default();
    let root = gdtf_ft
        .geometries
        .iter()
        .find(|g| g.name().is_some_and(|n| n.as_ref() == root_name));
    let mut references = Vec::new();
    if let Some(root) = root {
        collect_references(root, &IDENTITY, &mut references);
    } else {
        warnings.push(GdtfWarning::new(
            "IMPORT-MODE-GEOMETRY",
            format!("DMXMode[@Name='{label}']"),
            format!("Mode geometry '{root_name}' not found among the top-level geometries"),
        ));
    }

    let mut channel_functions: Vec<ChannelFunction> = Vec::new();
    // Cells from GeometryReferences: (reference name) -> first offset.
    let mut cells: BTreeMap<String, CellAt> = BTreeMap::new();
    for gdtf_channel in &gdtf_mode.dmx_channels {
        let geometry = gdtf_channel.geometry.as_ref().to_string();
        let instances: Vec<&Reference> =
            references.iter().filter(|r| r.template == geometry).collect();
        let Some(base) = channel_function(gdtf_channel, &geometry, warnings) else {
            continue;
        };
        if instances.is_empty() {
            let mut cf = base;
            if let DmxBreak::Value(b) = gdtf_channel.dmx_break {
                cf.dmx_break = b.clamp(1, 255) as u8;
            }
            channel_functions.push(cf);
            continue;
        }
        for r in instances {
            let mut cf = base.clone();
            cf.geometry = r.name.clone();
            let brk = match gdtf_channel.dmx_break {
                DmxBreak::Value(b) => r.breaks.iter().find(|(n, _)| i32::from(*n) == b),
                DmxBreak::Overwrite => r.breaks.last(),
            }
            .or(r.breaks.first())
            .copied()
            .unwrap_or((1, 1));
            cf.dmx_break = brk.0;
            if !cf.is_virtual {
                cf.dmx_offset = cf.dmx_offset.saturating_add(brk.1.saturating_sub(1) as u16);
                let entry = cells
                    .entry(r.name.clone())
                    .or_insert((r.template.clone(), cf.dmx_offset, r.position));
                entry.1 = entry.1.min(cf.dmx_offset);
            }
            channel_functions.push(cf);
        }
    }

    // Plain nested geometries: siblings carrying the same attributes.
    if cells.is_empty() {
        let mut by_geometry: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for cf in channel_functions.iter().filter(|cf| !cf.is_virtual && cf.geometry != root_name) {
            by_geometry
                .entry(cf.geometry.clone())
                .or_default()
                .push(cf.attribute_name.clone());
        }
        let mut groups: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
        for (g, mut attrs) in by_geometry {
            attrs.sort();
            groups.entry(attrs).or_default().push(g);
        }
        if let Some((_, geometries)) = groups.into_iter().filter(|(_, g)| g.len() >= 2).max_by_key(|(_, g)| g.len()) {
            for g in geometries {
                let first = channel_functions
                    .iter()
                    .filter(|cf| cf.geometry == g && !cf.is_virtual)
                    .map(|cf| cf.dmx_offset)
                    .min()
                    .unwrap_or(0);
                let position = root
                    .and_then(|root| find_position(root, &IDENTITY, &g))
                    .unwrap_or_default();
                cells.insert(g.clone(), (g, first, position));
            }
        }
    }

    // Order cells by DMX offset and point their channels at them.
    let mut ordered: Vec<(String, CellAt)> = cells.into_iter().collect();
    ordered.sort_by_key(|(name, (_, offset, _))| (*offset, name.clone()));
    let cells: Vec<Cell> = ordered
        .into_iter()
        .enumerate()
        .map(|(i, (name, (template, dmx_offset, position)))| Cell {
            index: i as u32 + 1,
            geometry: name,
            template,
            dmx_offset,
            position,
        })
        .collect();
    for cf in &mut channel_functions {
        cf.cell = cells
            .iter()
            .position(|c| c.geometry == cf.geometry)
            .map(|i| i as u32);
    }
    channel_functions.sort_by_key(|cf| (cf.is_virtual, cf.dmx_break, cf.dmx_offset));

    // The footprint of the first break: its highest slot.
    let dmx_footprint = channel_functions
        .iter()
        .filter(|cf| !cf.is_virtual && cf.dmx_break == 1)
        .map(|cf| cf.dmx_offset + u16::from(cf.byte_count))
        .max()
        .unwrap_or(0);

    let relations = gdtf_mode
        .relations
        .iter()
        .map(|r| Relation {
            name: name_of(r.name.as_ref()),
            master: r.master.to_string(),
            follower: r.follower.to_string(),
            kind: format!("{:?}", r.type_),
        })
        .collect();

    let mut mode = Mode {
        meta: NodeMeta::new(mode_name),
        channel_functions,
        dmx_footprint,
        geometry: root_name,
        cells,
        relations,
    };

    let result = VirtualChannelResolver::resolve(&mut mode);
    if result.unresolved > 0 {
        warnings.push(GdtfWarning::new(
            "IMPORT-VIRTUAL-UNRESOLVED",
            format!("DMXMode[@Name='{label}']"),
            format!("{} virtual channel(s) unresolved in this mode", result.unresolved),
        ));
    }
    mode
}

/// One GDTF DMX channel as a LumaCore channel (offset relative to its
/// geometry, break 1); None when it has no logical channel.
fn channel_function(
    gdtf_channel: &DmxChannel,
    geometry: &str,
    warnings: &mut Vec<GdtfWarning>,
) -> Option<ChannelFunction> {
    let is_virtual = gdtf_channel.offset.is_none();
    let (dmx_offset, byte_count) = match &gdtf_channel.offset {
        Some(offsets) if !offsets.is_empty() => {
            ((offsets[0].max(1) - 1) as u16, offsets.len().min(255) as u8)
        }
        _ => (0u16, 1u8),
    };
    let logical = gdtf_channel.logical_channels.first()?;
    let attribute_name = logical
        .attribute
        .first()
        .map(|n| n.as_ref().to_string())
        .unwrap_or_else(|| "NoFeature".to_string());
    if gdtf_channel.logical_channels.len() > 1 {
        warnings.push(GdtfWarning::new(
            "IMPORT-MULTI-LOGICAL-CHANNEL",
            format!("DMXChannel[@Geometry='{geometry}']"),
            format!(
                "Channel '{attribute_name}' has {} logical channels. Only the first is used.",
                gdtf_channel.logical_channels.len()
            ),
        ));
    }
    let first = logical.channel_functions.first();
    let default_value = first.map(|cf| cf.default.value().min(255) as u8).unwrap_or(0);
    let default_fraction = first.map(|cf| fraction(cf.default));
    let physical_range = match logical.channel_functions.as_slice() {
        [cf] if cf.physical_from != cf.physical_to => Some((cf.physical_from, cf.physical_to)),
        _ => None,
    };
    let functions = logical
        .channel_functions
        .iter()
        .map(|cf| GdtfFunction {
            name: name_of(cf.name.as_ref()),
            attribute: cf.attribute.to_string(),
            dmx_from: fraction(cf.dmx_from),
            physical_from: cf.physical_from,
            physical_to: cf.physical_to,
            wheel: cf.wheel.as_ref().map(|n| n.to_string()),
            emitter: cf.emitter.as_ref().map(|n| n.to_string()),
            filter: cf.filter.as_ref().map(|n| n.to_string()),
            channel_sets: cf
                .channel_sets
                .iter()
                .map(|set| ChannelSet {
                    name: name_of(set.name.as_ref()),
                    dmx_from: fraction(set.dmx_from),
                    physical_from: set.physical_from,
                    physical_to: set.physical_to,
                    wheel_slot: set.wheel_slot_index,
                })
                .collect(),
        })
        .collect();
    Some(ChannelFunction {
        attribute_type: attribute_name_to_type(&attribute_name),
        attribute_name,
        dmx_offset,
        byte_count,
        default_value,
        default_fraction,
        physical_range,
        is_virtual,
        resolved: !is_virtual,
        geometry: geometry.to_string(),
        cell: None,
        dmx_break: 1,
        highlight: gdtf_channel.highlight.map(fraction),
        snap: logical.snap,
        functions,
    })
}

/// Map a GDTF attribute name to a LumaCore `AttributeType`.
///
/// Uses a simple prefix-matching strategy. Unknown attributes become
/// `AttributeType::Unknown(name)`.
fn attribute_name_to_type(name: &str) -> AttributeType {
    match name {
        "Dimmer" => AttributeType::Dimmer,
        "Shutter" | "StrobeMode" => AttributeType::Shutter,
        "ColorAdd_R" | "ColorSub_R" => AttributeType::ColorRed,
        "ColorAdd_G" | "ColorSub_G" => AttributeType::ColorGreen,
        "ColorAdd_B" | "ColorSub_B" => AttributeType::ColorBlue,
        "ColorAdd_W" => AttributeType::ColorWhite,
        "ColorAdd_A" => AttributeType::ColorAmber,
        "Pan" | "PanRotate" => AttributeType::Pan,
        "Tilt" | "TiltRotate" => AttributeType::Tilt,
        "Zoom" | "ZoomMSpeed" => AttributeType::Zoom,
        "Focus" | "FocusMSpeed" | "AutoFocus" => AttributeType::Focus,
        "Iris" | "IrisMSpeed" | "IrisMode" => AttributeType::Iris,
        "Frost" | "FrostMSpeed" => AttributeType::Frost,
        "NoFeature" => AttributeType::NoFeature,
        _ if name.starts_with("Gobo") => AttributeType::Gobo,
        _ if name.starts_with("ColorAdd_") || name.starts_with("ColorSub_") => AttributeType::Color,
        _ if name.starts_with("Color") => AttributeType::Color,
        _ if name.starts_with("Strobe") => AttributeType::Shutter,
        _ if name.starts_with("Effects") || name.starts_with("Animation") => AttributeType::Effect,
        _ if name.starts_with("Control") || name.starts_with("Media") => AttributeType::Control,
        _ if name.starts_with("Speed") || name.contains("MSpeed") => AttributeType::Speed,
        _ if name.starts_with("Macro") => AttributeType::Macro,
        _ => AttributeType::Unknown(name.to_string()),
    }
}

/// Parse a fixture category from GDTF metadata.
///
/// GDTF does not have an explicit "category" field but the `LongName` often
/// contains keywords. Falls back to `Other("Unknown")` if no match.
fn parse_fixture_category(long_name: &str) -> FixtureCategory {
    let lower = long_name.to_lowercase();
    if lower.contains("spot") {
        FixtureCategory::Spot
    } else if lower.contains("wash") {
        FixtureCategory::Wash
    } else if lower.contains("beam") {
        FixtureCategory::Beam
    } else if lower.contains("strobe") {
        FixtureCategory::Strobe
    } else if lower.contains("effect") {
        FixtureCategory::Effect
    } else if lower.contains("led") || lower.contains("rgb") {
        FixtureCategory::Led
    } else if lower.contains("conventional") || lower.contains("profile") || lower.contains("par ")
    {
        FixtureCategory::Conventional
    } else {
        FixtureCategory::Other("Unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_path(name: &str) -> PathBuf {
        PathBuf::from(env!("LUMACORE_ROOT"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn import_virtual_channel_gdtf_returns_fixture_type() {
        // Use virtual_channel.gdtf as the primary "happy path" test fixture.
        // Note: allWorking.gdtf from libMVRgdtf contains "-nan(ind)" in wheel slot
        // color data which the gdtf crate (correctly) rejects as invalid XML.
        // virtual_channel.gdtf is clean and tests the two-pass resolver.
        let path = fixtures_path("virtual_channel.gdtf");
        let result = import_gdtf(&path);
        assert!(
            result.is_ok(),
            "virtual_channel.gdtf should import successfully: {:?}",
            result.err()
        );

        let (ft, warnings) = result.unwrap();
        // Should have at least one revision
        assert!(
            !ft.revisions.is_empty(),
            "should have at least one revision"
        );
        let revision = ft.latest_revision().unwrap();
        // Should have at least one mode
        assert!(
            !revision.modes.is_empty(),
            "revision should have at least one mode"
        );

        // Log any warnings for inspection (not a failure)
        for w in &warnings {
            println!("Warning: {w}");
        }
    }

    #[test]
    fn a_nan_wheel_colour_reads_as_white_not_a_refusal() {
        // allWorking.gdtf (libMVRgdtf) writes "-nan(ind)" in a WheelSlot colour; the vendored
        // reader (vendor/gdtf, decisions §40 1) takes it as white instead of refusing the file.
        let (ft, _) = import_gdtf(&fixtures_path("allWorking.gdtf")).expect("reads with the lenient wheel colour");
        assert!(!ft.revisions.is_empty());
    }

    #[test]
    fn a_stripped_capture_export_reads_with_the_specs_defaults() {
        // Capture's export of a type: one `Dummy` attribute without PhysicalUnit, a FeatureGroup
        // and a FixtureType without the strings the strict crate demanded (decisions §40 2).
        let (ft, _) = import_gdtf(&fixtures_path("capture-stripped.gdtf")).expect("reads");
        assert_eq!(ft.manufacturer, "Martin");
        let rev = &ft.revisions[0];
        assert_eq!(rev.modes.len(), 1, "{:?}", rev.modes.iter().map(|m| m.meta.name()).collect::<Vec<_>>());
        assert_eq!(rev.modes[0].dmx_footprint, 27);
    }

    #[test]
    fn import_xml_broken_gdtf_returns_error() {
        let path = fixtures_path("XMLBroken.gdtf");
        let result = import_gdtf(&path);
        // XMLBroken.gdtf should fail with an error, not a panic
        assert!(
            result.is_err(),
            "XMLBroken.gdtf should return an error, got a successful import"
        );
        // Should be InvalidGdtf or MalformedXml, not a panic
        match result.unwrap_err() {
            GdtfError::InvalidGdtf(_) | GdtfError::MalformedXml(_) => {}
            other => panic!("Expected InvalidGdtf or MalformedXml, got: {other:?}"),
        }
    }

    #[test]
    fn import_virtual_channel_gdtf_resolves_virtual_channels() {
        let path = fixtures_path("virtual_channel.gdtf");
        let result = import_gdtf(&path);
        assert!(
            result.is_ok(),
            "virtual_channel.gdtf should import (possibly with warnings): {:?}",
            result.err()
        );

        let (ft, warnings) = result.unwrap();
        // Log warnings
        for w in &warnings {
            println!("Warning: {w}");
        }

        // Should have revisions
        assert!(!ft.revisions.is_empty());
        let revision = ft.latest_revision().unwrap();
        assert!(!revision.modes.is_empty(), "should have at least one mode");

        // Check that at least some channels were processed
        let first_mode = &revision.modes[0];
        println!(
            "Mode '{}' has {} channel functions",
            first_mode.meta.identity.name.as_deref().unwrap_or("<unnamed>"),
            first_mode.channel_functions.len()
        );
    }

    #[test]
    fn import_nonexistent_file_returns_io_error() {
        let path = PathBuf::from("/nonexistent/path/fixture.gdtf");
        let result = import_gdtf(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GdtfError::Io(_)));
    }

    #[test]
    fn import_virtual_channel_gdtf_produces_correct_footprint() {
        let path = fixtures_path("virtual_channel.gdtf");
        let (ft, _) = import_gdtf(&path).expect("virtual_channel.gdtf should import");

        let revision = ft.latest_revision().unwrap();
        for mode in &revision.modes {
            // footprint should be positive for any non-trivial mode
            println!(
                "Mode '{}': footprint={}, channels={}",
                mode.meta.identity.name.as_deref().unwrap_or("<unnamed>"),
                mode.dmx_footprint,
                mode.channel_functions.len()
            );
            // A mode with channels should have a positive footprint (unless all virtual)
            let physical_count = mode
                .channel_functions
                .iter()
                .filter(|cf| !cf.is_virtual)
                .count();
            if physical_count > 0 {
                assert!(
                    mode.dmx_footprint > 0,
                    "mode with physical channels should have positive footprint"
                );
            }
        }
    }

    #[test]
    fn geometry_references_become_cells_at_their_break_offset() {
        let (ft, _) = import_gdtf(&fixtures_path("virtual_channel.gdtf")).unwrap();
        let rev = ft.latest_revision().unwrap();
        assert_eq!(rev.file_hash.len(), 64, "SHA-256 hex");
        let mode = &rev.modes[0];
        assert_eq!(mode.geometry, "Body");
        assert_eq!(mode.dmx_footprint, 2);
        assert_eq!(mode.cells.len(), 1);
        assert_eq!((mode.cells[0].geometry.as_str(), mode.cells[0].dmx_offset), ("Reference 1", 1));
        let cell_dimmer = mode
            .channel_functions
            .iter()
            .find(|cf| cf.geometry == "Reference 1")
            .unwrap();
        assert_eq!((cell_dimmer.dmx_offset, cell_dimmer.cell), (1, Some(0)));
        let body = mode.channel_functions.iter().find(|cf| cf.geometry == "Body").unwrap();
        assert_eq!(body.highlight, Some(1.0), "Highlight=\"255/1\"");
        assert!(!rev.geometries.is_empty());
    }

    #[test]
    fn highlight_snap_sets_wheels_and_emitters_are_kept() {
        let (ft, _) = import_gdtf(&fixtures_path("AuraXB.gdtf")).unwrap();
        let rev = ft.latest_revision().unwrap();
        assert!(!rev.wheels.is_empty() && !rev.emitters.is_empty());
        let mode = &rev.modes[0];
        assert!(mode.channel_functions.iter().any(|cf| cf.highlight.is_some()));
        assert!(mode.channel_functions.iter().any(|cf| cf.snap));
        assert!(
            mode.channel_functions
                .iter()
                .flat_map(|cf| &cf.functions)
                .any(|f| !f.channel_sets.is_empty())
        );
    }

    #[test]
    fn refreshing_a_revision_keeps_its_guids() {
        let bytes = std::fs::read(fixtures_path("virtual_channel.gdtf")).unwrap();
        let (ft, _) = import_gdtf_bytes(&bytes).unwrap();
        let mut rev = ft.revisions[0].clone();
        let (rev_guid, mode_guid) = (rev.meta.identity.guid, rev.modes[0].meta.identity.guid);
        rev.modes[0].channel_functions.clear();
        refresh_revision(&mut rev, &bytes).unwrap();
        assert_eq!((rev.meta.identity.guid, rev.modes[0].meta.identity.guid), (rev_guid, mode_guid));
        assert_eq!(rev.modes[0].channel_functions.len(), 3, "derived again from the file");
    }
}
