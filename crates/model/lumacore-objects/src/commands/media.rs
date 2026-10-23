//! Pictures, icons, profile files and asset drives (decisions §30 "12a2 /
//! 12a3 hardening", §8).
//!
//! - `Image n` — the show's Images pool (pictures for show data).
//! - `UserProfile [<n>|"name"] Image|Icon n` — a profile's own pools (owner
//!   first, like `DataPool 2 Group 3`; a bare `UserProfile` is your own).
//!   `Icon n` alone is your own profile's icon: the show has no icons, the
//!   standard set ships with LumaCore.
//! - `Import Image [<n>] "stage.jpg"` reads `library/media/`; `Export Image 3`
//!   writes the file back there. `Copy UserProfile Icon 3 At Image 10` copies
//!   between pools (new GUID).
//! - `Export UserProfile ["Bob"]` / `Import UserProfile "Bob" [/Overwrite]`:
//!   `library/userprofiles/<name>.lumaprofile`, the profile with its pools.
//! - `AssetRoot` / `Asset`: large media on asset drives; `Set AssetRoot
//!   "Video" Folder "/Volumes/Media"` maps a root on this station.
//!
//! Profiles' pools and look may be changed by the users working in that
//! profile and by Admins.

use crate::commands::flags as F;
use std::collections::BTreeMap;

use lumacore_tree::{AssetRoot, MediaFile, Root, UserProfile};

use crate::errors::{render_code, reply};
use crate::objects::{Doc, ObjectDef, Scope, TreeDef, Undo, Verb, filter, out, quote};
use crate::props;
use crate::text::render_success;
use crate::vfs::{DynChild, NodeView, btree_to_children, object_view};
use crate::{AppState, DispatchResult};
use crate::claims::Claims;

type Edit<'a> = &'a mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>;

/// Profile files (`Export UserProfile`).
pub const PROFILES_DIR: &str = "library/userprofiles";

/// [`PROFILES_DIR`] (tests keep theirs out of the repo).
fn profiles_dir() -> String {
    if cfg!(test) {
        std::env::temp_dir().join("lumacore-test-userprofiles").to_string_lossy().into_owned()
    } else {
        PROFILES_DIR.to_string()
    }
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

fn words<'a>(args: &[&'a str]) -> Vec<&'a str> {
    F::positional(args, &[&F::DRIVE])
}

// Media's options (decisions §31: declared on the objects that take them).
pub const AS: F::FlagDef = F::FlagDef::new("As", "Import it under this name").value("name");
pub const HASH: F::FlagDef = F::FlagDef::new("Hash", "The file's SHA-256 (64 hex digits)").value("sha256");
pub const SIZE: F::FlagDef = F::FlagDef::new("Size", "The file's size in bytes").value("bytes");
const VERIFY: F::FlagDef = F::FlagDef::new("Verify", "Check every file's hash");
const MISSING: F::FlagDef = F::FlagDef::new("Missing", "Only the files that are missing");

// ── Owners and pools ─────────────────────────────────────────────────────────

/// Which pool a picture lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pool {
    /// `ShowData.Images`.
    Show,
    /// A profile's `Images` (by number).
    ProfileImages(u32),
    /// A profile's `Icons`.
    ProfileIcons(u32),
}

impl Pool {
    fn label(self, root: &Root) -> String {
        let profile = |no: u32| {
            root.show_data
                .user_profiles
                .get(&no)
                .and_then(|p| p.meta.identity.name.clone())
                .map_or(format!("UserProfile {no}"), |n| format!("UserProfile {}", quote(&n)))
        };
        match self {
            Pool::Show => "Image".into(),
            Pool::ProfileImages(no) => format!("{} Image", profile(no)),
            Pool::ProfileIcons(no) => format!("{} Icon", profile(no)),
        }
    }

    fn profile(self) -> Option<u32> {
        match self {
            Pool::Show => None,
            Pool::ProfileImages(no) | Pool::ProfileIcons(no) => Some(no),
        }
    }
}

fn pool(root: &Root, p: Pool) -> Option<&BTreeMap<u32, MediaFile>> {
    let show = &root.show_data;
    match p {
        Pool::Show => Some(&show.images),
        Pool::ProfileImages(no) => show.user_profiles.get(&no).map(|p| &p.content.images),
        Pool::ProfileIcons(no) => show.user_profiles.get(&no).map(|p| &p.content.icons),
    }
}

fn pool_mut(root: &mut Root, p: Pool) -> Option<&mut BTreeMap<u32, MediaFile>> {
    let show = &mut root.show_data;
    match p {
        Pool::Show => Some(&mut show.images),
        Pool::ProfileImages(no) => show.profile_mut(no).map(|p| &mut p.content.images),
        Pool::ProfileIcons(no) => show.profile_mut(no).map(|p| &mut p.content.icons),
    }
}

/// The profile the acting user works in.
fn own_profile(state: &AppState) -> Option<u32> {
    state.root.show_data.profile_id_for_user(state.effective_user_name())
}

/// A profile by number or name.
fn profile_no(state: &AppState, word: &str) -> Option<u32> {
    let word = unquote(word);
    let profiles = &state.root.show_data.user_profiles;
    if let Ok(no) = word.parse::<u32>() {
        return profiles.contains_key(&no).then_some(no);
    }
    profiles
        .iter()
        .find(|(_, p)| p.meta.identity.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(&word)))
        .map(|(&no, _)| no)
}

/// Users of the profile and Admins may change it (§30 "12a2"); nobody
/// changes the locked template Default (§30 "12c2", E-USER-002).
pub fn may_change_profile(state: &AppState, no: u32) -> Result<(), String> {
    if state.root.show_data.is_default_profile(no) {
        return Err(crate::errors::with_code(
            crate::commands::profiles::DEFAULT_LOCKED,
            "E-USER-002",
        ));
    }
    if state.is_admin() || own_profile(state) == Some(no) {
        return Ok(());
    }
    Err(crate::errors::with_code(
        format!(
            "Only the users of {} and Admins may change it.",
            Pool::ProfileImages(no).label(&state.root).trim_end_matches(" Image")
        ),
        "E-RIGHTS-001",
    ))
}

/// After `UserProfile`: `[<n>|"name"] Image|Icon …` → the pool and the rest.
/// None = no pool named (a profile-level verb: Import / Export a file).
fn profile_pool<'a>(state: &AppState, args: &'a [&'a str]) -> Result<Option<(Pool, &'a [&'a str])>, String> {
    let kind = |w: &str| {
        if IMAGE.matches(w) {
            Some(true)
        } else if ICON.matches(w) {
            Some(false)
        } else {
            None
        }
    };
    let make = |no: u32, image: bool| if image { Pool::ProfileImages(no) } else { Pool::ProfileIcons(no) };
    match args {
        [first, rest @ ..] if kind(first).is_some() => {
            let no = own_profile(state).ok_or("You have no profile.")?;
            Ok(Some((make(no, kind(first) == Some(true)), rest)))
        }
        [owner, second, rest @ ..] if kind(second).is_some() => {
            let no = profile_no(state, owner).ok_or_else(|| format!("No UserProfile {owner}."))?;
            Ok(Some((make(no, kind(second) == Some(true)), rest)))
        }
        _ => Ok(None),
    }
}

/// The free number after the highest.
fn next_no(map: &BTreeMap<u32, MediaFile>) -> u32 {
    map.keys().next_back().map_or(1, |n| n + 1)
}

fn changed(state: &mut AppState) {
    state.show_dirty = true;
}

// ── Pool verbs (shared by Image, Icon and UserProfile …) ─────────────────────

fn guard(state: &AppState, p: Pool) -> Result<(), String> {
    match p.profile() {
        Some(no) => may_change_profile(state, no),
        None => Ok(()),
    }
}

/// `[<n>] "file"` → import from `library/media/`.
fn pool_import(state: &mut AppState, p: Pool, args: &[&str]) -> String {
    let w = words(args);
    let (no, file) = match w.as_slice() {
        [no, file, ..] if no.parse::<u32>().is_ok() => (no.parse::<u32>().ok(), *file),
        [file, ..] => (None, *file),
        [] => {
            // The station's "which file?" ask (decisions §38 5): the files element on media/.
            let label = p.label(&state.root);
            state.asking = Some(
                crate::dialogs::Ask::new(format!("Import {label}"), "A picture file of the library becomes an image.")
                    .file("file", "File", "media")
                    .button("Import", lumacore_wire::ButtonKind::Primary, format!("Import {label} {{file}}"))
                    .cancel("Cancel")
                    .flags("the file: Import Image \"file\" [/Drive \"usb1\"]"),
            );
            return render_code(
                format!("Usage: Import {label} [<n>] \"file\" (a file in {}/)", lumacore_luma::media::LIBRARY),
                "E-CMD-006",
            );
        }
    };
    if let Err(e) = guard(state, p) {
        return reply(e, "E-RIGHTS-001");
    }
    let drive = F::value(args, &F::DRIVE);
    let dir = match drive.as_deref() {
        None => lumacore_luma::media::library_dir(),
        Some(d) => match state.library_folder(Some(d), "media", false) {
            Ok(dir) => dir,
            Err(e) => return e,
        },
    };
    let Some(map) = pool(&state.root, p) else {
        return render_code("No such profile.", "E-CMD-007");
    };
    let no = no.unwrap_or_else(|| next_no(map));
    if map.contains_key(&no) && !F::has(args, &F::OVERWRITE) {
        return render_code(
            format!("{} {no} exists. Add /Overwrite to replace it.", p.label(&state.root)),
            "E-CMD-010",
        );
    }
    let file_mf = match lumacore_luma::media::import_file_in(&state.root.show_data, &dir, file, None) {
        Ok(mf) => mf,
        Err(e) => return reply(e, "E-MEDIA-001"),
    };
    let label = p.label(&state.root);
    let pool = pool_mut(&mut state.root, p).expect("checked above");
    // Overwriting keeps the object's GUID (§21 "6d").
    let file_mf = match pool.get(&no) {
        Some(old) => MediaFile { meta: lumacore_base::NodeMeta { identity: old.meta.identity.clone(), ..file_mf.meta }, ..file_mf },
        None => file_mf,
    };
    let msg = format!(
        "{label} {no} \"{}\" — {}×{} {}, {} KB",
        file_mf.meta.name().unwrap_or_default(),
        file_mf.width,
        file_mf.height,
        file_mf.ext,
        file_mf.size.div_ceil(1024)
    );
    pool.insert(no, file_mf);
    changed(state);
    render_success(&msg)
}

fn pool_item(state: &AppState, p: Pool, word: Option<&&str>) -> Result<(u32, MediaFile), String> {
    let label = p.label(&state.root);
    let map = pool(&state.root, p).ok_or("No such profile.")?;
    let word = word.ok_or_else(|| format!("Usage: … {label} <n>"))?;
    let found = match word.parse::<u32>() {
        Ok(no) => map.get(&no).map(|m| (no, m.clone())),
        Err(_) => {
            let name = unquote(word);
            map.iter().find(|(_, m)| m.meta.name().unwrap_or_default().eq_ignore_ascii_case(&name)).map(|(&n, m)| (n, m.clone()))
        }
    };
    found.ok_or_else(|| format!("No {label} {word}."))
}

fn pool_delete(state: &mut AppState, p: Pool, args: &[&str]) -> String {
    let (no, item) = match pool_item(state, p, args.first()) {
        Ok(found) => found,
        Err(e) => return render_code(e, "E-CMD-007"),
    };
    if let Err(e) = guard(state, p) {
        return reply(e, "E-RIGHTS-001");
    }
    let label = p.label(&state.root);
    pool_mut(&mut state.root, p).expect("found above").remove(&no);
    changed(state);
    // Links to it stay and are listed (List Reference /Broken), never dropped.
    render_success(&format!("Deleted {label} {no} \"{}\"", item.meta.name().unwrap_or_default()))
}

fn pool_label(state: &mut AppState, p: Pool, args: &[&str]) -> String {
    let (no, _) = match pool_item(state, p, args.first()) {
        Ok(found) => found,
        Err(e) => return render_code(e, "E-CMD-007"),
    };
    let Some(name) = args.get(1).map(|n| unquote(n)).filter(|n| !n.is_empty()) else {
        return render_code(format!("Usage: Label {} {no} \"name\"", p.label(&state.root)), "E-CMD-006");
    };
    if let Err(e) = guard(state, p) {
        return reply(e, "E-RIGHTS-001");
    }
    let label = p.label(&state.root);
    if let Some(m) = pool_mut(&mut state.root, p).and_then(|m| m.get_mut(&no)) {
        m.meta.identity.name = Some(name.clone());
    }
    changed(state);
    render_success(&format!("{label} {no} = \"{name}\""))
}

/// `Copy <pool> <n> At <pool> [<m>]` — between any two pools; the copy gets a
/// new GUID.
fn pool_copy(state: &mut AppState, p: Pool, args: &[&str]) -> String {
    let (_, item) = match pool_item(state, p, args.first()) {
        Ok(found) => found,
        Err(e) => return render_code(e, "E-CMD-007"),
    };
    let usage = || render_code(
        "Usage: Copy UserProfile Icon 3 At Image 10 (any of Image, Icon, UserProfile [<n>] Image|Icon)",
        "E-CMD-006",
    );
    let Some(at) = args.iter().position(|a| a.eq_ignore_ascii_case("at")) else {
        return usage();
    };
    let target = &args[at + 1..];
    let parsed: Result<Option<(Pool, &[&str])>, String> = match target.first() {
        Some(w) if crate::objects::USER_PROFILE.matches(w) => profile_pool(state, &target[1..]),
        Some(w) if IMAGE.matches(w) => Ok(Some((Pool::Show, &target[1..]))),
        Some(w) if ICON.matches(w) => match own_profile(state) {
            Some(no) => Ok(Some((Pool::ProfileIcons(no), &target[1..]))),
            None => Err("You have no profile.".into()),
        },
        _ => Ok(None),
    };
    let (to, rest) = match parsed {
        Ok(Some(found)) => found,
        Ok(None) => return usage(),
        Err(e) => return render_code(e, "E-CMD-007"),
    };
    if let Err(e) = guard(state, to) {
        return reply(e, "E-RIGHTS-001");
    }
    let Some(map) = pool(&state.root, to) else {
        return render_code("No such profile.", "E-CMD-007");
    };
    let no = rest.first().and_then(|n| n.parse::<u32>().ok()).unwrap_or_else(|| next_no(map));
    if map.contains_key(&no) {
        return render_code(format!("{} {no} exists.", to.label(&state.root)), "E-CMD-010");
    }
    let label = to.label(&state.root);
    let mut copy = item;
    copy.meta.identity.guid = uuid::Uuid::new_v4();
    let name = copy.meta.name().unwrap_or_default().to_string();
    pool_mut(&mut state.root, to).expect("checked above").insert(no, copy);
    changed(state);
    render_success(&format!("Copied to {label} {no} \"{name}\""))
}

/// `Export <pool> <n>`: the file back into `library/media/`.
fn pool_export(state: &mut AppState, p: Pool, args: &[&str]) -> String {
    let (_, item) = match pool_item(state, p, args.first()) {
        Ok(found) => found,
        Err(e) => return render_code(e, "E-CMD-007"),
    };
    let Some(bytes) = lumacore_luma::media::stored(&item.sha256, &item.ext) else {
        return render_code(
            format!("This station doesn't have the file of \"{}\" (yet).", item.meta.name().unwrap_or_default()),
            "E-CMD-012",
        );
    };
    match lumacore_luma::media::save_upload(&format!("{}.{}", item.meta.name().unwrap_or_default(), item.ext), &bytes) {
        Ok(name) => render_success(&format!("Exported to {}/{name}", lumacore_luma::media::LIBRARY)),
        Err(e) => reply(e, "E-CMD-012"),
    }
}

fn pool_list(state: &AppState, p: Pool) -> String {
    let Some(map) = pool(&state.root, p) else {
        return render_code("No such profile.", "E-CMD-007");
    };
    let label = p.label(&state.root);
    if map.is_empty() {
        return format!("{label}: none");
    }
    let mut lines = vec![format!("{label}:")];
    for (no, m) in map {
        let here = if lumacore_luma::media::has(&m.sha256, &m.ext) { "" } else { "  (file missing here)" };
        lines.push(format!(
            "  {no:>3}  {:<24} {:>5}×{:<5} {:<4} {:>6} KB{here}",
            quote(m.meta.name().unwrap_or_default()),
            m.width,
            m.height,
            m.ext,
            m.size.div_ceil(1024)
        ));
    }
    lines.join("\n")
}

fn pool_info(state: &AppState, p: Pool, args: &[&str]) -> String {
    match pool_item(state, p, args.first()) {
        Ok((no, m)) => format!(
            "{} {no} \"{}\"\n  GUID     {}\n  File     {}.{} ({} bytes, {}×{})\n  SHA-256  {}\n  From     {}\n  Here     {}",
            p.label(&state.root),
            m.meta.name().unwrap_or_default(),
            m.meta.identity.guid,
            m.meta.name().unwrap_or_default(),
            m.ext,
            m.size,
            m.width,
            m.height,
            m.sha256,
            m.source,
            if lumacore_luma::media::has(&m.sha256, &m.ext) { "in the store" } else { "missing (fetched from the master in a LumaSession)" }
        ),
        Err(e) => render_code(e, "E-CMD-007"),
    }
}

/// One verb on a pool.
fn on_pool(state: &mut AppState, verb: Verb, p: Pool, args: &[&str]) -> String {
    match verb {
        Verb::Import => pool_import(state, p, args),
        Verb::Export => pool_export(state, p, args),
        Verb::Delete => pool_delete(state, p, args),
        Verb::Label => pool_label(state, p, args),
        Verb::Copy => pool_copy(state, p, args),
        Verb::Info if !args.is_empty() => pool_info(state, p, args),
        Verb::List | Verb::Info | Verb::Call => pool_list(state, p),
        other => render_code(format!("{} doesn't take {}.", p.label(&state.root), other.label()), "E-CMD-004"),
    }
}

// ── Image (the show's pool) ──────────────────────────────────────────────────

fn image_verb(verb: Verb) -> fn(&mut AppState, &[&str]) -> DispatchResult {
    match verb {
        Verb::Import => |s, a| out(on_pool(s, Verb::Import, Pool::Show, a)),
        Verb::Export => |s, a| out(on_pool(s, Verb::Export, Pool::Show, a)),
        Verb::Delete => |s, a| out(on_pool(s, Verb::Delete, Pool::Show, a)),
        Verb::Label => |s, a| out(on_pool(s, Verb::Label, Pool::Show, a)),
        Verb::Copy => |s, a| out(on_pool(s, Verb::Copy, Pool::Show, a)),
        Verb::Info => |s, a| out(on_pool(s, Verb::Info, Pool::Show, a)),
        _ => |s, a| out(on_pool(s, Verb::List, Pool::Show, a)),
    }
}

fn media_reach(verb: Verb) -> crate::commands::registry::Reach {
    use crate::commands::registry::Reach;
    if matches!(verb, Verb::Import | Verb::Export) { Reach::Drive } else { Reach::Show }
}

fn images_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.images)
}
fn image_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.images.get(&id).map(|t| object_view(id, t))
}
fn image_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let m = root.show_data.images.get_mut(&id).ok_or("no such image")?;
    props::edit_json(m, edit)
}
fn image_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(state.root.show_data.images.keys().map(u32::to_string), prefix)
}

const NAME_PROP: [props::Property; 1] = [props::Property {
    name: "Name",
    access: crate::props::Access::Field("name"),
    kind: props::PropKind::Text,
    rights: crate::commands::registry::Rights::Program,
    doc: "display name",
    ..crate::props::Property::BASE
}];

pub static IMAGE: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Image",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[F::OVERWRITE.on(&[Verb::Import]), F::DRIVE.on(&[Verb::Import])],
        aliases: &["images"],
        doc: Doc {
            display: "Image",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Import Image [<n>] \"file\" | Delete|Label|Copy|Export|Info Image <n> | List Image | UserProfile [<n>] Image <n>",
            description: "The show's pictures, for show data (decisions §30 \"12a2\"). A picture is \
                          embedded in the .luma beside its manifest (GUID, SHA-256, size); files come \
                          from library/media/ (16 MB each, 256 MB per show). A profile has its own \
                          Images and Icons: `UserProfile Image 3` (owner first).",
            examples: &[
                "Import Image \"stage.jpg\"                -- into the show's pool",
                "Import UserProfile Image \"bg.jpg\"       -- into your profile's pool",
                "Label Image 1 \"Stage\"",
                "Copy UserProfile Icon 3 At Image 10     -- a copy with a new GUID",
                "Export Image 1                          -- back to library/media/",
                "List Image",
            ],
            related: &["icon", "userprofile", "assetroot"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: media_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Import, |s, a| image_verb(Verb::Import)(s, a)),
        (Verb::Export, |s, a| image_verb(Verb::Export)(s, a)),
        (Verb::Delete, |s, a| image_verb(Verb::Delete)(s, a)),
        (Verb::Label, |s, a| image_verb(Verb::Label)(s, a)),
        (Verb::Copy, |s, a| image_verb(Verb::Copy)(s, a)),
        (Verb::List, |s, a| image_verb(Verb::List)(s, a)),
        (Verb::Info, |s, a| image_verb(Verb::Info)(s, a)),
    ],
    delete_ask: None,
    complete: Some(image_complete),
    props: Some(&crate::props::Props { lists: &[&NAME_PROP], apply: Some(image_write), may: None, ..crate::props::Props::BASE }),
    tree: Some(TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::Images,
        children: images_children,
        object: image_view,
    }),
};

// ── Icon (your profile's pool) ───────────────────────────────────────────────

fn own_icons(state: &AppState) -> Result<Pool, String> {
    own_profile(state).map(Pool::ProfileIcons).ok_or_else(|| "You have no profile.".into())
}

fn icon_on(state: &mut AppState, verb: Verb, args: &[&str]) -> DispatchResult {
    out(match own_icons(state) {
        Ok(p) => on_pool(state, verb, p, args),
        Err(e) => render_code(e, "E-CMD-011"),
    })
}

fn icon_complete(state: &AppState, prefix: &str) -> Vec<String> {
    let keys = own_profile(state)
        .and_then(|no| state.root.show_data.user_profiles.get(&no))
        .map(|p| p.content.icons.keys().map(u32::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    filter(keys, prefix)
}

pub static ICON: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Icon",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[F::OVERWRITE.on(&[Verb::Import]), F::DRIVE.on(&[Verb::Import])],
        aliases: &["icons"],
        doc: Doc {
            display: "Icon",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Import Icon [<n>] \"file\" | Delete|Label|Copy|Export|Info Icon <n> | UserProfile [<n>|\"name\"] Icon <n>",
            description: "A profile's own icons, for its own UI (views, view buttons, keys, screens) — \
                          they travel with the profile. `Icon 3` is your profile's; `UserProfile \"Bob\" \
                          Icon 3` names another. The show itself has no icons: the standard set (Lucide \
                          + cat glyphs) ships with LumaCore; pictures for show data are `Image`s.",
            examples: &[
                "Import Icon \"star.svg\"",
                "Import UserProfile \"Bob\" Icon \"star.svg\"",
                "Copy Icon 3 At Image 10   -- into the show's pool",
                "List Icon",
            ],
            related: &["image", "userprofile"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: crate::objects::on_master,
        reach: media_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Import, |s, a| icon_on(s, Verb::Import, a)),
        (Verb::Export, |s, a| icon_on(s, Verb::Export, a)),
        (Verb::Delete, |s, a| icon_on(s, Verb::Delete, a)),
        (Verb::Label, |s, a| icon_on(s, Verb::Label, a)),
        (Verb::Copy, |s, a| icon_on(s, Verb::Copy, a)),
        (Verb::List, |s, a| icon_on(s, Verb::List, a)),
        (Verb::Info, |s, a| icon_on(s, Verb::Info, a)),
    ],
    delete_ask: None,
    complete: Some(icon_complete),
    props: None,
    tree: None,
};

// ── UserProfile verbs: its pools, export / import ────────────────────────────

/// `<verb> UserProfile …`: a pool verb when a pool is named, else the
/// profile's own (List / Info / Dump the profile, Import / Export its file).
pub fn profile_verb(
    state: &mut AppState,
    verb: Verb,
    args: &[&str],
    fallback: fn(&mut AppState, &[&str]) -> DispatchResult,
) -> DispatchResult {
    match profile_pool(state, args) {
        Ok(Some((p, rest))) => out(on_pool(state, verb, p, rest)),
        Ok(None) => match verb {
            Verb::Export => out(profile_export(state, args)),
            Verb::Import => out(profile_import(state, args)),
            _ => fallback(state, args),
        },
        Err(e) => out(render_code(e, "E-CMD-007")),
    }
}

fn file_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
}

/// `Export UserProfile [<n>|"name"] ["file"]` → `library/userprofiles/<name>.lumaprofile`.
fn profile_export(state: &mut AppState, args: &[&str]) -> String {
    let w = words(args);
    let no = match w.first() {
        Some(word) => match profile_no(state, word) {
            Some(no) => no,
            None => return render_code(format!("No UserProfile {word}."), "E-CMD-007"),
        },
        None => match own_profile(state) {
            Some(no) => no,
            None => return render_code("You have no profile.", "E-CMD-011"),
        },
    };
    let profile = &state.root.show_data.user_profiles[&no];
    let name = w
        .get(1)
        .map(|f| unquote(f).trim_end_matches(".lumaprofile").to_string())
        .or_else(|| profile.meta.name().map(str::to_string))
        .unwrap_or_else(|| format!("profile{no}"));
    // A typed file may name a sub-folder (issue 07); a profile's own name is
    // made a file name first.
    let typed = w.get(1).is_some();
    let name = if typed { name } else { file_name(&name) };
    let dir = match F::value(args, &F::DRIVE) {
        None => profiles_dir(),
        Some(d) => match state.library_folder(Some(&d), "userprofiles", true) {
            Ok(dir) => dir,
            Err(e) => return e,
        },
    };
    let path = match lumacore_luma::drive::library_file(&dir, &format!("{name}.lumaprofile")) {
        Ok(path) => path,
        Err(e) => return render_code(e, "E-CMD-008"),
    };
    let (bytes, missing) = match lumacore_luma::show_file::encode_profile(no, profile) {
        Ok(b) => b,
        Err(e) => return reply(e, "E-CMD-012"),
    };
    let written = path
        .parent()
        .map_or(Ok(()), lumacore_luma::drive::create_dir_all)
        .and_then(|_| lumacore_luma::drive::write_atomic(&path, &bytes));
    let path = path.display().to_string();
    match written {
        Ok(()) => {
            let note = if missing.is_empty() {
                String::new()
            } else {
                format!(" ({} picture files weren't on this station)", missing.len())
            };
            render_success(&format!("UserProfile {no} exported to {path}{note}"))
        }
        Err(e) => render_code(format!("Cannot write {path}: {e}"), "E-CMD-012"),
    }
}

/// `/As "name"` in an Import line, and the arguments without it. An empty
/// name (the ask's empty field) is none.
fn take_as<'a>(args: &[&'a str]) -> (Option<String>, Vec<&'a str>) {
    (F::value(args, &AS).filter(|n| !n.is_empty()), F::without(args, &AS))
}

/// `Import UserProfile "Bob" [/As "name"] [/Overwrite]` ←
/// `library/userprofiles/Bob.lumaprofile`. With `/As` it is a new profile
/// with new GUIDs (a copy, the next free name). Without, the same GUID
/// replaces that profile (with /Overwrite), else it is a new profile with
/// its GUIDs kept (a move between shows, §21 "6d"). A bare Import asks for
/// the file. Never a second template: an imported profile is never locked.
/// Admin's (declared: `profiles::rights`).
fn profile_import(state: &mut AppState, args: &[&str]) -> String {
    let (as_name, args) = take_as(args);
    let args = args.as_slice();
    let Some(file) = words(args).first().map(|f| unquote(f)) else {
        return crate::commands::profiles::ask_import(state, as_name.as_deref().unwrap_or(""));
    };
    let dir = match F::value(args, &F::DRIVE) {
        None => profiles_dir(),
        Some(d) => match state.library_folder(Some(&d), "userprofiles", false) {
            Ok(dir) => dir,
            Err(e) => return e,
        },
    };
    // A file in library/userprofiles/ (sub-folders allowed; issue 07).
    let path = match lumacore_luma::drive::library_file(&dir, &format!("{}.lumaprofile", file.trim_end_matches(".lumaprofile"))) {
        Ok(path) => path.display().to_string(),
        Err(e) => return render_code(e, "E-MEDIA-001"),
    };
    let bytes = match lumacore_luma::drive::read(&path) {
        Ok(b) => b,
        Err(e) => return render_code(format!("Cannot read {path}: {e}"), "E-MEDIA-001"),
    };
    let (mut profile, media) = match lumacore_luma::show_file::decode_profile(&bytes) {
        Ok(p) => p,
        Err(e) => return reply(e, "E-CMD-012"),
    };
    profile.locked = false;
    // `/As "name"`: a copy with new GUIDs under that name.
    if let Some(name) = as_name {
        let name = crate::commands::users::free_profile_name(state, &name);
        let embedded: u64 = media.iter().map(|(_, b)| b.len() as u64).sum();
        if lumacore_luma::media::embedded_bytes(&state.root.show_data) + embedded > lumacore_tree::media::MAX_SHOW_BYTES {
            return render_code("The show would embed more than 256 MB.", "E-MEDIA-004");
        }
        for (ext, data) in &media {
            let _ = lumacore_luma::media::store(data, ext);
        }
        let show = &mut state.root.show_data;
        let no = show.user_profiles.keys().next_back().map_or(1, |n| n + 1);
        let copy = profile.copy_renewed(Some(name.clone()));
        show.user_profiles.insert(no, std::sync::Arc::new(copy));
        // It keeps its own attribute layer and learns what it lacks (§30 "12f-a").
        crate::commands::attributes::adopt_profile(state, no, false);
        changed(state);
        return render_success(&format!("UserProfile {no} \"{name}\" imported from {path} (a copy, new GUIDs)"));
    }
    let show = &state.root.show_data;
    let existing = show.user_profiles.iter().find(|(_, p)| p.meta.identity.guid == profile.meta.identity.guid).map(|(&n, _)| n);
    if let Some(no) = existing {
        if !F::has(args, &F::OVERWRITE) {
            // Ask (decisions §30 "12b"); one-shot paths add /Overwrite.
            let name = show.user_profiles.get(&no).and_then(|p| p.meta.identity.name.clone()).unwrap_or_default();
            let line = format!("Import UserProfile \"{file}\" /Overwrite");
            state.asking = Some(
                crate::dialogs::Ask::new(
                    format!("Import UserProfile \"{file}\""),
                    format!(
                        "This show already has the profile \"{name}\" (UserProfile {no}, same GUID). Replace it with the one from {file}.lumaprofile?"
                    ),
                )
                .meta("same GUID", Some(lumacore_wire::Tone::Warn))
                .button("Overwrite", lumacore_wire::ButtonKind::Danger, line)
                .cancel("Cancel")
                .flags("/Overwrite"),
            );
            return render_code(
                format!("UserProfile {no} is this profile (same GUID). Add /Overwrite to replace it."),
                "E-CMD-010",
            );
        }
        if let Err(e) = may_change_profile(state, no) {
            return reply(e, "E-RIGHTS-001");
        }
    }
    let embedded: u64 = media.iter().map(|(_, b)| b.len() as u64).sum();
    if lumacore_luma::media::embedded_bytes(show) + embedded > lumacore_tree::media::MAX_SHOW_BYTES {
        return render_code("The show would embed more than 256 MB.", "E-MEDIA-004");
    }
    for (ext, data) in &media {
        let _ = lumacore_luma::media::store(data, ext);
    }
    let no = existing.unwrap_or_else(|| show.user_profiles.keys().next_back().map_or(1, |n| n + 1));
    let name = profile.meta.identity.name.clone().unwrap_or_default();
    // A profile from another show must not bring GUIDs this show uses elsewhere.
    let mut trial = state.root.show_data.clone();
    trial.user_profiles.insert(no, std::sync::Arc::new(profile.clone()));
    let report = trial.identity();
    if !report.duplicates.is_empty() {
        return render_code(
            "The profile shares GUIDs with other objects in this show; it was not imported.",
            "E-SHOW-003",
        );
    }
    // Windows of a type this build doesn't know go, like on a show load.
    let dropped = profile.drop_unknown_windows();
    for note in &dropped {
        lumacore_sysmon::sysmon_note("Show", format!("[WARN] {note}"));
    }
    state.root.show_data.user_profiles.insert(no, std::sync::Arc::new(profile));
    crate::commands::attributes::adopt_profile(state, no, false);
    changed(state);
    let mut reply = render_success(&format!(
        "UserProfile {no} \"{name}\" {} from {path}",
        if existing.is_some() { "replaced" } else { "imported" }
    ));
    for note in dropped {
        reply.push('\n');
        reply.push_str(&note);
    }
    reply
}

// ── Set UserProfile … Theme / LockScreen ─────────────────────────────────────

const TEXT: props::PropKind = props::PropKind::Text;
const TAKE: &[&str] = &["None", "Take", "Offer"];
const GUEST: crate::commands::registry::Rights = crate::commands::registry::Rights::Guest;
const ADJUST: props::PropKind = props::PropKind::Int { min: -100, max: 100 };

macro_rules! colour {
    ($name:literal, $field:literal) => {
        props::Property {
            name: $name,
            section: "Look",
            access: crate::props::Access::Field(concat!("settings.theme.palette.", $field)),
            kind: props::PropKind::Colour,
            rights: GUEST,
            doc: concat!("palette colour ", $name, " (#rrggbb; none = the preset's)"),
            ..crate::props::Property::BASE
        }
    };
}

/// A profile's writable look (§30 "12a2"): users of the profile and Admins.
pub static PROFILE_PROPS: [props::Property; 38] = [
    props::Property { name: "Name", access: crate::props::Access::Field("meta.name"), kind: TEXT, rights: crate::commands::registry::Rights::Admin, doc: "display name", ..crate::props::Property::BASE },
    props::Property { name: "Note", access: crate::props::Access::Field("meta.note"), kind: TEXT, rights: GUEST, doc: "operator note", ..crate::props::Property::BASE },
    // The behaviour (decisions §30 "12b2": every preference a declared prop,
    // so the Users & Profiles window and EditSetting get them typed).
    props::Property { name: "MeowKey", access: crate::props::Access::Field("settings.preferences.meow_key"), kind: props::PropKind::Choice(&["Shift", "Alt", "Ctrl"]), rights: GUEST, doc: "the held key that acts as Meow", ..crate::props::Property::BASE },
    props::Property { name: "HoldTime", access: crate::props::Access::Field("settings.preferences.hold_ms"), kind: props::PropKind::Int { min: 100, max: 3000 }, rights: GUEST, doc: "press and hold, ms", ..crate::props::Property::BASE },
    props::Property { name: "DoublePress", access: crate::props::Access::Field("settings.preferences.double_click_ms"), kind: props::PropKind::Int { min: 100, max: 1000 }, rights: GUEST, doc: "a second press within this many ms is a double press", ..crate::props::Property::BASE },
    props::Property { name: "GridCursor", access: crate::props::Access::Field("settings.preferences.grid_cursor_move"), kind: props::PropKind::Choice(&["AppendX", "NewLine", "None"]), rights: GUEST, doc: "where a selection moves the grid cursor", ..crate::props::Property::BASE },
    props::Property { name: "HoldFollows", access: crate::props::Access::Field("settings.preferences.hold_follows"), kind: props::PropKind::OnOff, rights: GUEST, doc: "Hold full screen: the desk follows — a pane held full screen on a phone or tablet is maximised on the shared screen for every client (Off = only on that device)", ..crate::props::Property::BASE },
    props::Property { name: "PoolDrag", access: crate::props::Access::Field("settings.preferences.pool_drag"), kind: props::PropKind::Choice(&["Move", "Off"]), rights: GUEST, doc: "dragging a pool button moves it (Ctrl copies); Off on touch screens", ..crate::props::Property::BASE },
    props::Property { name: "CreateOops", access: crate::props::Access::Field("settings.preferences.oops.create_oops"), kind: props::PropKind::OnOff, rights: GUEST, doc: "file undo steps at all", ..crate::props::Property::BASE },
    props::Property { name: "OopsViews", access: crate::props::Access::Field("settings.preferences.oops.oops_views"), kind: props::PropKind::OnOff, rights: GUEST, doc: "windows, views, screens are undo steps", ..crate::props::Property::BASE },
    props::Property { name: "OopsProgrammer", access: crate::props::Access::Field("settings.preferences.oops.oops_programmer"), kind: props::PropKind::OnOff, rights: GUEST, doc: "programmer values are undo steps", ..crate::props::Property::BASE },
    props::Property { name: "OopsSelection", access: crate::props::Access::Field("settings.preferences.oops.oops_selection"), kind: props::PropKind::OnOff, rights: GUEST, doc: "selections are undo steps", ..crate::props::Property::BASE },
    props::Property { name: "OopsConfirmation", access: crate::props::Access::Field("settings.preferences.oops.confirmation"), kind: props::PropKind::Choice(&["Never", "Main", "Always"]), rights: GUEST, doc: "when Oops asks first", ..crate::props::Property::BASE },
    props::Property { name: "ShowDescriptions", access: crate::props::Access::Field("settings.show_descriptions"), kind: props::PropKind::OnOff, rights: GUEST, doc: "hints describe the words while typing", ..crate::props::Property::BASE },
    props::Property { name: "ClearBuffer", access: crate::props::Access::Field("settings.clear_buffer"), kind: props::PropKind::OnOff, rights: GUEST, doc: "the command line clears after each line", ..crate::props::Property::BASE },
    props::Property { name: "Suggestions", access: crate::props::Access::Field("settings.preferences.suggestions"), kind: props::PropKind::Choice(&["Auto", "TabOnly", "Off"]), rights: GUEST, section: "Command line", default: Some("Auto"), doc: "when the command line's suggestion list opens: Auto, only on Tab, or never", ..crate::props::Property::BASE },
    props::Property { name: "Verbosity", access: crate::props::Access::Field("settings.verbosity"), kind: props::PropKind::Choice(&["Summary", "Verbose", "Debug"]), rights: GUEST, section: "Command line", default: Some("Summary"), doc: "how much a reply tells (a line's /v or /d overrides it)", ..crate::props::Property::BASE },
    props::Property { name: "NotificationMode", access: crate::props::Access::Field("settings.preferences.notification_mode"), kind: props::PropKind::Choice(&["timed", "permanent"]), rights: GUEST, doc: "feedback cards close after 5 s or stay", ..crate::props::Property::BASE },
    props::Property { name: "PopupDim", access: crate::props::Access::Field("settings.preferences.popup_dim"), kind: props::PropKind::OnOff, rights: GUEST, doc: "dim the interface behind a pop-up (a tap outside closes it)", ..crate::props::Property::BASE },
    props::Property { name: "NavLog", access: crate::props::Access::Field("settings.preferences.nav_log"), kind: props::PropKind::Switch, rights: GUEST, section: "Command line", default: Some("On"), doc: "NAV's gesture log: three lines above the command line with the lines NAV made and what it copied (L in NAV)", ..crate::props::Property::BASE },
    props::Property { name: "SubAttributesFollow", access: crate::props::Access::Field("settings.preferences.sub_attributes_follow"), kind: props::PropKind::Choice(&["Ask", "Always", "Never"]), rights: GUEST, doc: "moving a main attribute brings its sub-attributes", ..crate::props::Property::BASE },
    // The MVR import preset (decisions §34 "Import MVR hardened" 2): the Options step's settings.
    props::Property { name: "MvrMatch", access: crate::props::Access::Field("settings.preferences.mvr.match_key"), kind: props::PropKind::Choice(&["Auto", "UUID", "FID", "FixtureName"]), rights: GUEST, section: "MVR import", default: Some("Auto"), doc: "how a file fixture finds its draft fixture: Auto = UUID, then FID, then the name", ..crate::props::Property::BASE },
    props::Property { name: "MvrLost", access: crate::props::Access::Field("settings.preferences.mvr.lost"), kind: props::PropKind::Choice(&["Mark", "Delete"]), rights: GUEST, section: "MVR import", default: Some("Mark"), doc: "a fixture in scope the file no longer has", ..crate::props::Property::BASE },
    props::Property { name: "MvrRotation", access: crate::props::Access::Field("settings.preferences.mvr.rotation"), kind: props::PropKind::Choice(&["Take", "Ignore"]), rights: GUEST, section: "MVR import", default: Some("Take"), doc: "the file's rotation with its position, or ignored", ..crate::props::Property::BASE },
    props::Property { name: "MvrMissing", access: crate::props::Access::Field("settings.preferences.mvr.missing"), kind: props::PropKind::Choice(&["Ask", "Recreate", "Parent", "None"]), rights: GUEST, section: "MVR import", default: Some("Ask"), doc: "a new fixture whose MVR group the show no longer has: ask, recreate the group, its parent, no group", ..crate::props::Property::BASE },
    props::Property { name: "MvrIdType", access: crate::props::Access::Field("settings.preferences.mvr.id_type"), kind: props::PropKind::Choice(&["Fixture", "Channel", "Generic", "Houselights", "NonDim", "Media", "Fog", "Effect", "Pyro", "Marker"]), rights: GUEST, section: "MVR import", default: Some("Generic"), doc: "the id type a CustomId the file never typed gets (some consoles write none; decisions §40 9, 10)", ..crate::props::Property::BASE },
    props::Property { name: "MvrAddresses", access: crate::props::Access::Field("settings.preferences.mvr.addresses"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Offer"), doc: "what the file may change: addresses", ..crate::props::Property::BASE },
    props::Property { name: "MvrIds", access: crate::props::Access::Field("settings.preferences.mvr.ids"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Offer"), doc: "what the file may change: FID, unit number, custom id", ..crate::props::Property::BASE },
    props::Property { name: "MvrNames", access: crate::props::Access::Field("settings.preferences.mvr.names"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Offer"), doc: "what the file may change: names", ..crate::props::Property::BASE },
    props::Property { name: "MvrPosition", access: crate::props::Access::Field("settings.preferences.mvr.position"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Offer"), doc: "what the file may change: position and rotation", ..crate::props::Property::BASE },
    props::Property { name: "MvrRevisions", access: crate::props::Access::Field("settings.preferences.mvr.revisions"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Offer"), doc: "what the file may change: the fixture type's revision and mode", ..crate::props::Property::BASE },
    props::Property { name: "MvrNewFixtures", access: crate::props::Access::Field("settings.preferences.mvr.new_fixtures"), kind: props::PropKind::Choice(TAKE), rights: GUEST, section: "MVR import", default: Some("Take"), doc: "fixtures the file adds", ..crate::props::Property::BASE },
    props::Property { name: "MvrFidOffset", access: crate::props::Access::Field("settings.preferences.mvr.fid_offset"), kind: props::PropKind::Int { min: -100000, max: 100000 }, rights: GUEST, section: "MVR import", default: Some("0"), doc: "added to the file's fixture numbers", ..crate::props::Property::BASE },
    props::Property { name: "MvrUniverseOffset", access: crate::props::Access::Field("settings.preferences.mvr.universe_offset"), kind: props::PropKind::Int { min: -10000, max: 10000 }, rights: GUEST, section: "MVR import", default: Some("0"), doc: "added to the file's universes", ..crate::props::Property::BASE },
    props::Property { name: "MvrX", access: crate::props::Access::Field("settings.preferences.mvr.x_mm"), kind: props::PropKind::Int { min: -1000000, max: 1000000 }, rights: GUEST, section: "MVR import", unit: "mm", default: Some("0"), doc: "the rig offset, X", ..crate::props::Property::BASE },
    props::Property { name: "MvrY", access: crate::props::Access::Field("settings.preferences.mvr.y_mm"), kind: props::PropKind::Int { min: -1000000, max: 1000000 }, rights: GUEST, section: "MVR import", unit: "mm", default: Some("0"), doc: "the rig offset, Y", ..crate::props::Property::BASE },
    props::Property { name: "MvrZ", access: crate::props::Access::Field("settings.preferences.mvr.z_mm"), kind: props::PropKind::Int { min: -1000000, max: 1000000 }, rights: GUEST, section: "MVR import", unit: "mm", default: Some("0"), doc: "the rig offset, Z", ..crate::props::Property::BASE },
    props::Property { name: "MvrRotate", access: crate::props::Access::Field("settings.preferences.mvr.rotate_deg"), kind: props::PropKind::Int { min: -360, max: 360 }, rights: GUEST, section: "MVR import", unit: "°", default: Some("0"), doc: "the rig rotation about Z", ..crate::props::Property::BASE },
];

/// A profile's theme (decisions §33 60, 63): the built-in base and its adjusters.
pub static THEME_PROPS: [props::Property; 7] = [
    props::Property { name: "Base", access: crate::props::Access::Field("settings.theme.preset"), kind: props::PropKind::Choice(lumacore_tree::media::THEME_PRESETS), rights: GUEST, doc: "built-in preset", section: "Look", default: Some("Dark"), ..crate::props::Property::BASE },
    props::Property { name: "Brightness", access: crate::props::Access::Field("settings.theme.adjust.brightness"), kind: ADJUST, rights: GUEST, doc: "adjuster, 0 = as the preset", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
    props::Property { name: "Contrast", access: crate::props::Access::Field("settings.theme.adjust.contrast"), kind: ADJUST, rights: GUEST, doc: "adjuster", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
    props::Property { name: "Saturation", access: crate::props::Access::Field("settings.theme.adjust.saturation"), kind: ADJUST, rights: GUEST, doc: "adjuster", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
    props::Property { name: "AccentHue", access: crate::props::Access::Field("settings.theme.adjust.accent_hue"), kind: props::PropKind::Int { min: -180, max: 180 }, rights: GUEST, doc: "turn the accent, degrees", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
    // The background adjusters (archive §30 "12g", replacing Tint): the neutral greys toward one hue.
    props::Property { name: "BackgroundHue", access: crate::props::Access::Field("settings.theme.adjust.background_hue"), kind: props::PropKind::Int { min: 0, max: 360 }, rights: GUEST, unit: "°", doc: "the hue the neutral greys turn toward", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
    props::Property { name: "BackgroundSaturation", access: crate::props::Access::Field("settings.theme.adjust.background_saturation"), kind: props::PropKind::Int { min: 0, max: 100 }, rights: GUEST, doc: "how far the neutral greys turn toward the background hue, 0 = the preset's", section: "Look", default: Some("0"), ..crate::props::Property::BASE },
];

/// The theme's colours (decisions §33 60, 62: screen colours are Paint; none = the base's).
pub static PAINT_PROPS: [props::Property; 14] = [
    colour!("Background", "background"),
    colour!("Surface", "surface"),
    colour!("Raised", "raised"),
    colour!("Line", "line"),
    colour!("Text", "text"),
    colour!("TextDim", "text_dim"),
    colour!("Accent", "accent"),
    colour!("Info", "info"),
    colour!("Ok", "ok"),
    colour!("Warn", "warn"),
    colour!("Alert", "alert"),
    colour!("Programmer", "programmer"),
    colour!("Blind", "blind"),
    colour!("Park", "park"),
];

/// A profile's lock screen (decisions §33 60).
pub static LOCK_SCREEN_PROPS: [props::Property; 4] = [
    props::Property { name: "Image", access: crate::props::Access::Field("settings.lock_screen.image"), kind: props::PropKind::OwnLink { kind: "Image", items: profile_image_items }, rights: GUEST, doc: "a picture of the profile's Images (number or name; none = built-in)", section: "Lock screen", ..crate::props::Property::BASE },
    props::Property { name: "ShowClock", access: crate::props::Access::Field("settings.lock_screen.show_clock"), kind: props::PropKind::OnOff, rights: GUEST, doc: "big clock and date", section: "Lock screen", default: Some("On"), ..crate::props::Property::BASE },
    props::Property { name: "Text", access: crate::props::Access::Field("settings.lock_screen.text"), kind: TEXT, rights: GUEST, doc: "a line under the clock", section: "Lock screen", ..crate::props::Property::BASE },
    props::Property { name: "AutoLock", access: crate::props::Access::Field("settings.lock_screen.auto_lock_minutes"), kind: props::PropKind::Int { min: 0, max: 1440 }, rights: GUEST, doc: "lock after idle minutes (0 = never)", section: "Lock screen", default: Some("0"), ..crate::props::Property::BASE },
];


/// The profile a Theme / LockScreen line means: the `UserProfile <x>` in front, else your own.
fn look_profile(state: &AppState) -> String {
    crate::commands::attributes::scoped_profile()
        .or_else(|| state.root.show_data.profile_id_for_user(state.effective_user_name()))
        .map(|n| n.to_string())
        .unwrap_or_default()
}

/// The profile's JSON: a Theme / Paint / LockScreen property's field lives in it.
fn profile_json(root: &Root, no: u32) -> Option<serde_json::Value> {
    serde_json::to_value(root.show_data.user_profiles.get(&no)?).ok()
}

macro_rules! look_props {
    ($lists:expr) => {
        props::Props {
            lists: $lists,
            keyed: Some(props::Keyed { rows: |s| vec![look_profile(s)], row: |_, _| None }),
            implicit: Some(look_profile),
            label: Some(|_, k| format!("UserProfile {k}")),
            apply: Some(profile_write),
            may: Some(profile_may),
            view: Some(profile_json),
            ..props::Props::BASE
        }
    };
}

static THEME_SETTINGS: props::Props = props::Props {
    // `Theme Paint …`: the colours of the same profile.
    within: Some(|_, _| Ok(Box::new(()))),
    ..look_props!(&[&THEME_PROPS])
};
static PAINT_SETTINGS: props::Props = look_props!(&[&PAINT_PROPS]);
static LOCK_SCREEN_SETTINGS: props::Props = look_props!(&[&LOCK_SCREEN_PROPS]);

/// `EditSetting Theme [Paint]` / `EditSetting LockScreen` (and the word typed alone): the
/// settings of the objects the line names, one-of-a-kind, so their words are the whole target.
fn edit_look(state: &mut AppState, target: &lumacore_syntax::ast::Target) -> DispatchResult {
    let words = crate::lum::vocabulary();
    let names: Vec<String> = target
        .items
        .iter()
        .flat_map(|(_, item)| match item {
            lumacore_syntax::ast::Item::Path(segments) => segments.iter().map(|s| words.word(s.object.node).name.clone()).collect(),
            _ => Vec::new(),
        })
        .collect();
    let line: Vec<&str> = names.iter().map(String::as_str).collect();
    out(crate::commands::edit_setting::cmd_edit_setting(state, &line))
}

macro_rules! look_edit {
    ($name:ident, $keyword:literal) => {
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "EditSetting", object = $keyword)]
        pub struct $name {
            #[lum(target)]
            pub target: lumacore_syntax::ast::Target,
        }

        impl crate::commands::typed::Command for $name {
            fn run(self, state: &mut AppState) -> DispatchResult {
                edit_look(state, &self.target)
            }
        }
    };
}

look_edit!(EditTheme, "Theme");
look_edit!(EditPaint, "Paint");
look_edit!(EditLockScreen, "LockScreen");

macro_rules! look_object {
    ($keyword:literal, $verbs:expr, $children:expr, $settings:expr, $syntax:literal, $description:literal, $examples:expr) => {
        ObjectDef {
            decl: crate::objects::ObjectDecl {
                keyword: $keyword,
                lum: crate::objects::LumShape::ONE.children($children),
                // Typed alone it opens its settings (decisions §33 63).
                default_verb: Verb::EditSetting,
                aliases: &[],
                doc: Doc {
                    display: $keyword,
                    category: crate::commands::help::HelpCategory::System,
                    also_applies_to: &[],
                    syntax: $syntax,
                    description: $description,
                    examples: $examples,
                    related: &["userprofile", "theme", "lockscreen", "paint"],
                    alias_depth: crate::commands::registry::AliasDepth::Base,
                    hidden: false,
                },
                family: None,
                call_undo: Undo::ReadOnly,
                call_class: crate::commands::registry::CommandClass::Commit,
                scope: crate::objects::on_master,
                reach: crate::objects::show_reach,
                rights: crate::commands::profiles::rights,
                planned: &[],
                flags: &[],
            },
            pool: None,
            verbs: $verbs,
            delete_ask: None,
            complete: None,
            props: Some(&$settings),
            tree: None,
        }
    };
}

pub static THEME: ObjectDef = look_object!(
    "Theme",
    &[(Verb::EditSetting, crate::commands::typed::typed::<EditTheme>)],
    &[&PAINT.decl],
    THEME_SETTINGS,
    "Set [UserProfile <n>] Theme \"<Property>\" <value> | Reset [UserProfile <n>] Theme",
    "A profile's look (decisions §33 60, 63): the built-in base (Dark, Daylight, Night) and its \
     adjusters; its colours are its Paint. Without a UserProfile in front it is your own.",
    &["Set Theme \"Base\" Night", "Set UserProfile \"Bob\" Theme \"Contrast\" 10", "Reset Theme", "Theme"]
);

pub static PAINT: ObjectDef = look_object!(
    "Paint",
    &[(Verb::EditSetting, crate::commands::typed::typed::<EditPaint>)],
    &[],
    PAINT_SETTINGS,
    "Set [UserProfile <n>] Theme Paint \"<Colour>\" \"#rrggbb\" | Reset … Theme Paint",
    "A theme's screen colours, sRGB (decisions §33 62); none set = the base's.",
    &["Set Theme Paint \"Accent\" \"#9a7a08\"", "Reset Theme Paint \"Accent\""]
);

pub static LOCK_SCREEN: ObjectDef = look_object!(
    "LockScreen",
    &[(Verb::EditSetting, crate::commands::typed::typed::<EditLockScreen>)],
    &[],
    LOCK_SCREEN_SETTINGS,
    "Set [UserProfile <n>] LockScreen \"<Property>\" <value> | Reset … LockScreen",
    "What a locked device shows (decisions §33 60): the profile's Image, the clock, a text, and \
     the minutes before it locks itself.",
    &["Set LockScreen \"Image\" 1", "Set LockScreen \"AutoLock\" 10", "Reset LockScreen"]
);

/// The Users & Profiles window's tab of a profile setting (decisions §30
/// "12b2": Look · Lock screen · Input & keys · Oops · Command line).
pub fn profile_section(name: &str) -> &'static str {
    // tidy: not-keywords (profile setting names, grouped into tabs)
    match name {
        "MeowKey" | "HoldTime" | "DoublePress" | "GridCursor" | "PoolDrag" | "HoldFollows" | "PopupDim" => "Input & keys",
        "SubAttributesFollow" => "Attributes",
        n if n.starts_with("Oops") || n == "CreateOops" => "Oops",
        "ShowDescriptions" | "ClearBuffer" | "NotificationMode" => "Command line",
        _ => "Look",
    }
}

/// Write a profile: the value was set in its JSON; the lock screen's picture is looked up in the profile's own
/// Images (a number or name becomes its GUID).
pub fn profile_write(root: &mut Root, _dp: Option<u32>, id: u32, edit: Edit) -> Result<(), String> {
    let profile = root.show_data.profile_mut(id).ok_or("no such profile")?;
    props::edit_json(profile, &mut |json| {
        edit(json)?;
        if let Some(word) = json["settings"]["lock_screen"]["image"].as_str().map(str::to_string)
            && uuid::Uuid::parse_str(&word).is_err()
        {
            let images = json["content"]["images"].as_object().cloned().unwrap_or_default();
            let found = images.iter().find(|(no, m)| {
                **no == word || m["meta"]["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(&word))
            });
            match found {
                Some((_, m)) => json["settings"]["lock_screen"]["image"] = m["meta"]["guid"].clone(),
                None => return Err(format!("the profile has no Image {word} (Import UserProfile Image \"file\")")),
            }
        }
        Ok(())
    })
}

/// A profile's own Images (its LockScreen.Image pick list).
fn profile_image_items(root: &Root, profile: u32) -> Vec<lumacore_wire::PickItem> {
    root.show_data
        .user_profiles
        .get(&profile)
        .map(|p| {
            p.content.images
                .iter()
                .map(|(no, m)| lumacore_wire::PickItem {
                    no: *no,
                    name: m.meta.name().unwrap_or_default().to_string(),
                    note: if m.width > 0 { format!("{}×{}", m.width, m.height) } else { m.ext.clone() },
                    guid: m.meta.identity.guid.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Users of the profile and Admins (`props::WriteDef::may`).
pub fn profile_may(state: &AppState, id: u32) -> Result<(), String> {
    may_change_profile(state, id)
}

/// The look of a profile, for clients (`/api/me`): theme and lock screen with
/// the picture's file named by hash.
pub fn look_of(profile: &UserProfile) -> serde_json::Value {
    let image = profile
        .settings.lock_screen
        .image
        .and_then(|g| profile.content.images.values().find(|m| m.meta.identity.guid == g))
        .map(|m| serde_json::json!({ "sha256": m.sha256, "ext": m.ext, "width": m.width, "height": m.height }));
    serde_json::json!({
        "theme": profile.settings.theme,
        "lock_screen": {
            "image": image,
            "show_clock": profile.settings.lock_screen.show_clock,
            "text": profile.settings.lock_screen.text,
            "auto_lock_minutes": profile.settings.lock_screen.auto_lock_minutes,
        },
    })
}

// ── Asset drives ─────────────────────────────────────────────────────────────

fn root_no(root: &Root, word: &str) -> Option<u32> {
    let word = unquote(word);
    let roots = &root.show_data.assets.roots;
    if let Ok(no) = word.parse::<u32>() {
        return roots.contains_key(&no).then_some(no);
    }
    // By GUID: the background hash's follow-up line (a rename meanwhile
    // can't break it).
    if let Ok(guid) = uuid::Uuid::parse_str(&word) {
        return roots.iter().find(|(_, r)| r.meta.identity.guid == guid).map(|(&n, _)| n);
    }
    roots.iter().find(|(_, r)| r.name.eq_ignore_ascii_case(&word)).map(|(&n, _)| n)
}

/// This station's folder of a root.
fn root_folder<'a>(state: &'a AppState, root: &AssetRoot) -> Option<&'a String> {
    state.root.station_settings.asset_roots.get(&root.name.to_lowercase())
}

fn assetroot_store(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let w = words(args);
    let (no, name) = match w.as_slice() {
        [no, name, ..] if no.parse::<u32>().is_ok() => (no.parse::<u32>().ok(), unquote(name)),
        [name, ..] => (None, unquote(name)),
        [] => return out(render_code("Usage: Store AssetRoot [<n>] \"Video\"", "E-CMD-006")),
    };
    let roots = &mut state.root.show_data.assets.roots;
    if roots.values().any(|r| r.name.eq_ignore_ascii_case(&name)) {
        return out(render_code(format!("AssetRoot \"{name}\" exists."), "E-CMD-010"));
    }
    let no = no.unwrap_or_else(|| roots.keys().next_back().map_or(1, |n| n + 1));
    if roots.contains_key(&no) {
        return out(render_code(format!("AssetRoot {no} exists."), "E-CMD-010"));
    }
    roots.insert(no, AssetRoot { meta: lumacore_base::NodeMeta::default(), name: name.clone() });
    changed(state);
    out(render_success(&format!(
        "AssetRoot {no} \"{name}\" — map it on each station: Set AssetRoot \"{name}\" \"Folder\" \"/Volumes/Media\""
    )))
}

fn assetroot_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some(no) = args.first().and_then(|w| root_no(&state.root, w)) else {
        return out(render_code("Usage: Delete AssetRoot <n|\"name\">", "E-CMD-006"));
    };
    let assets = &state.root.show_data.assets;
    let guid = assets.roots[&no].meta.identity.guid;
    if assets.files.values().any(|a| a.root == guid) {
        return out(render_code("Assets use this root — delete or move them first.", "E-CMD-010"));
    }
    let gone = state.root.show_data.assets.roots.remove(&no).expect("found above");
    changed(state);
    out(render_success(&format!("Deleted AssetRoot {no} \"{}\"", gone.name)))
}

fn assetroot_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (Some(no), Some(name)) = (args.first().and_then(|w| root_no(&state.root, w)), args.get(1).map(|n| unquote(n))) else {
        return out(render_code("Usage: Label AssetRoot <n> \"name\"", "E-CMD-006"));
    };
    // Every station's folder mapping follows the name when the change is
    // published there (`AppState::follow_renames`, R1-69).
    state.root.show_data.assets.roots.get_mut(&no).expect("found above").name = name.clone();
    changed(state);
    out(render_success(&format!("AssetRoot {no} = \"{name}\"")))
}

fn assetroot_list(state: &mut AppState, _: &[&str]) -> DispatchResult {
    let roots = &state.root.show_data.assets.roots;
    if roots.is_empty() {
        return out("AssetRoot: none (Store AssetRoot \"Video\")".into());
    }
    let mut lines = vec!["AssetRoot (folder on this station):".to_string()];
    for (no, r) in roots {
        let folder = match root_folder(state, r) {
            Some(f) if lumacore_luma::drive::metadata(f).is_ok() => f.clone(),
            Some(f) => format!("{f}  (not there)"),
            None => "not mapped here".into(),
        };
        lines.push(format!("  {no:>3}  {:<20} {folder}", quote(&r.name)));
    }
    out(lines.join("\n"))
}

fn assetroot_scope(verb: Verb, args: &[&str]) -> Scope {
    if verb == Verb::Set && args.iter().any(|a| a.eq_ignore_ascii_case("folder")) {
        Scope::Station
    } else {
        Scope::Master
    }
}

fn assetroots_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.assets.roots)
}
fn assetroot_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.assets.roots.get(&id).map(|t| object_view(id, t))
}
fn assetroot_complete(state: &AppState, prefix: &str) -> Vec<String> {
    filter(state.root.show_data.assets.roots.values().map(|r| format!("\"{}\"", r.name)), prefix)
}

pub static ASSET_ROOT: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "AssetRoot",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[],
        aliases: &["assetroots"],
        doc: Doc {
            display: "AssetRoot",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Store AssetRoot \"Video\" | Set AssetRoot \"Video\" \"Folder\" <path|none> | Delete|Label AssetRoot <n> | List AssetRoot",
            description: "A named place for large show-critical media that stays outside the .luma \
                          (decisions §8). The show names the root; each station maps it to a folder \
                          or drive of its own (station-local, like a network to a NIC).",
            examples: &[
                "Store AssetRoot \"Video\"",
                "Set AssetRoot \"Video\" \"Folder\" \"/Volumes/Media/Video\"",
                "List AssetRoot",
            ],
            related: &["asset", "image"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: assetroot_scope,
        reach: crate::objects::show_reach,
        rights: crate::rights::setup,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Info, assetroot_list),
        (Verb::Store, assetroot_store),
        (Verb::Delete, assetroot_delete),
        (Verb::Label, assetroot_label),
        (Verb::List, assetroot_list),
    ],
    delete_ask: None,
    complete: Some(assetroot_complete),
    props: Some(&crate::settings_props::ASSET_ROOT_SETTINGS),
    tree: Some(TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::AssetRoots,
        children: assetroots_children,
        object: assetroot_view,
    }),
};

/// SHA-256 of a large file, read in pieces.
fn hash_file(path: &str) -> std::io::Result<(String, u64)> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = lumacore_luma::drive::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut size = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((hasher.finalize().iter().map(|b| format!("{b:02x}")).collect(), size))
}

/// `Import Asset "Video" "clips/intro.mov" ["name"]`: a file under this
/// station's folder of the root; the show keeps root + path + SHA-256 + size.
fn asset_import(state: &mut AppState, args: &[&str]) -> DispatchResult {
    // `/Hash <sha256> /Size <bytes>`: the background hash's follow-up line
    // (R1-70, Q42) — the file was hashed off the state thread.
    let bad = |what: &str| out(render_code(format!("{what} Usage: … /Hash <64 hex digits> /Size <bytes>"), "E-CMD-008"));
    let mut hashed: (Option<String>, Option<u64>) = (None, None);
    if F::has(args, &HASH) {
        match F::value(args, &HASH).filter(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())) {
            Some(h) => hashed.0 = Some(h.to_ascii_lowercase()),
            None => return bad("/Hash takes a SHA-256 (64 hex digits)."),
        }
    }
    if F::has(args, &SIZE) {
        match F::value(args, &SIZE).and_then(|v| v.parse().ok()) {
            Some(n) => hashed.1 = Some(n),
            None => return bad("/Size takes a number of bytes."),
        }
    }
    let plain = F::without(&F::without(args, &HASH), &SIZE);
    if hashed.0.is_some() != hashed.1.is_some() {
        return bad("/Hash and /Size go together.");
    }
    let args: &[&str] = &plain;
    let w = words(args);
    let [root_word, rel, rest @ ..] = w.as_slice() else {
        return out(render_code("Usage: Import Asset \"<root>\" \"path/in/root\" [\"name\"]", "E-CMD-006"));
    };
    let Some(rno) = root_no(&state.root, root_word) else {
        return out(render_code(format!("No AssetRoot {root_word}."), "E-CMD-007"));
    };
    let root = state.root.show_data.assets.roots[&rno].clone();
    let rel = unquote(rel).replace('\\', "/");
    if rel.split('/').any(|p| p == "..") || rel.starts_with('/') {
        return out(render_code("The path must stay inside the root.", "E-MEDIA-001"));
    }
    let Some(folder) = root_folder(state, &root).cloned() else {
        return out(render_code(
            format!("AssetRoot \"{}\" isn't mapped on this station (Set AssetRoot … \"Folder\" …).", root.name),
            "E-CMD-010",
        ));
    };
    let path = format!("{}/{rel}", folder.trim_end_matches('/'));
    let name = rest.first().map(|n| unquote(n)).unwrap_or_else(|| {
        rel.rsplit('/').next().unwrap_or(&rel).to_string()
    });
    // The follow-up line quotes path and name: a `"` can't be in them.
    if rel.contains('"') || name.contains('"') {
        return out(render_code("A file or asset name can't contain '\"': rename the file or give another name.", "E-CMD-008"));
    }
    let (sha256, size) = match hashed {
        (Some(sha), Some(size)) => (sha, size),
        _ => {
            // Hash on a worker; the follow-up line stores it (R1-70).
            if let Err(e) = lumacore_luma::drive::metadata(&path) {
                return out(render_code(format!("Cannot read {path}: {e}"), "E-MEDIA-001"));
            }
            let line_root = root.meta.identity.guid;
            let (line_rel, line_name) = (rel.clone(), name.clone());
            lumacore_luma::background::spawn(state.effective_user_name(), &format!("Import Asset {line_rel}"), move || {
                let (sha, size) = hash_file(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
                Ok(format!(
                    "Import Asset {line_root} \"{line_rel}\" \"{line_name}\" /Hash {sha} /Size {size}"
                ))
            });
            return out(render_success(&format!(
                "Hashing {root_name}:{rel} in the background — the asset is added when it's done (System Monitor).",
                root_name = root.name
            )));
        }
    };
    let files = &mut state.root.show_data.assets.files;
    let no = files.keys().next_back().map_or(1, |n| n + 1);
    files.insert(no, lumacore_tree::Asset {
        meta: lumacore_base::NodeMeta::new(Some(name.clone())),
        root: root.meta.identity.guid,
        path: rel.clone(),
        sha256,
        size,
    });
    changed(state);
    out(render_success(&format!("Asset {no} \"{name}\" — {}:{rel}, {} MB", root.name, size / (1024 * 1024))))
}

fn asset_delete(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let Some(no) = args.first().and_then(|w| w.parse::<u32>().ok()) else {
        return out(render_code("Usage: Delete Asset <n>", "E-CMD-006"));
    };
    match state.root.show_data.assets.files.remove(&no) {
        Some(a) => {
            changed(state);
            out(render_success(&format!("Deleted Asset {no} \"{}\" (the file stays on the drive)", a.meta.name().unwrap_or_default())))
        }
        None => out(render_code(format!("No Asset {no}."), "E-CMD-007")),
    }
}

fn asset_label(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let (Some(no), Some(name)) = (args.first().and_then(|w| w.parse::<u32>().ok()), args.get(1).map(|n| unquote(n))) else {
        return out(render_code("Usage: Label Asset <n> \"name\"", "E-CMD-006"));
    };
    match state.root.show_data.assets.files.get_mut(&no) {
        Some(a) => {
            a.meta.identity.name = Some(name.clone());
            changed(state);
            out(render_success(&format!("Asset {no} = \"{name}\"")))
        }
        None => out(render_code(format!("No Asset {no}."), "E-CMD-007")),
    }
}

/// Whether an asset is on this station: its root mapped, the file there with
/// the right size (`/Verify` also checks the SHA-256).
fn asset_status(state: &AppState, a: &lumacore_tree::Asset, verify: bool) -> String {
    let Some(root) = state.root.show_data.assets.roots.values().find(|r| r.meta.identity.guid == a.root) else {
        return "missing root".into();
    };
    let Some(folder) = root_folder(state, root) else {
        return format!("missing — {} not mapped here", root.name);
    };
    let path = format!("{}/{}", folder.trim_end_matches('/'), a.path);
    match lumacore_luma::drive::metadata(&path) {
        Err(_) => "missing".into(),
        Ok(m) if m.len() != a.size => "differs (size)".into(),
        Ok(_) if verify => match hash_file(&path) {
            Ok((sha, _)) if sha == a.sha256 => "ok (verified)".into(),
            Ok(_) => "differs (SHA-256)".into(),
            Err(e) => format!("unreadable: {e}"),
        },
        Ok(_) => "ok".into(),
    }
}

fn asset_list(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let verify = F::has(args, &VERIFY);
    if verify {
        return verify_in_background(state);
    }
    let only_missing = F::has(args, &MISSING);
    let assets = &state.root.show_data.assets;
    if assets.files.is_empty() {
        return out("Asset: none (Import Asset \"<root>\" \"path\")".into());
    }
    let mut lines = vec!["Asset (on this station):".to_string()];
    for (no, a) in &assets.files {
        let status = asset_status(state, a, verify);
        if only_missing && status.starts_with("ok") {
            continue;
        }
        let root = assets.roots.values().find(|r| r.meta.identity.guid == a.root).map_or("?", |r| r.name.as_str());
        lines.push(format!("  {no:>3}  {:<20} {root}:{}  {} MB  {status}", quote(a.meta.name().unwrap_or_default()), a.path, a.size / (1024 * 1024)));
    }
    out(lines.join("\n"))
}

/// `List Asset /Verify`: every asset's SHA-256 checked on a worker (R1-70,
/// Q42); the result goes to the System Monitor.
fn verify_in_background(state: &mut AppState) -> DispatchResult {
    let jobs: Vec<(u32, String, Option<String>, String, u64)> = state
        .root
        .show_data
        .assets
        .files
        .iter()
        .map(|(no, a)| {
            let folder = state
                .root
                .show_data
                .assets
                .roots
                .values()
                .find(|r| r.meta.identity.guid == a.root)
                .and_then(|r| root_folder(state, r))
                .map(|f| format!("{}/{}", f.trim_end_matches('/'), a.path));
            (*no, a.meta.name().unwrap_or_default().to_string(), folder, a.sha256.clone(), a.size)
        })
        .collect();
    if jobs.is_empty() {
        return out("Asset: none (Import Asset \"<root>\" \"path\")".into());
    }
    let count = jobs.len();
    lumacore_luma::background::spawn(state.effective_user_name(), "List Asset /Verify", move || {
        let lines: Vec<String> = jobs
            .into_iter()
            .map(|(no, name, path, sha, size)| {
                let status = match path {
                    None => "missing — its root isn't mapped here".to_string(),
                    Some(p) => match hash_file(&p) {
                        Ok((s, n)) if s == sha && n == size => "ok (verified)".to_string(),
                        Ok(_) => "differs (SHA-256)".to_string(),
                        Err(e) => format!("unreadable: {e}"),
                    },
                };
                format!("Asset {no} \"{name}\": {status}")
            })
            .collect();
        for line in &lines {
            lumacore_sysmon::sysmon_note("Asset", line.clone());
        }
        // The follow-up: a plain list (read-only) so the user sees it ran.
        Ok("List Asset".to_string())
    });
    out(render_success(&format!("Verifying {count} asset(s) in the background — results in the System Monitor.")))
}

/// Listing assets (and `/Verify`) reports on this station's drives: it runs
/// here, also on a member; the rest changes the show (the master).
fn asset_scope(verb: Verb, _: &[&str]) -> Scope {
    if matches!(verb, Verb::List | Verb::Call | Verb::Info) { Scope::Station } else { Scope::Master }
}

fn asset_reach(verb: Verb) -> crate::commands::registry::Reach {
    use crate::commands::registry::Reach;
    if matches!(verb, Verb::Import | Verb::List | Verb::Call) { Reach::Drive } else { Reach::Show }
}

fn assets_children(root: &Root) -> Vec<DynChild> {
    btree_to_children(&root.show_data.assets.files)
}
fn asset_view(root: &Root, id: u32) -> Option<NodeView> {
    root.show_data.assets.files.get(&id).map(|t| object_view(id, t))
}

pub static ASSET: ObjectDef = ObjectDef {
    decl: crate::objects::ObjectDecl {
        keyword: "Asset",
        lum: crate::objects::LumShape::ids(1),
        default_verb: crate::objects::Verb::Info,
        flags: &[HASH.on(&[Verb::Import]), SIZE.on(&[Verb::Import]), VERIFY.on(&[Verb::List]), MISSING.on(&[Verb::List])],
        aliases: &["assets"],
        doc: Doc {
            display: "Asset",
            category: crate::commands::help::HelpCategory::System,
            also_applies_to: &[],
            syntax: "Import Asset \"<root>\" \"path\" [\"name\"] | Delete|Label Asset <n> | List Asset [/Missing] [/Verify]",
            description: "A large file on an asset drive (decisions §8): the show keeps its root, path, \
                          SHA-256 and size, never the file. List Asset shows whether each is on this \
                          station; a missing asset is listed, never a crash. Preloading into memory and \
                          the operator-approved media sync come later.",
            examples: &[
                "Import Asset \"Video\" \"clips/intro.mov\"",
                "List Asset /Missing",
                "List Asset /Verify      -- also checks every SHA-256 (slow)",
            ],
            related: &["assetroot", "image"],
            alias_depth: crate::commands::registry::AliasDepth::Base,
            hidden: false,
        },
        family: None,
        call_undo: Undo::ReadOnly,
        call_class: crate::commands::registry::CommandClass::Commit,
        scope: asset_scope,
        reach: asset_reach,
        rights: crate::rights::content,
        planned: &[],
    },
    pool: None,
    verbs: &[
        (Verb::Info, asset_list),
        (Verb::Import, asset_import),
        (Verb::Delete, asset_delete),
        (Verb::Label, asset_label),
        (Verb::List, asset_list),
    ],
    delete_ask: None,
    complete: None,
    props: None,
    tree: Some(TreeDef {
        door: None,
        datapool: None,
        name: None,
        node: crate::tree_path::Node::AssetFiles,
        children: assets_children,
        object: asset_view,
    }),
};
