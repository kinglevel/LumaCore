// ██╗    ██╗███████╗██████╗ ██╗   ██╗██╗
// ██║    ██║██╔════╝██╔══██╗██║   ██║██║
// ██║ █╗ ██║█████╗  ██████╔╝██║   ██║██║
// ██║███╗██║██╔══╝  ██╔══██╗██║   ██║██║
// ╚███╔███╔╝███████╗██████╔╝╚██████╔╝██║
//  ╚══╝╚══╝ ╚══════╝╚═════╝  ╚═════╝ ╚═╝
// FILE: webui

//! The WebUI (decisions §36): every file the station's page (`web/index.html`) and the app's
//! connect page (`web/connect.html`) are made of — the kit, its layouts, the vendored libraries —
//! embedded at build time, so a server binary carries its UI and the app shows its connect page
//! before any station is found. `--web-root` serves the `web/` folder from disk instead (live
//! edits without a rebuild); that choice is `lumacore-http`'s.
//!
//! Owns:          the `web/` files, FILES, get.
//! Doesn't own:   serving them (lumacore-http, the app's `lumacore://` handler), what they do.
//! Entry points:  FILES, get, INDEX, CONNECT.
//! Decisions:     §36 1–3, §30 "12a3" (the connect page), §30 (the kit).

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Every file under `web/`: (URL path, bytes, MIME type), sorted by path.
pub const FILES: &[(&str, &[u8], &str)] = include!(concat!(env!("OUT_DIR"), "/files.rs"));

/// The station's page.
pub const INDEX: &str = "/index.html";

/// The app's connect page.
pub const CONNECT: &str = "/connect.html";

/// The file at a URL path (a query or fragment is ignored; `/` is the station's page).
pub fn get(path: &str) -> Option<(&'static [u8], &'static str)> {
    let path = path.split(['?', '#']).next().unwrap_or("/");
    let path = if path == "/" { INDEX } else { path };
    FILES.iter().find(|(p, _, _)| *p == path).map(|(_, b, m)| (*b, *m))
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
