// ███████╗██╗   ██╗███████╗███████╗██╗███╗   ██╗ ██████╗
// ██╔════╝██║   ██║╚══███╔╝╚══███╔╝██║████╗  ██║██╔════╝
// █████╗  ██║   ██║  ███╔╝   ███╔╝ ██║██╔██╗ ██║██║  ███╗
// ██╔══╝  ██║   ██║ ███╔╝   ███╔╝  ██║██║╚██╗██║██║   ██║
// ██║     ╚██████╔╝███████╗███████╗██║██║ ╚████║╚██████╔╝
// ╚═╝      ╚═════╝ ╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝
// FILE: fuzzing

//! Entry points for the cargo-fuzz targets: the decoders and the command line, each taking any
//! bytes. The rule every one checks: no panic and no hang, whatever comes in; a codec that also
//! encodes gives back what it decoded. Not part of the sweep's API.
//!
//! Owns:          the entry points the cargo-fuzz targets in tests/fuzz/ call, and their catalog.
//! Doesn't own:   the targets themselves (tests/fuzz/fuzz_targets/), the decoders (their crates).
//! Entry points:  osc, stream_frame, command_line, token, commands.
//! Decisions:     §25 "Step 10 hardening", §34 20.10.

use std::sync::{Arc, OnceLock};

use lumacore_objects::{AppState, DispatchResult};
use lumacore_luma::drive;

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// An OSC packet on an input line.
pub fn osc(data: &[u8]) {
    let _ = lumacore_osc::parse_osc_packet(data);
}

/// A LumaSession stream frame (u32 length + zstd + JSON `StreamMsg`);
/// a frame that decodes encodes back to the same message.
pub fn stream_frame(data: &[u8]) {
    use lumacore_lumasession::stream::{read_frame, write_frame};
    let Ok(msg) = read_frame(&mut std::io::Cursor::new(data)) else {
        return;
    };
    let mut out = Vec::new();
    write_frame(&mut out, &msg).expect("a decoded frame encodes");
    let again = read_frame(&mut std::io::Cursor::new(&out)).expect("an encoded frame decodes");
    assert_eq!(
        serde_json::to_value(&msg).ok(),
        serde_json::to_value(&again).ok(),
        "stream frame round trip"
    );
}

/// A command line: lexer, statements, scope/reach/undo, the selection
/// parser, password masking, then dispatch on a fresh station with the disk
/// shut. A panic inside dispatch is caught there and answered with
/// E-INTERNAL-001 — here it is raised again so the fuzzer sees it.
pub fn command_line(data: &[u8]) {
    let Ok(line) = std::str::from_utf8(data) else {
        return;
    };
    use lumacore_objects::commands::{registry, tokens};
    let _ = tokens::lex(line);
    let _ = tokens::statements(line);
    let words = tokens::words(&tokens::lex_lenient(line));
    let _ = lumacore_objects::parser::parse_selection(&words);
    let _ = registry::line_scope(line);
    let reach = registry::line_reach(line);
    let _ = lumacore_objects::commands::undo_of_line(line);
    let masked = lumacore_objects::passwords::mask_secrets(line);
    assert_eq!(lumacore_objects::passwords::mask_secrets(&masked), masked, "masking is idempotent");
    if reach != registry::Reach::Show || line.len() > 4096 {
        return;
    }
    catalog();
    let mut state = AppState::new();
    drive::set_off(true);
    let out = lumacore_objects::commands::dispatch(&mut state, line);
    drive::set_off(false);
    if let DispatchResult::Output(text) = out
        && text.contains("Internal error in ")
    {
        panic!("{}", lumacore_objects::text::strip_ansi(&text));
    }
}

/// A login token (`v2.<user>.<expiry>.<fingerprint>.<nonce>.<hmac>`).
pub fn token(data: &[u8]) {
    static STATE: OnceLock<std::sync::Mutex<AppState>> = OnceLock::new();
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    catalog();
    let state = STATE.get_or_init(|| std::sync::Mutex::new(AppState::new()));
    let state = state.lock().unwrap_or_else(|e| e.into_inner());
    let _ = lumacore_objects::passwords::verify_token(&state, text);
}

/// Bytes choose each next word among what completion offers, from the demo
/// show; the line runs through the sweep's world and its checks. A panic or
/// a Show line reaching the disk is raised.
pub fn commands(data: &[u8]) {
    static BASE: OnceLock<Arc<String>> = OnceLock::new();
    let base = BASE
        .get_or_init(|| {
            let root = lumacore_objects::demo::repo_root();
            let path = lumacore_objects::demo::ensure(&root).map(|(p, _)| p).unwrap_or_else(|_| lumacore_objects::demo::demo_path(&root));
            Arc::new(std::fs::read(path).ok().and_then(|b| lumacore_luma::show_file::decode(&b).ok()).map(|c| c.json).unwrap_or_default())
        })
        .clone();
    catalog();
    let Ok(mut world) = crate::world::World::new(base) else {
        return;
    };
    let roles = crate::world::ROLES;
    for chunk in data.split(|b| *b == 0xff).take(8) {
        let Some((&role, words)) = chunk.split_first() else {
            continue;
        };
        let mut line = String::new();
        for &b in words.iter().take(8) {
            let probe = if line.is_empty() { String::new() } else { format!("{line} ") };
            let (_, offered) = lumacore_suggest::complete_input(&probe, probe.chars().count(), &world.master);
            let word = match offered.get(b as usize % offered.len().max(1)) {
                Some(w) if b < 0xe0 => w.clone(),
                _ => (b % 32).to_string(),
            };
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word.trim());
        }
        if line.is_empty() || lumacore_objects::commands::registry::line_reach(&line) != lumacore_objects::commands::registry::Reach::Show {
            continue;
        }
        let (outcome, findings) = world.run(roles[role as usize % roles.len()], &line, None);
        if let Some(f) = findings.iter().find(|f| f.check == "panic" || f.check == "drive") {
            panic!("{} — {}\n{}", f.key(), f.detail, world.history.join("\n"));
        }
        if outcome.ended {
            return;
        }
    }
}

/// A fuzz target runs with what the sweep builds on (`crate::register`), installed once.
fn catalog() {
    lumacore_registry::install_with(crate::register);
}
