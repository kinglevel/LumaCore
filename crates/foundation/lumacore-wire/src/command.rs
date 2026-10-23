//! The UDP command port: one JSON request, one JSON reply. Clients use it for
//! Health and to start / stop / command a station; stations for LumaSession
//! requests (Invite, Dismiss, Yield).

use std::net::UdpSocket;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{REMOTE_CMD_PORT, StationKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RemoteRequest {
    Health,
    Login {
        user: String,
    },
    Command {
        user: String,
        cmd: String,
    },
    Shutdown,
    /// A master invites this station into its LumaSession (an invite):
    /// accepted when the station's Invite setting is on; it then joins the
    /// sender's LumaSession stream at `stream_port`.
    Invite {
        lumasession: String,
        from: String,
        stream_port: u16,
    },
    /// The master dismisses this station from its LumaSession.
    Dismiss {
        from: String,
    },
    /// Two masters met and the other one chose Use Mine (§21 "6c"): this
    /// master saves its show, sends its members to `from` and joins it.
    Yield {
        lumasession: String,
        from: String,
        epoch: u64,
        stream_port: u16,
    },
    /// A member's session clock ping to its master (decisions §24 "Session
    /// clock (researched)"): `t1` is the member's session time (µs) when
    /// sent; `report` is its sync, listed on the master. Answered at the
    /// socket with a [`TimeReply`], never through the state thread.
    Time {
        seq: u64,
        t1: u64,
        from: String,
        #[serde(default)]
        report: Option<ClockReport>,
    },
}

/// The answer to [`RemoteRequest::Time`]: the master's session time when
/// the ping was read (`t2`) and when the reply left (`t3`), in µs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeReply {
    pub seq: u64,
    pub t1: u64,
    pub t2: u64,
    pub t3: u64,
    /// The replying station is a LumaSession master.
    pub master: bool,
}

/// What a member says about its session clock (PTP terms).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ClockReport {
    pub state: String,
    pub offset_us: i64,
    pub path_delay_us: u64,
    pub jitter_us: u64,
    pub frequency_ppb: i64,
    pub last_sync_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteReply {
    pub ok: bool,
    pub message: String,
    pub server_pid: u32,
    pub active_user: Option<String>,
    pub show_path: Option<String>,
    pub debug_mode: bool,
    pub build_id: String,
    pub station_name: Option<String>,
    pub session_count: Option<u32>,
    /// Station priority advertised in Health replies.
    #[serde(default)]
    pub station_priority: Option<String>,
    /// The LumaSession the replying station belongs to.
    #[serde(default)]
    pub lumasession: Option<String>,
    /// What the station is and the protocols it speaks (0 = an older station).
    #[serde(default)]
    pub kind: StationKind,
    #[serde(default)]
    pub client_protocol: u32,
    #[serde(default)]
    pub session_protocol: u32,
    /// The command cleared the terminal (`Clear`): a terminal client clears
    /// its screen too — it needs no command knowledge of its own.
    #[serde(default)]
    pub clear_screen: bool,
}

/// Send a request to the command port of `target` and wait for the reply.
pub fn send_request(
    target: &str,
    request: &RemoteRequest,
    timeout: Duration,
) -> Result<RemoteReply, String> {
    send_request_to(target, REMOTE_CMD_PORT, request, timeout)
}

pub fn send_request_to(
    target: &str,
    port: u16,
    request: &RemoteRequest,
    timeout: Duration,
) -> Result<RemoteReply, String> {
    let payload = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    let socket = UdpSocket::bind(("0.0.0.0", 0)).map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    socket
        .send_to(&payload, (target, port))
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 65535];
    let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
    serde_json::from_slice(&buf[..len]).map_err(|e| e.to_string())
}

pub fn try_health(target: &str, timeout: Duration) -> Option<RemoteReply> {
    send_request(target, &RemoteRequest::Health, timeout).ok()
}
