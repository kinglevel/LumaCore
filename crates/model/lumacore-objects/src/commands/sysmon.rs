//! Sysmon notifications from commands: one `Cmd` line per command a user
//! runs (System Monitor: feedback on user commands) and tree change
//! detection.

use crate::AppState;

/// Counts of the show's objects around a line (a sysmon line names what came and went). Counts
/// only: whether the show changed at all is the change path's answer (`change::finish`), never
/// a second walk of the tree — on a rig a walk costs more than the line (decisions §40 "40f
/// built"). A read-only line has no change path and is reported by its counts alone.
pub struct TreeCounts {
    fixtures: usize,
    universes: usize,
    outputs: usize,
    groups: usize,
    sequences: usize,
    executors: usize,
    macros: usize,
    user_profiles: usize,
    fixture_types: usize,
    effects: usize,
    claims: usize,
    datapools: usize,
}

/// The subtrees whose modification the sysmon line names (a selection, a programmer, a
/// master's move or Usage never did; the gesture channel stays quiet).
pub fn reported(subtree: &lumacore_tree::ShowSubtree) -> bool {
    use lumacore_tree::ShowSubtree as S;
    matches!(subtree, S::Devices | S::Playback | S::DataPool(_) | S::Protocols | S::Profile(_) | S::ShowSettings)
}

pub fn tree_snapshot(state: &AppState) -> TreeCounts {
    let sd = &state.root.show_data;
    TreeCounts {
        fixtures: sd.devices.dmx.patch.fixtures.len(),
        universes: sd.devices.dmx.patch.universes.len(),
        outputs: sd.protocols.all_lines().count(),
        groups: sd.datapools.values().map(|dp| dp.groups.len()).sum(),
        sequences: sd.sequences().count(),
        executors: sd.playback.executors.len(),
        macros: sd.datapools.values().map(|dp| dp.macros.len()).sum(),
        user_profiles: sd.user_profiles.len(),
        fixture_types: sd.devices.dmx.patch.fixture_types.len(),
        effects: sd.playback.effects.len(),
        claims: state.root.runtime.claims.len(),
        datapools: sd.datapools.len(),
    }
}

/// Plain text of command output for sysmon.
pub fn strip_ansi_for_sysmon(s: &str) -> String {
    crate::text::strip_ansi(s).trim().to_string()
}

pub fn report_tree_changes(
    state: &mut AppState,
    pre: &TreeCounts,
    post: &TreeCounts,
    cmd: &str,
    output: &str,
    modified: bool,
) {
    let masked = crate::passwords::mask_secrets(cmd);
    let cmd_trimmed = masked.trim();
    if cmd_trimmed.is_empty() {
        return;
    }

    let user = state.effective_user_name().to_string();

    // Build a list of specific count deltas for informative output.
    let mut changes = Vec::new();

    fn delta(name: &str, before: usize, after: usize, changes: &mut Vec<String>) {
        if after > before {
            changes.push(format!("{} +{}", name, after - before));
        } else if after < before {
            changes.push(format!("{} -{}", name, before - after));
        }
    }

    delta("Fixtures", pre.fixtures, post.fixtures, &mut changes);
    delta("Universes", pre.universes, post.universes, &mut changes);
    delta("Outputs", pre.outputs, post.outputs, &mut changes);
    delta("Groups", pre.groups, post.groups, &mut changes);
    delta("Sequences", pre.sequences, post.sequences, &mut changes);
    delta("Executors", pre.executors, post.executors, &mut changes);
    delta("Macros", pre.macros, post.macros, &mut changes);
    delta("Effects", pre.effects, post.effects, &mut changes);
    delta(
        "UserProfiles",
        pre.user_profiles,
        post.user_profiles,
        &mut changes,
    );
    delta(
        "FixtureTypes",
        pre.fixture_types,
        post.fixture_types,
        &mut changes,
    );
    delta("Claims", pre.claims, post.claims, &mut changes);
    delta("DataPools", pre.datapools, post.datapools, &mut changes);

    // Extract the first non-empty line of command output (stripped of ANSI) as detail.
    let detail = strip_ansi_for_sysmon(output)
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string());

    if changes.is_empty() {
        // No count moved: a modification the change path saw, or nothing.
        if !modified {
            return;
        }
        if let Some(detail) = detail {
            state.sysmon_pending.push((
                "Show".to_string(),
                format!("Tree modified by '{user}': {detail}"),
            ));
        } else {
            state.sysmon_pending.push((
                "Show".to_string(),
                format!("Tree modified by '{user}': {cmd_trimmed}"),
            ));
        }
    } else {
        let change_summary = changes.join(", ");
        if let Some(detail) = detail {
            state.sysmon_pending.push((
                "Show".to_string(),
                format!("Tree changed by '{user}': {detail} ({change_summary})"),
            ));
        } else {
            state.sysmon_pending.push((
                "Show".to_string(),
                format!("Tree changed by '{user}': {cmd_trimmed} ({change_summary})"),
            ));
        }
    }
}

/// One `Cmd` line for a command a user ran (typed, from a UI dialog, or over
/// UDP): `[LD@Screen 1] Store Group 5 → ok` or, as an error line,
/// `[LD] Store Group 99 → No group 99`. Passwords are masked; long output is
/// not copied (the first line of an error only). `via` replaces the screen
/// (e.g. "udp").
pub fn log_command(
    state: &mut AppState,
    cmd: &str,
    result: &crate::DispatchResult,
    via: Option<&str>,
) {
    use crate::DispatchResult;
    // `Self` as resolved where the line ran (decisions §30 "12c").
    let resolved = state.resolved_line.take();
    let cmd = crate::passwords::mask_secrets(resolved.as_deref().unwrap_or(cmd).trim());
    if cmd.is_empty() {
        return;
    }
    // Echo writes its own line.
    if cmd
        .split_whitespace()
        .next()
        .is_some_and(|v| v.eq_ignore_ascii_case("echo"))
    {
        return;
    }
    let user = state.effective_user_name().to_string();
    let place = match via {
        Some(v) => Some(v.to_string()),
        None => state.current_live_screen.and_then(|vs| {
            let store = state.root.runtime.live_screens.read().ok()?;
            store.get(vs).map(|s| format!("Screen {}", s.screen))
        }),
    };
    let who = match place {
        Some(p) => format!("{user}@{p}"),
        None => user,
    };
    let (error, outcome) = match result {
        // The outcome from the reply (decisions §31 "typed replies"); the message for the log
        // from its first error line.
        DispatchResult::Output(out) if out.failed => {
            let plain = strip_ansi_for_sysmon(out);
            let line = plain
                .lines()
                .find(|l| l.trim_start().starts_with("Error:"))
                .or_else(|| plain.lines().find(|l| !l.trim().is_empty()))
                .unwrap_or_default();
            (true, line.trim().trim_start_matches("Error:").trim().chars().take(160).collect())
        }
        DispatchResult::Output(_) => (false, "ok".to_string()),
        DispatchResult::Exit => (false, "shutdown".to_string()),
        DispatchResult::Logout => (false, "logged out".to_string()),
        DispatchResult::ContextSwitch(_) => (false, "ok".to_string()),
        DispatchResult::Ask(ask) => (false, format!("asks: {}", ask.title())),
    };
    let level = if error { "[ERROR] " } else { "" };
    state.sysmon_pending.push((
        "Cmd".to_string(),
        format!("{level}[{who}] {cmd} → {outcome}"),
    ));
}
