// ██████╗ ██╗   ██╗███╗   ██╗
// ██╔══██╗██║   ██║████╗  ██║
// ██████╔╝██║   ██║██╔██╗ ██║
// ██╔══██╗██║   ██║██║╚██╗██║
// ██║  ██║╚██████╔╝██║ ╚████║
// ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: run

//! The overnight run: the exhaustive sweep, then random sequences, then every cargo-fuzz target —
//! resumable. Everything lives in one report directory (target/fuzz-report/): run.json,
//! inflight.txt, sweep_done.txt / sweep_plan.txt, findings.jsonl, replays/ and summary.md. On
//! start: completed → a new run; stopped → continue; still running but its pid gone → crashed: the
//! line in flight is a finding, skipped, and the run continues. A new build runs the cases it added
//! first; --fresh starts over.
//!
//! Owns:          the overnight run: the exhaustive sweep, random sequences, every cargo-fuzz target; its report directory, resume, watchdog, replay.
//! Doesn't own:   running one line (world.rs), the binary (the station's src/bin/lumacore-sweep.rs).
//! Entry points:  long, replay, Options, RunState.
//! Decisions:     §25 "Step 10 hardening", §34 24.1.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::lines::{self, Case, Rng};
use crate::vocab::Vocab;
use crate::world::{Finding, OWNER, Outcome, ROLES, World};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A line running longer than this is a hang: the watchdog ends the process.
pub const HANG: Duration = Duration::from_secs(60);
/// Lines between checkpoints.
const CHECKPOINT: u64 = 100;
/// Lines per random sequence before the world starts over.
const SEQUENCE: usize = 200;
/// A progress line at least this often, so a watcher sees the run move.
const PROGRESS: Duration = Duration::from_secs(5);
/// The sandbox's `library/` is copied fresh once lines have written more than
/// this into it (every `SaveShow` keeps a ~1 MB show); checked per checkpoint.
const SANDBOX_CAP: u64 = 64 * 1024 * 1024;

/// Time-based progress: `due()` is true once per [`PROGRESS`].
struct Progress(Instant);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Progress {
    fn new() -> Self {
        Self(Instant::now())
    }

    fn due(&mut self) -> bool {
        if self.0.elapsed() < PROGRESS {
            return false;
        }
        self.0 = Instant::now();
        true
    }
}

/// `1h05m` / `4m12s` / `9s`.
fn span(d: Duration) -> String {
    let s = d.as_secs();
    match s {
        0..60 => format!("{s}s"),
        60..3600 => format!("{}m{:02}s", s / 60, s % 60),
        _ => format!("{}h{:02}m", s / 3600, s % 3600 / 60),
    }
}

/// A command line cut to fit one progress line.
fn short(line: &str) -> String {
    const MAX: usize = 60;
    if line.chars().count() <= MAX {
        line.to_string()
    } else {
        format!("{}…", line.chars().take(MAX).collect::<String>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Running,
    Stopped,
    Completed,
}

/// One stretch of the run by one build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub build: String,
    pub started_ms: u64,
    pub ended_ms: u64,
    /// How it ended: completed, stopped, crashed, hang.
    pub end: String,
    pub lines: u64,
}

/// `run.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub status: Status,
    pub pid: u32,
    pub build: String,
    pub started_ms: u64,
    pub heartbeat_ms: u64,
    /// `sweep`, `random`, `fuzz:<target>` or `done`.
    pub phase: String,
    pub seed: u64,
    /// Random sequences finished.
    pub iteration: u64,
    pub random_ms: u64,
    pub fuzz_done: Vec<String>,
    pub lines: u64,
    pub segments: Vec<Segment>,
    /// Lines that took the process down (never run again this run).
    pub killers: Vec<String>,
    /// Findings by kind (`check · word`) → how often.
    pub counts: BTreeMap<String, u64>,
    /// Per word: [runs, ok, errors].
    pub coverage: BTreeMap<String, [u64; 3]>,
}

/// What to run.
#[derive(Debug, Clone)]
pub struct Options {
    pub dir: PathBuf,
    pub fresh: bool,
    pub random_minutes: u64,
    pub fuzz_minutes: u64,
    /// The cargo-fuzz crate (`tests/fuzz/`); None skips fuzzing.
    pub fuzz_dir: Option<PathBuf>,
    /// The show the sweep starts from.
    pub base: PathBuf,
}

/// Shared with the signal handler and the watchdog.
struct Live {
    stop: Arc<AtomicBool>,
    /// When the line in flight started (ms), 0 = none.
    line_started: Arc<AtomicU64>,
}

/// Wall-clock ms (the one copy is the base's, decisions §34 25.14).
pub use lumacore_base::clock::now_ms;

/// Run (or continue) the overnight run. Returns the process exit code:
/// 0 completed, 130 stopped.
pub fn long(opts: &Options, build: &str) -> Result<i32, String> {
    std::fs::create_dir_all(opts.dir.join("replays")).map_err(|e| e.to_string())?;
    // Absolute paths: the line phases work inside the sandbox.
    let absolute = |p: &Path| std::path::absolute(p).map_err(|e| format!("{}: {e}", p.display()));
    let opts = &Options {
        dir: absolute(&opts.dir)?,
        base: absolute(&opts.base)?,
        fuzz_dir: opts.fuzz_dir.as_deref().map(absolute).transpose()?,
        ..opts.clone()
    };
    let mut run = start(opts, build)?;
    let live = Live {
        stop: Arc::new(AtomicBool::new(false)),
        line_started: Arc::new(AtomicU64::new(0)),
    };
    #[cfg(unix)]
    for sig in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM, signal_hook::consts::SIGHUP] {
        signal_hook::flag::register(sig, live.stop.clone()).map_err(|e| e.to_string())?;
    }
    watchdog(opts.dir.clone(), live.line_started.clone());

    let base = Arc::new(base_json(&opts.base)?);
    let mut report = Report::open(&opts.dir, build)?;
    let segment_start = run.lines;

    let sandbox = Sandbox::enter(&opts.dir)?;
    if run.phase == "sweep" {
        sweep(opts, &mut run, &base, &live, &mut report, &sandbox)?;
        if !stopped(&live) {
            run.phase = "random".into();
            save(&opts.dir, &mut run)?;
        }
    }
    if run.phase == "random" && !stopped(&live) {
        random(opts, &mut run, &base, &live, &mut report, &sandbox)?;
        if !stopped(&live) {
            run.phase = "fuzz".into();
            save(&opts.dir, &mut run)?;
        }
    }
    drop(sandbox);
    if run.phase.starts_with("fuzz") && !stopped(&live) {
        fuzz(opts, &mut run, &live, &mut report)?;
        if !stopped(&live) {
            run.phase = "done".into();
        }
    }

    let end = if stopped(&live) { "stopped" } else { "completed" };
    run.status = if stopped(&live) { Status::Stopped } else { Status::Completed };
    run.segments.push(Segment {
        build: build.to_string(),
        started_ms: report.segment_started,
        ended_ms: now_ms(),
        end: end.to_string(),
        lines: run.lines - segment_start,
    });
    save(&opts.dir, &mut run)?;
    let _ = std::fs::remove_file(opts.dir.join("inflight.txt"));
    write_summary(&opts.dir, &run)?;
    println!("sweep {end}: {} lines, {} kinds of findings — {}", run.lines, run.counts.len(), opts.dir.join("summary.md").display());
    Ok(if stopped(&live) { 130 } else { 0 })
}

fn stopped(live: &Live) -> bool {
    live.stop.load(Ordering::Relaxed)
}

/// Read `run.json` and decide: new run, continue, or record a crash first.
fn start(opts: &Options, build: &str) -> Result<RunState, String> {
    let path = opts.dir.join("run.json");
    let old: Option<RunState> = std::fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok());
    let fresh = || RunState {
        status: Status::Running,
        pid: std::process::id(),
        build: build.to_string(),
        started_ms: now_ms(),
        heartbeat_ms: now_ms(),
        phase: "sweep".into(),
        seed: now_ms(),
        iteration: 0,
        random_ms: 0,
        fuzz_done: Vec::new(),
        lines: 0,
        segments: Vec::new(),
        killers: Vec::new(),
        counts: BTreeMap::new(),
        coverage: BTreeMap::new(),
    };
    let mut run = match old {
        Some(old) if !opts.fresh && old.status != Status::Completed => {
            if old.status == Status::Running && alive(old.pid) {
                return Err(format!("a sweep is already running (pid {})", old.pid));
            }
            let mut run = old;
            if run.status == Status::Running {
                // The process died: the line in flight took it down.
                let hung = opts.dir.join("hung.txt").exists();
                let inflight = std::fs::read_to_string(opts.dir.join("inflight.txt")).unwrap_or_default();
                let end = if hung { "hang" } else { "crashed" };
                println!("the last run {end} (pid {}) on: {}", run.pid, inflight.trim());
                run.segments.push(Segment {
                    build: run.build.clone(),
                    started_ms: run.heartbeat_ms,
                    ended_ms: run.heartbeat_ms,
                    end: end.to_string(),
                    lines: 0,
                });
                if !inflight.trim().is_empty() {
                    let mut report = Report::open(&opts.dir, &run.build)?;
                    let (role, line) = inflight.trim().split_once('\t').unwrap_or(("?", inflight.trim()));
                    let finding = Finding {
                        check: end.to_string(),
                        word: super::world::word_of(line),
                        detail: format!("the process {end} on this line as {role}"),
                    };
                    let phase = run.phase.clone();
                    report.record(&mut run, &finding, &phase, &[inflight.trim().to_string()]);
                    run.killers.push(line.to_string());
                    if run.phase == "sweep" {
                        report.done(lines::fnv(line))?;
                    }
                }
                let _ = std::fs::remove_file(opts.dir.join("hung.txt"));
            } else {
                println!("continuing the run from {} (phase {})", run.segments.last().map_or("start", |s| s.end.as_str()), run.phase);
            }
            if run.build != build {
                println!("new build {build} (was {}): its new cases run first", run.build);
            }
            run.build = build.to_string();
            run
        }
        Some(old) => {
            archive(&opts.dir, &old)?;
            fresh()
        }
        None => fresh(),
    };
    run.status = Status::Running;
    run.pid = std::process::id();
    save(&opts.dir, &mut run)?;
    Ok(run)
}

/// Keep a finished run's files under `runs/<started>/`, start clean.
fn archive(dir: &Path, old: &RunState) -> Result<(), String> {
    let to = dir.join("runs").join(old.started_ms.to_string());
    std::fs::create_dir_all(&to).map_err(|e| e.to_string())?;
    for name in ["run.json", "sweep_done.txt", "sweep_plan.txt", "findings.jsonl", "summary.md", "replays"] {
        let from = dir.join(name);
        if from.exists() {
            let _ = std::fs::rename(&from, to.join(name));
        }
    }
    std::fs::create_dir_all(dir.join("replays")).map_err(|e| e.to_string())
}

fn alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn save(dir: &Path, run: &mut RunState) -> Result<(), String> {
    run.heartbeat_ms = now_ms();
    let json = serde_json::to_string_pretty(run).map_err(|e| e.to_string())?;
    let tmp = dir.join("run.json.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(tmp, dir.join("run.json")).map_err(|e| e.to_string())
}

/// Ends the process when one line runs longer than [`HANG`].
fn watchdog(dir: PathBuf, line_started: Arc<AtomicU64>) {
    std::thread::Builder::new()
        .name("sweep-watchdog".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(5));
                let started = line_started.load(Ordering::Relaxed);
                if started != 0 && now_ms().saturating_sub(started) > HANG.as_millis() as u64 {
                    let _ = std::fs::write(dir.join("hung.txt"), now_ms().to_string());
                    eprintln!("a line ran over {} s: stopping (the next start skips it)", HANG.as_secs());
                    std::process::exit(3);
                }
            }
        })
        .ok();
}

/// Findings, replays and the done list on disk.
struct Report {
    dir: PathBuf,
    build: String,
    done: std::fs::File,
    segment_started: u64,
}

impl Report {
    fn open(dir: &Path, build: &str) -> Result<Self, String> {
        let done = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("sweep_done.txt"))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            dir: dir.to_path_buf(),
            build: build.to_string(),
            done,
            segment_started: now_ms(),
        })
    }

    fn done(&mut self, id: u64) -> Result<(), String> {
        writeln!(self.done, "{id:016x}").map_err(|e| e.to_string())
    }

    /// Count a finding; the first of its kind is written out with a replay.
    fn record(&mut self, run: &mut RunState, f: &Finding, phase: &str, history: &[String]) {
        let count = run.counts.entry(f.key()).or_insert(0);
        *count += 1;
        if *count > 1 {
            return;
        }
        let n = run.counts.len();
        let replay = self.dir.join("replays").join(format!("{n:04}.txt"));
        let _ = std::fs::write(&replay, history.join("\n") + "\n");
        let entry = serde_json::json!({
            "time_ms": now_ms(),
            "build": self.build,
            "phase": phase,
            "check": f.check,
            "word": f.word,
            "detail": f.detail,
            "line": history.last(),
            "replay": format!("cargo run --profile sweep -p lumacore-station --bin lumacore-sweep -- replay {}", replay.display()),
        });
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(self.dir.join("findings.jsonl")) {
            let _ = writeln!(file, "{entry}");
        }
        println!("finding #{n}: {} — {}", f.key(), f.detail);
    }
}

fn inflight(dir: &Path, live: &Live, role: &str, line: &str) {
    let _ = std::fs::write(dir.join("inflight.txt"), format!("{role}\t{line}"));
    live.line_started.store(now_ms(), Ordering::Relaxed);
}

fn landed(live: &Live) {
    live.line_started.store(0, Ordering::Relaxed);
}

fn cover(run: &mut RunState, word: &str, outcome: &Outcome) {
    let c = run.coverage.entry(word.to_string()).or_insert([0; 3]);
    c[0] += 1;
    if outcome.error.is_some() {
        c[2] += 1;
    } else {
        c[1] += 1;
    }
}

/// The exhaustive plan, each case as every role from a fresh world.
fn sweep(
    opts: &Options,
    run: &mut RunState,
    base: &Arc<String>,
    live: &Live,
    report: &mut Report,
    sandbox: &Sandbox,
) -> Result<(), String> {
    let mut world = World::new(base.clone())?;
    let vocab = Vocab::new(&world.master);
    let plan = lines::exhaustive(&vocab, &world.master);
    let done: BTreeSet<u64> = read_ids(&opts.dir.join("sweep_done.txt"));
    let old_plan: BTreeSet<u64> = read_ids(&opts.dir.join("sweep_plan.txt"));
    let ids: Vec<String> = plan.iter().map(|c| format!("{:016x}", c.id())).collect();
    std::fs::write(opts.dir.join("sweep_plan.txt"), ids.join("\n") + "\n").map_err(|e| e.to_string())?;
    // Cases this build added go first, then the rest still to do.
    let (new, old): (Vec<&Case>, Vec<&Case>) = plan
        .iter()
        .filter(|c| !done.contains(&c.id()) && !run.killers.contains(&c.line))
        .partition(|c| !old_plan.is_empty() && !old_plan.contains(&c.id()));
    println!(
        "sweep: {} cases ({} done, {} new in this build), {} words skipped (tree shell / drive)",
        plan.len(),
        done.len(),
        new.len(),
        vocab.skipped.len()
    );
    let todo: Vec<&Case> = new.into_iter().chain(old).collect();
    let started = Instant::now();
    let mut progress = Progress::new();
    for (i, case) in todo.iter().enumerate() {
        if stopped(live) {
            break;
        }
        let mut admin: Option<Outcome> = None;
        for role in ROLES {
            world.reset()?;
            // Each role starts from the same disk: a Drive line of the one
            // before (Admin's NewShow) never changes the next (issue 20 c).
            if lumacore_objects::commands::registry::line_reach(&case.line) == lumacore_objects::commands::registry::Reach::Drive {
                sandbox.refresh()?;
            }
            inflight(&opts.dir, live, role, &case.line);
            let (outcome, mut findings) = world.run(role, &case.line, case.expect.as_ref());
            if i % 10 == 0 && role == "Admin" {
                let mut heavy = Vec::new();
                world.heavy(&mut |check: &str, detail: String| {
                    heavy.push(Finding {
                        check: check.to_string(),
                        word: super::world::word_of(&case.line),
                        detail,
                    })
                });
                findings.extend(heavy);
            }
            landed(live);
            run.lines += 1;
            let word = super::world::word_of(&case.line);
            cover(run, &word, &outcome);
            // Rights: refused with the rights or claim code, else like Admin.
            match &admin {
                None => admin = Some(outcome.clone()),
                Some(a) => {
                    let refused = matches!(&outcome.error, Some(Some(c)) if c == "E-RIGHTS-001" || c == "E-CLAIM-001");
                    // What the acting user owns differs by role (their
                    // profile, programmer, selection, usage — named `self`),
                    // and a replaced show changes everything: compared without
                    // those (issue 20 d). Another user's subtrees still count.
                    let shared = |changed: &[String]| -> Vec<String> {
                        changed.iter().filter(|c| !c.contains("self")).cloned().collect()
                    };
                    let differs = a.error.is_some() != outcome.error.is_some()
                        || (!a.replaced && !outcome.replaced && shared(&a.changed) != shared(&outcome.changed));
                    if !refused && differs {
                        findings.push(Finding {
                            check: "role-differs".into(),
                            word: word.clone(),
                            detail: format!(
                                "as {role}: error {:?}, changed [{}]; as Admin: error {:?}, changed [{}]",
                                outcome.error,
                                outcome.changed.join(", "),
                                a.error,
                                a.changed.join(", ")
                            ),
                        });
                    }
                }
            }
            for f in &findings {
                report.record(run, f, "sweep", &world.history);
            }
        }
        report.done(case.id())?;
        if run.lines % CHECKPOINT < ROLES.len() as u64 {
            save(&opts.dir, run)?;
            sandbox.trim()?;
        }
        if progress.due() || i + 1 == todo.len() {
            let rate = (i + 1) as f64 / started.elapsed().as_secs_f64().max(0.001);
            let left = Duration::from_secs_f64((todo.len() - i - 1) as f64 / rate.max(0.001));
            println!(
                "sweep {}/{} · {:.1} cases/s · {} left · {} kinds of findings · last: {}",
                i + 1,
                todo.len(),
                rate,
                span(left),
                run.counts.len(),
                short(&case.line)
            );
        }
    }
    save(&opts.dir, run)
}

fn read_ids(path: &Path) -> BTreeSet<u64> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| u64::from_str_radix(l.trim(), 16).ok())
        .collect()
}

/// Random sequences until the time is used up; each sequence is replayable
/// from (seed, iteration) and starts from a fresh sandbox, like a replay.
fn random(
    opts: &Options,
    run: &mut RunState,
    base: &Arc<String>,
    live: &Live,
    report: &mut Report,
    sandbox: &Sandbox,
) -> Result<(), String> {
    let budget = opts.random_minutes * 60_000;
    let mut world = World::new(base.clone())?;
    let vocab = Vocab::new(&world.master);
    let roles: Vec<&str> = ROLES.iter().copied().chain([OWNER]).collect();
    println!("random: {} min left (seed {}, sequence {})", budget.saturating_sub(run.random_ms) / 60_000, run.seed, run.iteration);
    let mut progress = Progress::new();
    let mut last = String::new();
    while run.random_ms < budget && !stopped(live) {
        let began = Instant::now();
        let mut rng = Rng(run.seed ^ run.iteration.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        sandbox.refresh()?;
        world.reset()?;
        for _ in 0..SEQUENCE {
            if stopped(live) {
                break;
            }
            let line = lines::random_line(&mut rng, &world.master, &vocab);
            if run.killers.contains(&line) {
                continue;
            }
            let role = *rng.pick(&roles).unwrap_or(&"Admin");
            inflight(&opts.dir, live, role, &line);
            let (outcome, findings) = world.run(role, &line, None);
            landed(live);
            run.lines += 1;
            cover(run, &super::world::word_of(&line), &outcome);
            for f in &findings {
                report.record(run, f, "random", &world.history);
            }
            if outcome.ended {
                world.reset()?;
            }
            if progress.due() {
                let spent = run.random_ms + began.elapsed().as_millis() as u64;
                println!(
                    "random: {} left · sequence {} · {} lines · {} kinds of findings · last: {role} › {}",
                    span(Duration::from_millis(budget.saturating_sub(spent))),
                    run.iteration,
                    run.lines,
                    run.counts.len(),
                    short(&line)
                );
            }
            last = line;
        }
        run.iteration += 1;
        run.random_ms += began.elapsed().as_millis() as u64;
        save(&opts.dir, run)?;
    }
    println!("random: {} · {} sequences · {} lines · {} kinds of findings · last: {}", if stopped(live) { "stopped" } else { "done" }, run.iteration, run.lines, run.counts.len(), short(&last));
    Ok(())
}

/// Every cargo-fuzz target for `fuzz_minutes` each (nightly).
fn fuzz(opts: &Options, run: &mut RunState, live: &Live, report: &mut Report) -> Result<(), String> {
    let Some(fuzz_dir) = &opts.fuzz_dir else {
        return Ok(());
    };
    let list = std::process::Command::new("cargo")
        .args(["+nightly", "fuzz", "list", "--fuzz-dir"])
        .arg(fuzz_dir)
        .output();
    let targets: Vec<String> = match list {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect(),
        _ => {
            println!("fuzz: skipped (needs `rustup toolchain install nightly` and `cargo install cargo-fuzz`)");
            return Ok(());
        }
    };
    for target in targets {
        if stopped(live) || run.fuzz_done.contains(&target) {
            continue;
        }
        run.phase = format!("fuzz:{target}");
        save(&opts.dir, run)?;
        let artifacts = fuzz_dir.join("artifacts").join(&target);
        let before = list_files(&artifacts);
        let log_path = opts.dir.join(format!("fuzz-{target}.log"));
        let log = std::fs::File::create(&log_path).map_err(|e| e.to_string())?;
        println!("fuzz {target}: {} min", opts.fuzz_minutes);
        let mut child = std::process::Command::new("cargo")
            .args(["+nightly", "fuzz", "run", "--fuzz-dir"])
            .arg(fuzz_dir)
            .arg(&target)
            .arg("--")
            .arg(format!("-max_total_time={}", opts.fuzz_minutes * 60))
            .args(["-fork=1", "-ignore_crashes=1", "-ignore_ooms=1", "-ignore_timeouts=1", "-print_final_stats=1"])
            .stdout(log.try_clone().map_err(|e| e.to_string())?)
            .stderr(log)
            .spawn()
            .map_err(|e| e.to_string())?;
        let began = Instant::now();
        let mut progress = Progress::new();
        loop {
            if let Some(_status) = child.try_wait().map_err(|e| e.to_string())? {
                break;
            }
            if progress.due() {
                println!(
                    "fuzz {target}: {} / {} min · {}",
                    span(began.elapsed()),
                    opts.fuzz_minutes,
                    last_stat(&log_path)
                );
            }
            if stopped(live) {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        for file in list_files(&artifacts).difference(&before) {
            let kind = file.split('-').next().unwrap_or("crash").to_string();
            let finding = Finding {
                check: format!("fuzz-{kind}"),
                word: target.clone(),
                detail: format!("{}", artifacts.join(file).display()),
            };
            let phase = run.phase.clone();
            report.record(run, &finding, &phase, &[format!("cargo +nightly fuzz run {target} {}", artifacts.join(file).display())]);
        }
        run.fuzz_done.push(target);
        save(&opts.dir, run)?;
    }
    Ok(())
}

/// The newest libFuzzer status line (`#123 pulse cov: … exec/s: …`), or
/// what the build is doing before the fuzzer starts.
fn last_stat(log: &Path) -> String {
    let text = std::fs::read_to_string(log).unwrap_or_default();
    let line = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| l.starts_with('#') || l.starts_with("Compiling") || l.starts_with("Finished"))
        .or_else(|| text.lines().rev().map(str::trim).find(|l| !l.is_empty()))
        .unwrap_or("starting");
    short(line)
}

fn list_files(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default()
}

/// `summary.md`: findings by kind, words never accepted, what was skipped.
fn write_summary(dir: &Path, run: &RunState) -> Result<(), String> {
    let mut s = String::new();
    s.push_str(&format!(
        "# Command sweep + fuzz — {}\n\nBuild `{}` · phase `{}` · {} lines · seed {} · {} random sequences\n\n",
        chrono::DateTime::from_timestamp_millis(run.heartbeat_ms as i64).map(|t| t.to_rfc3339()).unwrap_or_default(),
        run.build,
        run.phase,
        run.lines,
        run.seed,
        run.iteration
    ));
    s.push_str("## Segments\n\n| build | lines | end |\n|---|---|---|\n");
    for seg in &run.segments {
        s.push_str(&format!("| {} | {} | {} |\n", seg.build, seg.lines, seg.end));
    }
    s.push_str("\n## Findings (kind → count; details in findings.jsonl)\n\n");
    if run.counts.is_empty() {
        s.push_str("None.\n");
    }
    let mut counts: Vec<(&String, &u64)> = run.counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (key, n) in counts {
        s.push_str(&format!("- {key}: {n}\n"));
    }
    if !run.killers.is_empty() {
        s.push_str("\n## Lines that took the process down\n\n");
        for k in &run.killers {
            s.push_str(&format!("- `{k}`\n"));
        }
    }
    // Per first word: a word no line ever got accepted with.
    let mut words: BTreeMap<&str, [u64; 2]> = BTreeMap::new();
    for (word, [runs, ok, _]) in &run.coverage {
        let w = words.entry(word.split(' ').next().unwrap_or("")).or_insert([0; 2]);
        w[0] += runs;
        w[1] += ok;
    }
    s.push_str("\n## Unreachable: words never accepted (every line with them was an error)\n\n");
    for (word, [runs, ok]) in &words {
        if *ok == 0 {
            s.push_str(&format!("- `{word}` ({runs} lines)\n"));
        }
    }
    s.push_str(&format!("\n{} words run, {} never accepted.\n", words.len(), words.values().filter(|w| w[1] == 0).count()));
    std::fs::write(dir.join("summary.md"), s).map_err(|e| e.to_string())
}

/// Run a replay file (`role<TAB>line` per line) from a fresh world and print
/// what each line answered and found.
pub fn replay(file: &Path, base: &Path) -> Result<(), String> {
    let base = Arc::new(base_json(base)?);
    let text = std::fs::read_to_string(file).map_err(|e| e.to_string())?;
    let dir = std::path::absolute(file.parent().unwrap_or(Path::new("."))).map_err(|e| e.to_string())?;
    let _sandbox = Sandbox::enter(&dir)?;
    let mut world = World::new(base)?;
    for row in text.lines().filter(|l| !l.trim().is_empty()) {
        let (role, line) = row.split_once('\t').unwrap_or(("Admin", row));
        let (outcome, findings) = world.run(role, line, None);
        println!("{role:>6} › {line}\n        error {:?}, changed [{}]", outcome.error, outcome.changed.join(", "));
        for f in findings {
            println!("        ✗ {} — {}", f.key(), f.detail);
        }
        if outcome.ended {
            world.reset()?;
        }
    }
    Ok(())
}

/// The base show's JSON (a `.luma` container, `lumacore_luma::show_file`).
fn base_json(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    lumacore_luma::show_file::decode(&bytes).map(|c| c.json)
}

/// The sweep's sandbox (decisions §29): a throwaway copy of `library/` in
/// `<report>/sandbox/` — without the station identity, login secrets and
/// logs — that the process works in while lines run. The drive door lets
/// only its paths through (`drive::set_sandbox`); leaving restores both.
/// Lines save shows into it, so it is copied again before it grows past
/// [`SANDBOX_CAP`].
struct Sandbox {
    home: PathBuf,
    root: PathBuf,
}

impl Sandbox {
    fn enter(report: &Path) -> Result<Self, String> {
        let home = std::env::current_dir().map_err(|e| e.to_string())?;
        let root = report.join("sandbox");
        if root.exists() {
            std::fs::remove_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        }
        copy_library(&home.join("library"), &root.join("library"))?;
        std::env::set_current_dir(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        lumacore_luma::drive::set_sandbox(Some(root.clone()));
        Ok(Self { home, root })
    }

    /// Throw away what lines wrote and copy `library/` again.
    fn refresh(&self) -> Result<(), String> {
        // A background job (an asset hash) may still read the old files.
        if !lumacore_luma::background::wait_idle(std::time::Duration::from_secs(60)) {
            return Err("a background job is still running after 60 s; the sandbox wasn't refreshed".into());
        }
        let library = self.root.join("library");
        std::fs::remove_dir_all(&library).map_err(|e| format!("{}: {e}", library.display()))?;
        copy_library(&self.home.join("library"), &library)
    }

    /// [`Self::refresh`] once the sandbox has grown past [`SANDBOX_CAP`].
    fn trim(&self) -> Result<(), String> {
        let size = dir_size(&self.root.join("library"));
        if size > SANDBOX_CAP {
            println!("sandbox: {} MB written by lines, copied library/ again", size / (1024 * 1024));
            self.refresh()?;
        }
        Ok(())
    }
}

/// Bytes in the files under `dir`.
fn dir_size(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        lumacore_luma::drive::set_sandbox(None);
        let _ = std::env::set_current_dir(&self.home);
    }
}

/// Copy `library/` for the sandbox, leaving out what identifies this
/// station or holds secrets (`stations/`, `station_settings.json`) and logs.
fn copy_library(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let Ok(entries) = std::fs::read_dir(from) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if matches!(name.as_ref(), "stations" | "station_settings.json" | "logs") {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(name.as_ref()));
        if src.is_dir() {
            copy_library(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", src.display()))?;
        }
    }
    Ok(())
}
