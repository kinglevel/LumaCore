//! The in-process `lumacore` REPL: the catalog, a station state, then the terminal (decisions
//! §34 24.1).

use clap::Parser;
use lumacore_server::app::{self, CommonCli};

#[derive(Parser, Debug)]
#[command(
    name = "lumacore",
    version = "0.1.0",
    about = "Terminal-first DMX lighting console"
)]
struct Cli {
    #[command(flatten)]
    common: CommonCli,

    /// Execute a command and exit (non-interactive mode).
    #[arg(long = "cmd", short = 'C')]
    command: Option<String>,

    /// Show help for a specific command and exit.
    #[arg(long = "CmdHelp", value_name = "COMMAND")]
    cmd_help: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    app::init_logging(&cli.common.log_level);
    log::info!("LumaCore v0.1.0 starting");

    // The assembler installs the catalog before anything reads it (decisions §34 24.1).
    if let Err(err) = lumacore_station::install_catalog() {
        eprintln!("{err}");
        std::process::exit(1);
    }
    let (mut state, _) = match app::build_state(&cli.common) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    if let Some(ref cmd_str) = cli.cmd_help {
        run_command_list(&mut state, cmd_str);
        std::process::exit(0);
    }

    if let Some(ref cmd) = cli.command {
        run_command_list(&mut state, cmd);
        std::process::exit(0);
    }

    let exit_code = lumacore_terminal::repl::run_repl(&mut state);
    std::process::exit(exit_code);
}

fn run_command_list(state: &mut lumacore_objects::AppState, raw: &str) {
    use lumacore_objects::commands::dispatch;
    use lumacore_objects::DispatchResult;

    let commands: Vec<&str> = raw.split(';').map(|s| s.trim()).collect();
    for c in commands {
        if c.is_empty() {
            continue;
        }
        match dispatch(state, c) {
            DispatchResult::Output(output) => {
                if !output.is_empty() {
                    println!("{output}");
                }
            }
            DispatchResult::Exit => break,
            DispatchResult::Logout => break,
            DispatchResult::ContextSwitch(_) => {}
            DispatchResult::Ask(ask) => eprintln!("{}", ask.refusal()),
        }
    }
}
