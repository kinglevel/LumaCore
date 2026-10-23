// ███████╗██╗██╗  ██╗████████╗██╗   ██╗██████╗ ███████╗    ███╗   ███╗██╗   ██╗██████╗
// ██╔════╝██║╚██╗██╔╝╚══██╔══╝██║   ██║██╔══██╗██╔════╝    ████╗ ████║██║   ██║██╔══██╗
// █████╗  ██║ ╚███╔╝    ██║   ██║   ██║██████╔╝█████╗      ██╔████╔██║██║   ██║██████╔╝
// ██╔══╝  ██║ ██╔██╗    ██║   ██║   ██║██╔══██╗██╔══╝      ██║╚██╔╝██║╚██╗ ██╔╝██╔══██╗
// ██║     ██║██╔╝ ██╗   ██║   ╚██████╔╝██║  ██║███████╗    ██║ ╚═╝ ██║ ╚████╔╝ ██║  ██║
// ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝    ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═╝
// FILE: fixture_mvr

//! A fixture's MVR part (decisions §34 "34g4 patch hardened" 2–6): every field of MVR 1.6's
//! `<Fixture>`, typed, links as GUIDs; the world matrix and the angles read from it.
//!
//! MVR's `uuid` is the fixture's GUID and its `Name` the fixture's name (both in the meta, §12
//! "Tags part 1b hardened"); `FixtureIDNumeric` is the fixture's pool number; the next MVR's
//! `Comment` is the meta's note. Links to MVR's 3D objects (focus points, the other end of a
//! connection) wait for `Root.ShowData.3DSpatial` and are kept as data, never dropped.
//!
//! Owns:          MvrFixture, Matrix (world, mm) and its angles, IdType, the MVR records.
//! Doesn't own:   the MVR file (import / export), the metadata pools (devices_meta.rs).
//! Entry points:  MvrFixture, Matrix, IdType.
//! Decisions:     §34 "34g4 patch hardened", §12 "Tags part 1b hardened".

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fixture::FixtureBreak;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// What MVR 1.6 says about one fixture, besides its uuid, name and number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvrFixture {
    /// Where it sits in the MVR tree: a layer, a group, or the fixture it is a child of
    /// (MVR's `ChildList`). None = straight under the patch.
    #[serde(default)]
    pub parent: Option<Uuid>,
    /// Its place in the world: one matrix, exact (point 4).
    #[serde(default)]
    pub matrix: Matrix,
    /// The fixture type, revision and mode (GUID links, point 6; export derives `GDTFSpec` /
    /// `GDTFMode`).
    pub fixture_type: Uuid,
    pub revision: Uuid,
    pub mode: Uuid,
    /// MVR's `<Addresses>`: one per break.
    pub breaks: Vec<FixtureBreak>,
    /// The fixture this one is a multipatch of.
    #[serde(default)]
    pub multipatch: Option<Uuid>,
    /// `Classing`: a class of `Devices.Metadata.MVR`.
    #[serde(default)]
    pub class: Option<Uuid>,
    /// `Position`: a position of `Devices.Metadata.MVR`.
    #[serde(default)]
    pub position: Option<Uuid>,
    /// `Focus`: a focus point (3DSpatial, not hardened yet: kept as data).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<Uuid>,
    #[serde(default)]
    pub function: String,
    /// `FixtureID`: the drawing's text id.
    #[serde(default)]
    pub fixture_id: String,
    /// `FixtureIDNumeric`: the fixture number, `Fixture 849` — none on a folder (decisions §40
    /// 5, 7: the pool's key is a slot, never the number). Unique in the show.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fid: Option<u32>,
    /// Where it sits among its parent's children (decisions §40 6): spaced 10, 20, 30 …, the
    /// MVR's order kept; an insert in the middle takes the midpoint.
    #[serde(default)]
    pub order: u32,
    #[serde(default)]
    pub unit_number: u32,
    #[serde(default)]
    pub child_position: String,
    /// The second id (point 5, decisions §40 7): MVR's fixed list of id types (`IdType`)
    /// and the number in it (`CustomId`, `Fixture Media 7003`); unique within its type.
    #[serde(default)]
    pub id_type: IdType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<u32>,
    #[serde(default)]
    pub cast_shadow: bool,
    #[serde(default)]
    pub dmx_invert_pan: bool,
    #[serde(default)]
    pub dmx_invert_tilt: bool,
    /// `Addresses` / `Network`: its network addresses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub network: Vec<NetworkAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protocols: Vec<MvrProtocol>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alignments: Vec<Alignment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overwrites: Vec<Overwrite>,
    /// `Connections`: the other end is any MVR object (3DSpatial): kept as data.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<Connection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<CieColor>,
    /// `Mappings`: each names a mapping definition of `Devices.Metadata.MVR`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<Mapping>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gobo: Option<Gobo>,
}

/// MVR's 4×3 matrix: the rows are where the local x, y and z axes point, then the offset, in
/// millimetres (`{u}{v}{w}{o}`). A fixture keeps its world matrix; export writes the local one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    pub u: [f64; 3],
    pub v: [f64; 3],
    pub w: [f64; 3],
    pub o: [f64; 3],
}

/// MVR's `IdType` (its fixed list; 0 = none; the spec skips 3, one console's own Universal /
/// Generic slot, which LumaCore fills — decisions §40 10: Generic is the default type of a CID).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdType {
    #[default]
    None,
    Fixture,
    Channel,
    Generic,
    Houselights,
    NonDim,
    Media,
    Fog,
    Effect,
    Pyro,
    Marker,
}

/// One network address (`<Network>`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkAddress {
    pub geometry: String,
    #[serde(default)]
    pub ipv4: String,
    #[serde(default)]
    pub subnetmask: String,
    #[serde(default)]
    pub ipv6: String,
    #[serde(default)]
    pub dhcp: bool,
    #[serde(default)]
    pub hostname: String,
}

/// One `<Protocol>` the fixture speaks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MvrProtocol {
    #[serde(default)]
    pub geometry: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub transmission: String,
}

/// One `<Alignment>`: a beam geometry's up and direction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alignment {
    pub geometry: String,
    pub up: [f64; 3],
    pub direction: [f64; 3],
}

/// One `<Overwrite>`: a wheel slot / emitter / filter swapped for another.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Overwrite {
    pub universal: String,
    #[serde(default)]
    pub target: String,
}

/// One `<Connection>`: own geometry → another object's geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub own: String,
    pub other: String,
    pub to_object: Uuid,
}

/// A colour as CIE 1931 xyY.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CieColor {
    pub x: f64,
    pub y: f64,
    pub big_y: f64,
}

/// One `<Mapping>` onto a mapping definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mapping {
    pub link_def: Uuid,
    #[serde(default)]
    pub ux: i64,
    #[serde(default)]
    pub uy: i64,
    #[serde(default)]
    pub ox: i64,
    #[serde(default)]
    pub oy: i64,
    #[serde(default)]
    pub rz: f64,
}

/// The `<Gobo>`: an image file and its rotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gobo {
    pub file: String,
    #[serde(default)]
    pub rotation: f64,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl MvrFixture {
    /// A fixture of this type, revision and mode at these breaks, nothing else said.
    pub fn new(fixture_type: Uuid, revision: Uuid, mode: Uuid, breaks: Vec<FixtureBreak>) -> Self {
        Self {
            parent: None,
            matrix: Matrix::default(),
            fixture_type,
            revision,
            mode,
            breaks,
            multipatch: None,
            class: None,
            position: None,
            focus: None,
            function: String::new(),
            fixture_id: String::new(),
            fid: None,
            order: 0,
            unit_number: 0,
            child_position: String::new(),
            id_type: IdType::None,
            cid: None,
            cast_shadow: false,
            dmx_invert_pan: false,
            dmx_invert_tilt: false,
            network: Vec::new(),
            protocols: Vec::new(),
            alignments: Vec::new(),
            custom_commands: Vec::new(),
            overwrites: Vec::new(),
            connections: Vec::new(),
            color: None,
            mappings: Vec::new(),
            gobo: None,
        }
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
            w: [0.0, 0.0, 1.0],
            o: [0.0; 3],
        }
    }
}

impl Matrix {
    /// Where it is, in metres.
    pub fn position_m(&self) -> [f64; 3] {
        self.o.map(|mm| mm / 1000.0)
    }

    /// Move one axis (0 = x) to `metres`; the rotation is kept as it is.
    pub fn set_position_m(&mut self, axis: usize, metres: f64) {
        self.o[axis] = metres * 1000.0;
    }

    /// The rotation as angles in degrees, about x, then y, then z (world axes: R = Rz · Ry · Rx).
    /// Only read from the matrix: reading never changes it.
    pub fn rotation_deg(&self) -> [f64; 3] {
        let [u, v, w] = self.axes();
        // The axes are the columns of R: R[r][c] = axis_c[r].
        let r = |row: usize, col: usize| [u, v, w][col][row];
        let (a, b, c);
        let sb = (-r(2, 0)).clamp(-1.0, 1.0);
        b = sb.asin();
        if sb.abs() > 1.0 - 1e-9 {
            // Straight up or down: one angle is free; it is put on z.
            a = 0.0;
            c = (-r(0, 1)).atan2(r(1, 1));
        } else {
            a = r(2, 1).atan2(r(2, 2));
            c = r(1, 0).atan2(r(0, 0));
        }
        [a, b, c].map(|x| clean(x.to_degrees()))
    }

    /// Turn one axis (0 = x) to `degrees`, keeping the other two angles, the scale and the
    /// offset.
    pub fn set_rotation_deg(&mut self, axis: usize, degrees: f64) {
        let mut angles = self.rotation_deg();
        angles[axis] = degrees;
        let scale = [self.u, self.v, self.w].map(norm);
        let [a, b, c] = angles.map(f64::to_radians);
        let (sa, ca, sb, cb, sc, cc) = (a.sin(), a.cos(), b.sin(), b.cos(), c.sin(), c.cos());
        // The columns of Rz · Ry · Rx.
        let x = [cb * cc, cb * sc, -sb];
        let y = [sa * sb * cc - ca * sc, sa * sb * sc + ca * cc, sa * cb];
        let z = [ca * sb * cc + sa * sc, ca * sb * sc - sa * cc, ca * cb];
        self.u = x.map(|e| e * scale[0]);
        self.v = y.map(|e| e * scale[1]);
        self.w = z.map(|e| e * scale[2]);
    }

    /// The three axes without their scale.
    fn axes(&self) -> [[f64; 3]; 3] {
        [self.u, self.v, self.w].map(|a| {
            let n = norm(a);
            if n == 0.0 { a } else { a.map(|e| e / n) }
        })
    }
}

impl IdType {
    pub const NAMES: &'static [&'static str] =
        &["None", "Fixture", "Channel", "Generic", "Houselights", "NonDim", "Media", "Fog", "Effect", "Pyro", "Marker"];

    /// The type a name says, in any case (`NAMES`).
    pub fn from_name(name: &str) -> Option<Self> {
        let canonical = Self::NAMES.iter().find(|n| n.eq_ignore_ascii_case(name))?;
        serde_json::from_value(serde_json::Value::String(canonical.to_string())).ok()
    }

    /// Its name as declared (`NAMES`).
    pub fn name(self) -> &'static str {
        serde_json::to_value(self).ok().and_then(|v| Self::NAMES.iter().find(|n| v.as_str() == Some(**n)).copied()).unwrap_or("None")
    }

    /// MVR's number for it (the spec's list skips 3; Generic takes it).
    pub fn number(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Fixture => 1,
            Self::Channel => 2,
            Self::Generic => 3,
            Self::Houselights => 4,
            Self::NonDim => 5,
            Self::Media => 6,
            Self::Fog => 7,
            Self::Effect => 8,
            Self::Pyro => 9,
            Self::Marker => 10,
        }
    }

    /// The type of MVR's number (anything else is none).
    pub fn from_number(n: u32) -> Self {
        Self::ALL.into_iter().find(|t| t.number() == n).unwrap_or(Self::None)
    }

    const ALL: [Self; 11] = [
        Self::None,
        Self::Fixture,
        Self::Channel,
        Self::Generic,
        Self::Houselights,
        Self::NonDim,
        Self::Media,
        Self::Fog,
        Self::Effect,
        Self::Pyro,
        Self::Marker,
    ];
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

fn norm(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// A position or angle as shown and typed back: at most 3 places, no trailing zeros (`2.5`, `-90`).
pub fn decimal(v: f64) -> String {
    let t = format!("{v:.3}");
    let t = t.trim_end_matches('0').trim_end_matches('.');
    if t == "-0" { "0".into() } else { t.to_string() }
}

/// An angle as read: float noise dropped (to 1e-9 degrees), -0 as 0.
fn clean(deg: f64) -> f64 {
    let r = (deg * 1e9).round() / 1e9;
    if r == 0.0 { 0.0 } else { r }
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
