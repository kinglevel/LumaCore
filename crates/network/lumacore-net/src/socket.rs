// ███████╗ ██████╗  ██████╗██╗  ██╗███████╗████████╗
// ██╔════╝██╔═══██╗██╔════╝██║ ██╔╝██╔════╝╚══██╔══╝
// ███████╗██║   ██║██║     █████╔╝ █████╗     ██║
// ╚════██║██║   ██║██║     ██╔═██╗ ██╔══╝     ██║
// ███████║╚██████╔╝╚██████╗██║  ██╗███████╗   ██║
// ╚══════╝ ╚═════╝  ╚═════╝╚═╝  ╚═╝╚══════╝   ╚═╝
// FILE: socket

//! The station's sockets: the UDP command port, the shell listener, the discovery sender, and
//! command-port requests and replies (decisions §23 "Step 8").
//!
//! Owns:          binding the command port, the shell listener and the discovery sender; reading a request and sending a reply on the command port.
//! Doesn't own:   what a request does (the server's handler), the error text (lumacore-state errors).
//! Entry points:  bind_server_socket_at, bind_shell_listener_at, bind_discovery_sender, recv_request, send_reply.
//! Decisions:     §23 "Step 8", §34 20.6, 22.

use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::time::Duration;

use lumacore_wire::{REMOTE_CMD_PORT, REMOTE_SHELL_PORT, RemoteReply, RemoteRequest};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

pub fn bind_server_socket() -> std::io::Result<UdpSocket> {
    bind_server_socket_at(REMOTE_CMD_PORT)
}

pub fn bind_server_socket_at(port: u16) -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind(("0.0.0.0", port))?;
    socket.set_read_timeout(Some(Duration::from_millis(250)))?;
    Ok(socket)
}

pub fn bind_shell_listener() -> std::io::Result<TcpListener> {
    bind_shell_listener_at(REMOTE_SHELL_PORT)
}

pub fn bind_shell_listener_at(port: u16) -> std::io::Result<TcpListener> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub fn bind_discovery_sender() -> std::io::Result<UdpSocket> {
    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_multicast_ttl_v4(1)?;
    Ok(socket)
}

pub fn recv_request(socket: &UdpSocket) -> std::io::Result<(RemoteRequest, SocketAddr)> {
    let mut buf = [0u8; 65535];
    let (len, addr) = socket.recv_from(&mut buf)?;
    let request = serde_json::from_slice(&buf[..len])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok((request, addr))
}

/// The most a UDP reply can carry (IPv4: 65,535 − 8 UDP − 20 IP header bytes).
pub const MAX_REPLY_BYTES: usize = 65_507;

/// Send `reply`; one too large for a packet is replaced by a refusal whose message `too_large`
/// writes from the reply's size (the station renders it with its code, E-WIRE-002) instead of
/// being lost, and a send that fails is logged.
pub fn send_reply(socket: &UdpSocket, addr: SocketAddr, reply: &RemoteReply, too_large: impl FnOnce(usize) -> String) {
    let Ok(mut bytes) = serde_json::to_vec(reply) else { return };
    if bytes.len() > MAX_REPLY_BYTES {
        let refusal = RemoteReply {
            ok: false,
            message: too_large(bytes.len()),
            clear_screen: false,
            ..reply.clone()
        };
        let Ok(small) = serde_json::to_vec(&refusal) else { return };
        bytes = small;
    }
    if let Err(err) = socket.send_to(&bytes, addr) {
        log::warn!("UDP command port: reply to {addr} not sent: {err}");
    }
}
