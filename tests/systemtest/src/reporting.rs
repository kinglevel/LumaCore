
use crate::defs::*;
use crate::harness::*;

/// Regenerate the demo show from scratch with the one demo builder
/// (`lumacore_objects::demo`), in the systemtest work directory
/// (`target/systemtest/work/library/shows`), as a show file (the container
/// with its GDTF, decisions §29).
///
/// This ensures the demo show always matches the compiled ShowData structure.
/// Must be called once before any tests run.
pub fn ensure_demo_show() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        println!("gen-demo: regenerating demo.luma from current code...");
        enter_work_dir();
        let root = work_dir();
        // The sources are the repo's (decisions §40 15); the file lands in the work dir.
        let built = lumacore_objects::demo::build(&lumacore_objects::demo::repo_root());
        for w in &built.warnings {
            println!("gen-demo:   WARN {w}");
        }
        let demo_path = root.join("library/shows/demo.luma");
        std::fs::create_dir_all(demo_path.parent().unwrap()).ok();
        std::fs::write(&demo_path, built.file()).expect("failed to write demo.luma");
        let sd = &built.state.root.show_data;
        println!(
            "gen-demo: \u{2713} demo.luma ({} fixtures, {} universes, {} lines, {} users)",
            sd.devices.dmx.patch.fixtures.len(),
            sd.devices.dmx.patch.universes.len(),
            sd.protocols.all_lines().count(),
            sd.user_profiles.len(),
        );
    });
}

pub fn git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn git_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn write_metrics(report_dir: &std::path::Path, summary: &RunSummary) {
    let verdict = if summary.failed > 0 { "fail" } else { "pass" };

    let snapshot = MetricsSnapshot {
        timestamp: chrono_like_now(),
        git_commit: git_commit(),
        git_branch: git_branch(),
        command_batch_wall_ms: summary.command_batch_wall_ms.unwrap_or(0),
        verdict: verdict.to_string(),
    };

    // Write metrics.json
    let metrics_path = report_dir.join("metrics.json");
    if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
        let _ = std::fs::write(&metrics_path, &json);
        println!("metrics:  {}", metrics_path.display());
    }

    // Append to metrics_history.jsonl
    let history_path = report_dir.join("metrics_history.jsonl");
    if let Ok(line) = serde_json::to_string(&snapshot) {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }
}

/// Simple ISO 8601 timestamp without requiring chrono.
fn chrono_like_now() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Approximate -- good enough for metrics tracking
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch to Y-M-D (simplified)
    let (year, month, day) = epoch_days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn epoch_days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from Howard Hinnant
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
