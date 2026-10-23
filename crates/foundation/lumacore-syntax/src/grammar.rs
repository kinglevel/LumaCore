//! The grammar as text — exactly the `grammar` block of `docs/user/syntax/lum.md` (decision 3;
//! the test `the_grammar_is_lum_md` fails when they differ), and the vocabulary's word list.

use crate::Vocabulary;

/// The grammar this crate implements.
pub const GRAMMAR: &str = r#"(* A line *)
line          = statement { ";" statement } ;
statement     = alias | fence | hint | lum ;
hint          = WORD "?" ;                     (* the word's quick hint *)

(* Lum *)
lum           = [ select-op ] [ verb [ phase ] ] [ if-early ] [ target ] { slot } [ if ] { option } ;
select-op     = "+" | "-" ;                    (* only with no verb: add to / take out of the selection *)
phase         = "On" | "Off" ;                 (* hold verbs only: Top, Flash, Temp, Swap *)
target        = item { set-op item } ;
item          = path | ROOT-PATH | "Self" | ids ;   (* bare ids: the verb's default object, else yours *)
path          = segment { segment } ;          (* owner first; each segment a declared child *)
segment       = OBJECT [ id-kind ] [ ids ] ;
id-kind       = WORD ;                         (* a kind its object declares for its ids, ids after it:
                                                  Fixture Houselights 4 — MVR's ID type + CID *)
ids           = range { set-op range } ;
range         = id [ "Thru" [ id ] ] | "Thru" [ id ] ;
id            = NUMBER | QUOTED ;
set-op        = "+" | "-" ;
slot          = HELPER [ value ] | value ;     (* which slots, in which order: the signature *)
value         = NUMBER | relative | spread | WORD | QUOTED | LINE | SELF-READ | target ;
relative      = ( "+" | "-" | "*" | "/" ) NUMBER ;
spread        = NUMBER "Thru" NUMBER ;
if-early      = "If" target "EndIf" ;          (* printed as `if` at the end *)
if            = "If" target ;
option        = "/" NAME [ value ] ;           (* whether it takes a value: declared on the option *)

(* Aliases: typed lines only *)
alias         = "!" NAME { any-token } ;       (* the typed words are appended to the alias line *)

(* The tree-shell fence *)
fence         = FENCE-WORD { fence-arg } ;
fence-arg     = TREE-PATH | "-" LETTER [ value ] | "--" NAME [ value ] | QUOTED | WORD | NUMBER ;

(* Tokens *)
NUMBER        = [ "+" | "-" ] DIGITS { "." DIGITS } ;   (* a glued sign belongs to the number *)
QUOTED        = '"' { any char but '"' } '"' ;          (* text, never a keyword *)
LINE          = '`' { any char but '`' } '`' ;          (* a command line as a value, never text *)
WORD          = LETTER { LETTER | DIGIT | "_" } [ "+" | "-" ] ;  (* case-insensitive; a glued
                                                  trailing sign only before a space, ";" or the end: Go- *)
ROOT-PATH     = "Root" "." PART { "." PART } ;          (* the tree form; with SELF-READ the only place a dot nests *)
PART          = NAME | DIGITS | QUOTED ;                  (* a node name as stored, a number, quoted text *)
SELF-READ     = "Self" "." NAME { "." NAME } ;
TREE-PATH     = ".." | "/" | "~" [ NAME ] | ROOT-PATH | PART { "." PART } ;   (* ~ = the local library, ~usb1 = a drive's *)
OBJECT, HELPER, NAME, FENCE-WORD
              = WORD from the vocabulary (the registries) ;
operators     = "+" "-" "*" "/" standing alone (space on both sides) ;"#;

/// The grammar's own words: never a verb, object or value in a [`Vocabulary`].
pub const KEYWORDS: &[&str] = &["Thru", "If", "EndIf", "Self"];

/// Whether `name` can be written as one WORD token (grammar `WORD`): a letter, then letters,
/// digits or `_`, and at most one trailing `+` / `-` (`Go-`).
pub fn is_word(name: &str) -> bool {
    let body = name.strip_suffix(['+', '-']).unwrap_or(name);
    let mut chars = body.chars();
    chars.next().is_some_and(|c| c.is_alphabetic()) && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Every word the vocabulary knows with its roles, one per line, sorted — a golden record of
/// the language's words (the station blesses it like `tree_paths.golden.txt`).
pub fn grammar_listing(vocab: &Vocabulary) -> String {
    let mut lines: Vec<String> = vocab
        .words()
        .map(|(_, w)| {
            let mut roles = Vec::new();
            if let Some(v) = &w.verb {
                roles.push(if v.hold { "verb (hold)".to_string() } else { "verb".to_string() });
            }
            if let Some(o) = &w.object {
                let mut role = String::from("object");
                if o.counted {
                    role.push_str(&format!(" ids:{}", o.id_parts.max(1)));
                }
                if !o.children.is_empty() {
                    role.push_str(&format!(" children:{}", o.children.join(",")));
                }
                if !o.id_kinds.is_empty() {
                    role.push_str(&format!(" kinds:{}", o.id_kinds.join(",")));
                }
                roles.push(role);
            }
            if w.helper {
                roles.push("helper".into());
            }
            if w.value {
                roles.push("value".into());
            }
            if w.property {
                roles.push("property".into());
            }
            if let Some(f) = &w.fence {
                let flags: Vec<String> = f.flags.iter().map(|(n, v)| if *v { format!("{n}=") } else { n.clone() }).collect();
                roles.push(if flags.is_empty() { "fence".into() } else { format!("fence flags:{}", flags.join(",")) });
            }
            format!("{} — {}", w.name, roles.join(" · "))
        })
        .collect();
    lines.extend(vocab.options().iter().map(|o| format!("/{} — option{}", o.name, if o.takes_value { " (value)" } else { "" })));
    lines.sort_by_key(|l| l.to_lowercase());
    lines.join("\n") + "\n"
}
