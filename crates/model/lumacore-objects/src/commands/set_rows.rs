//! One `Set` line for many rows (decisions §30 "12d hardening", multi-edit):
//! `Set Output 1 Thru 3 "Rate" 44`, `Set Interface "en0" + "en1" "Shell" No`. Every
//! grid gesture ends as one such line naming its rows; it runs one `Set` per
//! row inside the one line, so it is one Oops step. A value `21 Thru 30`
//! spreads over the rows; relative values (`+ 10`, `* 2`) apply per row.
//! Rows refused (no such row, no such property) are skipped and named with
//! their codes; when every row is refused the reply is the error.

use crate::AppState;
use crate::DispatchResult;
use crate::errors::render_code;

/// One row word, or a numeric range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowSpec<'a> {
    One(&'a str),
    Range(&'a str, &'a str),
}

fn is_thru(t: &str) -> bool {
    t.eq_ignore_ascii_case("thru")
}

/// The rows at the start of `args` (`1 Thru 3 + 5`, `en0 + en1`) and how
/// many tokens they took. A single row is one spec; nothing = empty.
pub fn split<'a>(args: &[&'a str]) -> (Vec<RowSpec<'a>>, usize) {
    let Some(first) = args.first() else {
        return (Vec::new(), 0);
    };
    let mut specs = vec![RowSpec::One(first)];
    let mut i = 1;
    while i + 1 < args.len() {
        let (sep, next) = (args[i], args[i + 1]);
        if sep == "+" {
            specs.push(RowSpec::One(next));
        } else if is_thru(sep) {
            let Some(RowSpec::One(from)) = specs.pop() else {
                break;
            };
            specs.push(RowSpec::Range(from, next));
        } else {
            break;
        }
        i += 2;
    }
    (specs, i)
}

/// Whether the specs name more than one row (a list or a range).
pub fn is_multi(specs: &[RowSpec]) -> bool {
    specs.len() > 1 || matches!(specs.first(), Some(RowSpec::Range(..)))
}

/// `a Thru b` over `n` rows: the values spread linearly (whole numbers when
/// both ends are). None = not a spread.
pub fn spread(raw: &str, n: usize) -> Option<Vec<String>> {
    let words: Vec<&str> = raw.split_whitespace().collect();
    let [a, t, b] = words.as_slice() else {
        return None;
    };
    if !is_thru(t) {
        return None;
    }
    let (fa, fb) = (a.parse::<f64>().ok()?, b.parse::<f64>().ok()?);
    let whole = a.parse::<i64>().is_ok() && b.parse::<i64>().is_ok();
    Some(
        (0..n)
            .map(|i| {
                let v = if n <= 1 { fa } else { fa + (fb - fa) * i as f64 / (n - 1) as f64 };
                if whole { format!("{}", v.round() as i64) } else { format!("{:.2}", v).trim_end_matches('0').trim_end_matches('.').to_string() }
            })
            .collect(),
    )
}

/// An error reply's message and code (`… [E-SET-001]`), plain: the outcome from the reply
/// (decisions §31 "typed replies"), the message from its first error line.
fn refusal(reply: &crate::Reply) -> Option<(String, String)> {
    if !reply.failed {
        return None;
    }
    let code = reply.code.clone().unwrap_or_else(|| "E-SET-003".to_string());
    let plain = crate::text::strip_ansi(reply);
    let line = plain.lines().find(|l| l.trim_start().starts_with("Error:")).unwrap_or_default();
    let msg = line.trim_start().trim_start_matches("Error:").trim();
    let msg = msg.strip_suffix(&format!("[{code}]")).unwrap_or(msg).trim();
    Some((msg.to_string(), code))
}

/// Run `one(row, value)` for every row (values spread when `raw` is `a Thru
/// b`), and answer once: the rows set, the rows skipped with their codes.
pub fn run_rows(
    state: &mut AppState,
    rows: &[String],
    labels: &[String],
    prop: &str,
    raw: &str,
    one: impl FnMut(&mut AppState, &str, &str) -> DispatchResult,
) -> DispatchResult {
    let values = spread(raw, rows.len()).unwrap_or_else(|| vec![raw.to_string(); rows.len()]);
    run_each(state, rows, labels, prop, &values, raw, one)
}

/// Run `one(row, value)` for every row with its own value, and answer once: the rows set, the
/// rows skipped with their codes. `shown` is the value as typed (a spread `21 Thru 30`).
pub fn run_each(
    state: &mut AppState,
    rows: &[String],
    labels: &[String],
    prop: &str,
    values: &[String],
    shown: &str,
    mut one: impl FnMut(&mut AppState, &str, &str) -> DispatchResult,
) -> DispatchResult {
    let raw = shown;
    let mut done: Vec<String> = Vec::new();
    let mut skipped: Vec<(String, String, String)> = Vec::new();
    for ((row, label), value) in rows.iter().zip(labels).zip(values) {
        match one(state, row, value) {
            DispatchResult::Output(text) => match refusal(&text) {
                Some((msg, code)) => skipped.push((label.clone(), msg, code)),
                None => done.push(label.clone()),
            },
            other => return other,
        }
    }
    let skipped_text = skipped
        .iter()
        .map(|(l, m, c)| format!("{l}: {m} [{c}]"))
        .collect::<Vec<_>>()
        .join("; ");
    if done.is_empty() {
        let code = skipped.first().map_or("E-SET-004".to_string(), |s| s.2.clone());
        let msg = if skipped.is_empty() {
            "Set: no rows to set.".to_string()
        } else {
            format!("Set {prop}: every row refused — {}", skipped.iter().map(|(l, m, _)| format!("{l}: {m}")).collect::<Vec<_>>().join("; "))
        };
        return DispatchResult::Output(crate::Reply::error(msg, &code));
    }
    let shown = if values.iter().all(|v| v == &values[0]) {
        values[0].clone()
    } else {
        raw.to_string()
    };
    let mut text = format!("{}: {prop} = {}", done.join(", "), shown.trim_matches('"'));
    if !skipped.is_empty() {
        text.push_str(&format!(" · skipped {skipped_text}"));
    }
    DispatchResult::Output((crate::text::render_success(&text)).into())
}

/// An object whose delete handler takes one row (decisions §30 "12d"):
/// lines, networks, stations, users, profiles. Dispatch asks once for every
/// Delete (decisions §30 "Merge of bugfixes"); for these its question names
/// the count and the rows, and rows that can't go are refused before it.
pub struct RowDelete {
    /// The object word lines use (`Output`).
    pub keyword: &'static str,
    /// What a row is called in the question, singular (`line`).
    pub noun: &'static str,
    /// The numbers of the rows there are (a range names only those).
    pub numbers: fn(&AppState) -> Vec<u32>,
    /// Why a row can't go, or None.
    pub refused: fn(&AppState, &str) -> Option<String>,
    /// What deleting a row also affects (shown in the question), or None:
    /// "used by Input 3 (OSC)" for a user (decisions §31 "OSC line users").
    pub affects: fn(&AppState, &str) -> Option<String>,
    /// One row is asked by the handler itself, with a richer question
    /// (what goes with a user): dispatch doesn't ask then.
    pub own_ask: bool,
}

/// The rows a Delete names: (rows, refusals), or the reply when it names
/// none / isn't a row list (None = not a row list: the handler says what it
/// takes).
/// The rows that go and the refusals.
type Targets = (Vec<String>, Vec<String>);

fn delete_targets(state: &AppState, rd: &RowDelete, args: &[&str]) -> Option<Result<Targets, String>> {
    let words: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with('/')).collect();
    let keyword = rd.keyword;
    // Numbers: the one range rule (R1-34, Q6) — missing numbers skipped,
    // open ranges (`3 Thru`, `Thru 5`), `+` / `-`, only existing rows visited.
    let numeric = !words.is_empty()
        && words.iter().all(|w| w.parse::<u32>().is_ok() || is_thru(w) || *w == "+" || *w == "-");
    if numeric {
        let have: std::collections::BTreeMap<u32, ()> = (rd.numbers)(state).into_iter().map(|n| (n, ())).collect();
        let rows: Vec<String> = crate::parser::number_list(&words, &have)?.iter().map(u32::to_string).collect();
        // A single number that isn't there is the handler's to name.
        let rows = if rows.is_empty() && words.len() == 1 { vec![words[0].to_string()] } else { rows };
        return Some(refuse_some(state, rd, rows));
    }
    let (specs, used) = split(&words);
    if specs.is_empty() || used < words.len() {
        return None;
    }
    let mut rows: Vec<String> = Vec::new();
    for spec in specs {
        match spec {
            RowSpec::One(w) => rows.push(w.to_string()),
            RowSpec::Range(a, b) => {
                let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) else {
                    return Some(Err(render_code(format!("{keyword} {a} Thru {b}: Thru takes numbers."), "E-CMD-008")));
                };
                // The one range rule: only the rows that exist.
                let have = (rd.numbers)(state);
                rows.extend((a.min(b)..=a.max(b)).filter(|n| have.contains(n)).map(|n| n.to_string()));
            }
        }
    }
    Some(refuse_some(state, rd, rows))
}

/// The rows that can go and the refusals of the others (an empty list is
/// "no such rows").
fn refuse_some(state: &AppState, rd: &RowDelete, mut rows: Vec<String>) -> Result<Targets, String> {
    if rows.is_empty() {
        return Err(render_code(format!("Delete {}: no such rows.", rd.keyword), "E-CMD-007"));
    }
    // Rows that can't go are refused at once, never asked about.
    let mut refusals = Vec::new();
    rows.retain(|r| match (rd.refused)(state, r) {
        Some(why) => {
            refusals.push(why);
            false
        }
        None => true,
    });
    if rows.is_empty() {
        return Err(refusals.join("\n"));
    }
    Ok((rows, refusals))
}

/// A row word as a line names it (names with spaces quoted, quotes kept).
fn row_word(r: &str) -> String {
    if r.starts_with('"') {
        return r.to_string();
    }
    if r.chars().all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c)) { r.to_string() } else { format!("\"{r}\"") }
}

/// The one Delete question for a row object (dispatch's `delete_first`):
/// "Delete 3 lines (Output 1, 4, 7)?", its button runs the rows that can go
/// as one statement with `/NoConfirmation` (one Oops step).
pub fn delete_ask(state: &AppState, rd: &RowDelete, args: &[&str]) -> crate::objects::DeleteAsk {
    use crate::objects::DeleteAsk;
    let (rows, refusals) = match delete_targets(state, rd, args) {
        // Not a row list: the handler answers (usage), nothing is deleted.
        None => return DeleteAsk::Handler,
        Some(Err(reply)) => return DeleteAsk::Refuse(reply),
        Some(Ok(found)) => found,
    };
    if rd.own_ask && rows.len() == 1 && refusals.is_empty() {
        return DeleteAsk::Handler;
    }
    let keyword = rd.keyword;
    let flags: Vec<&str> = args.iter().copied().filter(|a| a.starts_with('/')).collect();
    let extra = if flags.is_empty() { String::new() } else { format!(" {}", flags.join(" ")) };
    let shown: Vec<String> = rows.iter().map(|r| r.trim_matches('"').to_string()).collect();
    let (title, text) = if rows.len() == 1 {
        (format!("Delete {keyword} {}", shown[0]), format!("Delete {keyword} {}?", shown[0]))
    } else {
        (
            format!("Delete {} {keyword}s", rows.len()),
            format!("Delete {} {}s ({keyword} {})?", rows.len(), rd.noun, shown.join(", ")),
        )
    };
    let line = format!(
        "Delete {keyword} {}{extra} /NoConfirmation",
        rows.iter().map(|r| row_word(r)).collect::<Vec<_>>().join(" + ")
    );
    let mut ask = crate::dialogs::Ask::new(title, text);
    if rows.len() > 1 {
        ask = ask.meta("one Oops step", Some(lumacore_wire::Tone::Warn));
    }
    for row in &rows {
        if let Some(affects) = (rd.affects)(state, row) {
            ask = ask.row(format!("{keyword} {}", row.trim_matches('"')), affects);
        }
    }
    if !refusals.is_empty() {
        ask = ask.note(format!("Not deleted: {}", crate::text::strip_ansi(&refusals.join("; ")).replace("Error: ", "")));
    }
    DeleteAsk::Ask(Box::new(
        ask.button("Delete", lumacore_wire::ButtonKind::Danger, line)
            .cancel("Cancel")
            .flags("/NoConfirmation"),
    ))
}

/// `Delete <object> <rows>` for a [`RowDelete`] object: refuses the rows that
/// can't go and deletes the others, one row at a time inside the one line
/// (one Oops step). It never asks: dispatch asked already (or the line came
/// from a stored key / macro line, which never asks). `one` deletes one row
/// (its word, the other flags passed on, `/NoConfirmation` unless a single
/// row is the handler's own question).
pub fn delete_rows(
    state: &mut AppState,
    rd: &RowDelete,
    args: &[&str],
    mut one: impl FnMut(&mut AppState, &[&str]) -> DispatchResult,
) -> DispatchResult {
    let confirmed = crate::commands::flags::has(args, &crate::commands::flags::NO_CONFIRMATION);
    let flags: Vec<&str> = args.iter().copied().filter(|a| a.starts_with('/') && !crate::commands::flags::is(a, &crate::commands::flags::NO_CONFIRMATION)).collect();
    let (rows, refusals) = match delete_targets(state, rd, args) {
        None => return one(state, args),
        Some(Err(reply)) => return DispatchResult::Output(reply.into()),
        Some(Ok(found)) => found,
    };
    // A single row of an own-ask object keeps the handler's question.
    let pass_confirm = confirmed || !(rd.own_ask && rows.len() == 1);
    let mut outs = refusals;
    for r in &rows {
        let mut row_args: Vec<&str> = vec![r.as_str()];
        row_args.extend(flags.iter().copied());
        if pass_confirm {
            row_args.push("/NoConfirmation");
        }
        match one(state, &row_args) {
            DispatchResult::Output(o) => outs.push(String::from(o)),
            other if rows.len() == 1 => return other,
            _ => {}
        }
    }
    DispatchResult::Output((outs.join("\n")).into())
}

/// A typed Delete's rows (their keys, from the resolver): the ones that can't go are refused
/// first, each other one is deleted by `one` (its key and `/NoConfirmation` — dispatch asked
/// once), unless a single row of an own-ask object keeps its handler's question.
pub fn delete_resolved(
    state: &mut AppState,
    rd: &RowDelete,
    keys: Vec<String>,
    mut one: impl FnMut(&mut AppState, &[&str]) -> DispatchResult,
) -> DispatchResult {
    let (rows, refusals) = match refuse_some(state, rd, keys) {
        Ok(found) => found,
        Err(reply) => return DispatchResult::Output(reply.into()),
    };
    let pass_confirm = !(rd.own_ask && rows.len() == 1);
    let mut outs = refusals;
    for r in &rows {
        let mut row_args: Vec<&str> = vec![r.as_str()];
        if pass_confirm {
            row_args.push("/NoConfirmation");
        }
        match one(state, &row_args) {
            DispatchResult::Output(o) => outs.push(String::from(o)),
            other if rows.len() == 1 => return other,
            _ => {}
        }
    }
    DispatchResult::Output((outs.join("\n")).into())
}

/// A typed Delete of rows (`Delete Output 1 Thru 3 + 5`, `Delete Network "Rig"`): the resolver
/// names them, rows that can't go are refused first; dispatch asked once before (decisions §30
/// "Merge of bugfixes").
pub fn delete_typed(
    state: &mut AppState,
    obj: &'static crate::objects::ObjectDef,
    rows: &crate::commands::set_rows::RowDelete,
    target: &lumacore_syntax::ast::Target,
    one: fn(&mut AppState, &[&str]) -> crate::DispatchResult,
) -> crate::DispatchResult {
    match crate::resolve::resolve_one(state, target, obj) {
        Ok(r) => crate::commands::set_rows::delete_resolved(state, rows, r.members.iter().map(|m| m.no.to_string()).collect(), one),
        Err(e) => crate::objects::out(e),
    }
}

/// A Store's slot as (number, name): `Store Output 3 "Rig"`, `Store Output "Rig"`, `Store Output`.
pub fn store_slot(state: &AppState, obj: &'static crate::objects::ObjectDef, target: Option<&lumacore_syntax::ast::Target>, name: Option<String>) -> Result<(Option<u32>, Option<String>), String> {
    use crate::resolve::Slot;
    Ok(match crate::resolve::slot(state, target, obj)?.0 {
        Slot::Number(n) => (Some(n), name),
        Slot::NewName(n) => (None, Some(n)),
        Slot::Next => (None, name),
        Slot::Key(k) => return Err(crate::errors::render_code(format!("'{k}' isn't a {} number.", obj.keyword), "E-CMD-008")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_split_on_plus_and_thru() {
        let a = ["1", "Thru", "3", "+", "5", "Rate", "44"];
        let (specs, used) = split(&a);
        assert_eq!(specs, vec![RowSpec::Range("1", "3"), RowSpec::One("5")]);
        assert_eq!(used, 5);
        let (specs, used) = split(&["en0", "+", "en1", "Shell", "No"]);
        assert_eq!((specs.len(), used), (2, 3));
        let (specs, used) = split(&["3", "Rate", "+", "10"]);
        assert!(!is_multi(&specs));
        assert_eq!(used, 1);
    }

    #[test]
    fn a_thru_b_spreads() {
        assert_eq!(spread("21 Thru 30", 4).unwrap(), vec!["21", "24", "27", "30"]);
        assert_eq!(spread("1 thru 2", 3).unwrap(), vec!["1", "2", "2"]);
        assert!(spread("+ 10", 3).is_none());
        assert!(spread("44", 3).is_none());
    }
}
