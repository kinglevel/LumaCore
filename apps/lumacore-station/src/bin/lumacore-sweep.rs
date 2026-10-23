//! The overnight command sweep + fuzz run (decisions §25 "Step 10
//! hardening"). Started by `scripts/unix/fuzz_long`.
//!
//!     lumacore-sweep long [--minutes N] [--fresh] [--no-fuzz]
//!     lumacore-sweep replay <file>

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lumacore_objects::demo;
use lumacore_sweep::run;

#[derive(Parser)]
#[command(about = "LumaCore overnight command sweep and fuzz run")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
    /// Report directory.
    #[arg(long, global = true)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run or continue the overnight run.
    Long {
        /// Minutes for the random phase and for each fuzz target.
        #[arg(long, default_value_t = 30)]
        minutes: u64,
        /// Start over even if a run is unfinished.
        #[arg(long)]
        fresh: bool,
        /// Skip the cargo-fuzz targets.
        #[arg(long)]
        no_fuzz: bool,
    },
    /// Replay the lines that led to a finding.
    Replay { file: PathBuf },
}

fn main() {
    lumacore_station::install_catalog().expect("the catalog");
    let cli = Cli::parse();
    let root = demo::repo_root();
    let base = demo::demo_path(&root);
    let dir = cli.dir.unwrap_or_else(|| root.join("target/fuzz-report"));
    let code = match cli.cmd {
        Cmd::Long { minutes, fresh, no_fuzz } => {
            let opts = run::Options {
                dir,
                fresh,
                random_minutes: minutes,
                fuzz_minutes: minutes,
                fuzz_dir: (!no_fuzz).then(|| root.join("tests/fuzz")),
                base,
            };
            // The build id stays the same between uncommitted rebuilds: add the
            // binary's time so a rebuild counts as a new build.
            let built = std::env::current_exe()
                .and_then(std::fs::metadata)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            let build = format!("{}@{built}", lumacore_objects::build_info().build_id);
            match run::long(&opts, &build) {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("lumacore-sweep: {e}");
                    1
                }
            }
        }
        Cmd::Replay { file } => match run::replay(&file, &base) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("lumacore-sweep: {e}");
                1
            }
        },
    };
    std::process::exit(code);
}
