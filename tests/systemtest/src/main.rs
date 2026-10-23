mod defs;
mod harness;
mod reporting;
mod test_domains;

use harness::{run_all_groups, selected_group_names, target_systemtest_dir};
use reporting::{ensure_demo_show, write_metrics};

fn main() {
    lumacore_station::install_catalog().expect("the catalog freezes");
    ensure_demo_show();
    let selected_groups = selected_group_names();
    let summary = run_all_groups(&selected_groups);

    let report_dir = target_systemtest_dir();
    std::fs::create_dir_all(&report_dir).expect("create systemtest report dir");
    let report_path = report_dir.join("last_run.json");
    let summary_json = serde_json::to_string_pretty(&summary).expect("serialize run summary");
    std::fs::write(&report_path, &summary_json).expect("write systemtest report");

    // Write metrics.json and append to metrics_history.jsonl
    write_metrics(&report_dir, &summary);

    println!("manifest: {}", summary.build_manifest);
    println!("report:   {}", report_path.display());
    println!(
        "summary:  {} passed / {} failed / {} skipped",
        summary.passed, summary.failed, summary.skipped
    );

    if let Some(wall_ms) = summary.command_batch_wall_ms {
        println!("batch:    {wall_ms}ms (representative command batch)");
    }

    if summary.failed > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::harness::run_all_groups;

    #[test]
    fn systemtest_full_suite() {
        lumacore_station::install_catalog().expect("the catalog freezes");
        let summary = run_all_groups(&[]);
        println!(
            "systemtest: {} passed / {} failed / {} skipped",
            summary.passed, summary.failed, summary.skipped
        );
        assert!(
            summary.failed == 0,
            "systemtest: {} test(s) failed -- run `cargo run -p systemtest` for details",
            summary.failed
        );
        assert!(
            summary.passed > 0,
            "systemtest: no tests passed (expected at least 1)"
        );
    }
}
