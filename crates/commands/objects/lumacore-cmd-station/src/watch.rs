// ██╗    ██╗ █████╗ ████████╗ ██████╗██╗  ██╗
// ██║    ██║██╔══██╗╚══██╔══╝██╔════╝██║  ██║
// ██║ █╗ ██║███████║   ██║   ██║     ███████║
// ██║███╗██║██╔══██║   ██║   ██║     ██╔══██║
// ╚███╔███╔╝██║  ██║   ██║   ╚██████╗██║  ██║
//  ╚══╝╚══╝ ╚═╝  ╚═╝   ╚═╝    ╚═════╝╚═╝  ╚═╝
// FILE: watch

//! Watch and unwatch commands.
//!
//! Only read-only lines are watched (R1-65): a watch re-runs its line every refresh.
//!
//! Owns:          Watch / Unwatch, `<line> /Watch` (its hook), which lines may be watched, the Watch object.
//! Doesn't own:   re-running the line (the kernel's dispatch), the watches' store (AppState).
//! Entry points:  WATCH, try_watch, handle_watch, handle_unwatch, watchable.
//! Decisions:     §29, R1-65.

use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::commands::dispatch;
use lumacore_objects::commands::handle_list_settings;
use lumacore_objects::commands::help::HelpCategory;
use lumacore_objects::commands::registry::{AliasDepth, CommandClass, CommandFamily, Doc};
use lumacore_objects::objects::{ObjectDef, out};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Whether `line` may be watched (R1-65): only read-only lines — a watch
/// re-runs its line every refresh, so a line that changes anything (an
/// Oops step, a replication op, a line the master runs) is never watched.
pub fn watchable(line: &str) -> bool {
    matches!(
        lumacore_objects::commands::undo_of_line(line),
        lumacore_objects::commands::registry::Undo::ReadOnly
    ) && lumacore_objects::commands::registry::line_scope(line)
        != lumacore_objects::commands::registry::Scope::Master
}

/// The refusal for a line that isn't read-only.
fn not_watchable(line: &str) -> String {
    lumacore_objects::errors::render_code(
        format!("'{line}' changes things — only read-only lines can be watched."),
        "E-CMD-010",
    )
}

/// Check if any token is a /watch flag. If found, register the watch and return a result.
///
/// Supports: `/watch` (default 200ms) and `/watch=500` (custom interval).
/// The command stored is the original line with the /watch flag stripped.
pub fn try_watch(
    state: &mut AppState,
    tokens: &[String],
    _original_line: &str,
) -> Option<DispatchResult> {
    // Don't intercept a line that already calls Watch — handle_watch manages that
    if let Some(first) = tokens.first()
        && let Ok(lumacore_objects::commands::registry::Word::Object(o)) =
            lumacore_objects::commands::registry::resolve(first)
        && std::ptr::eq(o, &crate::watch::WATCH)
    {
        return None;
    }

    let mut interval_ms: Option<u32> = None;
    let mut watch_token_idx: Option<usize> = None;

    if let Some(i) = tokens.iter().position(|t| {
        lumacore_objects::commands::flags::is(t, &lumacore_objects::commands::flags::WATCH)
    }) {
        // `/Watch` alone every 200 ms; `/Watch=500` its own (floor 50 ms); a value that
        // isn't a number isn't a watch.
        match lumacore_objects::commands::flags::value(
            &[tokens[i].as_str()],
            &lumacore_objects::commands::flags::WATCH,
        ) {
            None => interval_ms = Some(200),
            Some(v) => interval_ms = v.parse::<u32>().ok().map(|ms| ms.max(50)),
        }
        watch_token_idx = Some(i);
    }

    let interval_ms = interval_ms?;
    let watch_idx = watch_token_idx?;

    // Build the command without the /watch token
    let cmd_tokens: Vec<&str> = tokens
        .iter()
        .enumerate()
        .filter(|&(i, _)| i != watch_idx)
        .map(|(_, t)| t.as_str())
        .collect();

    if cmd_tokens.is_empty() {
        return Some(DispatchResult::Output(
            (lumacore_objects::errors::render_code(
                "/watch needs a command. Example: clock /watch",
                "E-CMD-006",
            ))
            .into(),
        ));
    }

    let watch_cmd = cmd_tokens.join(" ");

    // Check if this command is already being watched
    if state
        .watches()
        .iter()
        .any(|w| w.command.to_lowercase() == watch_cmd.to_lowercase())
    {
        return Some(DispatchResult::Output(
            (lumacore_objects::errors::render_code(
                format!(
                    "'{}' is already being watched. Use 'unwatch' to stop.",
                    watch_cmd
                ),
                "E-CMD-010",
            ))
            .into(),
        ));
    }

    if !watchable(&watch_cmd) {
        return Some(DispatchResult::Output((not_watchable(&watch_cmd)).into()));
    }

    // Execute the command once to verify it produces output
    let test_line = watch_cmd.clone();
    let test_result = dispatch(state, &test_line);
    match &test_result {
        DispatchResult::Output(s) if !s.is_empty() => { /* good, has output */ }
        DispatchResult::Output(_) => {
            return Some(DispatchResult::Output(
                (lumacore_objects::errors::render_code(
                    format!("'{}' produced no output — nothing to watch.", watch_cmd),
                    "E-CMD-009",
                ))
                .into(),
            ));
        }
        _ => {
            return Some(DispatchResult::Output(
                (lumacore_objects::errors::render_code(
                    "Only commands that produce output can be watched.",
                    "E-CMD-010",
                ))
                .into(),
            ));
        }
    };

    state.watches_mut().push(lumacore_tree::WatchTarget {
        command: watch_cmd.clone(),
        interval_ms,
    });

    Some(DispatchResult::Output(
        (lumacore_objects::text::render_success(&format!(
            "Watching '{}' ({}ms). Use 'unwatch' to stop.",
            watch_cmd, interval_ms
        )))
        .into(),
    ))
}

/// Handle: watch <command> [/watch=ms]
///
/// Standalone watch command. Wraps the /watch flag logic.
/// Example: watch clock, watch "info clock", watch clock /watch=500
pub fn handle_watch(state: &mut AppState, args: &[&str]) -> String {
    if args.is_empty() {
        // Bare `watch` — show active watches
        return handle_list_settings(state);
    }

    // Check if any arg is already a /watch flag — if so, pass through as-is
    // Extract interval from /watch=NNN if present, otherwise default 200
    let interval_ms =
        lumacore_objects::commands::flags::value(args, &lumacore_objects::commands::flags::WATCH)
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(200);

    // Build the command string (without any /watch flags)
    let cmd_parts: Vec<&str> = args
        .iter()
        .copied()
        .filter(|a| {
            !lumacore_objects::commands::flags::is(a, &lumacore_objects::commands::flags::WATCH)
        })
        .collect();

    if cmd_parts.is_empty() {
        return lumacore_objects::errors::render_code(
            "Watch what? Usage: watch clock, watch \"info clock\"",
            "E-CMD-006",
        );
    }

    // Expand watch on objects — `watch fixture` → `list fixture`: the default
    // view of an object is its List (objects publish whether they have one).
    let listable = lumacore_objects::objects::find(cmd_parts[0])
        .is_some_and(|o| o.supports(lumacore_objects::objects::Verb::List));
    let cmd_parts = if listable {
        let mut expanded = vec!["list"];
        expanded.extend_from_slice(&cmd_parts);
        expanded
    } else {
        cmd_parts
    };

    let watch_cmd = cmd_parts.join(" ");

    // Check duplicate
    if state
        .watches()
        .iter()
        .any(|w| w.command.to_lowercase() == watch_cmd.to_lowercase())
    {
        return lumacore_objects::errors::render_code(
            format!(
                "'{}' is already being watched. Use 'unwatch' to stop.",
                watch_cmd
            ),
            "E-CMD-010",
        );
    }

    if !watchable(&watch_cmd) {
        return not_watchable(&watch_cmd);
    }

    // Test the command
    let test_result = dispatch(state, &watch_cmd);
    match &test_result {
        DispatchResult::Output(s) if !s.is_empty() => {}
        DispatchResult::Output(_) => {
            return lumacore_objects::errors::render_code(
                format!("'{}' produced no output — nothing to watch.", watch_cmd),
                "E-CMD-009",
            );
        }
        _ => {
            return lumacore_objects::errors::render_code(
                "Only commands that produce output can be watched.",
                "E-CMD-010",
            );
        }
    }

    state.watches_mut().push(lumacore_tree::WatchTarget {
        command: watch_cmd.clone(),
        interval_ms: interval_ms.max(50),
    });

    lumacore_objects::text::render_success(&format!(
        "Watching '{}' ({}ms). Use 'unwatch' to stop.",
        watch_cmd, interval_ms
    ))
}

/// Handle: unwatch [command|index|all]
pub fn handle_unwatch(state: &mut AppState, args: &[&str]) -> String {
    if state.watches().is_empty() {
        return lumacore_objects::errors::render_code("No active watches.", "E-CMD-009");
    }

    if args.is_empty() || args[0].to_lowercase() == "all" {
        let count = state.watches().len();
        state.watches_mut().clear();
        return lumacore_objects::text::render_success(&format!(
            "Stopped {} watch{}.",
            count,
            if count == 1 { "" } else { "es" }
        ));
    }

    let target = args[0];

    // Try as numeric index (1-based)
    if let Ok(idx) = target.parse::<usize>() {
        if idx >= 1 && idx <= state.watches().len() {
            let removed = state.watches_mut().remove(idx - 1);
            return lumacore_objects::text::render_success(&format!(
                "Stopped watching '{}'.",
                removed.command
            ));
        } else {
            return lumacore_objects::errors::render_code(
                format!(
                    "Watch index {} out of range (1-{}).",
                    idx,
                    state.watches().len()
                ),
                "E-CMD-008",
            );
        }
    }

    // Try as command name match
    let target_lower = target.to_lowercase();
    if let Some(pos) = state
        .watches()
        .iter()
        .position(|w| w.command.to_lowercase() == target_lower)
    {
        let removed = state.watches_mut().remove(pos);
        return lumacore_objects::text::render_success(&format!(
            "Stopped watching '{}'.",
            removed.command
        ));
    }

    lumacore_objects::errors::render_code(
        format!(
            "No watch found for '{}'. Use 'list watch' to see active watches.",
            target
        ),
        "E-CMD-007",
    )
}

/// `watch clock` — calling Watch starts a watch.
pub(crate) fn watch_call(state: &mut AppState, args: &[&str]) -> DispatchResult {
    out(handle_watch(state, args))
}

lumacore_objects::listed_object!(
lum lumacore_objects::objects::LumShape::ids(1),
// `Watch List Fixture` starts a watch until §33 3e makes Watch a verb (decision 29).
default Call,
    station WATCH,
    "Watch",
    ["watches"],
    "Live watches",
    [
        List => |s, a| out(lumacore_objects::commands::list::watches(s, a)),
    ],
    call: watch_call,
    extra: (Doc {
        display: "Watch",
        category: HelpCategory::System,
        also_applies_to: &[],
        syntax: "Watch <command | object> [/watch=<ms>] | List Watch",
        description: "Watch any read-only command or object live with auto-refresh \
                      (a line that changes anything is refused). Default interval is 200ms. \
                      Objects (fixture, group, universe, selection, etc.) auto-expand \
                      to their list view. You can also use the `/watch` flag on any command directly. \
                      `List Watch` shows the running watches.",
        examples: &[
            "watch selection        -- live selection view",
            "watch fixture          -- live fixture list",
            "watch clock            -- watch wall clock (200ms)",
            "watch clock /watch=500 -- 500ms interval",
            "list watch             -- running watches",
        ],
        related: &["unwatch", "list"],
        alias_depth: AliasDepth::Full,
        hidden: false,
    }, Some(CommandFamily::Inspection), lumacore_objects::commands::registry::UI, CommandClass::Session));
