//  █████╗ ██████╗ ██████╗
// ██╔══██╗██╔══██╗██╔══██╗
// ███████║██████╔╝██████╔╝
// ██╔══██║██╔═══╝ ██╔═══╝
// ██║  ██║██║     ██║
// ╚═╝  ╚═╝╚═╝     ╚═╝
// FILE: app

//!
//! Owns:          CommonCli (the options every binary shares), logging, building the AppState from the station's files and the startup show, the .starting marker.
//! Doesn't own:   the catalog (the assembler installs it), the server's start (builder).
//! Entry points:  CommonCli, init_logging, init_server_logging, build_state, resolve_show_path.
//! Decisions:     §20, §30 "12d part 5", R1-17, §34 24.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use clap::Parser;

use lumacore_wire::{REMOTE_CMD_PORT, REMOTE_HTTP_PORT, REMOTE_SHELL_PORT};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

#[derive(Parser, Debug, Clone)]
pub struct CommonCli {
    /// Optional path to a GDTF file to import on startup.
    pub gdtf_file: Option<String>,

    /// Log verbosity level.
    #[arg(long, default_value = "warn", value_parser = ["error", "warn", "info", "debug", "trace"])]
    pub log_level: String,

    /// Enable debug mode (unlocks dangerous commands).
    #[arg(long)]
    pub debug: bool,

    /// Load a .luma show file on startup.
    #[arg(long)]
    pub show: Option<String>,

    /// Login as a specific user profile.
    #[arg(long)]
    pub user: Option<String>,

    /// UDP command listener port for lumacore-server.
    #[arg(long, default_value_t = REMOTE_CMD_PORT)]
    pub cmd_port: u16,

    /// TCP shell listener port for lumacore-server.
    #[arg(long, default_value_t = REMOTE_SHELL_PORT)]
    pub shell_port: u16,

    /// UDP multicast discovery listener port for session/network discovery.
    #[arg(long, default_value_t = lumacore_wire::SESSION_DISCOVERY_PORT)]
    pub discovery_port: u16,

    /// HTTP engine port (REST, WebSocket, static files).
    #[arg(long, default_value_t = REMOTE_HTTP_PORT)]
    pub http_port: u16,

    /// TCP LumaSession stream port (members connect to their master here).
    #[arg(long, default_value_t = lumacore_wire::REMOTE_STREAM_PORT)]
    pub stream_port: u16,

    /// Run as station <name>: its own settings, sACN CID and tokens in
    /// library/stations/<name>/ (shows and GDTF stay shared). For several
    /// servers on one machine.
    #[arg(long)]
    pub station: Option<String>,

    /// Base for this server's ports: command N+3, shell N+4, HTTP N+6,
    /// stream N+8 (discovery stays shared). Overrides those port flags.
    #[arg(long)]
    pub port_base: Option<u16>,

    /// LumaSession loss timeout in seconds (default 5). Debug builds only,
    /// a hidden disable-timeouts switch.
    #[cfg(debug_assertions)]
    #[arg(long)]
    pub session_timeout: Option<u64>,

    /// Disable the HTTP engine entirely.
    #[arg(long)]
    pub no_http: bool,

    /// Serve the WebUI from this folder instead of the files built into the binary (live edits;
    /// the scripts pass `crates/ui/lumacore-webui/web`, decisions §36 2).
    #[arg(long)]
    pub web_root: Option<String>,

    /// CleanStart: start with an empty show, startup show None and
    /// default station settings (the show files stay).
    #[arg(long)]
    pub clean_start: bool,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub fn init_logging(level: &str) {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level))
        .try_init();
}

pub fn init_server_logging(level: &str) {
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level));

    if let Some(path) = open_server_log_path()
        && let Ok(file) = OpenOptions::new().create(true).append(true).open(&path)
    {
        builder.target(env_logger::Target::Pipe(Box::new(MutexWriter(Mutex::new(
            file,
        )))));
    }

    let _ = builder.try_init();
}

pub fn resolve_show_path(show_arg: &str) -> String {
    if Path::new(show_arg).exists() {
        show_arg.to_string()
    } else {
        let name = show_arg.trim_end_matches(".luma");
        format!("library/shows/{name}.luma")
    }
}

/// The demo show is built from its sources when it is missing or stale (decisions §40 15: the
/// one door, `lumacore_objects::demo::ensure`; a build that fails is logged and the load says
/// why).
fn ensure_demo(path: &str) {
    match lumacore_objects::demo::ensure_if_demo(Path::new(path)) {
        Some(note) if note.starts_with("[ERROR]") => log::error!("{note}"),
        Some(note) => log::info!("{note}"),
        None => {}
    }
}

pub fn build_state(common: &CommonCli) -> Result<(lumacore_objects::AppState, Option<String>), String> {
    crate::require_catalog()?;
    let mut state = lumacore_objects::AppState::new();
    state.debug_mode = common.debug;

    state.local_settings = lumacore_luma::station_config::LocalSettings::load();
    state
        .local_settings
        .apply_to(&mut state.root.station_settings);

    let resolved_show = if common.clean_start {
        None
    } else if let Some(show_arg) = common.show.as_deref() {
        let path = resolve_show_path(show_arg);
        ensure_demo(&path);
        load_startup_show(&mut state, &path, |state| lumacore_objects::objects::load_show(state, &path))
            .map_err(|e| format!("Error loading show file '{path}': {e}"))?;
        // An older show opened upgraded (decisions 0289): said now, and unsaved until a save.
        if let Some(u) = state.upgraded.take() {
            state.show_dirty = true;
            lumacore_sysmon::sysmon_early("Show", format!("{path}: {} — unsaved until SaveShow", u.line()));
            for note in &u.notes {
                lumacore_sysmon::sysmon_early("Show", format!("  {note}"));
            }
        }
        state.current_show = Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned());
        Some(path)
    } else if let Some(name) = state.local_settings.startup_show_name() {
        // Startup show: by default the
        // last show, so a restart or a rebuild comes back where it was.
        let loaded = guarded_by_marker(&lumacore_luma::station_config::dir(), &name, || {
            load_startup_show(&mut state, &name, |state| lumacore_cmd_showfile::load_show(state, &name))
        });
        match loaded.unwrap_or_else(|skipped| {
            // Skipped first; the recovery ask (`lumacore_objects::recovery::startup`) then
            // offers a newer copy of this show (merge of bugfixes, §30).
            state.skipped_startup_show = Some(name.clone());
            Err(skipped)
        }) {
            Ok(msg) => {
                log::info!("startup show: {msg}");
                if msg.contains("upgraded") {
                    lumacore_sysmon::sysmon_early("Show", msg.clone());
                }
                Some(
                    lumacore_cmd_showfile::show_path(&name)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
            Err(e) => {
                log::warn!("startup show \"{name}\" not loaded: {e}");
                None
            }
        }
    } else {
        None
    };

    lumacore_cmd_showfile::ensure_admin_user(&mut state);
    // A split base noted before a restart, for the show loaded now (R1-22).
    lumacore_objects::session_link::restore_split_base(&mut state);
    if common.clean_start {
        lumacore_cmd_showfile::clean_start(&mut state);
        log::warn!("CleanStart: empty show, startup show None, station settings reset");
    }

    if let Some(user) = common.user.as_deref() {
        login_or_create_user(&mut state, user);
    }

    if let Some(path) = common.gdtf_file.as_deref() {
        let args = vec![path];
        let result = lumacore_objects::commands::import::handle_import(&mut state, &args);
        log::info!("{result}");
    }

    Ok((state, resolved_show))
}

/// The `.starting` marker (R1-17, Q38): written before the startup show
/// loads and removed after, whatever the result. Still there at the next
/// start, the last start died hard while loading (an abort, out of memory,
/// a kill) — the startup show is skipped this time and the System Monitor
/// says why, so a show that kills the process can't stop every start.
/// `Err` = skipped (E-SHOW-006), `Ok(result)` = the load ran.
fn guarded_by_marker<T>(dir: &str, name: &str, load: impl FnOnce() -> Result<T, String>) -> Result<Result<T, String>, String> {
    let marker = format!("{dir}/.starting");
    if let Ok(was) = lumacore_luma::drive::read_to_string(&marker) {
        let _ = lumacore_luma::drive::remove_file(&marker);
        let line = lumacore_objects::errors::with_code(
            format!(
                "the last start died while loading the startup show \"{}\": not loaded this time (LoadShow it by hand)",
                was.trim()
            ),
            "E-SHOW-006",
        );
        lumacore_sysmon::sysmon_early("Show", format!("[ERROR] {line}"));
        return Err(line);
    }
    let _ = lumacore_luma::drive::create_dir_all(dir);
    let _ = lumacore_luma::drive::write(&marker, name);
    let result = load();
    let _ = lumacore_luma::drive::remove_file(&marker);
    Ok(result)
}

/// Load the startup show (`--show` or the station's startup show) inside the
/// panic guard (R1-17): a show that panics while it loads must not stop
/// every start. On a panic the partly loaded state is thrown away (a fresh
/// state with this station's settings), the System Monitor gets an ERROR
/// with the report number, and the caller goes on with no show.
fn load_startup_show<T>(
    state: &mut lumacore_objects::AppState,
    name: &str,
    load: impl FnOnce(&mut lumacore_objects::AppState) -> Result<T, String>,
) -> Result<T, String> {
    match lumacore_sysmon::panic_report::guarded(|| load(state)) {
        Ok(result) => result,
        Err(report) => {
            let local = std::mem::take(&mut state.local_settings);
            let debug = state.debug_mode;
            *state = lumacore_objects::AppState::new();
            state.debug_mode = debug;
            local.apply_to(&mut state.root.station_settings);
            state.local_settings = local;
            let line = lumacore_objects::errors::with_code(
                format!(
                    "startup show \"{name}\" panicked on load ({}:{} — {}) · report #{}",
                    report.file, report.line, report.message, report.number
                ),
                "E-INTERNAL-001",
            );
            lumacore_sysmon::sysmon_early("Show", format!("[ERROR] {line}"));
            Err(line)
        }
    }
}

fn login_or_create_user(state: &mut lumacore_objects::AppState, user: &str) {
    if user
        .trim()
        .eq_ignore_ascii_case(lumacore_objects::INTERNAL_ROOT_USER)
    {
        log::warn!(
            "'{}' is reserved for the internal runtime user.",
            lumacore_objects::INTERNAL_ROOT_USER
        );
        return;
    }
    if let Some(u) = state.root.show_data.user(user.trim()) {
        state.user = u.login.clone();
        return;
    }
    // A new user (a Programmer) with their own profile, a copy of Default
    // (decisions §30 "12c2": rights live on the user).
    let display_name = {
        let mut chars = user.trim().chars();
        match chars.next() {
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            None => return,
        }
    };
    let Ok(profile) = lumacore_objects::commands::users::new_profile(state, &display_name, None) else {
        return;
    };
    let show = &mut state.root.show_data;
    let no = show.users.keys().next_back().map_or(1, |n| n + 1);
    let guid = show.user_profiles[&profile].meta.identity.guid;
    show.users.insert(
        no,
        lumacore_tree::User::new(no, display_name.clone(), lumacore_tree::UserRights::Program, guid),
    );
    state.user = display_name.clone();
    log::info!("Auto-created user '{display_name}' (Programmer, profile {profile})");
}

fn open_server_log_path() -> Option<PathBuf> {
    let dir = PathBuf::from("library/logs/server");
    fs::create_dir_all(&dir).ok()?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(dir.join(format!("server_{ts}.log")))
}

struct MutexWriter<T>(Mutex<T>);

impl<T: std::io::Write> std::io::Write for MutexWriter<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log file lock poisoned"))?
            .write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log file lock poisoned"))?
            .flush()
    }
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
