// ███████╗██████╗  ██████╗ ███╗   ██╗████████╗███████╗███╗   ██╗██████╗
// ██╔════╝██╔══██╗██╔═══██╗████╗  ██║╚══██╔══╝██╔════╝████╗  ██║██╔══██╗
// █████╗  ██████╔╝██║   ██║██╔██╗ ██║   ██║   █████╗  ██╔██╗ ██║██║  ██║
// ██╔══╝  ██╔══██╗██║   ██║██║╚██╗██║   ██║   ██╔══╝  ██║╚██╗██║██║  ██║
// ██║     ██║  ██║╚██████╔╝██║ ╚████║   ██║   ███████╗██║ ╚████║██████╔╝
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚══════╝╚═╝  ╚═══╝╚═════╝
// FILE: frontend
//! The UI kit is the only way to build the page (decisions §30): colours, icons, gestures, asks,
//! text selection, sockets, font sizes, tables and edit actions each have one home in `kit/`.
//!
//! Owns:          the frontend rules and their scanners
//! Doesn't own:   the kit itself (crates/ui/lumacore-webui/web/kit), the router side (lumacore-http)
//! Entry points:  the #[test] rules below
//! Decisions:     §30 "12b", "12b2", "12c2", "12d", "12e"

use std::path::PathBuf;

use crate::files::workspace;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// The theme module: the only frontend files that may hold raw colours.
const THEME_MODULE: &[&str] = &["tokens.css", "theme.js"];

/// The kit files that are the grid / slab themselves (and their Showroom).
const GRID_FILES: &[&str] = &["grid.js", "showroom12d.js"];

const SLAB_FILES: &[&str] = &["slab.js", "setup.js", "showroom12d.js", "showroom.js", "context.js"];

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

// *************************************************************************************************
// Files and lines
// *************************************************************************************************

/// The frontend files the kit rules cover (vendored libraries excluded).
fn frontend_files() -> Vec<PathBuf> {
    let web = workspace().join("crates/ui/lumacore-webui/web");
    let mut out = vec![web.join("index.html"), web.join("connect.html")];
    for entry in std::fs::read_dir(web.join("kit")).expect("read web/kit").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "js" || e == "css" || e == "html") {
            out.push(path);
        }
    }
    out
}

/// A line that is only a comment (`//`, `/*`, `*`, `<!--`).
fn comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with("<!--")
}

/// A `tidy: <tag> (<reason>)` escape on this line or the one before.
fn escaped(lines: &[&str], n: usize, tag: &str) -> bool {
    let mark = format!("tidy: {tag} (");
    lines[n].contains(&mark) || n > 0 && lines[n - 1].contains(&mark)
}

// *************************************************************************************************
// Scanners
// *************************************************************************************************

fn hand_gesture(line: &str) -> Option<&'static str> {
    if line.contains("draggable") || line.contains("dragstart") || line.contains("ondrop") {
        return Some("HTML5 drag");
    }
    let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    if (compact.contains("window.addEventListener('pointermove'") || compact.contains("document.addEventListener('pointermove'"))
        || compact.contains("window.addEventListener(\"pointermove\"")
    {
        return Some("a window-level pointermove (a hand-made drag)");
    }
    if line.contains("showRing(") {
        return Some("a hand-made hold (its own ring and timer)");
    }
    None
}

/// Raw colours in `line`: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(`,
/// `rgba(`, `hsl(`, `hsla(`. Selectors like `#ctx` or `#add-x` don't count
/// (a hex run followed by a word character or `-` is a name).
fn raw_colours(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let b = line.as_bytes();
    for (i, _) in line.match_indices('#') {
        let run = b[i + 1..].iter().take_while(|c| c.is_ascii_hexdigit()).count();
        let next = b.get(i + 1 + run).copied().unwrap_or(b' ');
        let name_goes_on = next.is_ascii_alphanumeric() || next == b'-' || next == b'_';
        if matches!(run, 3 | 4 | 6 | 8) && !name_goes_on {
            found.push(line[i..i + 1 + run].to_string());
        }
    }
    for f in ["rgb(", "rgba(", "hsl(", "hsla("] {
        let mut from = 0;
        while let Some(at) = line[from..].find(f) {
            let at = from + at;
            let before = line[..at].chars().last().unwrap_or(' ');
            if !before.is_ascii_alphanumeric() {
                found.push(f.to_string());
            }
            from = at + f.len();
        }
    }
    found
}

/// Emoji and symbol glyphs used as icons: Misc Technical, Geometric Shapes,
/// Misc Symbols, Dingbats, Misc Symbols and Arrows, and the emoji planes.
fn icon_glyph(c: char) -> bool {
    matches!(c as u32, 0x2300..=0x23FF | 0x25A0..=0x27BF | 0x2B00..=0x2BFF | 0x1F000..=0x1FAFF)
}

/// Raw pixel font sizes in a CSS `line` (decisions §30 "12c2"): a
/// `font-size:` or `font:` whose value names a number of px. Token
/// definitions (`--lc-…: 14px`) are where sizes live and don't count.
fn raw_font_px(line: &str) -> Option<String> {
    let t = line.trim_start();
    if t.starts_with("--") {
        return None;
    }
    for key in ["font-size:", "font:"] {
        for (i, _) in line.match_indices(key) {
            // `-font-size:` would be a custom property (`--x-font-size:`).
            if line[..i].ends_with('-') {
                continue;
            }
            let value: String = line[i + key.len()..].chars().take_while(|c| *c != ';' && *c != '}' && *c != '"' && *c != '`').collect();
            let b = value.as_bytes();
            for (j, _) in value.match_indices("px") {
                if j > 0 && b[j - 1].is_ascii_digit() {
                    return Some(format!("{key}{}", value.trim_end()));
                }
            }
        }
    }
    None
}

/// Tile-grid tracks that don't fit every box (decisions §30 "12c2"): a fixed
/// column count (`repeat(4, …)`) or a raw px minimum in an auto-fill /
/// auto-fit grid (`minmax(150px, …)`). Tile grids take their minimum from a
/// token (`--lc-tile-min`, `--lc-cell-min`, `--lc-chip-min`, `--lc-pool-cell`).
fn raw_grid_tracks(line: &str) -> Option<String> {
    let at = line.find("grid-template-columns:")?;
    let value: String = line[at..].chars().take_while(|c| *c != ';' && *c != '}').collect();
    let compact: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    for (i, _) in compact.match_indices("repeat(") {
        let rest = &compact[i + 7..];
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return Some(value);
        }
        if (rest.starts_with("auto-fill,minmax(") || rest.starts_with("auto-fit,minmax("))
            && rest.split_once("minmax(").is_some_and(|(_, m)| m.starts_with(|c: char| c.is_ascii_digit()))
        {
            return Some(value);
        }
    }
    None
}

/// Table markup: `<table`, or hand-made grid rows (`role="grid"` / `role="row"`).
fn table_markup(line: &str) -> Option<&'static str> {
    ["<table", "role=\"grid\"", "role=\"row\"", "role='grid'", "role='row'"]
        .into_iter()
        .find(|m| line.contains(m))
}

/// The slab's own words as a hand-made button or action-row item:
/// `>Delete</button>`, `label: 'New user…'`.
fn slab_action(line: &str) -> Option<String> {
    const WORDS: &[&str] = &["New", "Copy", "Cut", "Paste", "Delete", "Import", "Export"];
    // tidy: not-keywords (the slab's action labels, not command words)
    for w in WORDS {
        for (pre, post) in [(">", "</button>"), ("label: '", "'"), ("label: \"", "\"")] {
            let start = format!("{pre}{w}");
            for (i, _) in line.match_indices(&start) {
                let rest = &line[i + start.len()..];
                let Some(end) = rest.find(post) else { continue };
                let tail = &rest[..end];
                // `New`, `Delete…`, `New user…`, `Delete window` — not `Newer` or a sentence.
                if tail.chars().next().is_some_and(|c| c.is_ascii_alphanumeric()) || tail.chars().count() > 16 {
                    continue;
                }
                return Some(format!("{w}{tail}"));
            }
        }
    }
    None
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

// *************************************************************************************************
// Look
// Colours, icons, type scale (§30 "12c2").
// *************************************************************************************************

#[test]
fn frontend_colours_only_in_the_theme() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if THEME_MODULE.contains(&name.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if comment_line(line) {
                continue;
            }
            for c in raw_colours(line) {
                failures.push(format!("{}:{}: raw colour {c} — use a theme token (--lc-*)", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "raw colours outside kit/tokens.css:\n{}", failures.join("\n"));
}

#[test]
fn colour_scanner_tells_colours_from_names() {
    assert_eq!(raw_colours("color: #fff;"), vec!["#fff"]);
    assert_eq!(raw_colours("background: #121212"), vec!["#121212"]);
    assert!(raw_colours("#ctx .ci, #add-window, #bk-desc, #cb-line").is_empty());
    assert_eq!(raw_colours("a: rgba(0,0,0,.5)"), vec!["rgba("]);
    assert!(raw_colours("--lc-ring-ms: 1ms; var(--lc-bg)").is_empty());
}

#[test]
fn frontend_icons_come_from_the_kit() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if comment_line(line) {
                continue;
            }
            if let Some(c) = line.chars().find(|c| icon_glyph(*c)) {
                failures.push(format!("{}:{}: glyph {c:?} — use <luma-icon> (web/vendor/lucide/icons.txt)", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "emoji / symbol glyphs as icons:\n{}", failures.join("\n"));
}

/// One type scale and fitting tile grids (decisions §30 "12c2"): no raw px
/// font sizes outside the token definitions (`--lc-fs*` in tokens.css; text
/// never below 12 px), and no tile grid with a fixed column count or a raw
/// minimum. A deliberate one carries `tidy: px-ok (<reason>)` or
/// `tidy: grid-ok (<reason>)` on its line.
#[test]
fn frontend_font_sizes_are_tokens() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name == "tokens.css" || name == "connect.html" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if comment_line(line) {
                continue;
            }
            if let Some(v) = raw_font_px(line).filter(|_| !line.contains("tidy: px-ok (")) {
                failures.push(format!("{}:{}: {v} — use a --lc-fs* token (kit/tokens.css)", path.display(), n + 1));
            }
            if let Some(v) = raw_grid_tracks(line).filter(|_| !line.contains("tidy: grid-ok (")) {
                failures.push(format!("{}:{}: {v} — tile grids auto-fill from a token minimum", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "raw font sizes / tile grid tracks:\n{}", failures.join("\n"));
}

#[test]
fn font_and_grid_scanners() {
    assert!(raw_font_px("a { font-size: 13px; }").is_some());
    assert!(raw_font_px("b { font: 600 15px/1 var(--lc-font); }").is_some());
    assert!(raw_font_px("c { font-size: var(--lc-fs-small); }").is_none());
    assert!(raw_font_px("  --lc-fs: 14px;").is_none());
    assert!(raw_font_px("d { font: inherit; padding: 4px; }").is_none());
    assert!(raw_grid_tracks(".g { grid-template-columns: repeat(4, 1fr); }").is_some());
    assert!(raw_grid_tracks(".g { grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); }").is_some());
    assert!(raw_grid_tracks(".g { grid-template-columns: repeat(auto-fill, minmax(var(--lc-tile-min), 1fr)); }").is_none());
    assert!(raw_grid_tracks(".g { grid-template-columns: 34px 1fr; }").is_none());
    assert!(raw_grid_tracks(".g { grid-template-columns: repeat(var(--n), minmax(0, 1fr)); }").is_none());
}

// *************************************************************************************************
// Gestures and asks
// The kit's hold, drag and dialog (§30 "12b", "12e").
// *************************************************************************************************

/// One hold and one drag, the kit's (decisions §30 "12e hardening"): outside
/// `kit/` no HTML5 drag (`draggable`, `dragstart`, `ondrop` — dead on touch),
/// no window- or document-level `pointermove` listener (a hand-made drag) and
/// no hand-made hold ring (`showRing(` — a hand-made hold timer). The pointer
/// core (`kit/pointer.js`) and the kit's elements do these. One that must stay
/// carries `tidy: hold-ok (<reason>)` on its line or the line before.
#[test]
fn frontend_gestures_only_in_the_kit() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        if path.parent().is_some_and(|p| p.ends_with("kit")) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        let lines: Vec<&str> = text.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            if comment_line(line) || escaped(&lines, n, "hold-ok") {
                continue;
            }
            if let Some(g) = hand_gesture(line) {
                failures.push(format!("{}:{}: {g} — gestures are the kit's (kit/pointer.js: hold, drag)", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "hand-made gestures outside kit/:\n{}", failures.join("\n"));
}

#[test]
fn hand_gesture_scanner() {
    assert_eq!(hand_gesture("row.draggable = true;"), Some("HTML5 drag"));
    assert_eq!(hand_gesture("  window.addEventListener('pointermove', mv);"), Some("a window-level pointermove (a hand-made drag)"));
    assert_eq!(hand_gesture("LumaKit.pointer.showRing(x, y, 500);"), Some("a hand-made hold (its own ring and timer)"));
    assert_eq!(hand_gesture("LumaKit.pointer.hold(btn, {…})"), None);
}

/// Every question goes through the standard dialog (decisions §30 "12b"):
/// no browser dialogs (`alert`, `confirm`, `prompt` block a WebView) and no
/// hand-built ask overlays in the page.
#[test]
fn frontend_asks_only_through_the_kit_dialog() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if comment_line(line) {
                continue;
            }
            for bad in ["alert(", "confirm(", "prompt(", "ask-overlay", "prompt-overlay", "confirm-overlay"] {
                let hit = line.match_indices(bad).any(|(i, _)| {
                    let before = line[..i].chars().last().unwrap_or(' ');
                    !(before.is_ascii_alphanumeric() || before == '_' || before == '.' && !line[..i].ends_with("window."))
                });
                if hit {
                    failures.push(format!("{}:{}: {bad} — use LumaKit.dialog", path.display(), n + 1));
                }
            }
        }
    }
    assert!(failures.is_empty(), "asks outside the kit dialog:\n{}", failures.join("\n"));
}

/// Text selection is one kit rule (decisions §30 "12b2": only where you read
/// or type): `user-select` / `touch-callout` only in kit.css; elsewhere mark
/// selectable text with `.lc-selectable`.
#[test]
fn text_selection_only_in_the_kit_rule() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        if path.file_name().is_some_and(|n| n == "kit.css") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if !comment_line(line) && (line.contains("user-select") || line.contains("touch-callout")) {
                failures.push(format!("{}:{}", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "user-select outside kit.css (use .lc-selectable):\n{}", failures.join("\n"));
}

// *************************************************************************************************
// Feeds
// Everything live on /ws/feeds (§30 "12b2", "12d").
// *************************************************************************************************

/// One feed rule (decisions §30 "12b2 hardening"): a page opens two sockets
/// only — `/ws/feeds` (every live feed, as subscriptions on the kit's
/// FeedClient: one batch when a window opens, then batches per tick) and
/// `/ws/shell`. A new per-feed socket fails here (the router side is held
/// by lumacore-http's `the_router_has_only_the_feeds_and_shell_sockets_and_no_sse`).
#[test]
fn frontend_feeds_ride_one_socket() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        for (n, line) in text.lines().enumerate() {
            if comment_line(line) {
                continue;
            }
            for (i, _) in line.match_indices("/ws/") {
                let socket: String = line[i + 4..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
                if socket != "feeds" && socket != "shell" {
                    failures.push(format!("{}:{}: /ws/{socket} — subscribe a feed on /ws/feeds (kit/feeds.js)", path.display(), n + 1));
                }
            }
            // Only the FeedClient and the shell open a WebSocket.
            if line.contains("new WebSocket(") && name != "feeds.js" && !line.contains("/ws/shell") {
                failures.push(format!("{}:{}: a WebSocket outside kit/feeds.js and the shell", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "sockets besides /ws/feeds and /ws/shell:\n{}", failures.join("\n"));
}

/// Everything live rides /ws/feeds (decisions §30 "12d hardening"): no SSE
/// stream and no timer that fetches — fetch stays for one-shot loads. A
/// timer that only counts down to one fetch says `// tidy: not-a-poll (…)`.
#[test]
fn frontend_has_no_event_stream_or_fetch_poll() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        let lines: Vec<&str> = text.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            if comment_line(line) {
                continue;
            }
            if line.contains("EventSource") || line.contains("/api/events") || line.contains("/api/station/events") {
                failures.push(format!("{}:{}: an event stream — subscribe a feed on /ws/feeds", path.display(), n + 1));
            }
            // A timer whose body (this line and the next few) fetches, or
            // calls an async function of this file that does.
            let marked = line.contains("tidy: not-a-poll") || n > 0 && lines[n - 1].contains("tidy: not-a-poll");
            if line.contains("setInterval(") && !marked {
                let body = lines[n..(n + 6).min(lines.len())].join(" ");
                let calls = body
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '('))
                    .filter_map(|w| w.split('(').next().filter(|name| !name.is_empty() && w.contains('(')))
                    .filter(|name| *name != "setInterval");
                let mut fetches = body.contains("fetch(");
                for name in calls {
                    if let Some(f) = text.split(&format!("async function {name}(")).nth(1) {
                        fetches |= f.split("\n}").next().is_some_and(|b| b.contains("fetch("));
                    }
                }
                if fetches {
                    failures.push(format!("{}:{}: a timer that fetches — a feed pushes instead", path.display(), n + 1));
                }
            }
        }
    }
    assert!(failures.is_empty(), "polls / event streams in the frontend:\n{}", failures.join("\n"));
}

// *************************************************************************************************
// Grid and slab
// One list, one action block (§30 "12d").
// *************************************************************************************************

/// One grid for the whole system (decisions §30 "12d"): no table markup
/// outside `<luma-grid>` (kit/grid.js). A table not rebuilt yet carries
/// `tidy: table-ok (<reason>)` on its line or the line before.
#[test]
fn frontend_tables_only_in_the_grid() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if GRID_FILES.contains(&name.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        let lines: Vec<&str> = text.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            if comment_line(line) || escaped(&lines, n, "table-ok") {
                continue;
            }
            if let Some(m) = table_markup(line) {
                failures.push(format!("{}:{}: {m} — lists are a <luma-grid> (kit/grid.js)", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "table markup outside <luma-grid>:\n{}", failures.join("\n"));
}

/// The slab is the kit's one action block (decisions §30 "12d"; the one
/// context menu, kit/context.js, is the same on a selection, §34 round 3, 18): no
/// hand-made New / Copy / Cut / Paste / Delete / Import / Export buttons or
/// action-row items outside `<luma-slab>` (kit/slab.js). One not moved yet
/// carries `tidy: slab-ok (<reason>)` on its line or the line before.
#[test]
fn frontend_actions_only_in_the_slab() {
    let mut failures = Vec::new();
    for path in frontend_files() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if SLAB_FILES.contains(&name.as_str()) || name == "connect.html" {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read frontend file");
        let lines: Vec<&str> = text.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            if comment_line(line) || escaped(&lines, n, "slab-ok") {
                continue;
            }
            if let Some(a) = slab_action(line) {
                failures.push(format!("{}:{}: '{a}' — edit actions live in the slab (<luma-slab>, kit/slab.js)", path.display(), n + 1));
            }
        }
    }
    assert!(failures.is_empty(), "hand-made edit actions outside <luma-slab>:\n{}", failures.join("\n"));
}

#[test]
fn grid_and_slab_scanners() {
    assert_eq!(table_markup("x = `<table class=\"rt\">`"), Some("<table"));
    assert_eq!(table_markup("<div role=\"row\">"), Some("role=\"row\""));
    assert!(table_markup("<luma-grid></luma-grid>").is_none());
    assert_eq!(slab_action("<button class=\"x\">Delete</button>").as_deref(), Some("Delete"));
    assert_eq!(slab_action("{ label: 'New user…', line: 'Store User' }").as_deref(), Some("New user…"));
    assert_eq!(slab_action("{ label: 'Delete window', line }").as_deref(), Some("Delete window"));
    assert!(slab_action("{ label: 'Newest first' }").is_none());
    assert!(slab_action("<button>Copy the long sentence that is not a button</button>").is_none());
    assert!(slab_action("{ label: 'Load show…' }").is_none());
}
