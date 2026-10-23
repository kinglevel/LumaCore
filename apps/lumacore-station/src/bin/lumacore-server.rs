//! Headless LumaCore server. Startup, the state thread and shutdown live in
//! `lumacore_server` (the desktop app starts this binary as its local server).

use clap::{CommandFactory, FromArgMatches, Parser};
use lumacore_server::ServerConfig;
use lumacore_server::app::CommonCli;

/// Debug builds count allocations inside the render tick and the lines'
/// per-frame send (decisions §24): one in the tick is a panic there (held
/// frame, restart), one in a send is logged. Release builds pay nothing.
#[cfg(debug_assertions)]
#[global_allocator]
static ALLOC: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

#[derive(Parser, Debug)]
#[command(name = "lumacore-server", about = "Headless LumaCore server")]
struct Cli {
    #[command(flatten)]
    common: CommonCli,

    #[arg(long = "RootMonitor")]
    root_monitor: bool,
}

fn main() {
    // `--version` prints the build id (with the station's git stamp); the desktop app compares it
    // with the server already running on this machine.
    lumacore_station::stamp();
    let build: &'static str = Box::leak(lumacore_objects::build_info().build_id.into_boxed_str());
    let matches = Cli::command().version(build).get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    let config = ServerConfig {
        root_monitor: cli.root_monitor,
        ..ServerConfig::from(cli.common)
    };
    // The assembler installs the catalog; the server only checks it (decisions §34 24.1).
    if let Err(e) = lumacore_station::install_catalog() {
        eprintln!("lumacore-server: {e}");
        std::process::exit(1);
    }
    lumacore_server::serve(config);
}
