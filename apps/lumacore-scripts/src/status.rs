//! What the launcher sees of this machine's station, refreshed every 2 s by a
//! background thread (`watch`) so the menu never waits on it. Everything is
//! read locally: processes, the CLI's UDP health ask, two HTTP GETs on
//! 127.0.0.1, TCP connects to the ports, the binaries' times, git HEAD.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub const HTTP_PORT: u16 = 47656;
/// The TCP ports a station listens on (the command port is UDP: `healthy`).
pub const PORTS: [(&str, u16); 4] = [
    ("shell", 47654),
    ("http", HTTP_PORT),
    ("perfetto", 47657),
    ("stream", 47658),
];
pub const BINARIES: [&str; 3] = ["lumacore-server", "lumacore-cli", "lumacore-app"];

#[derive(Default)]
pub struct Status {
    pub checked: bool,
    /// PID and binary of each running lumacore-server.
    pub servers: Vec<(u32, String)>,
    pub apps: Vec<u32>,
    /// Running lumacore-cli processes, and how many of them are Meownitors.
    pub clis: usize,
    pub meownitors: usize,
    /// Answers `lumacore-cli --health` on 127.0.0.1 (one UDP request; the
    /// server logs nothing for it).
    pub healthy: bool,
    /// `GET /api/station`, when it answers.
    pub station: Option<Station>,
    pub web: Web,
    /// Each of `PORTS`: something accepts a connection on it.
    pub ports: [bool; 4],
    /// Build time of each of `BINARIES`, local (the scripts' default profile) then debug.
    pub binaries: [[Option<SystemTime>; 3]; 2],
    /// `git rev-parse --short=12 HEAD`.
    pub head: Option<String>,
}

pub struct Station {
    pub name: String,
    /// `0.1.0/release/linux-x86_64/<sha>[-dirty]`.
    pub build: String,
    pub role: String,
    pub show: Option<String>,
    pub unsaved: bool,
    pub busy: bool,
    pub started_ms: u64,
    pub exe: String,
    pub logged_in: Vec<String>,
}

impl Station {
    /// The commit stamp at the end of the build id.
    pub fn commit(&self) -> &str {
        self.build.rsplit('/').next().unwrap_or("")
    }

    pub fn uptime(&self) -> Option<Duration> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
        now.checked_sub(Duration::from_millis(self.started_ms))
    }

    /// The binary it runs was rebuilt after it started.
    pub fn rebuilt(&self) -> bool {
        std::fs::metadata(&self.exe)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .is_some_and(|t| t.as_millis() as u64 > self.started_ms + 1000)
    }
}

/// What `/` serves.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Web {
    #[default]
    Unknown,
    /// The WebUI (built in, or `--web-root`'s folder; decisions §36 2).
    Served,
    /// Nothing at `/`: a server from before the built-in WebUI.
    Off,
}

pub fn watch(root: PathBuf, status: Arc<Mutex<Status>>) {
    std::thread::spawn(move || {
        loop {
            let fresh = read(&root);
            if let Ok(mut s) = status.lock() {
                *s = fresh;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

fn read(root: &Path) -> Status {
    let procs = processes();
    let named = |name: &'static str| procs.iter().filter(move |p| p.name == name);
    let servers: Vec<(u32, String)> = named("lumacore-server")
        .map(|p| (p.pid, p.exe.clone()))
        .collect();
    let running = !servers.is_empty();
    let mut ports = [false; 4];
    if running {
        for (i, (_, port)) in PORTS.iter().enumerate() {
            ports[i] = tcp_open(*port);
        }
    }
    let station = if ports[1] {
        http_get("/api/station").and_then(|(code, body)| {
            (code == 200)
                .then(|| serde_json::from_str::<Value>(&body).ok())
                .flatten()
                .map(|v| station(&v))
        })
    } else {
        None
    };
    let web = match (&station, http_get("/")) {
        (None, _) | (_, None) => Web::Unknown,
        (Some(_), Some((200, _))) => Web::Served,
        (Some(_), Some(_)) => Web::Off,
    };
    let mut binaries = [[None; 3]; 2];
    for (p, profile) in ["local", "debug"].iter().enumerate() {
        for (b, name) in BINARIES.iter().enumerate() {
            binaries[p][b] = std::fs::metadata(binary(root, profile, name))
                .and_then(|m| m.modified())
                .ok();
        }
    }
    Status {
        checked: true,
        healthy: running && healthy(root),
        apps: named("lumacore-app").map(|p| p.pid).collect(),
        clis: named("lumacore-cli").count(),
        meownitors: named("lumacore-cli").filter(|p| p.meownitor).count(),
        servers,
        station,
        web,
        ports,
        binaries,
        head: git_head(root),
    }
}

pub fn binary(root: &Path, profile: &str, name: &str) -> PathBuf {
    let exe = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    root.join("target").join(profile).join(exe)
}

fn station(v: &Value) -> Station {
    let text = |v: &Value| v.as_str().unwrap_or("").to_string();
    Station {
        name: text(&v["name"]),
        build: text(&v["build"]),
        role: text(&v["status"]["role"]),
        show: v["show"].as_str().map(str::to_string),
        unsaved: v["status"]["unsaved"].as_bool().unwrap_or(false),
        busy: v["busy"].as_bool().unwrap_or(false),
        started_ms: v["started_ms"].as_u64().unwrap_or(0),
        exe: text(&v["exe"]),
        logged_in: v["users"]
            .as_array()
            .map(|users| {
                users
                    .iter()
                    .filter(|u| u["logged_in"].as_array().is_some_and(|l| !l.is_empty()))
                    .map(|u| text(&u["name"]))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

struct Proc {
    pid: u32,
    /// The binary's file name, without `.exe`.
    name: String,
    exe: String,
    meownitor: bool,
}

fn processes() -> Vec<Proc> {
    if cfg!(windows) {
        return BINARIES
            .iter()
            .flat_map(|name| {
                let filter = format!("IMAGENAME eq {name}.exe");
                let out = Command::new("tasklist")
                    .args(["/FI", &filter, "/FO", "CSV", "/NH"])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                    .unwrap_or_default();
                // "lumacore-server.exe","1234","Console","1","12,345 K"
                out.lines()
                    .filter_map(|l| l.split(',').nth(1)?.trim_matches('"').parse().ok())
                    .map(|pid| Proc {
                        pid,
                        name: name.to_string(),
                        exe: format!("{name}.exe"),
                        meownitor: false,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    let Ok(out) = Command::new("ps").args(["-A", "-o", "pid=,args="]).output() else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid, args) = line.split_once(' ')?;
            let pid: u32 = pid.parse().ok()?;
            let argv0 = args.split_whitespace().next()?;
            let name = Path::new(argv0).file_name()?.to_str()?;
            if !BINARIES.contains(&name) {
                return None;
            }
            // The real binary when /proc has it (argv0 may be relative).
            let exe = std::fs::read_link(format!("/proc/{pid}/exe"))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| argv0.to_string());
            Some(Proc {
                pid,
                name: name.to_string(),
                exe,
                meownitor: args.contains("--meownitor"),
            })
        })
        .collect()
}

fn healthy(root: &Path) -> bool {
    ["local", "release", "debug"]
        .iter()
        .map(|p| binary(root, p, "lumacore-cli"))
        .find(|cli| cli.exists())
        .is_some_and(|cli| {
            Command::new(cli)
                .args(["--target", "127.0.0.1", "--health"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        })
}

fn tcp_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

/// A plain HTTP/1.0 GET on 127.0.0.1 (no chunking, the server closes);
/// status code and body.
fn http_get(path: &str) -> Option<(u16, String)> {
    let addr = SocketAddr::from(([127, 0, 0, 1], HTTP_PORT));
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(300)).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(1))).ok()?;
    s.set_write_timeout(Some(Duration::from_secs(1))).ok()?;
    write!(s, "GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").ok()?;
    let mut raw = Vec::new();
    s.take(8 << 20).read_to_end(&mut raw).ok()?;
    let text = String::from_utf8_lossy(&raw);
    let (head, body) = text.split_once("\r\n\r\n")?;
    let code = head.split_whitespace().nth(1)?.parse().ok()?;
    Some((code, body.to_string()))
}

fn git_head(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// "12s", "5m", "3h", "2d".
pub fn age(d: Duration) -> String {
    let s = d.as_secs();
    match s {
        0..60 => format!("{s}s"),
        60..3600 => format!("{}m", s / 60),
        3600..86400 => format!("{}h", s / 3600),
        _ => format!("{}d", s / 86400),
    }
}
