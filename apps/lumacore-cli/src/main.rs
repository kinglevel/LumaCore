use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use lumacore_wire::{RemoteRequest, RemoteShellCode, RemoteShellKey, ShellClientPacket, ShellServerPacket};

#[derive(Parser, Debug)]
#[command(
    name = "lumacore-cli",
    version = "0.1.0",
    about = "Client terminal for lumacore-server"
)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    target: String,

    #[arg(long, default_value_t = lumacore_wire::REMOTE_CMD_PORT)]
    cmd_port: u16,

    #[arg(long, default_value_t = lumacore_wire::REMOTE_SHELL_PORT)]
    shell_port: u16,

    #[arg(long, default_value = "Admin")]
    user: String,

    /// Password for the user (else $LUMACORE_PASSWORD).
    #[arg(long)]
    password: Option<String>,

    #[arg(long = "cmd", short = 'C')]
    command: Option<String>,

    #[arg(long)]
    health: bool,

    #[arg(long)]
    shutdown: bool,

    #[arg(long)]
    meownitor: bool,
}

enum Mode {
    Unconnected,
    Cmdline { target: String, user: String },
}

/// Password sent with every shell hello (`--password` or $LUMACORE_PASSWORD).
static PASSWORD: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

fn main() -> ExitCode {
    let cli = Cli::parse();
    let _ = PASSWORD.set(
        cli.password
            .clone()
            .or_else(|| std::env::var("LUMACORE_PASSWORD").ok()),
    );

    if cli.health {
        return match lumacore_wire::send_request_to(
            &cli.target,
            cli.cmd_port,
            &RemoteRequest::Health,
            Duration::from_millis(500),
        ) {
            Ok(reply) => {
                println!("{}", reply.message);
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::from(1)
            }
        };
    }

    if cli.shutdown {
        return run_one(&cli.target, cli.cmd_port, RemoteRequest::Shutdown);
    }

    if let Some(cmd) = cli.command.as_deref() {
        return run_one(
            &cli.target,
            cli.cmd_port,
            RemoteRequest::Command {
                user: cli.user.clone(),
                cmd: cmd.to_string(),
            },
        );
    }

    if cli.meownitor {
        for _ in 0..20 {
            if lumacore_wire::send_request_to(
                &cli.target,
                cli.cmd_port,
                &RemoteRequest::Health,
                Duration::from_millis(500),
            )
            .is_ok()
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        print_meownitor_welcome(&cli.target);
        return run_monitor_session(&cli.target, cli.shell_port);
    }

    let mut mode = Mode::Unconnected;
    match login_user(&cli.target, cli.cmd_port, &cli.user) {
        Ok(_) => match run_command_line(&cli.target, cli.shell_port, &cli.user) {
            ShellOutcome::Disconnected => {}
            ShellOutcome::Exit(code) => return ExitCode::from(code),
            ShellOutcome::Error(err) => eprintln!("{err}"),
        },
        Err(err) => {
            eprintln!(
                "Failed to connect to server at {}:{} — {}",
                cli.target, cli.cmd_port, err
            );
        }
    }
    interactive_loop(&mut mode)
}

fn run_one(target: &str, cmd_port: u16, request: RemoteRequest) -> ExitCode {
    match lumacore_wire::send_request_to(target, cmd_port, &request, Duration::from_secs(2)) {
        Ok(reply) => {
            if !reply.message.is_empty() {
                println!("{}", reply.message);
            }
            if reply.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run_monitor_session(target: &str, shell_port: u16) -> ExitCode {
    let mut stream = match TcpStream::connect((target, shell_port)) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to connect to monitor at {target}:{shell_port} — {err}");
            return ExitCode::from(1);
        }
    };
    let _ = stream.set_nodelay(true);

    let hello = serde_json::to_vec(&ShellClientPacket::MonitorHello {
        protocol: lumacore_wire::CLIENT_PROTOCOL,
    })
    .unwrap();
    if stream.write_all(&hello).is_err()
        || stream.write_all(b"\n").is_err()
        || stream.flush().is_err()
    {
        eprintln!("Failed to send monitor hello");
        return ExitCode::from(1);
    }

    let _ = stream.set_nonblocking(true);
    let mut read_buf = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                read_buf.extend_from_slice(&chunk[..n]);
                while let Some(pos) = read_buf.iter().position(|b| *b == b'\n') {
                    let line = String::from_utf8_lossy(&read_buf[..pos]).to_string();
                    read_buf.drain(..=pos);
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(packet) = serde_json::from_str::<ShellServerPacket>(&line) {
                        match packet {
                            ShellServerPacket::Frame { ansi, .. } => {
                                print!("{ansi}");
                                let _ = io::stdout().flush();
                            }
                            ShellServerPacket::Error { message } => {
                                eprintln!("{message}");
                                return ExitCode::from(1);
                            }
                            ShellServerPacket::Closed { .. } => break,
                            // Terminal-only client has no panels; focus never leaves the terminal.
                            // Its suggestion list is the station's TUI overlay.
                            ShellServerPacket::Ui { .. }
                            | ShellServerPacket::Status { .. }
                            | ShellServerPacket::Suggestions { .. } => {}
                        }
                    }
                }
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }
    ExitCode::SUCCESS
}

fn interactive_loop(mode: &mut Mode) -> ExitCode {
    loop {
        print!("{}", prompt_for(mode));
        let _ = io::stdout().flush();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            return ExitCode::from(1);
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let upper = line.to_uppercase();

        match mode {
            Mode::Unconnected => {
                if upper.starts_with("EXIT") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let code = parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                    return ExitCode::from(code);
                }
                if upper == "HELP" {
                    print_bootstrap_help();
                    continue;
                }
                if let Some(user) = parse_login_command(line) {
                    let target = "127.0.0.1";
                    match login_user(target, lumacore_wire::REMOTE_CMD_PORT, user) {
                        Ok(_) => match run_command_line(target, lumacore_wire::REMOTE_SHELL_PORT, user) {
                            ShellOutcome::Disconnected => {}
                            ShellOutcome::Exit(code) => return ExitCode::from(code),
                            ShellOutcome::Error(err) => eprintln!("{err}"),
                        },
                        Err(err) => eprintln!("{err}"),
                    }
                } else if let Some(ip) = parse_connect_command(line, "CMDLINE") {
                    let user = "Admin".to_string();
                    match login_user(ip, lumacore_wire::REMOTE_CMD_PORT, &user) {
                        Ok(_) => {
                            println!(
                                "{}",
                                nu_ansi_term::Style::new().dimmed().paint(format!(
                                    "Connected to {} as {} in CMDLINE mode.",
                                    ip, user
                                ))
                            );
                            *mode = Mode::Cmdline {
                                target: ip.to_string(),
                                user,
                            };
                        }
                        Err(err) => eprintln!("{err}"),
                    }
                } else if let Some(ip) = parse_connect_command(line, "MEOWNITOR") {
                    // decisions §29 "11b hardening": planned (the System Monitor
                    // window following other stations is an idea).
                    println!(
                        "MEOWNITOR {ip} isn't built yet (planned). Use `lumacore-cli --meownitor --target {ip}`."
                    );
                } else {
                    println!("Unknown command. Type HELP.");
                }
            }
            Mode::Cmdline { target, user } => {
                if upper == "HELP" {
                    let _ = run_remote_command(target, lumacore_wire::REMOTE_CMD_PORT, user, "help");
                    continue;
                }
                if upper.starts_with("EXIT") || upper == "DISCONNECT" {
                    println!(
                        "{}",
                        nu_ansi_term::Style::new()
                            .dimmed()
                            .paint(format!("Disconnected from {} as {}.", target, user))
                    );
                    *mode = Mode::Unconnected;
                    continue;
                }
                if let Some(ip) = parse_connect_command(line, "CMDLINE") {
                    match login_user(ip, lumacore_wire::REMOTE_CMD_PORT, user) {
                        Ok(_) => {
                            *target = ip.to_string();
                            println!(
                                "{}",
                                nu_ansi_term::Style::new().dimmed().paint(format!(
                                    "Connected to {} as {} in CMDLINE mode.",
                                    target, user
                                ))
                            );
                        }
                        Err(err) => eprintln!("{err}"),
                    }
                    continue;
                }
                if let Some(new_user) = parse_login_command(line) {
                    match login_user(target, lumacore_wire::REMOTE_CMD_PORT, new_user) {
                        Ok(_) => {
                            match run_command_line(target, lumacore_wire::REMOTE_SHELL_PORT, new_user) {
                                ShellOutcome::Disconnected => {}
                                ShellOutcome::Exit(code) => return ExitCode::from(code),
                                ShellOutcome::Error(err) => eprintln!("{err}"),
                            }
                        }
                        Err(err) => eprintln!("{err}"),
                    }
                    continue;
                }
                let _ = run_remote_command(target, lumacore_wire::REMOTE_CMD_PORT, user, line);
            }
        }
    }
}

fn prompt_for(mode: &Mode) -> String {
    match mode {
        Mode::Unconnected => "Unconnected>".to_string(),
        Mode::Cmdline { target, user } => format!("Cmdline[{user}@{target}]>"),
    }
}

fn parse_connect_command<'a>(line: &'a str, verb: &str) -> Option<&'a str> {
    let mut parts = line.split_whitespace();
    let first = parts.next()?;
    if !first.eq_ignore_ascii_case(verb) {
        return None;
    }
    parts.next()
}

fn parse_user_command(line: &str) -> Option<&str> {
    let mut parts = line.split_whitespace();
    let first = parts.next()?;
    if !first.eq_ignore_ascii_case("USER") && !first.eq_ignore_ascii_case("LOGIN") {
        return None;
    }
    parts.next()
}

fn parse_login_command(line: &str) -> Option<&str> {
    parse_user_command(line)
}

fn login_user(target: &str, cmd_port: u16, user: &str) -> Result<String, String> {
    let reply = lumacore_wire::send_request_to(
        target,
        cmd_port,
        &RemoteRequest::Login {
            user: user.to_string(),
        },
        Duration::from_secs(2),
    )?;
    if reply.ok {
        Ok(reply.message)
    } else {
        Err(reply.message)
    }
}

fn run_remote_command(target: &str, cmd_port: u16, user: &str, line: &str) -> ExitCode {
    match lumacore_wire::send_request_to(
        target,
        cmd_port,
        &RemoteRequest::Command {
            user: user.to_string(),
            cmd: line.to_string(),
        },
        Duration::from_secs(2),
    ) {
        Ok(reply) => {
            // The station says when a command cleared the terminal; this
            // client needs no command knowledge of its own.
            if reply.clear_screen {
                clear_screen();
            } else if !reply.message.is_empty() {
                println!("{}", reply.message);
            }
            if reply.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

enum ShellOutcome {
    Disconnected,
    Exit(u8),
    Error(String),
}

/// How long a read waits for the station before the loop checks the keyboard.
const READ_WAIT: Duration = Duration::from_millis(5);
/// A write that can't finish in this time means the station is gone.
const WRITE_WAIT: Duration = Duration::from_secs(5);

/// Puts the terminal back (cursor, main screen, cooked mode) however the shell loop ends,
/// a panic included.
struct RawTerminal;

impl RawTerminal {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let guard = RawTerminal;
        let _ = crossterm::execute!(io::stdout(), EnterAlternateScreen, Hide);
        Ok(guard)
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn run_command_line(target: &str, shell_port: u16, user: &str) -> ShellOutcome {
    let mut stream = match TcpStream::connect((target, shell_port)) {
        Ok(stream) => stream,
        Err(err) => return ShellOutcome::Error(err.to_string()),
    };
    // Blocking writes (a packet is never left half sent) and short read waits, so one
    // thread can watch both the station and the keyboard.
    if let Err(err) = stream
        .set_read_timeout(Some(READ_WAIT))
        .and_then(|()| stream.set_write_timeout(Some(WRITE_WAIT)))
    {
        return ShellOutcome::Error(err.to_string());
    }

    let (width, height) = terminal::size().unwrap_or((120, 40));
    if send_shell_packet(
        &mut stream,
        &ShellClientPacket::Hello {
            user: user.to_string(),
            width,
            height,
            password: PASSWORD.get().cloned().flatten(),
            token: None,
            client_addr: None,
            protocol: lumacore_wire::CLIENT_PROTOCOL,
            // A CLI is not a device (it can't be locked; decisions §30 "12a3").
            device: None,
        },
    )
    .is_err()
    {
        return ShellOutcome::Error("failed to initialize shell session".to_string());
    }

    let _terminal = match RawTerminal::enter() {
        Ok(guard) => guard,
        Err(_) => return ShellOutcome::Error("failed to enable raw mode".to_string()),
    };

    let mut read_buf = Vec::<u8>::new();
    loop {
        if let Err(end) = drain_shell_packets(&mut stream, &mut read_buf) {
            return end;
        }

        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            let packet = match event::read() {
                Ok(Event::Key(key)) if key.kind == event::KeyEventKind::Press => {
                    to_remote_shell_key(key.code, key.modifiers)
                        .map(|key| ShellClientPacket::Key { key })
                }
                Ok(Event::Resize(width, height)) => Some(ShellClientPacket::Resize { width, height }),
                Ok(_) => None,
                Err(err) => return ShellOutcome::Error(err.to_string()),
            };
            if let Some(packet) = packet
                && let Err(err) = send_shell_packet(&mut stream, &packet)
            {
                return lost(&err);
            }
        }
    }
}

/// The station went away without saying goodbye (killed, network gone).
fn lost(err: &io::Error) -> ShellOutcome {
    ShellOutcome::Error(format!("Lost the connection to the station ({err})"))
}

fn drain_shell_packets(stream: &mut TcpStream, read_buf: &mut Vec<u8>) -> Result<(), ShellOutcome> {
    let mut chunk = [0u8; 8192];
    loop {
        match stream.read(&mut chunk) {
            // End of stream without a `Closed` packet: the station is gone.
            Ok(0) => {
                return Err(ShellOutcome::Error(
                    "Lost the connection to the station (it closed the connection)".to_string(),
                ));
            }
            Ok(n) => {
                read_buf.extend_from_slice(&chunk[..n]);
                while let Some(pos) = read_buf.iter().position(|b| *b == b'\n') {
                    let line = String::from_utf8_lossy(&read_buf[..pos]).to_string();
                    read_buf.drain(..=pos);
                    handle_shell_packet_line(&line)?;
                }
            }
            Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {
                return Ok(());
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(lost(&err)),
        }
    }
}

fn handle_shell_packet_line(line: &str) -> Result<(), ShellOutcome> {
    if line.trim().is_empty() {
        return Ok(());
    }
    let packet: ShellServerPacket =
        serde_json::from_str(line).map_err(|e| ShellOutcome::Error(e.to_string()))?;
    match packet {
        ShellServerPacket::Frame { ansi, .. } => {
            print!("{ansi}");
            let _ = io::stdout().flush();
            Ok(())
        }
        // The suggestion list is drawn by the station's TUI in the frames.
        ShellServerPacket::Ui { .. } | ShellServerPacket::Status { .. } | ShellServerPacket::Suggestions { .. } => Ok(()),
        ShellServerPacket::Error { message } => Err(ShellOutcome::Error(message)),
        ShellServerPacket::Closed { exit_code: 0, .. } => Err(ShellOutcome::Disconnected),
        ShellServerPacket::Closed { exit_code, .. } => {
            Err(ShellOutcome::Exit(u8::try_from(exit_code).unwrap_or(1)))
        }
    }
}

fn send_shell_packet(stream: &mut TcpStream, packet: &ShellClientPacket) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(packet)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    bytes.push(b'\n');
    stream.write_all(&bytes)?;
    stream.flush()
}

fn to_remote_shell_key(code: KeyCode, modifiers: KeyModifiers) -> Option<RemoteShellKey> {
    let code = match code {
        KeyCode::Char(ch) => RemoteShellCode::Char(ch),
        KeyCode::Enter => RemoteShellCode::Enter,
        KeyCode::Tab => RemoteShellCode::Tab,
        KeyCode::Backspace => RemoteShellCode::Backspace,
        KeyCode::Delete => RemoteShellCode::Delete,
        KeyCode::Home => RemoteShellCode::Home,
        KeyCode::End => RemoteShellCode::End,
        KeyCode::PageUp => RemoteShellCode::PageUp,
        KeyCode::PageDown => RemoteShellCode::PageDown,
        KeyCode::Up => RemoteShellCode::Up,
        KeyCode::Down => RemoteShellCode::Down,
        KeyCode::Left => RemoteShellCode::Left,
        KeyCode::Right => RemoteShellCode::Right,
        KeyCode::Esc => RemoteShellCode::Esc,
        KeyCode::Insert => RemoteShellCode::Insert,
        KeyCode::F(n) => RemoteShellCode::F(n),
        _ => return None,
    };
    // The wire's modifier bits (Shift 1, Alt 2, Ctrl 4), not crossterm's.
    use lumacore_wire::shell::modifiers as wire;
    let mut bits = 0;
    if modifiers.contains(KeyModifiers::SHIFT) {
        bits |= wire::SHIFT;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        bits |= wire::ALT;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        bits |= wire::CTRL;
    }
    Some(RemoteShellKey {
        code,
        modifiers: bits,
        repeat: None, // plain terminals don't report auto-repeat
    })
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

fn print_bootstrap_help() {
    println!("List of all available commands:");
    println!("Syntax : HELP (Lists all available commands)");
    println!("Syntax : EXIT [code] (Exit the terminal application)");
    println!("Syntax : CMDLINE [ip] (Connect with single-line responses)");
    println!("Syntax : LOGIN [name] (Login as an existing server user and open the session UI)");
}

fn print_meownitor_welcome(target: &str) {
    clear_screen();
    println!("{}", nu_ansi_term::Style::new().bold().paint("MEOWNITOR"));
    println!(
        "{}",
        nu_ansi_term::Style::new().dimmed().paint(format!(
            "Connected to {} — the station's System Monitor.",
            target
        ))
    );
}
