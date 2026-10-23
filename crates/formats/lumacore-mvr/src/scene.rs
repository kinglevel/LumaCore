// ███████╗ ██████╗███████╗███╗   ██╗███████╗
// ██╔════╝██╔════╝██╔════╝████╗  ██║██╔════╝
// ███████╗██║     █████╗  ██╔██╗ ██║█████╗
// ╚════██║██║     ██╔══╝  ██║╚██╗██║██╔══╝
// ███████║╚██████╗███████╗██║ ╚████║███████╗
// ╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═══╝╚══════╝
// FILE: scene

//! The scene as data (MVR 1.6 `GeneralSceneDescription`): the provider, the layers, the groups
//! nested in them, every fixture flattened with its parent and layer, the AUXData's classes and
//! positions, and the file list of the zip. The text rules of MVR live here: an address is
//! absolute (`513` = 2.001) or `universe.channel`; a matrix is `{u}{v}{w}{o}` in millimetres.
//!
//! Owns:          MvrFile, MvrScene, MvrLayer, MvrGroup, MvrFixture, MvrAddress, MvrNamed, MvrHead, from_xml.
//! Doesn't own:   the zip (read.rs), the element tree (xml.rs).
//! Entry points:  MvrScene::from_xml, MvrFile::head, MvrAddress::parse, parse_matrix.
//! Decisions:     §34 "34g4 patch hardened" 2–6, 10–12.

use uuid::Uuid;

use crate::MvrError;
use crate::xml::Node;


// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The zip: its scene, its entries and the GDTFs among them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrFile {
    pub scene: MvrScene,
    /// Every entry of the zip, as named.
    pub entries: Vec<String>,
}

/// The scene.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrScene {
    pub ver_major: u32,
    pub ver_minor: u32,
    pub provider: String,
    pub provider_version: String,
    pub layers: Vec<MvrLayer>,
    /// Every group, in tree order, with its parent (a layer or a group).
    pub groups: Vec<MvrGroup>,
    /// Every fixture, in tree order, with its parent (a layer, a group or a fixture).
    pub fixtures: Vec<MvrFixture>,
    pub classes: Vec<MvrNamed>,
    pub positions: Vec<MvrNamed>,
    /// Scene objects, trusses, supports, screens, projectors: counted, kept out (no `3DSpatial` yet).
    pub other_objects: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrLayer {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrGroup {
    pub uuid: Uuid,
    pub name: String,
    pub parent: Uuid,
    pub layer: Uuid,
}

/// A named thing of the AUXData (a class, a position).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrNamed {
    pub uuid: Uuid,
    pub name: String,
}

/// One `<Fixture>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrFixture {
    pub uuid: Uuid,
    pub name: String,
    /// A layer, a group or the fixture it is a child of.
    pub parent: Uuid,
    pub layer: Uuid,
    /// `{u}{v}{w}{o}`, mm; None when the file has none.
    pub matrix: Option<[[f64; 3]; 4]>,
    pub gdtf_spec: String,
    pub gdtf_mode: String,
    pub addresses: Vec<MvrAddress>,
    pub fixture_id: String,
    pub fixture_id_numeric: Option<u32>,
    pub unit_number: Option<u32>,
    pub custom_id: Option<u32>,
    pub custom_id_type: Option<u32>,
    pub classing: Option<Uuid>,
    pub position: Option<Uuid>,
    pub multipatch: Option<Uuid>,
}

/// One `<Address break="n">`: universe and channel, 1-based.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MvrAddress {
    pub dmx_break: u16,
    pub universe: u32,
    pub channel: u16,
}

/// The facts a file list shows (decisions §38 5: the `files` feed's MVR columns).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MvrHead {
    pub provider: String,
    pub provider_version: String,
    pub layers: usize,
    pub groups: usize,
    pub fixtures: usize,
    /// The distinct GDTF files the fixtures name.
    pub gdtf_named: Vec<String>,
    /// Of those, the ones the zip holds.
    pub gdtf_embedded: usize,
    /// The ones it lacks (the show's own store may still have them).
    pub gdtf_missing: Vec<String>,
}


// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl MvrAddress {
    /// `513` (absolute, 1-based) or `2.1` (universe.channel).
    pub fn parse(dmx_break: u16, text: &str) -> Option<Self> {
        let text = text.trim();
        if let Some((u, c)) = text.split_once('.') {
            let universe: u32 = u.trim().parse().ok()?;
            let channel: u16 = c.trim().parse().ok()?;
            return (universe >= 1 && (1..=512).contains(&channel)).then_some(Self { dmx_break, universe, channel });
        }
        let absolute: u32 = text.parse().ok()?;
        if absolute == 0 {
            return None;
        }
        Some(Self { dmx_break, universe: (absolute - 1) / 512 + 1, channel: ((absolute - 1) % 512 + 1) as u16 })
    }
}

/// `{u1,u2,u3}{v1,v2,v3}{w1,w2,w3}{x,y,z}` → four rows; None when the text isn't that.
pub fn parse_matrix(text: &str) -> Option<[[f64; 3]; 4]> {
    let mut rows = [[0.0; 3]; 4];
    let mut n = 0;
    for part in text.split('{').skip(1) {
        let inner = part.split('}').next()?;
        let mut cols = inner.split(',').map(|c| c.trim().parse::<f64>());
        let row = [cols.next()?.ok()?, cols.next()?.ok()?, cols.next()?.ok()?];
        if n >= 4 {
            return None;
        }
        rows[n] = row;
        n += 1;
    }
    (n == 4).then_some(rows)
}

fn uuid_of(node: &Node, what: &'static str) -> Result<Uuid, MvrError> {
    node.attr("uuid")
        .and_then(|u| Uuid::parse_str(u.trim()).ok())
        .ok_or_else(|| MvrError::BadUuid(what, node.attr("name").unwrap_or("").to_string()))
}

fn uuid_text(node: &Node, name: &str) -> Option<Uuid> {
    node.text_of(name).and_then(|t| Uuid::parse_str(t).ok())
}

fn number(node: &Node, name: &str) -> Option<u32> {
    node.text_of(name).and_then(|t| t.parse().ok())
}

impl MvrScene {
    /// The scene from the document's root (`GeneralSceneDescription`).
    pub fn from_xml(root: &Node) -> Result<Self, MvrError> {
        if root.name != "GeneralSceneDescription" {
            return Err(MvrError::BadXml(format!("the root is <{}>, not <GeneralSceneDescription>", root.name)));
        }
        let mut scene = MvrScene {
            ver_major: root.attr("verMajor").and_then(|v| v.parse().ok()).unwrap_or(1),
            ver_minor: root.attr("verMinor").and_then(|v| v.parse().ok()).unwrap_or(0),
            provider: root.attr("provider").unwrap_or("").to_string(),
            provider_version: root.attr("providerVersion").unwrap_or("").to_string(),
            ..MvrScene::default()
        };
        let Some(inner) = root.child("Scene") else { return Ok(scene) };
        if let Some(aux) = inner.child("AUXData") {
            for c in aux.children("Class") {
                scene.classes.push(MvrNamed { uuid: uuid_of(c, "Class")?, name: c.attr("name").unwrap_or("").to_string() });
            }
            for p in aux.children("Position") {
                scene.positions.push(MvrNamed { uuid: uuid_of(p, "Position")?, name: p.attr("name").unwrap_or("").to_string() });
            }
        }
        if let Some(layers) = inner.child("Layers") {
            for l in layers.children("Layer") {
                let uuid = uuid_of(l, "Layer")?;
                scene.layers.push(MvrLayer { uuid, name: l.attr("name").unwrap_or("").to_string() });
                if let Some(list) = l.child("ChildList") {
                    scene.walk(list, uuid, uuid)?;
                }
            }
        }
        Ok(scene)
    }

    /// A `ChildList`: groups nest, fixtures may hold fixtures, the rest is counted.
    fn walk(&mut self, list: &Node, parent: Uuid, layer: Uuid) -> Result<(), MvrError> {
        for item in &list.children {
            match item.name.as_str() {
                "GroupObject" => {
                    let uuid = uuid_of(item, "GroupObject")?;
                    self.groups.push(MvrGroup { uuid, name: item.attr("name").unwrap_or("").to_string(), parent, layer });
                    if let Some(inner) = item.child("ChildList") {
                        self.walk(inner, uuid, layer)?;
                    }
                }
                "Fixture" => {
                    let uuid = uuid_of(item, "Fixture")?;
                    self.fixtures.push(fixture(item, uuid, parent, layer));
                    if let Some(inner) = item.child("ChildList") {
                        self.walk(inner, uuid, layer)?;
                    }
                }
                _ => {
                    self.other_objects += 1;
                    if let Some(inner) = item.child("ChildList") {
                        self.walk(inner, parent, layer)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn fixture(node: &Node, uuid: Uuid, parent: Uuid, layer: Uuid) -> MvrFixture {
    let addresses = node
        .child("Addresses")
        .map(|a| {
            a.children("Address")
                .filter_map(|x| MvrAddress::parse(x.attr("break").and_then(|b| b.parse().ok()).unwrap_or(0), &x.text))
                .collect()
        })
        .unwrap_or_default();
    let fixture_id = node.text_of("FixtureID").unwrap_or("").to_string();
    MvrFixture {
        uuid,
        name: node.attr("name").unwrap_or("").to_string(),
        parent,
        layer,
        matrix: node.text_of("Matrix").and_then(parse_matrix),
        gdtf_spec: node.text_of("GDTFSpec").unwrap_or("").to_string(),
        gdtf_mode: node.text_of("GDTFMode").unwrap_or("").to_string(),
        addresses,
        fixture_id: fixture_id.clone(),
        // MVR 1.6's numeric id; before it (Capture, Vectorworks, Depence 1.4 / 1.5 files) the
        // number is the text of `FixtureID`.
        fixture_id_numeric: number(node, "FixtureIDNumeric").or_else(|| fixture_id.trim().parse().ok()),
        unit_number: number(node, "UnitNumber"),
        custom_id: number(node, "CustomId"),
        custom_id_type: number(node, "CustomIdType"),
        classing: uuid_text(node, "Classing"),
        position: uuid_text(node, "Position"),
        multipatch: uuid_text(node, "Multipatch"),
    }
}

impl MvrFile {
    /// The GDTF entries of the zip (any folder, any case of the extension).
    pub fn gdtf_entries(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(String::as_str).filter(|e| e.to_lowercase().ends_with(".gdtf"))
    }

    /// Whether the zip holds the GDTF a fixture names (by file name, any folder).
    pub fn has_gdtf(&self, spec: &str) -> bool {
        let want = spec.trim().to_lowercase();
        let want = want.rsplit('/').next().unwrap_or(&want).to_string();
        let want = if want.ends_with(".gdtf") { want } else { format!("{want}.gdtf") };
        self.gdtf_entries().any(|e| e.to_lowercase().rsplit('/').next() == Some(want.as_str()))
    }

    /// The facts a file list shows.
    pub fn head(&self) -> MvrHead {
        let mut named: Vec<String> = Vec::new();
        for f in &self.scene.fixtures {
            let spec = f.gdtf_spec.trim();
            if !spec.is_empty() && !named.iter().any(|n| n.eq_ignore_ascii_case(spec)) {
                named.push(spec.to_string());
            }
        }
        let missing: Vec<String> = named.iter().filter(|n| !self.has_gdtf(n)).cloned().collect();
        MvrHead {
            provider: self.scene.provider.clone(),
            provider_version: self.scene.provider_version.clone(),
            layers: self.scene.layers.len(),
            groups: self.scene.groups.len(),
            fixtures: self.scene.fixtures.len(),
            gdtf_embedded: named.len() - missing.len(),
            gdtf_named: named,
            gdtf_missing: missing,
        }
    }
}
