// ██████╗ ██████╗ ██╗██╗   ██╗███████╗███████╗
// ██╔══██╗██╔══██╗██║██║   ██║██╔════╝██╔════╝
// ██║  ██║██████╔╝██║██║   ██║█████╗  ███████╗
// ██║  ██║██╔══██╗██║╚██╗ ██╔╝██╔══╝  ╚════██║
// ██████╔╝██║  ██║██║ ╚████╔╝ ███████╗███████║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝╚══════╝
// FILE: drives

//! The Drive object (decisions §38): this station's drives — the local library, the volumes it
//! finds, the folders added by hand — as station-local keyed rows like Interface. `List Drive`,
//! `Info Drive "usb1"`, `Set Drive "usb1" "Label" "…"`, `Set Drive "ssdb" "Kind" "USB"` (the next
//! free id of the new kind, remembered), `Set Drive "usb1" "Library" Yes` (makes the library
//! tree; No is refused — a setting deletes nothing), `Store Drive ["label"] /Path "/srv/shows"`
//! (a Folder drive, the next free `folderN`), `Delete Drive "usb1"` (forgets a found drive —
//! its id is free again — or removes an added one; refused while an AssetRoot maps to it).
//! `Eject Drive "usb1"` is planned. The Drives menu is a grid on these properties (menus.rs).
//!
//! Owns:          the Drive object, its properties, the typed List / Info / Store / Delete, the planned Eject.
//! Doesn't own:   the live list and the table (lumacore-state `drives`), the probe (lumacore-drives), the watcher (the server).
//! Entry points:  DRIVE, DRIVE_SETTINGS, EjectPlanned.
//! Decisions:     §38.

use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::commands::flags as F;
use lumacore_objects::commands::registry::Rights;
use lumacore_objects::commands::typed::{Command, typed};
use lumacore_objects::drives::{self, LOCAL};
use lumacore_objects::objects::{ObjectDecl, ObjectDef, Verb, out};
use lumacore_objects::props::{Access, Home, Keyed, PropKind, Property, Props};
use lumacore_syntax::ast::{Id, Item, Range, Target, Value};
use lumacore_tree::{DriveKind, DriveMemory};

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

pub static DRIVE: ObjectDef = ObjectDef::new(ObjectDecl {
    keyword: "Drive",
    lum: lumacore_objects::objects::LumShape::ids(1),
    default_verb: Verb::Info,
    flags: &[F::PATH.on(&[Verb::Store])],
    aliases: &["drives"],
    doc: lumacore_objects::commands::registry::Doc::new(
        "Drive",
        "This station's drives: the local library, the volumes it finds (usb1, ssda), the folders you add (folder1) — each with the library tree LumaCore/library (decisions §38). Set Drive \"usb1\" \"Label\"|\"Kind\"|\"Library\" …; Store Drive /Path \"folder\"; Delete Drive \"usb1\"",
    ),
    family: None,
    call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
    call_class: lumacore_objects::commands::registry::CommandClass::Commit,
    scope: lumacore_objects::objects::on_station,
    reach: lumacore_objects::objects::drive_reach,
    rights: lumacore_objects::rights::setup,
    planned: &[],
})
.verbs(&[
    (Verb::List, typed::<ListDrive>),
    (Verb::Info, typed::<InfoDrive>),
    (Verb::Store, typed::<StoreDrive>),
    (Verb::Delete, typed::<DeleteDrive>),
])
.props(&DRIVE_SETTINGS)
.delete_ask(delete_ask);

const fn ro(name: &'static str, doc: &'static str, get: fn(&AppState, &str) -> String) -> Property {
    Property {
        name,
        kind: PropKind::Text,
        named: &[],
        words: ("", ""),
        rights: Rights::Setup,
        section: "Drive",
        unit: "",
        doc,
        default: None,
        access: Access::Keyed { get, set: |_, _, _| Err("read-only: what the station sees".to_string()) },
        home: Home::Station,
    }
}

pub static DRIVE_SETTINGS: Props = Props {
    lists: &[&[
        Property {
            name: "Label",
            kind: PropKind::Text,
            named: &[],
            words: ("", ""),
            rights: Rights::Setup,
            section: "Drive",
            unit: "",
            doc: "the display name beside the id (the volume's label at first)",
            default: None,
            access: Access::Keyed {
                get: |s, id| s.drive(id).map(|d| d.label.clone()).unwrap_or_default(),
                set: |s, id, v| {
                    let id = id.to_lowercase();
                    if id == LOCAL {
                        return Err("the local library keeps its name".to_string());
                    }
                    let m = s.root.station_settings.drives.get_mut(&id).ok_or_else(|| format!("No drive \"{id}\""))?;
                    m.label = v.trim_matches('"').to_string();
                    drives::refresh(s);
                    Ok(format!("Drive \"{id}\": Label {v}"))
                },
            },
            home: Home::Station,
        },
        Property {
            name: "Kind",
            kind: PropKind::Choice(&["SSD", "HDD", "USB", "Folder"]),
            named: &[],
            words: ("", ""),
            rights: Rights::Setup,
            section: "Drive",
            unit: "",
            doc: "what the drive is — corrects the OS once; the drive takes the next free id of the new kind, remembered",
            default: None,
            access: Access::Keyed {
                get: |s, id| s.drive(id).map(|d| d.kind.word().to_string()).unwrap_or_default(),
                set: |s, id, v| {
                    let id = id.to_lowercase();
                    if id == LOCAL {
                        return Err("the local library is what it is".to_string());
                    }
                    let kind = DriveKind::from_word(v.trim_matches('"')).ok_or_else(|| format!("Kind: SSD, HDD, USB or Folder, not {v}"))?;
                    let table = &mut s.root.station_settings.drives;
                    let mut m = table.remove(&id).ok_or_else(|| format!("No drive \"{id}\""))?;
                    m.kind = kind;
                    let new_id = drives::next_id(kind, table);
                    table.insert(new_id.clone(), m);
                    drives::refresh(s);
                    Ok(format!("Drive \"{id}\" → \"{new_id}\" ({})", kind.word()))
                },
            },
            home: Home::Station,
        },
        ro("Path", "the volume's root (the whole drive), or the folder it was added from", |s, id| s.drive(id).map(|d| d.mount.to_string_lossy().to_string()).unwrap_or_default()),
        Property {
            name: "Library",
            kind: PropKind::Switch,
            named: &[],
            words: ("Yes", "No"),
            rights: Rights::Setup,
            section: "Drive",
            unit: "",
            doc: "whether LumaCore/library exists on it; Yes makes it (the first write does too); No is refused — a setting deletes nothing",
            default: None,
            access: Access::Keyed {
                get: |s, id| if s.drive(id).is_some_and(|d| d.has_library()) { "Yes" } else { "No" }.to_string(),
                set: |s, id, v| {
                    if v != "Yes" {
                        return Err("a setting deletes nothing: the library tree stays (remove files in the tree shell)".to_string());
                    }
                    let lib = s.ensure_library(id)?;
                    drives::refresh(s);
                    Ok(format!("Drive \"{}\": library at {}", id.to_lowercase(), lib.display()))
                },
            },
            home: Home::Station,
        },
        ro("Free", "free space", |s, id| s.drive(id).filter(|d| d.present).map(|d| drives::gb(d.free)).unwrap_or_default()),
        ro("Total", "the volume's size", |s, id| s.drive(id).filter(|d| d.present).map(|d| drives::gb(d.total)).unwrap_or_default()),
        ro("State", "In (mounted, reachable) or Out", |s, id| s.drive(id).map(|d| d.state_word().to_string()).unwrap_or_default()),
        ro("Since", "when it was last seen", |s, id| s.drive(id).map(|d| since(d.since_ms)).unwrap_or_default()),
    ]],
    keyed: Some(Keyed {
        rows: |s| s.drives.iter().map(|d| d.id.clone()).collect(),
        row: |s, w| s.drive(w).map(|d| d.id.clone()),
    }),
    label: Some(|_, k| format!("Drive \"{k}\"")),
    after: |s| s.local_settings.store_from(&s.root.station_settings),
    ..Props::BASE
};

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// `List Drive`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "List", object = "Drive")]
pub struct ListDrive {
    #[lum(target)]
    pub target: Option<Target>,
}

impl Command for ListDrive {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(list(state))
    }
}

/// `Info Drive "usb1"` (bare: the list).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Info", object = "Drive")]
pub struct InfoDrive {
    #[lum(target)]
    pub target: Option<Target>,
}

impl Command for InfoDrive {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let names = self.target.as_ref().map(names).unwrap_or_default();
        match names.first() {
            None => out(list(state)),
            Some(w) => out(info(state, w).unwrap_or_else(|e| e)),
        }
    }
}

/// `Store Drive ["label"] /Path "/srv/shows"`: a Folder drive, the next free `folderN`.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Store", object = "Drive")]
pub struct StoreDrive {
    #[lum(target)]
    pub target: Option<Target>,
    #[lum(option = "Path")]
    pub path: Option<Value>,
}

impl Command for StoreDrive {
    fn run(self, state: &mut AppState) -> DispatchResult {
        let label = self.target.as_ref().map(names).unwrap_or_default().into_iter().next();
        out(store(state, label, self.path.as_ref().and_then(text)).unwrap_or_else(|e| e))
    }
}

/// `Delete Drive "usb1"`: forget a found drive, remove an added one (the plain question first).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Delete", object = "Drive")]
pub struct DeleteDrive {
    #[lum(target)]
    pub target: Target,
}

impl Command for DeleteDrive {
    fn run(self, state: &mut AppState) -> DispatchResult {
        out(delete(state, &names(&self.target)).unwrap_or_else(|e| e))
    }
}

/// `Eject Drive "usb1"`: planned — dispatch answers before this runs (E-CMD-003).
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Eject")]
pub struct EjectPlanned {
    #[lum(target)]
    pub target: Option<Target>,
}

impl Command for EjectPlanned {
    fn run(self, _: &mut AppState) -> DispatchResult {
        out(lumacore_objects::errors::render_code("Eject isn't built yet (planned)", "E-CMD-003"))
    }
}

fn list(state: &AppState) -> String {
    let mut out = String::from("Drives (this station):");
    for d in &state.drives {
        out.push_str(&format!(
            "\n  {:<14} {:<8} {:<6} {:<20} {:<12} {}",
            d.id,
            d.kind.word(),
            d.state_word(),
            d.label,
            if d.present { format!("{} free", drives::gb(d.free)) } else { String::new() },
            if d.has_library() { "library" } else { "no library yet" }
        ));
    }
    out
}

fn info(state: &AppState, w: &str) -> Result<String, String> {
    let d = state.drive(w).ok_or_else(|| lumacore_objects::errors::render_code(&format!("No drive \"{w}\" (List Drive)"), "E-CMD-007"))?;
    Ok(format!(
        "Drive \"{}\"\n  Label    {}\n  Kind     {}\n  State    {}\n  Path     {}\n  Library  {}\n  Free     {}\n  Total    {}\n  Since    {}",
        d.id,
        d.label,
        d.kind.word(),
        d.state_word(),
        d.mount.display(),
        if d.has_library() { d.library().display().to_string() } else { "not yet (made on the first write)".to_string() },
        if d.present { drives::gb(d.free) } else { "—".to_string() },
        if d.present { drives::gb(d.total) } else { "—".to_string() },
        since(d.since_ms),
    ))
}

fn store(state: &mut AppState, label: Option<String>, path: Option<String>) -> Result<String, String> {
    let Some(path) = path else {
        return Err(lumacore_objects::errors::render_code("Store Drive [\"label\"] /Path \"<folder>\"", "E-CMD-006"));
    };
    if !lumacore_luma::drive::metadata(&path).is_ok_and(|m| m.is_dir()) {
        return Err(lumacore_objects::errors::render_code(&format!("\"{path}\" is not a folder this station can reach"), "E-CMD-012"));
    }
    let label = label
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone()));
    let table = &mut state.root.station_settings.drives;
    if let Some((id, _)) = table.iter().find(|(_, m)| m.path.as_deref() == Some(path.as_str())) {
        return Err(format!("\"{path}\" is Drive \"{id}\" already"));
    }
    let id = drives::next_id(DriveKind::Folder, table);
    table.insert(id.clone(), DriveMemory { key: String::new(), kind: DriveKind::Folder, label: label.clone(), path: Some(path.clone()), last_seen: lumacore_tree::now_ms() });
    state.local_settings.store_from(&state.root.station_settings);
    drives::refresh(state);
    lumacore_sysmon::sysmon_note("Drives", format!("{id} \"{label}\" added ({path})"));
    Ok(lumacore_objects::text::render_success(&format!("Drive \"{id}\" ({label}) → {path}")))
}

/// Forgetting a found drive, removing an added one: the plain question, refused for the local
/// library and while an AssetRoot maps to it.
fn delete_ask(state: &AppState, args: &[&str]) -> lumacore_objects::defs::object::DeleteAsk {
    use lumacore_objects::defs::object::DeleteAsk;
    for w in args.iter().filter(|a| !F::is_flag(a)) {
        let Some(d) = state.drive(w) else { return DeleteAsk::Refuse(lumacore_objects::errors::render_code(&format!("No drive {w} (List Drive)"), "E-CMD-007")) };
        if d.kind == DriveKind::LocalLibrary {
            return DeleteAsk::Refuse("the local library is Drive 1 for good".to_string());
        }
        if let Some((_, root)) = mapped_root(state, &d.id) {
            return DeleteAsk::Refuse(format!("AssetRoot \"{root}\" is on Drive \"{}\" — map it elsewhere first (Set AssetRoot …)", d.id));
        }
    }
    DeleteAsk::Plain
}

/// The handler refuses what the question would (a `/NoConfirmation` line skips the question).
fn delete(state: &mut AppState, words: &[String]) -> Result<String, String> {
    if words.is_empty() {
        return Err(lumacore_objects::errors::render_code("Delete Drive \"<id>\" (List Drive)", "E-CMD-006"));
    }
    let mut gone = Vec::new();
    for w in words {
        let d = state.drive(w).ok_or_else(|| lumacore_objects::errors::render_code(&format!("No drive \"{w}\" (List Drive)"), "E-CMD-007"))?;
        if d.kind == DriveKind::LocalLibrary {
            return Err(lumacore_objects::errors::render_code("the local library is Drive 1 for good", "E-CMD-010"));
        }
        if let Some((_, root)) = mapped_root(state, &d.id) {
            return Err(lumacore_objects::errors::render_code(&format!("AssetRoot \"{root}\" is on Drive \"{}\" — map it elsewhere first", d.id), "E-CMD-010"));
        }
        let id = d.id.clone();
        if state.root.station_settings.drives.remove(&id).is_some() {
            gone.push(id);
        }
    }
    state.local_settings.store_from(&state.root.station_settings);
    drives::refresh(state);
    for id in &gone {
        lumacore_sysmon::sysmon_note("Drives", format!("{id} forgotten"));
    }
    Ok(lumacore_objects::text::render_success(&format!("Forgot Drive {}: its id is free again", gone.iter().map(|g| format!("\"{g}\"")).collect::<Vec<_>>().join(" + "))))
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The names a target holds (`"usb1"`, `Drive "usb1" + "usb2"`): a drive id is never a number.
fn names(t: &Target) -> Vec<String> {
    let mut out = Vec::new();
    let mut take = |ids: &lumacore_syntax::ast::Ids| {
        for (_, r) in &ids.ranges {
            if let Range::One(Id::Name(n)) = r {
                out.push(n.node.clone());
            }
        }
    };
    for (_, item) in &t.items {
        match item {
            Item::Ids(ids) => take(ids),
            Item::Path(segs) => {
                if let Some(ids) = segs.last().and_then(|s| s.ids.as_ref()) {
                    take(ids);
                }
            }
            _ => {}
        }
    }
    out
}

/// A `/Path` value as text.
fn text(v: &Value) -> Option<String> {
    match v {
        Value::Text(t) => Some(t.node.clone()),
        Value::Number(n) => Some(n.node.clone()),
        _ => None,
    }
}

/// An AssetRoot whose station folder lies on the drive (its mount path).
fn mapped_root(state: &AppState, id: &str) -> Option<(String, String)> {
    let d = state.drive(id)?;
    if !d.present {
        return None;
    }
    let mount = d.mount.to_string_lossy().to_string();
    state
        .root
        .station_settings
        .asset_roots
        .iter()
        .find(|(_, folder)| std::path::Path::new(folder.as_str()).starts_with(&mount))
        .map(|(name, folder)| (name.clone(), folder.clone()))
}

fn since(ms: u64) -> String {
    if ms == 0 {
        return "—".to_string();
    }
    chrono::DateTime::from_timestamp_millis(ms as i64)
        .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}
