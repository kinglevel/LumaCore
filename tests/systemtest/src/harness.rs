use lumacore_objects::AppState;
use lumacore_objects::commands::dispatch;
use lumacore_objects::DispatchResult;

use crate::defs::*;
use crate::reporting::ensure_demo_show;

use std::env;
use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("LUMACORE_ROOT"))
}

pub fn build_manifest_path() -> PathBuf {
    target_systemtest_dir().join("current_build/command_manifest.json")
}

pub fn target_systemtest_dir() -> PathBuf {
    // Absolute even for a relative CARGO_TARGET_DIR: the tests change the
    // working directory (see `enter_work_dir`).
    env::var("CARGO_TARGET_DIR")
        .map(|dir| repo_root().join(dir))
        .unwrap_or_else(|_| repo_root().join("target"))
        .join("systemtest")
}

/// Where the system tests run: a fresh `target/systemtest/work` with a copy of
/// the repository's GDTF library. LumaCore reads and writes `library/…`
/// relative to the working directory, so every show, log and station-settings
/// file the tests create lands here — never in the repository.
pub fn work_dir() -> PathBuf {
    target_systemtest_dir().join("work")
}

/// Create the work directory (once per process) and make it the working
/// directory.
pub fn enter_work_dir() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let work = work_dir();
        if work.exists() {
            std::fs::remove_dir_all(&work).expect("clear systemtest work dir");
        }
        let gdtf = work.join("library/gdtf");
        std::fs::create_dir_all(&gdtf).expect("create systemtest work dir");
        std::fs::create_dir_all(work.join("library/shows")).expect("create work shows dir");
        if let Ok(entries) = std::fs::read_dir(repo_root().join("library/gdtf")) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    std::fs::copy(entry.path(), gdtf.join(entry.file_name()))
                        .expect("copy GDTF into systemtest work dir");
                }
            }
        }
        env::set_current_dir(&work).expect("enter systemtest work dir");
    });
}

/// A station with the demo show loaded, as `AppState::load_show` leaves it.
///
/// The file is read once (35 ms: unzip + every fixture type from its GDTF);
/// each case gets a copy of what that load produced, put in place the way
/// `load_show` puts it (`replace_show_data` + the file's facts) — 0.6 ms.
/// Per thread, as `AppState` isn't `Sync`.
pub fn fresh_state() -> Result<AppState, String> {
    thread_local! {
        static LOADED: std::cell::OnceCell<Result<AppState, String>> = const { std::cell::OnceCell::new() };
    }
    LOADED.with(|loaded| {
        let template = loaded
            .get_or_init(|| {
                ensure_demo_show();
                let mut state = AppState::new();
                let demo = work_dir().join("library/shows/demo.luma");
                state.load_show(
                    demo.to_str()
                        .ok_or_else(|| "demo show path is not valid UTF-8".to_string())?,
                )?;
                Ok(state)
            })
            .as_ref()
            .map_err(Clone::clone)?;
        let mut state = AppState::new();
        state.replace_show_data(template.root.show_data.clone());
        state.saved = template.saved.clone();
        Ok(state)
    })
}

/// Act as `user` (a test switching hands; an in-session `Login Admin` line would ask for
/// Admin's password instead of switching).
pub fn switch_user(state: &mut AppState, user: &str) -> Result<(), String> {
    state
        .switch_to_existing_user(user)
        .ok_or_else(|| format!("unknown user '{user}'"))
        .map(|_| ())
}

pub fn dispatch_output(state: &mut AppState, command: &str) -> Result<String, String> {
    match dispatch(state, command) {
        DispatchResult::Output(text) => Ok(String::from(text)),
        DispatchResult::ContextSwitch(ctx) => {
            state.context = ctx;
            Ok(String::new())
        }
        DispatchResult::Logout => Ok("logout".to_string()),
        DispatchResult::Exit => Ok("exit".to_string()),
        DispatchResult::Ask(ask) => Ok(format!("{}: {}", ask.title(), ask.view().text)),
    }
}

pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }

    out
}

pub fn run_cmd(state: &mut AppState, command: &str) -> Result<(), String> {
    let output = dispatch_output(state, command)?;
    let normalized = strip_ansi(&output);
    let lower = normalized.to_lowercase();
    if lower.contains("error:") || lower.contains("unknown command") {
        Err(format!("`{command}` failed with `{output}`"))
    } else {
        Ok(())
    }
}

pub fn run_syntax_cmd(state: &mut AppState, command: &str) -> Result<(), String> {
    let output = dispatch_output(state, command)?;
    let normalized = strip_ansi(&output);
    let lower = normalized.to_lowercase();

    if lower.contains("unknown command")
        || lower.contains("selection parse error")
        || lower.contains("unknown node")
        || lower.contains("no such node")
        || lower.contains("usage:")
    {
        Err(format!("`{command}` failed with `{output}`"))
    } else {
        Ok(())
    }
}

pub fn assert_selected_fixtures(state: &AppState, expected: &[u32]) -> Result<(), String> {
    let programmer = state
        .current_programmer()
        .ok_or_else(|| "no active programmer".to_string())?;
    let expected_guids = state.root.show_data.fixture_guids(expected);
    if programmer.selected_fixtures == expected_guids {
        Ok(())
    } else {
        Err(format!(
            "expected selection {:?}, got {:?}",
            expected_guids, programmer.selected_fixtures
        ))
    }
}

/// One case; its lines go to `log` (cases run on worker threads, printed in order after).
pub fn run_test(test: TestDef, log: &mut Vec<String>) -> TestResult {
    for gate in &test.gates {
        if let Err(reason) = (gate.check)() {
            log.push(format!("skip test  {} [{}]: {}", test.name, gate.name, reason));
            return TestResult {
                name: test.name.to_string(),
                status: "skipped".to_string(),
                user: test.user.to_string(),
                reason: Some(format!("{}: {}", gate.name, reason)),
                steps: Vec::new(),
            };
        }
    }

    let mut state = match fresh_state() {
        Ok(state) => state,
        Err(err) => {
            return TestResult {
                name: test.name.to_string(),
                status: "failed".to_string(),
                user: test.user.to_string(),
                reason: Some(err),
                steps: Vec::new(),
            };
        }
    };

    if let Err(err) = switch_user(&mut state, test.user) {
        return TestResult {
            name: test.name.to_string(),
            status: "failed".to_string(),
            user: test.user.to_string(),
            reason: Some(err),
            steps: Vec::new(),
        };
    }

    log.push(format!("begin test {} [{}]", test.name, test.user));
    let mut steps = Vec::new();
    for step in test.steps {
        let result = match step.kind {
            StepKind::Verify(action) => action(&mut state),
            StepKind::SyntaxCommand(command) => match fresh_state() {
                Ok(mut syntax_state) => match switch_user(&mut syntax_state, test.user) {
                    Ok(()) => run_syntax_cmd(&mut syntax_state, command),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            },
        };
        match result {
            Ok(()) => {
                log.push(format!("ok   step    {} -> {}", test.name, step.name));
                steps.push(StepResult {
                    name: step.name.to_string(),
                    status: "passed".to_string(),
                    detail: None,
                });
            }
            Err(err) => {
                log.push(format!("FAIL step    {} -> {}: {}", test.name, step.name, err));
                steps.push(StepResult {
                    name: step.name.to_string(),
                    status: "failed".to_string(),
                    detail: Some(err.clone()),
                });
                return TestResult {
                    name: test.name.to_string(),
                    status: "failed".to_string(),
                    user: test.user.to_string(),
                    reason: Some(err),
                    steps,
                };
            }
        }
    }

    TestResult {
        name: test.name.to_string(),
        status: "passed".to_string(),
        user: test.user.to_string(),
        reason: None,
        steps,
    }
}

pub fn run_all_groups(selected_groups: &[String]) -> RunSummary {
    let manifest_path = build_manifest_path();
    let manifest_display = manifest_path.display().to_string();
    let mut groups = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    // Every case of the selected groups runs on the worker threads; the
    // groups' reports are put together from the results, in order.
    let mut shape = Vec::new();
    let mut cases = Vec::new();
    for group in crate::test_domains::curated_groups() {
        if should_run_group(&group, selected_groups) {
            shape.push((group.name, group.tests.len()));
            cases.extend(group.tests);
        }
    }
    let mut results = run_parallel(cases).into_iter();

    for (name, count) in shape {
        println!("begin group {name}");
        let mut group_result = GroupResult {
            name: name.to_string(),
            passed: 0,
            failed: 0,
            skipped: 0,
            tests: Vec::new(),
        };

        for _ in 0..count {
            let (log, test_result) = results.next().expect("one result per case");
            for line in log {
                println!("{line}");
            }
            match test_result.status.as_str() {
                "passed" => {
                    passed += 1;
                    group_result.passed += 1;
                }
                "failed" => {
                    failed += 1;
                    group_result.failed += 1;
                }
                "skipped" => {
                    skipped += 1;
                    group_result.skipped += 1;
                }
                _ => {}
            }
            group_result.tests.push(test_result);
        }

        groups.push(group_result);
    }

    // Time a representative command batch for the summary
    let command_batch_wall_ms = fresh_state()
        .ok()
        .map(|mut state| crate::test_domains::run_command_batch(&mut state));

    RunSummary {
        build_manifest: manifest_display,
        selected_groups: selected_groups.to_vec(),
        passed,
        failed,
        skipped,
        groups,
        command_batch_wall_ms,
    }
}

/// Run every case on one worker thread per core (each case starts from its own
/// fresh state); results come back in the cases' order, each with its log.
fn run_parallel(cases: Vec<TestDef>) -> Vec<(Vec<String>, TestResult)> {
    use std::sync::Mutex;
    let count = cases.len();
    let queue = Mutex::new(cases.into_iter().enumerate());
    let done = Mutex::new(Vec::with_capacity(count));
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers.min(count.max(1)) {
            std::thread::Builder::new()
                // As much stack as the main thread the binary runs on.
                .stack_size(8 << 20)
                .spawn_scoped(scope, || {
                    loop {
                        let next = queue.lock().expect("case queue").next();
                        let Some((index, test)) = next else { break };
                        let mut log = Vec::new();
                        let result = run_test(test, &mut log);
                        done.lock().expect("results").push((index, log, result));
                    }
                })
                .expect("start a systemtest worker");
        }
    });
    let mut done = done.into_inner().expect("results");
    done.sort_by_key(|(index, _, _)| *index);
    done.into_iter().map(|(_, log, result)| (log, result)).collect()
}

pub fn selected_group_names() -> Vec<String> {
    let mut selected = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--group" {
            if let Some(name) = args.next() {
                selected.push(name);
            }
        } else if let Some(name) = arg.strip_prefix("--group=") {
            selected.push(name.to_string());
        }
    }
    selected
}

fn should_run_group(group: &GroupDef, selected_groups: &[String]) -> bool {
    selected_groups.is_empty()
        || selected_groups
            .iter()
            .any(|name| name.eq_ignore_ascii_case(group.name))
}
