// ██████╗  █████╗ ██╗███╗   ██╗████████╗
// ██╔══██╗██╔══██╗██║████╗  ██║╚══██╔══╝
// ██████╔╝███████║██║██╔██╗ ██║   ██║
// ██╔═══╝ ██╔══██║██║██║╚██╗██║   ██║
// ██║     ██║  ██║██║██║ ╚████║   ██║
// ╚═╝     ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ╚═╝
// FILE: paint

//! The colour an object shows to one viewer (decisions object-paint-palette-free-theme-links):
//! its own Paint (a colour, or a theme slot link in the viewer's own theme) → else the painted tag
//! with the highest Priority (ties: the first in the tag tree; a tag without a Paint of its own
//! takes its parent's) → else its pool's colour. A link to a slot the viewer's theme lacks is kept,
//! said, and falls back the same way. Every feed and the TUI read colours only from here; a client
//! never resolves a Paint.
//!
//! Owns:          Painter (one viewer's resolution), Shown, From, Index (Paint and tags by GUID).
//! Doesn't own:   the Paint value and its parser (lumacore_base::paint), the theme's slots
//!                (lumacore_tree::media).
//! Entry points:  Painter::new, Painter::of, Painter::of_or, Painter::slot (tested on the rig in
//!                lumacore-objects `commands/paint_tests.rs`).
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links, 0205.

use std::collections::HashMap;

use lumacore_base::paint::hex_of;
use lumacore_base::{NodeMeta, Paint};
use lumacore_tree::ShowData;
use lumacore_tree::media::Theme;
use lumacore_tree::tags::{Tag, TagTree};
use serde::Serialize;
use uuid::Uuid;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One viewer's paint resolution: their theme's slots and the show's painted tags, read once per
/// feed build.
pub struct Painter {
    theme: Theme,
    /// Every tag that colours (own Paint or a parent's), by GUID.
    tags: HashMap<Uuid, TagColour>,
}

/// What a tag gives an object.
#[derive(Debug, Clone)]
struct TagColour {
    name: String,
    priority: i32,
    /// Its place in the tag tree, depth first (ties).
    order: usize,
    paint: Paint,
}

/// Every show object's own Paint and the tags it carries, by GUID (from the identity walk): what
/// a list of mixed objects (the Tree window, `ls`) paints its rows from.
pub struct Index(HashMap<Uuid, (Option<Paint>, Vec<Uuid>)>);

impl Index {
    pub fn of(show: &ShowData) -> Self {
        let walk = show.walk_identity();
        let mut map: HashMap<Uuid, (Option<Paint>, Vec<Uuid>)> = walk.objects.into_iter().map(|o| (o.guid, (o.paint, Vec::new()))).collect();
        for l in walk.links.iter().filter(|l| l.kind == "Tag") {
            if let Some(entry) = l.from.and_then(|f| map.get_mut(&f)) {
                entry.1.push(l.target);
            }
        }
        Self(map)
    }
}

/// Where an object's colour comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum From {
    /// Its own colour.
    Own,
    /// Its own link to a theme slot.
    Theme(String),
    /// A painted tag (its name and Priority).
    Tag { name: String, priority: i32 },
    /// Its pool's colour (no Paint, no painted tag).
    Pool,
}

/// The colour an object shows and where it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// `#rrggbb`; None = its pool's own colour (the client's `--lc-pool-*` token).
    pub hex: Option<String>,
    pub from: From,
    /// A theme slot it links that the viewer's theme lacks (kept, listed, fallen back from).
    pub broken: Option<String>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

impl Painter {
    /// The painter for `user`: their profile's theme (none = LumaCore's slots).
    pub fn new(show: &ShowData, user: &str) -> Self {
        let theme = show.profile_for_user(user).map(|p| p.settings.theme.clone()).unwrap_or_default();
        Self::with_theme(show, theme)
    }

    /// The painter for a theme (a profile the window shows, a test's).
    pub fn with_theme(show: &ShowData, theme: Theme) -> Self {
        let mut tags = HashMap::new();
        let mut order = 0;
        collect(&show.tags, None, &mut order, &mut tags);
        Self { theme, tags }
    }

    /// The colour of a theme slot in this viewer's theme (any case); None = no such slot.
    pub fn slot(&self, name: &str) -> Option<&str> {
        self.theme.slot(name)
    }

    /// A Paint as a colour for this viewer; None = a slot link their theme lacks.
    pub fn resolve(&self, p: &Paint) -> Option<String> {
        match p {
            Paint::Rgb(rgb) => Some(hex_of(*rgb)),
            Paint::Theme(slot) => self.slot(slot).map(str::to_ascii_lowercase),
        }
    }

    /// The colour an object shows (its own Paint, its tags, its pool).
    pub fn of(&self, meta: &NodeMeta) -> Shown {
        self.of_or(meta, None)
    }

    /// The colour an object shows when `default` is its Paint while it has none of its own (a
    /// feature group: the theme slot of its name).
    pub fn of_or(&self, meta: &NodeMeta, default: Option<&Paint>) -> Shown {
        self.of_parts(meta.annotation.paint.as_ref().or(default), &meta.annotation.tags)
    }

    /// The colour of an object known by its Paint and the tags it carries (a walk's entry).
    pub fn of_parts(&self, paint: Option<&Paint>, tags: &[Uuid]) -> Shown {
        let mut broken = None;
        if let Some(p) = paint {
            match self.resolve(p) {
                Some(hex) => {
                    let from = match p {
                        Paint::Rgb(_) => From::Own,
                        Paint::Theme(slot) => From::Theme(slot.clone()),
                    };
                    return Shown { hex: Some(hex), from, broken: None };
                }
                None => broken = p.slot().map(str::to_string),
            }
        }
        let best = tags
            .iter()
            .filter_map(|g| self.tags.get(g))
            .filter_map(|t| self.resolve(&t.paint).map(|hex| (t, hex)))
            .max_by(|(a, _), (b, _)| a.priority.cmp(&b.priority).then(b.order.cmp(&a.order)));
        match best {
            Some((t, hex)) => Shown { hex: Some(hex), from: From::Tag { name: t.name.clone(), priority: t.priority }, broken },
            None => Shown { hex: None, from: From::Pool, broken },
        }
    }

    /// The colour of the show object with GUID `g` (None = no such object).
    pub fn of_guid(&self, index: &Index, g: Uuid) -> Option<Shown> {
        index.0.get(&g).map(|(paint, tags)| self.of_parts(paint.as_ref(), tags))
    }

    /// A tag's own colour as it shows in the tag tree (its Paint, else its parent's); None = none.
    pub fn tag(&self, guid: Uuid) -> Option<String> {
        self.tags.get(&guid).and_then(|t| self.resolve(&t.paint))
    }
}

impl Shown {
    /// Where it comes from, as the inspector and the "Colour from" column say it: `own paint`,
    /// `Theme Position`, `tag Wash · 10`, `the pool`; a missing slot is named.
    pub fn from_text(&self) -> String {
        let base = match &self.from {
            From::Own => "own paint".to_string(),
            From::Theme(slot) => format!("Theme {slot}"),
            From::Tag { name, priority } => format!("tag {name} · {priority}"),
            From::Pool => "the pool".to_string(),
        };
        match &self.broken {
            Some(slot) => format!("{base} (Theme {slot} is missing)"),
            None => base,
        }
    }
}

// ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
// ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
// ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
// ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
// ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
// ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
// PART: HELPERS

/// The tags that colour, depth first: a tag without a Paint takes its parent's colour and, with
/// none of its own, its parent's Priority.
fn collect(level: &TagTree, parent: Option<&TagColour>, order: &mut usize, out: &mut HashMap<Uuid, TagColour>) {
    for tag in level.values() {
        let own = own_colour(tag, parent, *order);
        *order += 1;
        if let Some(c) = &own {
            out.insert(tag.meta.identity.guid, c.clone());
        }
        collect(&tag.children, own.as_ref().or(parent), order, out);
    }
}

fn own_colour(tag: &Tag, parent: Option<&TagColour>, order: usize) -> Option<TagColour> {
    let name = tag.name().to_string();
    match (&tag.meta.annotation.paint, parent) {
        (Some(p), _) => Some(TagColour { name, priority: tag.priority, order, paint: p.clone() }),
        (None, Some(up)) => Some(TagColour { name, priority: if tag.priority != 0 { tag.priority } else { up.priority }, order, paint: up.paint.clone() }),
        (None, None) => None,
    }
}
