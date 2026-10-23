//! lumacore-scripts: the developer launcher. A Plan pane (what to build and
//! run, as checkboxes, run as one `scripts/unix/dev` line), the local
//! station's status, and every script in `scripts/` as a menu.

mod plan;
mod status;

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

use plan::{Options, Plan, ROWS};
use status::{BINARIES, PORTS, Status, Web};

/// One menu line: a script in `scripts/unix/` (or `scripts/win/<name>.ps1`)
/// with fixed arguments, plus the profile / show when it takes them.
struct ScriptEntry {
    /// Section header rendered above this entry (starts a new group).
    section: Option<&'static str>,
    /// Base name without extension — the platform variant is resolved at runtime.
    base_name: &'static str,
    args: &'static [&'static str],
    label: &'static str,
    description: &'static str,
    /// Gets `--debug` when the debug profile is on.
    takes_profile: bool,
    /// Gets `--show <name>` when a show is set.
    takes_show: bool,
}

const fn entry(
    section: Option<&'static str>,
    base_name: &'static str,
    args: &'static [&'static str],
    label: &'static str,
    description: &'static str,
    takes_profile: bool,
    takes_show: bool,
) -> ScriptEntry {
    ScriptEntry {
        section,
        base_name,
        args,
        label,
        description,
        takes_profile,
        takes_show,
    }
}

#[rustfmt::skip]
const SCRIPT_ENTRIES: &[ScriptEntry] = &[
    entry(Some("Build"), "build", &["all"], "build all", "Server, CLI and app — nothing is started.", true, false),
    entry(None, "build", &["server"], "build server", "lumacore-server (perfetto,http).", true, false),
    entry(None, "build", &["app"], "build app", "lumacore-app alone (connects to any station).", true, false),
    entry(None, "build", &["bench", "local"], "build bench", "Wipe target/local, time a fresh build of every crate; a row in docs/dev/build-times.md.", false, false),
    entry(Some("Server"), "server", &["start"], "server start", "Build, restart in the background, wait for health.", true, true),
    entry(None, "server", &["stop"], "server stop", "Stop it cleanly (autosave), kill after 8 s.", false, false),
    entry(None, "server", &["status"], "server status", "PID, binary, uptime, health, ports.", true, false),
    entry(None, "connect", &[], "connect", "Attach the CLI to the running server.", true, false),
    entry(None, "start_server", &[], "start_server", "Preset: build + start server, CLI here.", true, true),
    entry(Some("App"), "app", &[], "app", "Build and start the desktop app (needs a display).", true, false),
    entry(Some("Test"), "test", &["quick"], "test quick", "cargo test, every crate.", false, false),
    entry(None, "test", &["e2e"], "test e2e", "UI end-to-end (Playwright).", false, false),
    entry(None, "test", &["smoke"], "test smoke", "Release server start / health / shutdown.", false, false),
    entry(None, "test", &["all"], "test all", "quick + e2e + smoke.", false, false),
    entry(None, "test", &["regen"], "test regen", "gen-docs + gen-demo after a registry or show change.", false, false),
    entry(None, "test", &["screenshots"], "screenshots", "Every window into target/screenshots/.", false, false),
    entry(Some("Fuzz (long run — never per step)"), "test", &["fuzz", "5"], "fuzz 5 min", "Short sweep + fuzz run; continues an unfinished run.", false, false),
    entry(None, "test", &["fuzz", "60"], "fuzz overnight", "60 min random + 60 min per fuzz target. Ctrl-C stops.", false, false),
    entry(None, "test", &["fuzz", "30", "--fresh"], "fuzz fresh", "Start a new 30 min run (drops the unfinished one).", false, false),
    entry(None, "test", &["fuzz-report"], "fuzz report", "target/fuzz-report/summary.md.", false, false),
    entry(Some("Logs (watch a run live — Ctrl-C stops watching)"), "logs", &[], "logs", "Follow the newest script run; moves on to the next run.", false, false),
    entry(None, "logs", &["fuzz"], "logs fuzz", "Follow the newest fuzz run (progress every 5 s).", false, false),
    entry(None, "logs", &["server"], "logs server", "Follow the local server's output.", false, false),
    entry(None, "logs", &["list"], "logs list", "The recent runs, newest first.", false, false),
    entry(Some("Developer"), "start_developer", &[], "start_developer", "Preset: server (+Tracy), CLI, Meownitor, Tracy windows.", true, true),
    entry(None, "stop_developer", &[], "stop_developer", "Stop the server, CLI, Meownitor and Tracy.", false, false),
    entry(Some("Setup"), "dev_init", &[], "dev_init", "Check tools, validate .env.local, build this launcher.", false, false),
];

fn script_path(repo_root: &Path, entry: &ScriptEntry) -> PathBuf {
    let scripts_dir = repo_root.join("scripts");
    if cfg!(windows) {
        return scripts_dir
            .join("win")
            .join(format!("{}.ps1", entry.base_name));
    }
    // Try the bare name first, then .sh
    let unix_dir = scripts_dir.join("unix");
    let direct = unix_dir.join(entry.base_name);
    if direct.exists() {
        direct
    } else {
        unix_dir.join(format!("{}.sh", entry.base_name))
    }
}

/// The arguments for an entry: its own, then the profile and show (Unix
/// flags, or the PowerShell parameters on Windows).
fn script_args(entry: &ScriptEntry, opts: &Options) -> Vec<String> {
    let mut args: Vec<String> = entry.args.iter().map(|a| a.to_string()).collect();
    if entry.takes_profile && opts.debug {
        args.push(
            if cfg!(windows) {
                "-DebugBuild"
            } else {
                "--debug"
            }
            .into(),
        );
    }
    if entry.takes_show && !opts.show.is_empty() {
        args.push(if cfg!(windows) { "-Show" } else { "--show" }.into());
        args.push(opts.show.clone());
    }
    args
}

fn build_command(repo_root: &Path, entry: &ScriptEntry, opts: &Options) -> Command {
    let path = script_path(repo_root, entry);
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        c.arg(&path);
        c
    } else {
        Command::new(&path)
    };
    cmd.args(script_args(entry, opts));
    cmd.current_dir(repo_root);
    cmd
}

/// The command line shown in the footer.
fn display_command(entry: &ScriptEntry, opts: &Options) -> String {
    let script = if cfg!(windows) {
        format!(".\\scripts\\win\\{}.ps1", entry.base_name)
    } else {
        format!("./scripts/unix/{}", entry.base_name)
    };
    std::iter::once(script)
        .chain(script_args(entry, opts))
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Plan,
    Scripts,
}

/// A text option being typed.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Editing {
    Show,
    User,
}

struct App {
    focus: Focus,
    /// The highlighted script.
    selected: usize,
    /// The Plan pane's cursor: row in `ROWS`, column 0 = Build, 1 = Run.
    row: usize,
    col: usize,
    plan: Plan,
    opts: Options,
    editing: Option<Editing>,
    /// One line under the panes (an empty plan, what just ran).
    message: Option<String>,
}

impl App {
    fn next(&mut self) {
        match self.focus {
            Focus::Scripts => self.selected = (self.selected + 1) % SCRIPT_ENTRIES.len(),
            Focus::Plan => {
                self.row = (self.row + 1) % ROWS.len();
                self.snap_col();
            }
        }
    }

    fn previous(&mut self) {
        match self.focus {
            Focus::Scripts => {
                self.selected = self
                    .selected
                    .checked_sub(1)
                    .unwrap_or(SCRIPT_ENTRIES.len() - 1)
            }
            Focus::Plan => {
                self.row = self.row.checked_sub(1).unwrap_or(ROWS.len() - 1);
                self.snap_col();
            }
        }
    }

    /// Rows without a Build box keep the cursor on Run.
    fn snap_col(&mut self) {
        if ROWS[self.row].build.is_none() {
            self.col = 1;
        }
    }

    fn cell(&self) -> Option<plan::Tick> {
        let row = &ROWS[self.row];
        if self.col == 0 { row.build } else { row.run }
    }

    fn selected_entry(&self) -> &'static ScriptEntry {
        &SCRIPT_ENTRIES[self.selected]
    }
}

fn repo_root() -> io::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .ok_or_else(|| io::Error::other("failed to resolve repo root"))
}

/// Run a script with the terminal handed over; its exit code.
fn run(mut cmd: Command, shown: &str, path: &Path) -> io::Result<i32> {
    if !path.exists() {
        eprintln!("{} doesn't exist on this platform.", path.display());
        return Ok(1);
    }
    println!("$ {shown}\n");
    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.code().unwrap_or(1))
}

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

fn dot(on: bool) -> Span<'static> {
    if on {
        Span::styled("●", Style::default().fg(Color::Green).bold())
    } else {
        Span::styled("○", dim())
    }
}

fn label(text: &'static str) -> Span<'static> {
    Span::styled(format!("{text:<9}"), Style::default().bold())
}

fn warn(text: String) -> Span<'static> {
    Span::styled(text, Style::default().fg(Color::Yellow).bold())
}

fn profile_of(exe: &str) -> &'static str {
    let sep = std::path::MAIN_SEPARATOR;
    if exe.contains(&format!("{sep}debug{sep}")) {
        "debug"
    } else if exe.contains(&format!("{sep}release{sep}")) {
        "release"
    } else if exe.contains(&format!("{sep}local{sep}")) {
        "local"
    } else {
        ""
    }
}

/// The Status pane: server, HTTP, web UI, ports, clients, binaries.
fn status_lines(s: &Status, debug: bool) -> Vec<Line<'static>> {
    if !s.checked {
        return vec![Line::from(Span::styled("checking…", dim()))];
    }
    let mut lines = Vec::new();

    // Server: processes + the UDP health ask.
    let mut server = vec![label("Server")];
    if s.servers.is_empty() {
        server.push(Span::styled("○ not running", dim().bold()));
    } else {
        let pids = s
            .servers
            .iter()
            .map(|(pid, exe)| format!("PID {pid} {}", profile_of(exe)))
            .collect::<Vec<_>>()
            .join(", ");
        if s.healthy {
            server.push(Span::styled(
                "● running",
                Style::default().fg(Color::Green).bold(),
            ));
            server.push(Span::raw(format!("  {pids} · healthy")));
        } else {
            server.push(warn("● running".into()));
            server.push(Span::raw(format!(
                "  {pids} · not answering (starting, stopping or stuck)"
            )));
        }
        if s.servers.len() > 1 {
            server.push(warn("  · more than one!".into()));
        }
    }
    lines.push(Line::from(server));

    // HTTP: /api/station.
    let mut http = vec![label("HTTP")];
    match &s.station {
        None if s.servers.is_empty() => http.push(Span::styled("○", dim())),
        None => http.push(warn(format!("○ no answer on :{}", status::HTTP_PORT))),
        Some(st) => {
            http.push(dot(true));
            let up = st.uptime().map(status::age).unwrap_or_default();
            let show = st.show.clone().unwrap_or_else(|| "no show".into());
            http.push(Span::raw(format!(
                " {} · {} · up {up} · {show}",
                st.name, st.role
            )));
            if st.unsaved {
                http.push(warn(" (unsaved)".into()));
            }
            if st.busy {
                http.push(warn(" · BUSY".into()));
            }
            if !st.logged_in.is_empty() {
                http.push(Span::raw(format!(
                    " · logged in: {}",
                    st.logged_in.join(", ")
                )));
            }
        }
    }
    lines.push(Line::from(http));

    // Build: the running build vs HEAD and the binary on disk.
    if let Some(st) = &s.station {
        let commit = st.commit().to_string();
        let mut build = vec![label("Build"), Span::raw(st.build.clone())];
        match &s.head {
            Some(head) if commit.trim_end_matches("-dirty") == head => {
                if commit.ends_with("-dirty") {
                    build.push(Span::styled("  = HEAD + local changes", dim()));
                } else {
                    build.push(Span::styled("  = HEAD", Style::default().fg(Color::Green)));
                }
            }
            Some(head) => build.push(warn(format!("  ≠ HEAD {head}"))),
            None => {}
        }
        if st.rebuilt() {
            build.push(warn("  · binary rebuilt since start (Run server)".into()));
        }
        lines.push(Line::from(build));
    }

    // Web UI: what / serves.
    let url = format!("http://127.0.0.1:{}/", status::HTTP_PORT);
    let web = match s.web {
        Web::Unknown => vec![label("Web UI"), Span::styled("○", dim())],
        Web::Served => vec![label("Web UI"), dot(true), Span::raw(format!(" {url}"))],
        Web::Off => vec![
            label("Web UI"),
            warn("▲ off".into()),
            Span::raw(" — an older server; restart it: Run server"),
        ],
    };
    lines.push(Line::from(web));

    // Ports.
    let mut ports = vec![label("Ports")];
    ports.push(dot(s.healthy));
    ports.push(Span::raw(" cmd 47653  "));
    for (i, (name, port)) in PORTS.iter().enumerate() {
        ports.push(dot(s.ports[i]));
        ports.push(Span::raw(format!(" {name} {port}  ")));
    }
    lines.push(Line::from(ports));

    // Clients.
    let mut clients = vec![label("Clients")];
    clients.push(dot(!s.apps.is_empty()));
    if s.apps.is_empty() {
        clients.push(Span::raw(" app  "));
    } else {
        let pids = s.apps.iter().map(|p| p.to_string()).collect::<Vec<_>>();
        clients.push(Span::raw(format!(" app PID {}  ", pids.join(", "))));
    }
    clients.push(dot(s.clis > s.meownitors));
    clients.push(Span::raw(format!(" cli {}  ", s.clis - s.meownitors)));
    clients.push(dot(s.meownitors > 0));
    clients.push(Span::raw(format!(" meownitor {}", s.meownitors)));
    lines.push(Line::from(clients));

    // Binaries of the chosen profile.
    let (p, profile) = if debug { (1, "debug") } else { (0, "local") };
    let mut bins = vec![
        label("Binaries"),
        Span::styled(format!("{profile}: "), dim()),
    ];
    let now = SystemTime::now();
    for (b, name) in BINARIES.iter().enumerate() {
        let short = name.trim_start_matches("lumacore-");
        match s.binaries[p][b] {
            Some(t) => {
                let ago = now.duration_since(t).map(status::age).unwrap_or_default();
                bins.push(Span::raw(format!("{short} {ago} ago  ")));
            }
            None => bins.push(Span::styled(format!("{short} not built  "), dim())),
        }
    }
    lines.push(Line::from(bins));
    lines
}

fn plan_lines(app: &App) -> Vec<Line<'static>> {
    let focused = app.focus == Focus::Plan;
    let mut lines = vec![Line::from(Span::styled(
        format!("  {:<12} {:<9} {:<12} ", "", "Build", "Run"),
        dim(),
    ))];
    for (r, row) in ROWS.iter().enumerate() {
        let cell = |c: usize, tick: Option<plan::Tick>| -> Span<'static> {
            let Some(t) = tick else {
                return Span::raw(" ".repeat(if c == 0 { 9 } else { 12 }));
            };
            let on = app.plan.on(t);
            let mut text = if on { "[x]" } else { "[ ]" }.to_string();
            if t == plan::Tick::RunCli && on {
                text.push_str(if app.plan.cli_window {
                    " window"
                } else {
                    " here"
                });
            }
            if t == plan::Tick::TracyFeature {
                text.push_str(" feat");
            }
            let width = if c == 0 { 9 } else { 12 };
            let text = format!("{text:<width$}");
            let style = if focused && app.row == r && app.col == c {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if on {
                Style::default().fg(Color::Green).bold()
            } else {
                Style::default()
            };
            Span::styled(text, style)
        };
        let marker = if focused && app.row == r { "> " } else { "  " };
        lines.push(Line::from(vec![
            Span::raw(marker),
            Span::raw(format!("{:<12} ", row.label)),
            cell(0, row.build),
            cell(1, row.run),
            Span::raw(" "),
            Span::styled(row.note, dim()),
        ]));
    }
    for w in app.plan.warnings() {
        lines.push(Line::from(warn(format!("  ▲ {w}"))));
    }
    lines
}

fn draw_ui(frame: &mut Frame, app: &App, status: &Status) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .title("LumaCore Scripts")
            .borders(Borders::ALL),
        area,
    );
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    let plan_height = (ROWS.len() + 1 + app.plan.warnings().len() + 2) as u16;
    let status_lines = status_lines(status, app.opts.debug);
    let status_height = status_lines.len() as u16 + 2;
    // Plan and Status side by side when there is room.
    let wide = inner.width >= 150;
    let top_height = if wide {
        plan_height.max(status_height)
    } else {
        plan_height + status_height
    };
    let layout = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(top_height),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(inner);

    // Help + options.
    let profile = if app.opts.debug { "debug" } else { "local" };
    let field = |which: Editing, value: &str, empty: &str| match (
        app.editing == Some(which),
        value.is_empty(),
    ) {
        (true, _) => format!("{value}▏"),
        (false, true) => empty.to_string(),
        (false, false) => value.to_string(),
    };
    let cyan = Style::default().fg(Color::Cyan).bold();
    let keys = match app.focus {
        Focus::Plan => {
            "Tab → scripts · ↑↓←→ move · Space ticks · Enter or r runs the plan · n clears · q exits"
        }
        Focus::Scripts => {
            "Tab → plan · ↑↓ move · Enter runs the script · r runs the plan · q exits"
        }
    };
    let help = Paragraph::new(vec![
        Line::from(keys),
        Line::from(vec![
            Span::raw("d Profile: "),
            Span::styled(profile, cyan),
            Span::raw("   s Show: "),
            Span::styled(field(Editing::Show, &app.opts.show, "(none)"), cyan),
            Span::raw("   u User: "),
            Span::styled(field(Editing::User, &app.opts.user, "Admin"), cyan),
            Span::raw("   c Clean start: "),
            Span::styled(if app.opts.clean_start { "on" } else { "off" }, cyan),
            Span::raw(if app.editing.is_some() {
                "   (type, Enter keeps)"
            } else {
                ""
            }),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(help, layout[0]);

    let (plan_area, status_area) = if wide {
        let cols =
            Layout::horizontal([Constraint::Length(88), Constraint::Min(40)]).split(layout[1]);
        (cols[0], cols[1])
    } else {
        let rows = Layout::vertical([Constraint::Length(plan_height), Constraint::Min(0)])
            .split(layout[1]);
        (rows[0], rows[1])
    };
    let border = |focused: bool| {
        if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        }
    };
    frame.render_widget(
        Paragraph::new(plan_lines(app)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border(app.focus == Focus::Plan))
                .title("Plan (build, then run — in this order)"),
        ),
        plan_area,
    );
    frame.render_widget(
        Paragraph::new(status_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status (every 2 s)"),
        ),
        status_area,
    );

    // Section headers are their own rows; track which row each entry is on.
    let mut items: Vec<ListItem> = Vec::new();
    let mut entry_to_visual: Vec<usize> = Vec::new();
    for entry in SCRIPT_ENTRIES {
        if let Some(section) = entry.section {
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }
            items.push(ListItem::new(Span::styled(
                section,
                Style::default().fg(Color::Yellow).bold(),
            )));
        }
        entry_to_visual.push(items.len());
        items.push(ListItem::new(format!(
            "  {:<18} {}",
            entry.label, entry.description
        )));
    }
    let scripts_focused = app.focus == Focus::Scripts;
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border(scripts_focused))
                .title("Scripts"),
        )
        .highlight_symbol(if scripts_focused { "> " } else { "  " })
        .highlight_style(if scripts_focused {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        });
    let mut state = ListState::default();
    state.select(Some(entry_to_visual[app.selected]));
    frame.render_stateful_widget(list, layout[2], &mut state);

    let command = match (&app.message, app.focus) {
        (Some(m), _) => Line::from(warn(m.clone())),
        (None, Focus::Plan) if app.plan.is_empty() => {
            Line::from(Span::styled("Tick what to build and run (Space)", dim()))
        }
        (None, Focus::Plan) => Line::from(plan::display(&app.plan, &app.opts)),
        (None, Focus::Scripts) => Line::from(display_command(app.selected_entry(), &app.opts)),
    };
    frame.render_widget(
        Paragraph::new(command).block(Block::default().borders(Borders::ALL).title("Command")),
        layout[3],
    );
}

/// What the loop leaves the menu for.
enum Action {
    Quit,
    Script,
    Plan,
}

fn main() -> io::Result<()> {
    // Ctrl-C while a script runs (server log, a fuzz run) stops the script,
    // not this menu: a handler of our own survives SIGINT, and the child's
    // exec resets it to the default.
    #[cfg(unix)]
    signal_hook::flag::register(
        signal_hook::consts::SIGINT,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )?;

    let root = repo_root()?;
    let status = Arc::new(Mutex::new(Status::default()));
    status::watch(root.clone(), Arc::clone(&status));

    let (plan, opts) = plan::load(&root);
    let mut app = App {
        focus: Focus::Plan,
        selected: 0,
        row: 1,
        col: 0,
        plan,
        opts,
        editing: None,
        message: None,
    };
    loop {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let action = loop {
            terminal.draw(|frame| {
                let status = status.lock().unwrap_or_else(|e| e.into_inner());
                draw_ui(frame, &app, &status)
            })?;

            // Wake now and then so the status stays current.
            if !event::poll(Duration::from_millis(500))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            app.message = None;
            if let Some(which) = app.editing {
                let text = match which {
                    Editing::Show => &mut app.opts.show,
                    Editing::User => &mut app.opts.user,
                };
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => app.editing = None,
                    KeyCode::Backspace => {
                        text.pop();
                    }
                    KeyCode::Char(c) => text.push(c),
                    _ => {}
                }
                continue;
            }
            match key.code {
                KeyCode::Tab | KeyCode::BackTab => {
                    app.focus = match app.focus {
                        Focus::Plan => Focus::Scripts,
                        Focus::Scripts => Focus::Plan,
                    }
                }
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                KeyCode::Left if app.focus == Focus::Plan && ROWS[app.row].build.is_some() => {
                    app.col = 0
                }
                KeyCode::Right if app.focus == Focus::Plan => app.col = 1,
                KeyCode::Char(' ') if app.focus == Focus::Plan => {
                    if let Some(t) = app.cell() {
                        app.plan.toggle(t);
                    }
                }
                KeyCode::Char('n') if app.focus == Focus::Plan => app.plan.clear(),
                KeyCode::Char('d') => app.opts.debug = !app.opts.debug,
                KeyCode::Char('s') => app.editing = Some(Editing::Show),
                KeyCode::Char('u') => app.editing = Some(Editing::User),
                KeyCode::Char('c') => app.opts.clean_start = !app.opts.clean_start,
                KeyCode::Char('r') | KeyCode::F(5) => break Action::Plan,
                KeyCode::Enter if app.focus == Focus::Plan => break Action::Plan,
                KeyCode::Enter => break Action::Script,
                KeyCode::Esc | KeyCode::Char('q') => break Action::Quit,
                _ => {}
            }
        };
        plan::save(&root, &app.plan, &app.opts);

        if matches!(action, Action::Plan) && app.plan.is_empty() {
            app.message = Some("Nothing ticked — Space ticks a box in the Plan".into());
            continue;
        }

        // Restore the terminal BEFORE running any script, so the child process
        // inherits a clean terminal state.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        let code = match action {
            Action::Quit => return Ok(()),
            Action::Script => {
                let entry = app.selected_entry();
                run(
                    build_command(&root, entry, &app.opts),
                    &display_command(entry, &app.opts),
                    &script_path(&root, entry),
                )?
            }
            Action::Plan => run(
                plan::command(&root, &app.plan, &app.opts),
                &plan::display(&app.plan, &app.opts),
                &plan::script(&root),
            )?,
        };
        // Keep the output readable: the menu would clear it at once.
        let outcome = if code == 0 {
            "Done".to_string()
        } else {
            format!("Script exited with code {code}")
        };
        eprintln!("\n{outcome}. Press Enter to return to the menu...");
        let _ = io::stdin().read_line(&mut String::new());
    }
}
