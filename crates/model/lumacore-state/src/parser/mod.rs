//! Selection expression parser for the LumaCore REPL.
//!
//! Parses fixture selection expressions from token slices produced by `tokenize()`.
//! Supports console-style selection algebra: ranges, add/sub, group refs, if filters.
//!
//! ## Grammar (informal)
//!
//! ```text
//! selection  = expr [if_clause]
//! expr       = term (('+' | '-') term)*
//! term       = atom ['Thru' atom]
//! atom       = number | '"name"' | 'Group' number | 'Group' '"name"' | 'Thru' (All)
//! if_clause  = 'If' predicate+
//! predicate  = 'Group' '"name"'
//!            | attr_name ('>' | '<' | '=') number
//! ```

pub mod resolve;

use std::fmt;

/// A fixture selection expression — the AST produced by `parse_selection`.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionExpr {
    /// A fixture selected by numeric ID.
    ById(u32),
    /// A fixture selected by name (case-insensitive match).
    ByName(String),
    /// All fixtures in a range (inclusive on both ends).
    Range(u32, u32),
    /// Terms joined left to right: the first, then each `+` (union) or `-`
    /// (difference) term. Flat — a long `1 + 2 + …` line never nests, so
    /// nothing walks it recursively (R1-10).
    Chain(Box<SelectionExpr>, Vec<(SelOp, SelectionExpr)>),
    /// Fixtures in a group, referenced by ID.
    Group(u32),
    /// Fixtures in a group, referenced by name (case-insensitive match).
    GroupByName(String),
    /// All fixtures in ShowData (keyword: bare `Thru`).
    All,
}

/// `+` or `-` between selection terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelOp {
    Add,
    Sub,
}

impl fmt::Display for SelectionExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectionExpr::ById(n) => write!(f, "{n}"),
            SelectionExpr::ByName(s) => write!(f, "\"{s}\""),
            SelectionExpr::Range(a, b) => write!(f, "{a} Thru {b}"),
            SelectionExpr::Chain(first, rest) => {
                write!(f, "{first}")?;
                for (op, term) in rest {
                    let op = if *op == SelOp::Add { '+' } else { '-' };
                    write!(f, " {op} {term}")?;
                }
                Ok(())
            }
            SelectionExpr::Group(n) => write!(f, "Group {n}"),
            SelectionExpr::GroupByName(s) => write!(f, "Group \"{s}\""),
            SelectionExpr::All => write!(f, "All"),
        }
    }
}

/// A filter predicate for the `If` clause of a selection.
#[derive(Debug, Clone, PartialEq)]
pub enum IfPredicate {
    /// Fixture must be a member of the named group (case-insensitive).
    InGroup(String),
    /// Fixture must be a member of the group with this numeric ID.
    InGroupById(u32),
    /// Attribute value must be greater than threshold (percent, 0.0–100.0).
    AttributeGt(String, f64),
    /// Attribute value must be less than threshold (percent, 0.0–100.0).
    AttributeLt(String, f64),
    /// Attribute value must equal threshold (percent, 0.0–100.0).
    AttributeEq(String, f64),
}

/// A fully parsed selection: an expression plus optional `If` filters.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSelection {
    pub expr: SelectionExpr,
    pub filters: Vec<IfPredicate>,
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parse a selection expression from a token slice.
///
/// Tokens come from `tokenize()` which already strips quotes from quoted tokens.
/// Quoted tokens are passed here with their quotes still present; we strip them
/// during name atom parsing.
///
/// Returns `Ok(ParsedSelection)` or `Err(message)`.
pub fn parse_selection(tokens: &[String]) -> Result<ParsedSelection, String> {
    if tokens.is_empty() {
        return Err("Empty selection".to_string());
    }

    let mut pos = 0usize;

    // Special case: bare "Thru" (no preceding atom) = All
    if tokens[0].to_lowercase() == "thru" && tokens.len() == 1 {
        return Ok(ParsedSelection {
            expr: SelectionExpr::All,
            filters: vec![],
        });
    }

    let expr = parse_add_sub(tokens, &mut pos)?;
    let filters = parse_if_clause(tokens, &mut pos)?;

    Ok(ParsedSelection { expr, filters })
}

// ─── Internal combinators ─────────────────────────────────────────────────────

fn parse_add_sub(tokens: &[String], pos: &mut usize) -> Result<SelectionExpr, String> {
    let first = parse_thru(tokens, pos)?;
    let mut rest = Vec::new();

    loop {
        let op = match tokens.get(*pos).map(|t| t.as_str()) {
            Some("+") => SelOp::Add,
            Some("-") => SelOp::Sub,
            _ => break,
        };
        *pos += 1;
        rest.push((op, parse_thru(tokens, pos)?));
    }

    Ok(if rest.is_empty() {
        first
    } else {
        SelectionExpr::Chain(Box::new(first), rest)
    })
}

fn parse_thru(tokens: &[String], pos: &mut usize) -> Result<SelectionExpr, String> {
    let is_thru = |t: Option<&String>| t.is_some_and(|t| t.eq_ignore_ascii_case("thru"));
    let is_number = |t: Option<&String>| t.is_some_and(|t| t.parse::<u32>().is_ok());
    // Open start (Q6): `Thru 5` = from the first; a bare `Thru` = all
    // (`Thru - Group 999`).
    if is_thru(tokens.get(*pos)) {
        *pos += 1;
        if is_number(tokens.get(*pos)) {
            let n = tokens[*pos].parse::<u32>().unwrap_or(0);
            *pos += 1;
            return Ok(SelectionExpr::Range(0, n));
        }
        return Ok(SelectionExpr::All);
    }
    let left = parse_atom(tokens, pos)?;

    if is_thru(tokens.get(*pos)) {
        *pos += 1; // consume "Thru"
        // Open end (Q6): `5 Thru` = to the last.
        if !is_number(tokens.get(*pos)) {
            return match left {
                SelectionExpr::ById(a) => Ok(SelectionExpr::Range(a, u32::MAX)),
                _ => Err("Thru requires numeric IDs on both sides".to_string()),
            };
        }
        let right = parse_atom(tokens, pos)?;
        match (&left, &right) {
            (SelectionExpr::ById(a), SelectionExpr::ById(b)) => {
                return Ok(SelectionExpr::Range(*a, *b));
            }
            _ => return Err("Thru requires numeric IDs on both sides".to_string()),
        }
    }

    Ok(left)
}

fn parse_atom(tokens: &[String], pos: &mut usize) -> Result<SelectionExpr, String> {
    let tok = tokens
        .get(*pos)
        .ok_or_else(|| "Unexpected end of selection".to_string())?;

    let lower = tok.to_lowercase();

    // "Group" keyword
    if lower == "group" {
        *pos += 1;
        let next = tokens
            .get(*pos)
            .ok_or_else(|| "Expected group ID or name after 'Group'".to_string())?;

        if next.starts_with('"') {
            // Quoted name
            let name = strip_quotes(next);
            *pos += 1;
            return Ok(SelectionExpr::GroupByName(name));
        } else if let Ok(id) = next.parse::<u32>() {
            *pos += 1;
            return Ok(SelectionExpr::Group(id));
        } else {
            return Err(format!("Expected group ID or quoted name, got: '{next}'"));
        }
    }

    // "Fixture" keyword inside an expression (`Group 5 - Fixture 2`).
    if lower == "fixture" {
        *pos += 1;
        return parse_atom(tokens, pos);
    }

    // Bare "All" keyword
    if lower == "all" {
        *pos += 1;
        return Ok(SelectionExpr::All);
    }

    // Quoted name → ByName
    if tok.starts_with('"') {
        let name = strip_quotes(tok);
        *pos += 1;
        return Ok(SelectionExpr::ByName(name));
    }

    // Numeric ID
    if let Ok(id) = tok.parse::<u32>() {
        *pos += 1;
        return Ok(SelectionExpr::ById(id));
    }

    Err(format!("Unexpected token in selection: '{tok}'"))
}

fn parse_if_clause(tokens: &[String], pos: &mut usize) -> Result<Vec<IfPredicate>, String> {
    if tokens
        .get(*pos)
        .map(|t| t.to_lowercase().as_str() == "if")
        .unwrap_or(false)
    {
        *pos += 1; // consume "If"
    } else {
        return Ok(vec![]);
    }

    let mut preds = Vec::new();

    // Parse one or more predicates
    while *pos < tokens.len() {
        let tok = &tokens[*pos];
        let lower = tok.to_lowercase();

        // "Group" predicate: If Group "name" | If Group id
        if lower == "group" {
            *pos += 1;
            let next = tokens.get(*pos).ok_or_else(|| {
                "Expected group name or ID after 'Group' in If clause".to_string()
            })?;
            if next.starts_with('"') {
                let name = strip_quotes(next);
                *pos += 1;
                preds.push(IfPredicate::InGroup(name));
            } else if let Ok(id) = next.parse::<u32>() {
                *pos += 1;
                preds.push(IfPredicate::InGroupById(id));
            } else {
                return Err(format!(
                    "Expected group name or ID after 'Group' in If clause, got: '{next}'"
                ));
            }
            continue;
        }

        // Attribute comparison: If Dimmer > 50 | If Pan < 100 | If Tilt = 0
        // Next token after attribute name must be an operator
        if let Some(op_tok) = tokens.get(*pos + 1) {
            let op = op_tok.as_str();
            if op == ">" || op == "<" || op == "=" || op == "==" {
                let attr = tok.to_string();
                *pos += 2; // consume attr + operator
                let val_tok = tokens
                    .get(*pos)
                    .ok_or_else(|| format!("Expected number after '{op}' in If clause"))?;
                let threshold = val_tok
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or_else(|| format!("Expected number, got: '{val_tok}'"))?;
                *pos += 1;
                let pred = match op {
                    ">" => IfPredicate::AttributeGt(attr, threshold),
                    "<" => IfPredicate::AttributeLt(attr, threshold),
                    "=" | "==" => IfPredicate::AttributeEq(attr, threshold),
                    _ => unreachable!(),
                };
                preds.push(pred);
                continue;
            }
        }

        // Unknown token — stop parsing predicates (may be consumed by outer context)
        break;
    }

    if preds.is_empty() {
        return Err("Expected predicate after 'If'".to_string());
    }

    Ok(preds)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Strip surrounding double quotes from a token.
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Convenience: tokenize a raw string for testing (minimal, handles quotes).
#[cfg(test)]
pub fn test_tokens(s: &str) -> Vec<String> {
    // Reuse the terminal tokenizer logic inline for tests
    let mut tokens = Vec::new();
    let chars = s.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in chars {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
                current.push('"');
            }
            '"' if in_quotes => {
                in_quotes = false;
                current.push('"');
                // Don't split here — keep quoted string as single token with quotes
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> ParsedSelection {
        let tokens = test_tokens(s);
        parse_selection(&tokens).unwrap_or_else(|e| panic!("parse_selection({s:?}) failed: {e}"))
    }

    // ── Basic atoms ──────────────────────────────────────────────────────────

    #[test]
    fn parse_by_id() {
        assert_eq!(p("1").expr, SelectionExpr::ById(1));
    }

    #[test]
    fn parse_by_name() {
        assert_eq!(
            p("\"Mac Aura\"").expr,
            SelectionExpr::ByName("Mac Aura".to_string())
        );
    }

    #[test]
    fn parse_all_thru_bare() {
        assert_eq!(p("Thru").expr, SelectionExpr::All);
    }

    #[test]
    fn parse_all_keyword() {
        assert_eq!(p("All").expr, SelectionExpr::All);
    }

    // ── Range ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_range() {
        assert_eq!(p("1 Thru 10").expr, SelectionExpr::Range(1, 10));
    }

    #[test]
    fn parse_range_case_insensitive() {
        assert_eq!(p("1 thru 10").expr, SelectionExpr::Range(1, 10));
        assert_eq!(p("1 THRU 10").expr, SelectionExpr::Range(1, 10));
    }

    // ── Add / Sub ────────────────────────────────────────────────────────────

    #[test]
    fn parse_add() {
        assert_eq!(
            p("1 + 5").expr,
            SelectionExpr::Chain(Box::new(SelectionExpr::ById(1)), vec![(SelOp::Add, SelectionExpr::ById(5))])
        );
    }

    #[test]
    fn parse_sub() {
        assert_eq!(
            p("1 Thru 10 - 5").expr,
            SelectionExpr::Chain(Box::new(SelectionExpr::Range(1, 10)), vec![(SelOp::Sub, SelectionExpr::ById(5))])
        );
    }

    // ── Group ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_group_by_id() {
        assert_eq!(p("Group 1").expr, SelectionExpr::Group(1));
    }

    #[test]
    fn parse_group_by_name() {
        assert_eq!(
            p("Group \"Front Wash\"").expr,
            SelectionExpr::GroupByName("Front Wash".to_string())
        );
    }

    // ── If predicates ────────────────────────────────────────────────────────

    #[test]
    fn parse_if_group_name() {
        let r = p("1 Thru 10 If Group \"Wash\"");
        assert_eq!(r.expr, SelectionExpr::Range(1, 10));
        assert_eq!(r.filters, vec![IfPredicate::InGroup("Wash".to_string())]);
    }

    #[test]
    fn parse_if_attribute_gt() {
        let r = p("1 Thru 10 If Dimmer > 50");
        assert_eq!(
            r.filters,
            vec![IfPredicate::AttributeGt("Dimmer".to_string(), 50.0)]
        );
    }

    #[test]
    fn parse_if_attribute_lt() {
        let r = p("All If Pan < 100");
        assert_eq!(
            r.filters,
            vec![IfPredicate::AttributeLt("Pan".to_string(), 100.0)]
        );
    }

    #[test]
    fn parse_if_attribute_eq() {
        let r = p("1 If Tilt = 0");
        assert_eq!(
            r.filters,
            vec![IfPredicate::AttributeEq("Tilt".to_string(), 0.0)]
        );
    }

    // ── Errors ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_is_error() {
        let tokens: Vec<String> = vec![];
        assert!(parse_selection(&tokens).is_err());
    }

    #[test]
    fn parse_if_without_predicate_is_error() {
        let tokens = test_tokens("1 If");
        assert!(parse_selection(&tokens).is_err());
    }
}

/// Object numbers named by `3`, `1 Thru 4`, `1 Thru` (to the last),
/// `Thru 4` (from the first), joined by `+` (add) and `-` (take out) — the
/// one range rule for every object word (R1-34, Q6): typed order, numbers
/// that don't exist are skipped (the list closes up, no holes), only
/// existing numbers are visited (R1-08: `Thru 4294967295` costs nothing).
/// None when the words aren't such a list.
pub fn number_list<T>(tokens: &[&str], existing: &std::collections::BTreeMap<u32, T>) -> Option<Vec<u32>> {
    // tidy: not-keywords (range grammar)
    let thru = |w: &&str| w.eq_ignore_ascii_case("thru");
    let mut out: Vec<u32> = Vec::new();
    let mut i = 0;
    let mut take_out = false;
    while i < tokens.len() {
        let (from, to, used) = match (tokens.get(i), tokens.get(i + 1), tokens.get(i + 2)) {
            // A bare `Thru` (`Thru - 3`): every number.
            (Some(t), None | Some(&"+" | &"-"), _) if thru(t) => (u32::MIN, u32::MAX, 1),
            (Some(t), Some(m), _) if thru(t) => (u32::MIN, m.parse::<u32>().ok()?, 2),
            (Some(n), Some(t), Some(m)) if thru(t) && m.parse::<u32>().is_ok() => {
                (n.parse::<u32>().ok()?, m.parse::<u32>().ok()?, 3)
            }
            (Some(n), Some(t), _) if thru(t) => (n.parse::<u32>().ok()?, u32::MAX, 2),
            (Some(n), _, _) => {
                let n = n.parse::<u32>().ok()?;
                (n, n, 1)
            }
            _ => return None,
        };
        let ids: Vec<u32> = if from <= to {
            existing.range(from..=to).map(|(k, _)| *k).collect()
        } else {
            existing.range(to..=from).rev().map(|(k, _)| *k).collect()
        };
        if take_out {
            out.retain(|n| !ids.contains(n));
        } else {
            for n in ids {
                if !out.contains(&n) {
                    out.push(n);
                }
            }
        }
        i += used;
        match tokens.get(i).copied() {
            Some("+") => take_out = false,
            Some("-") => take_out = true,
            None => break,
            Some(_) => return None,
        }
        i += 1;
    }
    Some(out)
}

#[cfg(test)]
#[test]
fn one_range_rule_for_every_object() {
    let existing: std::collections::BTreeMap<u32, ()> = [1, 2, 3, 5, 6, 8].into_iter().map(|n| (n, ())).collect();
    let l = |s: &str| number_list(&s.split_whitespace().collect::<Vec<_>>(), &existing);
    assert_eq!(l("1 Thru 6"), Some(vec![1, 2, 3, 5, 6]), "4 skipped, no hole");
    assert_eq!(l("6 Thru 1"), Some(vec![6, 5, 3, 2, 1]), "typed order");
    assert_eq!(l("5 Thru"), Some(vec![5, 6, 8]), "open end");
    assert_eq!(l("Thru 2"), Some(vec![1, 2]), "open start");
    assert_eq!(l("1 Thru 8 - 2 Thru 3"), Some(vec![1, 5, 6, 8]));
    assert_eq!(l("3 + 1 + 3"), Some(vec![3, 1]), "no duplicates");
    assert_eq!(l("1 Thru 4294967295"), Some(vec![1, 2, 3, 5, 6, 8]), "visits only what exists");
    assert_eq!(l("4"), Some(vec![]), "missing: empty, the caller says so");
    assert_eq!(l("1 Thru x"), None);
    assert_eq!(l("1 2"), None);
}
