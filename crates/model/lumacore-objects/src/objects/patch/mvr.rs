// ███╗   ███╗██╗   ██╗██████╗
// ████╗ ████║██║   ██║██╔══██╗
// ██╔████╔██║██║   ██║██████╔╝
// ██║╚██╔╝██║╚██╗ ██╔╝██╔══██╗
// ██║ ╚═╝ ██║ ╚████╔╝ ██║  ██║
// ╚═╝     ╚═╝  ╚═══╝  ╚═╝  ╚═╝
// FILE: mvr

//! The `Mvr` object (decisions §34 "Import MVR hardened" 1–4): `Import Mvr "file" [/Drive]
//! [/Scope <target>] [/Match …] …` reads an MVR against the holder's Devices draft and opens the
//! review on it (nothing written); `List Mvr` / `Info Mvr` show it; `Import Mvr /Cancel` drops it;
//! `Import Mvr {rows} /Apply`, a cell's `/Take` / `/Offer` and the placement asks come with part
//! 4b. The settings are the profile's MVR import preset (`UserProfile` "Mvr…" properties); the
//! line's options override them for one import. A bare `Import Mvr` asks for the file with the
//! files element.
//!
//! Owns:          MVR (the row), the options, the settings resolved from the preset and the line.
//! Doesn't own:   the file (`lumacore-mvr`), the review's rule (`review`), the preset's storage (the profile).
//! Entry points:  MVR, import::ImportMvr, list::{ListMvr, InfoMvr}, review::build, apply::apply.
//! Decisions:     §34 "Import MVR hardened" 1–4, "34g4 patch hardened" 10–12.

use lumacore_tree::{MvrLost, MvrMatch, MvrMissing, MvrRotation, MvrTake};

use crate::commands::flags as F;
use crate::objects::{ObjectDef, Verb};

pub mod apply;
pub mod import;
pub mod list;
pub mod review;
// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES


/// The settings a review is built with: the preset, then the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub match_key: MvrMatch,
    pub lost: MvrLost,
    pub rotation: MvrRotation,
    pub missing: MvrMissing,
    /// The id type of a CustomId the file never typed (decisions §40 9, 10).
    pub id_type: lumacore_fixtures::IdType,
    pub addresses: MvrTake,
    pub ids: MvrTake,
    pub names: MvrTake,
    pub position: MvrTake,
    pub revisions: MvrTake,
    pub new_fixtures: MvrTake,
    pub fid_offset: i64,
    pub universe_offset: i64,
    pub offset_mm: [i64; 3],
    pub rotate_deg: i64,
}

impl Settings {
    pub fn from_preset(p: &lumacore_tree::MvrImportPreset) -> Self {
        Self {
            match_key: p.match_key,
            lost: p.lost,
            rotation: p.rotation,
            missing: p.missing,
            id_type: p.id_type,
            addresses: p.addresses,
            ids: p.ids,
            names: p.names,
            position: p.position,
            revisions: p.revisions,
            new_fixtures: p.new_fixtures,
            fid_offset: p.fid_offset,
            universe_offset: p.universe_offset,
            offset_mm: [p.x_mm, p.y_mm, p.z_mm],
            rotate_deg: p.rotate_deg,
        }
    }

    pub fn for_review(&self) -> crate::mvr_review::ReviewSettings {
        crate::mvr_review::ReviewSettings {
            match_key: format!("{:?}", self.match_key),
            lost: format!("{:?}", self.lost),
            rotation: format!("{:?}", self.rotation),
            missing: format!("{:?}", self.missing),
            id_type: self.id_type.name().to_string(),
            addresses: self.addresses,
            ids: self.ids,
            names: self.names,
            position: self.position,
            revisions: self.revisions,
            new_fixtures: self.new_fixtures,
            fid_offset: self.fid_offset,
            universe_offset: self.universe_offset,
            offset_mm: self.offset_mm,
            rotate_deg: self.rotate_deg,
        }
    }
}
// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY


pub const SCOPE: F::FlagDef = F::FlagDef::new("Scope", "What the import works on: an MvrLayer, MvrGroup, MvrClass or Tag (absent: the whole patch)").value("target");
pub const MATCH: F::FlagDef = F::FlagDef::new("Match", "The match key: Auto (UUID, then FID, then the name), UUID, FID, FixtureName").value("key");
pub const LOST: F::FlagDef = F::FlagDef::new("Lost", "A fixture in scope the file no longer has: Mark or Delete").value("Mark | Delete");
pub const IGNORE_ROTATION: F::FlagDef = F::FlagDef::new("IgnoreRotation", "Take the file's position without its rotation");
pub const FID_OFFSET: F::FlagDef = F::FlagDef::new("FidOffset", "Added to the file's fixture numbers").value("n");
pub const UNIVERSE_OFFSET: F::FlagDef = F::FlagDef::new("UniverseOffset", "Added to the file's universes").value("n");
pub const X: F::FlagDef = F::FlagDef::new("X", "The rig offset, X (mm)").value("mm");
pub const Y: F::FlagDef = F::FlagDef::new("Y", "The rig offset, Y (mm)").value("mm");
pub const Z: F::FlagDef = F::FlagDef::new("Z", "The rig offset, Z (mm)").value("mm");
pub const ROTATE: F::FlagDef = F::FlagDef::new("Rotate", "The rig rotation about Z (degrees)").value("degrees");
pub const ADDRESSES: F::FlagDef = F::FlagDef::new("Addresses", "What the file may change in the addresses: None, Take, Offer").value("mode");
pub const IDS: F::FlagDef = F::FlagDef::new("Ids", "What the file may change in the ids: None, Take, Offer").value("mode");
pub const NAMES: F::FlagDef = F::FlagDef::new("Names", "What the file may change in the names: None, Take, Offer").value("mode");
pub const POSITION: F::FlagDef = F::FlagDef::new("Position", "What the file may change in position and rotation: None, Take, Offer").value("mode");
pub const REVISIONS: F::FlagDef = F::FlagDef::new("Revisions", "What the file may change in the types' revisions: None, Take, Offer").value("mode");
pub const NEW_FIXTURES: F::FlagDef = F::FlagDef::new("NewFixtures", "The fixtures the file adds: None, Take, Offer").value("mode");
pub const APPLY: F::FlagDef = F::FlagDef::new("Apply", "Write the ticked rows of the review into the draft");
pub const CANCEL: F::FlagDef = F::FlagDef::new("Cancel", "Drop the review; the draft as it was");
pub const TAKE: F::FlagDef = F::FlagDef::new("Take", "Take one cell of a review row").value("field");
pub const OFFER: F::FlagDef = F::FlagDef::new("Offer", "Offer one cell of a review row again").value("field");
pub const ID_TYPE: F::FlagDef = F::FlagDef::new("IdType", "The id type of a CustomId the file never typed (Generic by default, decisions §40 9)").value("type");
pub const GONE: F::FlagDef = F::FlagDef::new("Gone", "A new fixture's gone group: Recreate, Parent, None (`/Missing` is List Asset's)").value("choice");
pub const GROUP: F::FlagDef = F::FlagDef::new("Group", "The one gone group a /Gone answer is for").value("name");

pub static MVR: ObjectDef = ObjectDef::new(crate::objects::ObjectDecl {
    keyword: "Mvr",
    lum: crate::objects::LumShape::ids(1),
    default_verb: Verb::Info,
    flags: &[
        F::DRIVE.on(&[Verb::Import]),
        SCOPE.on(&[Verb::Import]),
        MATCH.on(&[Verb::Import]),
        LOST.on(&[Verb::Import]),
        IGNORE_ROTATION.on(&[Verb::Import]),
        ID_TYPE.on(&[Verb::Import]),
        FID_OFFSET.on(&[Verb::Import]),
        UNIVERSE_OFFSET.on(&[Verb::Import]),
        X.on(&[Verb::Import]),
        Y.on(&[Verb::Import]),
        Z.on(&[Verb::Import]),
        ROTATE.on(&[Verb::Import]),
        ADDRESSES.on(&[Verb::Import]),
        IDS.on(&[Verb::Import]),
        NAMES.on(&[Verb::Import]),
        POSITION.on(&[Verb::Import]),
        REVISIONS.on(&[Verb::Import]),
        NEW_FIXTURES.on(&[Verb::Import]),
        APPLY.on(&[Verb::Import]),
        CANCEL.on(&[Verb::Import]),
        TAKE.on(&[Verb::Import]),
        OFFER.on(&[Verb::Import]),
        GONE.on(&[Verb::Import]),
        GROUP.on(&[Verb::Import]),
    ],
    aliases: &[],
    doc: crate::commands::registry::Doc::new("Mvr", "An MVR file against the Devices draft: Import opens the review, List / Info show it"),
    family: None,
    call_undo: crate::commands::registry::Undo::ReadOnly,
    call_class: crate::commands::registry::CommandClass::Commit,
    scope: crate::objects::on_master,
    reach: |v| if v == Verb::Import { crate::commands::registry::Reach::Drive } else { crate::commands::registry::Reach::Show },
    rights: crate::rights::setup,
    planned: &[],
})
.verbs(&[
    (Verb::Import, crate::commands::typed::typed::<import::ImportMvr>),
    (Verb::List, crate::commands::typed::typed::<list::ListMvr>),
    (Verb::Info, crate::commands::typed::typed::<list::InfoMvr>),
]);
// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS


/// A line value as a word: a declared word, quoted text or a number.
pub(crate) fn word(v: &lumacore_syntax::ast::Value) -> String {
    use lumacore_syntax::ast::Value;
    match v {
        Value::Word(w) => crate::lum::vocabulary().word(w.node).name.clone(),
        Value::Text(t) => t.node.clone(),
        Value::Number(n) => n.node.clone(),
        other => format!("{other:?}"),
    }
}

/// A None / Take / Offer word.
pub(crate) fn take_mode(v: &lumacore_syntax::ast::Value, what: &str) -> Result<MvrTake, String> {
    let w = word(v);
    match w.to_ascii_lowercase().as_str() {
        "none" => Ok(MvrTake::None),
        "take" => Ok(MvrTake::Take),
        "offer" => Ok(MvrTake::Offer),
        _ => Err(crate::errors::render_code(format!("/{what} takes None, Take or Offer, not {w}."), "E-CMD-008")),
    }
}

/// A whole number option.
pub(crate) fn whole(v: &lumacore_syntax::ast::Value, what: &str) -> Result<i64, String> {
    word(v).parse::<i64>().map_err(|_| crate::errors::render_code(format!("/{what} takes a whole number, not {}.", word(v)), "E-CMD-008"))
}

#[cfg(test)]
mod tests;
