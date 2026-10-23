//! The Plan pane: what to build and what to run, as checkboxes. It runs as
//! one `scripts/unix/dev` (or `scripts\win\dev.ps1`) command line, so the
//! order, the checks and the logs live in the script, not here.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

/// One checkbox.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tick {
    Stop,
    BuildServer,
    BuildCli,
    BuildApp,
    TracyFeature,
    RunServer,
    RunCli,
    RunApp,
    RunTracy,
    RunMeownitor,
    RunWeb,
}

impl Tick {
    const ALL: [Tick; 11] = [
        Tick::Stop,
        Tick::BuildServer,
        Tick::BuildCli,
        Tick::BuildApp,
        Tick::TracyFeature,
        Tick::RunServer,
        Tick::RunCli,
        Tick::RunApp,
        Tick::RunTracy,
        Tick::RunMeownitor,
        Tick::RunWeb,
    ];

    /// The name in the saved state.
    fn key(self) -> &'static str {
        match self {
            Tick::Stop => "stop",
            Tick::BuildServer => "build:server",
            Tick::BuildCli => "build:cli",
            Tick::BuildApp => "build:app",
            Tick::TracyFeature => "build:tracy",
            Tick::RunServer => "run:server",
            Tick::RunCli => "run:cli",
            Tick::RunApp => "run:app",
            Tick::RunTracy => "run:tracy",
            Tick::RunMeownitor => "run:meownitor",
            Tick::RunWeb => "run:web",
        }
    }
}

/// One row of the pane: a label, its Build and Run boxes, a note.
pub struct Row {
    pub label: &'static str,
    pub build: Option<Tick>,
    pub run: Option<Tick>,
    pub note: &'static str,
}

/// In the order `dev` runs them (the CLI here always last).
pub const ROWS: &[Row] = &[
    Row {
        label: "stop first",
        build: None,
        run: Some(Tick::Stop),
        note: "stop server, CLI, Meownitor, Tracy, app",
    },
    Row {
        label: "server",
        build: Some(Tick::BuildServer),
        run: Some(Tick::RunServer),
        note: "restart with the web UI, show, clean start",
    },
    Row {
        label: "tracy",
        build: Some(Tick::TracyFeature),
        run: Some(Tick::RunTracy),
        note: "build = server feature · run = profiler",
    },
    Row {
        label: "meownitor",
        build: None,
        run: Some(Tick::RunMeownitor),
        note: "terminal window",
    },
    Row {
        label: "web UI",
        build: None,
        run: Some(Tick::RunWeb),
        note: "open in the browser",
    },
    Row {
        label: "app",
        build: Some(Tick::BuildApp),
        run: Some(Tick::RunApp),
        note: "attaches to the server, or any station",
    },
    Row {
        label: "cli",
        build: Some(Tick::BuildCli),
        run: Some(Tick::RunCli),
        note: "Space: off → here → window",
    },
];

pub struct Options {
    pub debug: bool,
    pub show: String,
    pub user: String,
    pub clean_start: bool,
}

pub struct Plan {
    ticks: Vec<Tick>,
    /// The CLI opens in its own window instead of attaching here.
    pub cli_window: bool,
}

impl Plan {
    pub fn on(&self, t: Tick) -> bool {
        self.ticks.contains(&t)
    }

    fn set(&mut self, t: Tick, on: bool) {
        self.ticks.retain(|x| *x != t);
        if on {
            self.ticks.push(t);
        }
    }

    /// Space on a box. The CLI's Run box cycles off → here → window.
    pub fn toggle(&mut self, t: Tick) {
        if t == Tick::RunCli {
            match (self.on(t), self.cli_window) {
                (false, _) => {
                    self.set(t, true);
                    self.cli_window = false;
                }
                (true, false) => self.cli_window = true,
                (true, true) => {
                    self.set(t, false);
                    self.cli_window = false;
                }
            }
            return;
        }
        let on = !self.on(t);
        self.set(t, on);
    }

    pub fn clear(&mut self) {
        self.ticks.clear();
        self.cli_window = false;
    }

    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    fn builds(&self) -> Vec<&'static str> {
        [
            (Tick::BuildServer, "server"),
            (Tick::BuildCli, "cli"),
            (Tick::BuildApp, "app"),
        ]
        .iter()
        .filter(|(t, _)| self.on(*t))
        .map(|(_, n)| *n)
        .collect()
    }

    fn runs(&self) -> Vec<&'static str> {
        let cli = if self.cli_window { "cli-window" } else { "cli" };
        [
            (Tick::RunServer, "server"),
            (Tick::RunTracy, "tracy"),
            (Tick::RunMeownitor, "meownitor"),
            (Tick::RunWeb, "web"),
            (Tick::RunApp, "app"),
            (Tick::RunCli, cli),
        ]
        .iter()
        .filter(|(t, _)| self.on(*t))
        .map(|(_, n)| *n)
        .collect()
    }

    /// The arguments for `dev` (Unix flags or the PowerShell parameters).
    pub fn args(&self, opts: &Options) -> Vec<String> {
        let win = cfg!(windows);
        let flag = |unix: &str, ps: &str| {
            if win {
                ps.to_string()
            } else {
                unix.to_string()
            }
        };
        let mut a = Vec::new();
        if self.on(Tick::Stop) {
            a.push(flag("--stop", "-Stop"));
        }
        let builds = self.builds();
        if !builds.is_empty() {
            a.push(flag("--build", "-Build"));
            a.push(builds.join(","));
        }
        // Tracy as a feature only matters for a server build (or a run,
        // which turns it on in the script anyway).
        if self.on(Tick::TracyFeature) && self.on(Tick::BuildServer) {
            a.push(flag("--tracy", "-Tracy"));
        }
        let runs = self.runs();
        if !runs.is_empty() {
            a.push(flag("--run", "-Run"));
            a.push(runs.join(","));
        }
        if self.on(Tick::RunServer) {
            if !opts.show.is_empty() {
                a.push(flag("--show", "-Show"));
                a.push(opts.show.clone());
            }
            if opts.clean_start {
                a.push(flag("--clean-start", "-CleanStart"));
            }
        }
        if self.on(Tick::RunCli) && !opts.user.is_empty() && opts.user != "Admin" {
            a.push(flag("--user", "-User"));
            a.push(opts.user.clone());
        }
        if opts.debug {
            a.push(flag("--debug", "-DebugBuild"));
        }
        a
    }

    /// What the plan will do that may surprise, shown under the pane.
    pub fn warnings(&self) -> Vec<&'static str> {
        let mut w = Vec::new();
        if self.on(Tick::TracyFeature) && !self.on(Tick::BuildServer) {
            w.push("tracy feature needs Build server");
        }
        if self.on(Tick::BuildServer) && !self.on(Tick::RunServer) {
            w.push("a running server keeps the old binary (tick Run server)");
        }
        w
    }
}

pub fn script(root: &Path) -> PathBuf {
    if cfg!(windows) {
        root.join("scripts").join("win").join("dev.ps1")
    } else {
        root.join("scripts").join("unix").join("dev")
    }
}

pub fn display(plan: &Plan, opts: &Options) -> String {
    let script = if cfg!(windows) {
        ".\\scripts\\win\\dev.ps1"
    } else {
        "./scripts/unix/dev"
    };
    std::iter::once(script.to_string())
        .chain(plan.args(opts).into_iter().map(|a| {
            if a.contains(' ') {
                format!("\"{a}\"")
            } else {
                a
            }
        }))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn command(root: &Path, plan: &Plan, opts: &Options) -> Command {
    let path = script(root);
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
        c.arg(&path);
        c
    } else {
        Command::new(&path)
    };
    cmd.args(plan.args(opts));
    cmd.current_dir(root);
    cmd
}

/// Saved between runs of the launcher (target/ is gitignored).
fn state_file(root: &Path) -> PathBuf {
    root.join("target").join("lumacore-scripts.json")
}

pub fn load(root: &Path) -> (Plan, Options) {
    let v: Value = std::fs::read_to_string(state_file(root))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let names: Vec<&str> = v["plan"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let plan = Plan {
        ticks: Tick::ALL
            .into_iter()
            .filter(|t| names.contains(&t.key()))
            .collect(),
        cli_window: v["cli_window"].as_bool().unwrap_or(false),
    };
    let text = |k: &str, default: &str| v[k].as_str().unwrap_or(default).to_string();
    let opts = Options {
        debug: v["debug"].as_bool().unwrap_or(false),
        show: text("show", ""),
        user: text("user", "Admin"),
        clean_start: v["clean_start"].as_bool().unwrap_or(false),
    };
    (plan, opts)
}

pub fn save(root: &Path, plan: &Plan, opts: &Options) {
    let v = json!({
        "plan": plan.ticks.iter().map(|t| t.key()).collect::<Vec<_>>(),
        "cli_window": plan.cli_window,
        "debug": opts.debug,
        "show": opts.show,
        "user": opts.user,
        "clean_start": opts.clean_start,
    });
    if let Ok(text) = serde_json::to_string_pretty(&v) {
        let _ = std::fs::create_dir_all(root.join("target"));
        let _ = std::fs::write(state_file(root), text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Options {
        Options {
            debug: false,
            show: "demo".into(),
            user: "Admin".into(),
            clean_start: false,
        }
    }

    #[test]
    fn a_plan_writes_the_dev_line_in_run_order() {
        let mut p = Plan {
            ticks: Vec::new(),
            cli_window: false,
        };
        for t in [
            Tick::RunCli,
            Tick::BuildCli,
            Tick::RunServer,
            Tick::BuildServer,
            Tick::RunWeb,
        ] {
            p.toggle(t);
        }
        if cfg!(windows) {
            return;
        }
        assert_eq!(
            p.args(&opts()),
            [
                "--build",
                "server,cli",
                "--run",
                "server,web,cli",
                "--show",
                "demo"
            ]
        );
        p.toggle(Tick::RunCli); // here → window
        assert!(
            p.args(&opts())
                .contains(&"server,web,cli-window".to_string())
        );
        p.toggle(Tick::RunCli); // window → off
        assert!(!p.on(Tick::RunCli));
    }

    #[test]
    fn show_and_user_ride_only_with_what_uses_them() {
        let mut p = Plan {
            ticks: vec![Tick::BuildServer],
            cli_window: false,
        };
        let mut o = opts();
        o.user = "LD".into();
        assert!(!p.args(&o).iter().any(|a| a == "demo" || a == "LD"));
        p.toggle(Tick::RunCli);
        assert!(p.args(&o).iter().any(|a| a == "LD"));
    }

    #[test]
    fn every_row_box_has_a_saved_name() {
        for row in ROWS {
            for t in [row.build, row.run].into_iter().flatten() {
                assert!(Tick::ALL.contains(&t), "{t:?}");
            }
        }
    }
}
