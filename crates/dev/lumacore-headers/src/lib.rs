// ██████╗  █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗
// ██╔══██╗██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗
// ██████╔╝███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝
// ██╔══██╗██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗
// ██████╔╝██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝
// FILE: banner

//! The ANSI Shadow banners every `.rs` file is laid out with (decisions §34): the one font
//! table and the one renderer. `./scripts/unix/headers "Logic"` prints a Part banner with this
//! crate; the station's tidy test re-renders every tag line with it and fails when the art above
//! differs, so nobody draws a banner by hand.
//!
//! Owns:          the ANSI Shadow letters, `render`, `banner`, `section_box`, the Part vocabulary.
//! Doesn't own:   where headers must stand (the tidy rules, crates/dev/lumacore-tidy).
//! Entry points:  render, banner, section_box, PARTS, Kind.
//! Decisions:     §34.

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

// *************************************************************************************************
// Width and the Part vocabulary
// *************************************************************************************************

/// The longest a header line may be, `// ` included (decisions §34: 100 columns).
pub const WIDTH: usize = 100;

/// A file's Parts, in the one order they stand in (decisions §34 3).
pub const PARTS: &[&str] = &["TYPES", "INIT", "REGISTRY", "COMMANDS", "LOGIC", "VIEWS", "IO", "HELPERS", "TESTS"];

/// What a banner opens: the whole file, or one of its Parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    File,
    Part,
}

impl Kind {
    /// The tag line's word (`// FILE: groups`, `// PART: LOGIC`).
    pub fn tag(self) -> &'static str {
        match self {
            Kind::File => "FILE",
            Kind::Part => "PART",
        }
    }
}

// *************************************************************************************************
// The font
// ANSI Shadow, as figlet draws it; letters stand side by side.
// *************************************************************************************************

/// ANSI Shadow, six rows a letter; letters stand side by side (figlet's "full width").
const FONT: &[(char, [&str; 6])] = &[
    ('A', [" █████╗ ", "██╔══██╗", "███████║", "██╔══██║", "██║  ██║", "╚═╝  ╚═╝"]),
    ('B', ["██████╗ ", "██╔══██╗", "██████╔╝", "██╔══██╗", "██████╔╝", "╚═════╝ "]),
    ('C', [" ██████╗", "██╔════╝", "██║     ", "██║     ", "╚██████╗", " ╚═════╝"]),
    ('D', ["██████╗ ", "██╔══██╗", "██║  ██║", "██║  ██║", "██████╔╝", "╚═════╝ "]),
    ('E', ["███████╗", "██╔════╝", "█████╗  ", "██╔══╝  ", "███████╗", "╚══════╝"]),
    ('F', ["███████╗", "██╔════╝", "█████╗  ", "██╔══╝  ", "██║     ", "╚═╝     "]),
    ('G', [" ██████╗ ", "██╔════╝ ", "██║  ███╗", "██║   ██║", "╚██████╔╝", " ╚═════╝ "]),
    ('H', ["██╗  ██╗", "██║  ██║", "███████║", "██╔══██║", "██║  ██║", "╚═╝  ╚═╝"]),
    ('I', ["██╗", "██║", "██║", "██║", "██║", "╚═╝"]),
    ('J', ["     ██╗", "     ██║", "     ██║", "██   ██║", "╚█████╔╝", " ╚════╝ "]),
    ('K', ["██╗  ██╗", "██║ ██╔╝", "█████╔╝ ", "██╔═██╗ ", "██║  ██╗", "╚═╝  ╚═╝"]),
    ('L', ["██╗     ", "██║     ", "██║     ", "██║     ", "███████╗", "╚══════╝"]),
    ('M', ["███╗   ███╗", "████╗ ████║", "██╔████╔██║", "██║╚██╔╝██║", "██║ ╚═╝ ██║", "╚═╝     ╚═╝"]),
    ('N', ["███╗   ██╗", "████╗  ██║", "██╔██╗ ██║", "██║╚██╗██║", "██║ ╚████║", "╚═╝  ╚═══╝"]),
    ('O', [" ██████╗ ", "██╔═══██╗", "██║   ██║", "██║   ██║", "╚██████╔╝", " ╚═════╝ "]),
    ('P', ["██████╗ ", "██╔══██╗", "██████╔╝", "██╔═══╝ ", "██║     ", "╚═╝     "]),
    ('Q', [" ██████╗ ", "██╔═══██╗", "██║   ██║", "██║▄▄ ██║", "╚██████╔╝", " ╚══▀▀═╝ "]),
    ('R', ["██████╗ ", "██╔══██╗", "██████╔╝", "██╔══██╗", "██║  ██║", "╚═╝  ╚═╝"]),
    ('S', ["███████╗", "██╔════╝", "███████╗", "╚════██║", "███████║", "╚══════╝"]),
    ('T', ["████████╗", "╚══██╔══╝", "   ██║   ", "   ██║   ", "   ██║   ", "   ╚═╝   "]),
    ('U', ["██╗   ██╗", "██║   ██║", "██║   ██║", "██║   ██║", "╚██████╔╝", " ╚═════╝ "]),
    ('V', ["██╗   ██╗", "██║   ██║", "██║   ██║", "╚██╗ ██╔╝", " ╚████╔╝ ", "  ╚═══╝  "]),
    ('W', ["██╗    ██╗", "██║    ██║", "██║ █╗ ██║", "██║███╗██║", "╚███╔███╔╝", " ╚══╝╚══╝ "]),
    ('X', ["██╗  ██╗", "╚██╗██╔╝", " ╚███╔╝ ", " ██╔██╗ ", "██╔╝ ██╗", "╚═╝  ╚═╝"]),
    ('Y', ["██╗   ██╗", "╚██╗ ██╔╝", " ╚████╔╝ ", "  ╚██╔╝  ", "   ██║   ", "   ╚═╝   "]),
    ('Z', ["███████╗", "╚══███╔╝", "  ███╔╝ ", " ███╔╝  ", "███████╗", "╚══════╝"]),
    ('0', [" ██████╗ ", "██╔═████╗", "██║██╔██║", "████╔╝██║", "╚██████╔╝", " ╚═════╝ "]),
    ('1', [" ██╗", "███║", "╚██║", " ██║", " ██║", " ╚═╝"]),
    ('2', ["██████╗ ", "╚════██╗", " █████╔╝", "██╔═══╝ ", "███████╗", "╚══════╝"]),
    ('3', ["██████╗ ", "╚════██╗", " █████╔╝", " ╚═══██╗", "██████╔╝", "╚═════╝ "]),
    ('4', ["██╗  ██╗", "██║  ██║", "███████║", "╚════██║", "     ██║", "     ╚═╝"]),
    ('5', ["███████╗", "██╔════╝", "███████╗", "╚════██║", "███████║", "╚══════╝"]),
    ('6', [" ██████╗ ", "██╔════╝ ", "███████╗ ", "██╔═══██╗", "╚██████╔╝", " ╚═════╝ "]),
    ('7', ["███████╗", "╚════██║", "    ██╔╝", "   ██╔╝ ", "   ██║  ", "   ╚═╝  "]),
    ('8', [" █████╗ ", "██╔══██╗", "╚█████╔╝", "██╔══██╗", "╚█████╔╝", " ╚════╝ "]),
    ('9', [" █████╗ ", "██╔══██╗", "╚██████║", " ╚═══██║", " █████╔╝", " ╚════╝ "]),
    ('-', ["      ", "      ", "█████╗", "╚════╝", "      ", "      "]),
    (' ', ["    ", "    ", "    ", "    ", "    ", "    "]),
];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The six rows of `text` in ANSI Shadow (A–Z, 0–9, `-`, space; `_` and `.` read as space).
/// None names a character the font doesn't have.
pub fn render(text: &str) -> Result<[String; 6], String> {
    let mut rows: [String; 6] = Default::default();
    for c in text.chars() {
        let c = match c.to_ascii_uppercase() {
            '_' | '.' => ' ',
            c => c,
        };
        let Some((_, glyph)) = FONT.iter().find(|(k, _)| *k == c) else {
            return Err(format!("the banner font has no '{c}' (A–Z, 0–9, -, space)"));
        };
        for (row, part) in rows.iter_mut().zip(glyph) {
            row.push_str(part);
        }
    }
    for row in rows.iter_mut() {
        *row = row.trim_end().to_string();
    }
    Ok(rows)
}

/// A banner as it stands in a file: the six rows behind `// `, then its tag line. Refused when a
/// row is wider than [`WIDTH`] (a shorter title, decisions §34 3).
pub fn banner(kind: Kind, title: &str) -> Result<Vec<String>, String> {
    let shown = match kind {
        Kind::Part => title.to_ascii_uppercase(),
        Kind::File => title.to_string(),
    };
    let rows = render(&shown)?;
    let mut out: Vec<String> = rows.iter().map(|r| format!("// {r}")).collect();
    if let Some(wide) = out.iter().find(|l| l.chars().count() > WIDTH) {
        return Err(format!("\"{title}\" is {} columns wide, more than {WIDTH}: use a shorter title", wide.chars().count()));
    }
    out.push(format!("// {}: {shown}", kind.tag()));
    Ok(out)
}

/// A Section's star box (decisions §34 4): a rule, the title, its purpose lines, a rule.
pub fn section_box(title: &str, purpose: &[&str]) -> Vec<String> {
    let rule = format!("// {}", "*".repeat(WIDTH - 3));
    let mut out = vec![rule.clone(), format!("// {title}")];
    out.extend(purpose.iter().map(|p| format!("// {p}")));
    out.push(rule);
    out
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_six_even_rows() {
        for (c, glyph) in FONT {
            let w = glyph[0].chars().count();
            assert!(glyph.iter().all(|r| r.chars().count() == w), "'{c}' has uneven rows");
        }
    }

    #[test]
    fn every_part_fits_the_width() {
        for p in PARTS {
            banner(Kind::Part, p).unwrap_or_else(|e| panic!("{p}: {e}"));
        }
    }

    #[test]
    fn a_banner_ends_with_its_tag_and_long_titles_are_refused() {
        let b = banner(Kind::Part, "logic").unwrap();
        assert_eq!(b.len(), 7);
        assert_eq!(b[6], "// PART: LOGIC");
        assert!(banner(Kind::Part, "OBJECT HANDLING TOO").is_err());
        assert!(render("é").is_err());
        assert_eq!(section_box("Store", &["Into a group."])[0].chars().count(), WIDTH);
    }
}
