// ██████╗  ██████╗  ██████╗ ██████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██╔══██╗
// ██║  ██║██║   ██║██║   ██║██████╔╝
// ██║  ██║██║   ██║██║   ██║██╔══██╗
// ██████╔╝╚██████╔╝╚██████╔╝██║  ██║
// ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
// FILE: door

//! The tree shell's `mv`, `cp` and `rm` (decisions §35 1, 19): one door per provider. On the show
//! tree they are written out as the Lum line they mean — `mv 3 7` in DataPool 1's Groups is `Move
//! DataPool 1 Group 3 At 7` — before rights, forwarding, the journal and Oops, so the object door
//! does the work (Swap, the asks). Under `Root.Drives` they work on this station's files through
//! the drive door: a taken name asks Overwrite / Keep both / Cancel, `rm` moves to a `.trash`
//! and says how to put it back; disk changes aren't Oops steps. Between the two they are refused.
//!
//! Owns:          write_out, the Mv / Cp / Rm fence words, the drive trash.
//! Doesn't own:   the object door's verbs (the kernel's `door`), the disk (lumacore-luma `drive`).
//! Entry points:  write_out, Mv, Cp, Rm.
//! Decisions:     §35 1, 19.

use std::path::{Path, PathBuf};

use lumacore_syntax::FenceLine;
use lumacore_wire::ButtonKind;

use lumacore_objects::commands::flags::FlagDef;
use lumacore_objects::commands::typed::FenceCommand;
use lumacore_objects::errors::render_code;
use lumacore_objects::{AppState, DispatchResult};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// `cp … --merge`: onto an object that exists, merge (where it can).
pub const MERGE: FlagDef = FlagDef::new("merge", "Onto an object that exists: merge into it");
/// `cp|mv … --overwrite`: replace what is there.
pub const OVERWRITE: FlagDef = FlagDef::new("overwrite", "Replace what is there");
/// `cp|mv … --keep`: a file that exists stays, the new one takes the next free name.
pub const KEEP: FlagDef = FlagDef::new(
    "keep",
    "A file that is there stays; the new one gets a free name",
);
/// `rm … -n`: no question.
pub const NO_ASK: FlagDef = FlagDef::new("n", "No question (as /NoConfirmation)");

pub struct Mv;
pub struct Cp;
pub struct Rm;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

impl FenceCommand for Mv {
    const WORD: &'static str = "mv";
    const PATHS: usize = 2;
    fn run(state: &mut AppState, line: &FenceLine) -> DispatchResult {
        run(state, "mv", line)
    }
}

impl FenceCommand for Cp {
    const WORD: &'static str = "cp";
    const PATHS: usize = 2;
    fn run(state: &mut AppState, line: &FenceLine) -> DispatchResult {
        run(state, "cp", line)
    }
}

impl FenceCommand for Rm {
    const WORD: &'static str = "rm";
    const PATHS: usize = 1;
    fn run(state: &mut AppState, line: &FenceLine) -> DispatchResult {
        run(state, "rm", line)
    }
}

//  ██████╗ ██████╗ ███╗   ███╗███╗   ███╗ █████╗ ███╗   ██╗██████╗ ███████╗
// ██╔════╝██╔═══██╗████╗ ████║████╗ ████║██╔══██╗████╗  ██║██╔══██╗██╔════╝
// ██║     ██║   ██║██╔████╔██║██╔████╔██║███████║██╔██╗ ██║██║  ██║███████╗
// ██║     ██║   ██║██║╚██╔╝██║██║╚██╔╝██║██╔══██║██║╚██╗██║██║  ██║╚════██║
// ╚██████╗╚██████╔╝██║ ╚═╝ ██║██║ ╚═╝ ██║██║  ██║██║ ╚████║██████╔╝███████║
//  ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝
// PART: COMMANDS

/// A line's `mv` / `cp` / `rm` on the show tree, written out as its Lum line (dispatch, before
/// anything reads the line); None for any other line and for paths under `Root.Drives`.
pub fn write_out(state: &AppState, line: &str) -> Option<Result<String, String>> {
    let word = line.split_whitespace().next()?.to_ascii_lowercase();
    if ![Mv::WORD, Cp::WORD, Rm::WORD].contains(&word.as_str()) {
        return None;
    }
    let fence = lumacore_objects::commands::typed::fence_line(line).ok()?;
    let src = absolute(state, fence.arg(0)?);
    if lumacore_objects::tree_path::names_drive(&src) {
        return None;
    }
    Some(lum_of(state, &word, &fence))
}

fn run(state: &mut AppState, word: &str, line: &FenceLine) -> DispatchResult {
    let Some(src) = line.arg(0).map(|p| absolute(state, p)) else {
        return out(Err(render_code(
            format!(
                "{word} what? {word} <path>{}",
                if word == "rm" { "" } else { " <to>" }
            ),
            "E-CMD-006",
        )));
    };
    if !lumacore_objects::tree_path::names_drive(&src) {
        // A stored line (a key, a macro) reaches here without the write-out: the same Lum line.
        return match lum_of(state, word, line) {
            Ok(lum) => lumacore_objects::commands::dispatch(state, &lum),
            Err(e) => out(Err(e)),
        };
    }
    out(match word {
        "rm" => remove_file(&src, line),
        _ => copy_or_move(state, word, &src, line),
    })
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The Lum line a show-tree `mv` / `cp` / `rm` means: the rows owner first, `At` the same kind.
fn lum_of(state: &AppState, word: &str, line: &FenceLine) -> Result<String, String> {
    let src = absolute(
        state,
        line.arg(0)
            .ok_or_else(|| render_code(format!("{word} what?"), "E-CMD-006"))?,
    );
    let (object, dp, rows) = rows_at(state, &src)?;
    let what = format!(
        "{}{} {}",
        owner(object, dp),
        object.keyword,
        lumacore_objects::commands::door::numbers_text(&rows)
    );
    if word == "rm" {
        return Ok(format!(
            "Delete {what}{}",
            if line.has("n") {
                " /NoConfirmation"
            } else {
                ""
            }
        ));
    }
    let to = line.arg(1).ok_or_else(|| {
        render_code(
            format!("{word} {} where to?", line.arg(0).unwrap_or("")),
            "E-CMD-006",
        )
    })?;
    let at = if to.bytes().all(|b| b.is_ascii_digit()) {
        to.to_string()
    } else {
        let dest = absolute(state, to);
        if lumacore_objects::tree_path::names_drive(&dest) {
            return Err(render_code(
                "Between the show and Root.Drives: that is Import / Export.",
                "E-CMD-020",
            ));
        }
        let (kind, dest_dp, n) = slot_at(state, &dest)?;
        if !std::ptr::eq(kind, object) {
            return Err(render_code(
                format!(
                    "A {} can't go into a {} — another kind.",
                    object.keyword, kind.keyword
                ),
                "E-CMD-020",
            ));
        }
        format!("{}{} {n}", owner(kind, dest_dp), kind.keyword)
    };
    let verb = if word == "mv" { "Move" } else { "Copy" };
    let flags = match (line.has("merge"), line.has("overwrite")) {
        (true, _) => " /Merge",
        (_, true) => " /Overwrite",
        _ => "",
    };
    Ok(format!("{verb} {what} At {at}{flags}"))
}

/// The rows a show-tree path names (an object or a collection's rows), on the door.
fn rows_at(
    state: &AppState,
    path: &str,
) -> Result<
    (
        &'static lumacore_objects::objects::ObjectDef,
        Option<u32>,
        Vec<u32>,
    ),
    String,
> {
    let target = lumacore_objects::resolve::target_of(path)?;
    let resolved = lumacore_objects::resolve::resolve_one(
        state,
        &target,
        lumacore_objects::objects::default_object(state),
    )?;
    if resolved.object.tree.as_ref().and_then(|t| t.door).is_none() {
        return Err(render_code(
            format!(
                "{path} can't be moved, copied or removed here (a {} isn't on the object door yet).",
                resolved.object.keyword
            ),
            "E-CMD-010",
        ));
    }
    if !resolved.named || resolved.members.is_empty() {
        return Err(render_code(
            format!(
                "{path}: name the {}s (a number, or a path to one).",
                resolved.object.keyword
            ),
            "E-CMD-006",
        ));
    }
    Ok((
        resolved.object,
        resolved.datapool,
        resolved.members.iter().map(|m| m.no).collect(),
    ))
}

/// Where a destination path points: an object (it may not exist yet: its collection and number).
fn slot_at(
    state: &AppState,
    path: &str,
) -> Result<
    (
        &'static lumacore_objects::objects::ObjectDef,
        Option<u32>,
        u32,
    ),
    String,
> {
    if let Ok(target) = lumacore_objects::resolve::target_of(path)
        && let Ok(r) = lumacore_objects::resolve::resolve_one(
            state,
            &target,
            lumacore_objects::objects::default_object(state),
        )
        && let [m] = r.members.as_slice()
        && r.named
    {
        return Ok((r.object, r.datapool, m.no));
    }
    let (parent, last) = path
        .rsplit_once('.')
        .ok_or_else(|| render_code(format!("{path}: where?"), "E-CMD-007"))?;
    let n: u32 = last
        .parse()
        .map_err(|_| render_code(format!("{path}: end it with a number."), "E-CMD-008"))?;
    let target = lumacore_objects::resolve::target_of(parent)?;
    let r = lumacore_objects::resolve::resolve_one(
        state,
        &target,
        lumacore_objects::objects::default_object(state),
    )?;
    Ok((r.object, r.datapool, n))
}

fn owner(object: &lumacore_objects::objects::ObjectDef, dp: Option<u32>) -> String {
    match (
        object.tree.as_ref().is_some_and(|t| t.datapool.is_some()),
        dp,
    ) {
        (true, Some(d)) => format!("DataPool {d} "),
        _ => String::new(),
    }
}

/// A path typed in the shell, absolute (`Root.…`).
fn absolute(state: &AppState, path: &str) -> String {
    lumacore_objects::vfs::dot_path_from_segments(&lumacore_objects::vfs::resolve_path(
        &state.navigation_path,
        path,
    ))
}

// ----- Drives -----

/// A reply, or an ask.
enum Done {
    Text(String),
    Ask(Box<lumacore_objects::dialogs::Ask>),
}

fn out(r: Result<Done, String>) -> DispatchResult {
    match r {
        Ok(Done::Text(t)) => DispatchResult::Output(lumacore_objects::Reply::ok(t)),
        Ok(Done::Ask(a)) => DispatchResult::Ask(a),
        Err(e) => DispatchResult::Output(e.into()),
    }
}

/// This line written out absolute, with its flags (the ask's buttons add one).
fn line_text(word: &str, paths: &[String], line: &FenceLine) -> String {
    let flags: Vec<String> = line
        .flags
        .iter()
        .map(|(f, _)| {
            if f.len() == 1 {
                format!("-{f}")
            } else {
                format!("--{f}")
            }
        })
        .collect();
    std::iter::once(word.to_string())
        .chain(paths.iter().cloned())
        .chain(flags)
        .collect::<Vec<_>>()
        .join(" ")
}

fn copy_or_move(
    state: &mut AppState,
    word: &str,
    src: &str,
    line: &FenceLine,
) -> Result<Done, String> {
    let from = lumacore_objects::vfs::resolve_drive_path(src)
        .ok_or_else(|| render_code(format!("{src}: no such file."), "E-CMD-007"))?;
    let to_arg = line
        .arg(1)
        .ok_or_else(|| render_code(format!("{word} {src} where to?"), "E-CMD-006"))?;
    let dest = absolute(state, to_arg);
    if !lumacore_objects::tree_path::names_drive(&dest) {
        return Err(render_code(
            "Between Root.Drives and the show: that is Import / Export.",
            "E-CMD-020",
        ));
    }
    if word == "cp" && from.is_dir() {
        return Err(render_code(
            format!("{src} is a folder — cp copies files."),
            "E-CMD-010",
        ));
    }
    let name = from
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut to = drive_dest(&dest, &name)?;
    if to == from {
        return Ok(Done::Text(format!("{} is already there.", from.display())));
    }
    if to.exists() {
        if line.has("keep") {
            to = free_name(&to);
        } else if !line.has("overwrite") {
            let base = line_text(word, &[src.to_string(), dest.clone()], line);
            return Ok(Done::Ask(Box::new(
                lumacore_objects::dialogs::Ask::new(
                    format!("{word} {name}"),
                    format!("{} is there already.", to.display()),
                )
                .button(
                    "Overwrite",
                    ButtonKind::Danger,
                    format!("{base} --overwrite"),
                )
                .button("Keep both", ButtonKind::Primary, format!("{base} --keep"))
                .cancel("Cancel")
                .flags("--overwrite or --keep"),
            )));
        }
    }
    let io = |e: std::io::Error| render_code(format!("{}: {e}", from.display()), "E-CMD-012");
    if word == "cp" {
        lumacore_luma::drive::copy(&from, &to).map_err(io)?;
        return Ok(Done::Text(format!(
            "{} copied to {}.",
            from.display(),
            to.display()
        )));
    }
    move_path(&from, &to).map_err(io)?;
    Ok(Done::Text(format!(
        "{} moved to {}.",
        from.display(),
        to.display()
    )))
}

fn remove_file(src: &str, line: &FenceLine) -> Result<Done, String> {
    let path = lumacore_objects::vfs::resolve_drive_path(src)
        .ok_or_else(|| render_code(format!("{src}: no such file."), "E-CMD-007"))?;
    if !line.has("n") {
        let base = line_text("rm", &[src.to_string()], line);
        return Ok(Done::Ask(Box::new(
            lumacore_objects::dialogs::Ask::new(
                "rm",
                format!(
                    "Move {} to the trash? It stays there 30 days.",
                    path.display()
                ),
            )
            .button("Move to trash", ButtonKind::Danger, format!("{base} -n"))
            .cancel("Cancel")
            .flags("-n"),
        )));
    }
    let trash = trash_dir(&path);
    lumacore_luma::drive::create_dir_all(&trash)
        .map_err(|e| render_code(format!("{}: {e}", trash.display()), "E-CMD-012"))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let to = free_name(&trash.join(format!("{stamp}_{name}")));
    move_path(&path, &to)
        .map_err(|e| render_code(format!("{}: {e}", path.display()), "E-CMD-012"))?;
    let back = format!(
        "mv {} {}",
        drive_dot(&to),
        path.parent().map(drive_dot).unwrap_or_default()
    );
    Ok(Done::Text(format!(
        "{} moved to the trash.\nPut back: {back}",
        path.display()
    )))
}

/// The file a destination path names: a folder that exists takes the source's name; else the
/// longest part that exists is the folder and the rest the new name (dots and all).
fn drive_dest(dest: &str, name: &str) -> Result<PathBuf, String> {
    if let Some(p) = lumacore_objects::vfs::resolve_drive_path(dest) {
        return Ok(if p.is_dir() { p.join(name) } else { p });
    }
    let parts: Vec<&str> = dest.split('.').collect();
    for k in (2..parts.len()).rev() {
        let folder = parts[..k].join(".");
        if let Some(dir) = lumacore_objects::vfs::resolve_drive_path(&folder).filter(|d| d.is_dir())
        {
            return Ok(dir.join(parts[k..].join(".")));
        }
    }
    Err(render_code(format!("{dest}: no such folder."), "E-CMD-007"))
}

/// `name 2.ext`, `name 3.ext` … — the first that isn't there.
fn free_name(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    (2..)
        .map(|n| dir.join(format!("{stem} {n}{ext}")))
        .find(|p| !p.exists())
        .expect("a free name")
}

/// Rename, or copy + remove across volumes (a file has no identity to lose).
fn move_path(from: &Path, to: &Path) -> std::io::Result<()> {
    match lumacore_luma::drive::rename(from, to) {
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices && from.is_file() => {
            lumacore_luma::drive::copy(from, to)?;
            lumacore_luma::drive::remove_file(from)
        }
        other => other,
    }
}

/// A drive's trash: a mounted volume's own `.trash`, else the library's.
fn trash_dir(path: &Path) -> PathBuf {
    let mut parts = path.components();
    #[cfg(unix)]
    {
        use std::path::Component;
        if let (Some(Component::RootDir), Some(Component::Normal(v)), Some(Component::Normal(name))) =
            (parts.next(), parts.next(), parts.next())
            && v == "Volumes"
        {
            return Path::new("/Volumes")
                .join(name)
                .join(lumacore_luma::backups::TRASH);
        }
    }
    #[cfg(windows)]
    {
        if let Some(std::path::Component::Prefix(p)) = parts.next() {
            return PathBuf::from(p.as_os_str())
                .join("\\")
                .join(lumacore_luma::backups::TRASH);
        }
    }
    lumacore_luma::backups::shows_dir()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
        .join(lumacore_luma::backups::TRASH)
}

/// A file's path in the tree (`Root.Drives.usb1.shows.a.luma`; decisions §38: the drive's id,
/// then the path on it — the local library is `Root.Drives.local.…`). A path on no drive keeps
/// its components after `Root.Drives`, as before the drives.
pub(crate) fn drive_dot(path: &Path) -> String {
    let parts = |p: &Path| -> Vec<String> {
        p.components()
            .filter_map(|c| match c {
                std::path::Component::Normal(n) => Some(n.to_string_lossy().to_string()),
                _ => None,
            })
            .collect()
    };
    let drives = lumacore_objects::tree_path::Node::Drives.path().to_string();
    let local = Path::new("library");
    let (id, rest): (String, Vec<String>) = if let Ok(rest) = path.strip_prefix(local) {
        (lumacore_objects::drives::LOCAL.to_string(), parts(rest))
    } else {
        lumacore_objects::drives::mounted_ids()
            .into_iter()
            .filter_map(|id| lumacore_objects::drives::mount_of(&id).map(|m| (id, m)))
            .filter_map(|(id, mount)| path.strip_prefix(&mount).ok().map(|rest| (id, parts(rest))))
            .max_by_key(|(_, rest)| usize::MAX - rest.len())
            .unwrap_or_else(|| (String::new(), parts(path)))
    };
    std::iter::once(drives).chain((!id.is_empty()).then_some(id)).chain(rest).collect::<Vec<_>>().join(".")
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
