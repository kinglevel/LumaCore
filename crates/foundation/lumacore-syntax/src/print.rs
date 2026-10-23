//! The canonical line (decisions §33): what is forwarded, stored in keys and macros, journalled.
//!
//! One spelling: words in their canonical name and case, one space between pieces, an early
//! `If … EndIf` moved to the end, `;` between statements. `print(parse(x))` parses back to the
//! same line.

use crate::ast::*;
use crate::parse::Line;
use crate::Vocabulary;

/// The canonical text of a parsed line.
pub fn print(line: &Line, vocab: &Vocabulary) -> String {
    line.statements.iter().map(|s| statement(s, vocab)).collect::<Vec<_>>().join(" ; ")
}

fn statement(s: &Statement, vocab: &Vocabulary) -> String {
    match &s.body {
        Body::Empty => String::new(),
        Body::Hint { word } => format!("{}?", word.node),
        Body::Alias { name, rest } => {
            if rest.node.is_empty() { format!("!{}", name.node) } else { format!("!{} {}", name.node, rest.node) }
        }
        Body::Fence { word, args } => {
            let mut out = vec![vocab.word(word.node).name.clone()];
            for a in args {
                out.push(match a {
                    FenceArg::Path(p) => p.node.clone(),
                    FenceArg::Text(t) => quote(&t.node),
                    FenceArg::Flag { name, long, value } => {
                        let flag = format!("{}{}", if *long { "--" } else { "-" }, name.node);
                        match value {
                            Some(v) => format!("{flag} {}", v.node),
                            None => flag,
                        }
                    }
                });
            }
            out.join(" ")
        }
        Body::Lum(lum) => lum_text(lum, vocab),
    }
}

fn lum_text(lum: &Lum, vocab: &Vocabulary) -> String {
    let mut out = Vec::new();
    if let Some(op) = &lum.select_op {
        out.push(op_text(op.node).to_string());
    }
    if let Some(verb) = &lum.verb {
        out.push(vocab.word(verb.node).name.clone());
    }
    if let Some(phase) = &lum.phase {
        out.push(match phase.node {
            Phase::On => "On".into(),
            Phase::Off => "Off".into(),
        });
    }
    if let Some(t) = &lum.target {
        out.push(target(t, vocab));
    }
    for slot in &lum.slots {
        out.push(match slot {
            Slot::Helper { word, value: Some(v) } => format!("{} {}", vocab.word(word.node).name, value(v, vocab)),
            Slot::Helper { word, value: None } => vocab.word(word.node).name.clone(),
            Slot::Value(v) => value(v, vocab),
        });
    }
    if let Some(c) = &lum.condition {
        out.push(format!("If {}", target(&c.target, vocab)));
    }
    for o in &lum.options {
        out.push(match &o.value {
            Some(v) => format!("/{} {}", o.name.node, value(v, vocab)),
            None => format!("/{}", o.name.node),
        });
    }
    out.join(" ")
}

/// A target as a line writes it (`Group 1 Thru 3`, `DataPool 2 Group "Front"`).
pub fn target_text(t: &Target, vocab: &Vocabulary) -> String {
    target(t, vocab)
}

fn target(t: &Target, vocab: &Vocabulary) -> String {
    let mut out = String::new();
    for (op, it) in &t.items {
        if let Some(op) = op {
            out.push(' ');
            out.push_str(op_text(op.node));
            out.push(' ');
        }
        out.push_str(&item(it, vocab));
    }
    out
}

fn item(it: &Item, vocab: &Vocabulary) -> String {
    match it {
        Item::Path(segments) => segments
            .iter()
            .map(|s| {
                let name = &vocab.word(s.object.node).name;
                let kind = s.kind.as_ref().map(|k| format!(" {}", k.node)).unwrap_or_default();
                match &s.ids {
                    Some(ids) => format!("{name}{kind} {}", ids_text(ids)),
                    None => name.clone(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        Item::Tree(parts) => {
            let mut parts = parts.node.clone();
            if parts.first().is_some_and(|p| p.eq_ignore_ascii_case("Root")) {
                parts[0] = "Root".into();
            }
            parts.join(".")
        }
        Item::SelfRef(_) => "Self".into(),
        Item::Ids(ids) => ids_text(ids),
    }
}

fn ids_text(ids: &Ids) -> String {
    let mut out = String::new();
    for (op, r) in &ids.ranges {
        if let Some(op) = op {
            out.push(' ');
            out.push_str(op_text(op.node));
            out.push(' ');
        }
        out.push_str(&match r {
            Range::One(id) => id_text(id),
            Range::Thru { from, to, .. } => match (from, to) {
                (Some(f), Some(t)) => format!("{} Thru {}", id_text(f), id_text(t)),
                (Some(f), None) => format!("{} Thru", id_text(f)),
                (None, Some(t)) => format!("Thru {}", id_text(t)),
                (None, None) => "Thru".into(),
            },
        });
    }
    out
}

fn id_text(id: &Id) -> String {
    match id {
        Id::Number(n) => n.node.clone(),
        Id::Name(n) => quote(&n.node),
    }
}

fn value(v: &Value, vocab: &Vocabulary) -> String {
    match v {
        Value::Number(n) => n.node.clone(),
        Value::Relative { op, number } => format!("{} {}", op.node, number.node),
        Value::Spread { from, to } => format!("{} Thru {}", from.node, to.node),
        Value::Word(w) => vocab.word(w.node).name.clone(),
        Value::Text(t) => quote(&t.node),
        Value::Line(l) => format!("`{}`", l.node),
        Value::Lines(ls) => ls.iter().map(|l| format!("`{}`", l.node)).collect::<Vec<_>>().join(" "),
        Value::Read(parts) => format!("Self.{}", parts.node.join(".")),
        Value::Target(t) => target(t, vocab),
    }
}

fn op_text(op: SetOp) -> &'static str {
    match op {
        SetOp::Plus => "+",
        SetOp::Minus => "-",
    }
}

fn quote(text: &str) -> String {
    format!("\"{text}\"")
}
