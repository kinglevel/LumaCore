//! A minimal blocking JSON client for the few calls the app makes before a
//! window shows the server's UI (`/api/station`, `/api/login`, `/api/me`,
//! `/api/shows`). HTTP/1.0, so the server answers with a plain body and closes.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::Value;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// `(status, body)`; Err = not reachable / not a LumaCore answer.
/// The most a reply may be (bytes).
const MAX_REPLY: u64 = 1024 * 1024;

pub fn get(host: &str, port: u16, path: &str, token: Option<&str>) -> Result<(u16, Value), String> {
    request("GET", host, port, path, token, None)
}

fn request(
    method: &str,
    host: &str,
    port: u16,
    path: &str,
    token: Option<&str>,
    body: Option<&Value>,
) -> Result<(u16, Value), String> {
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("{host}: {e}"))?
        .next()
        .ok_or_else(|| format!("{host}: no address"))?;
    let mut stream = TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT)
        .map_err(|e| format!("{host}:{port}: {e}"))?;
    stream.set_read_timeout(Some(IO_TIMEOUT)).ok();
    stream.set_write_timeout(Some(IO_TIMEOUT)).ok();

    let body = body.map(|b| b.to_string()).unwrap_or_default();
    let host_header = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let mut req =
        format!("{method} {path} HTTP/1.0\r\nHost: {host_header}\r\nAccept: application/json\r\n");
    if let Some(token) = token {
        req.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    if method != "GET" {
        req.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    req.push_str("\r\n");
    req.push_str(&body);
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;

    // A station's replies are small; a peer that sends more is cut off (decisions §31).
    let mut raw = Vec::new();
    stream.take(MAX_REPLY).read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&raw);
    let (head, body) = text.split_once("\r\n\r\n").ok_or("malformed HTTP reply")?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or("malformed HTTP status")?;
    let value = serde_json::from_str(body)
        .map_err(|_| format!("{host}:{port} is not a LumaCore server"))?;
    Ok((status, value))
}
