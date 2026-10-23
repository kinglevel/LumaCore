// ██████╗ ███████╗ █████╗ ██████╗
// ██╔══██╗██╔════╝██╔══██╗██╔══██╗
// ██████╔╝█████╗  ███████║██║  ██║
// ██╔══██╗██╔══╝  ██╔══██║██║  ██║
// ██║  ██║███████╗██║  ██║██████╔╝
// ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝
// FILE: read

//! The streaming walk: quick-xml events, nothing kept but the `Groups` element's content and a
//! count of the pool's other children. `read_from` takes any reader so a big export on a drive
//! is never read whole into memory; `read` takes bytes (an upload, a test).
//!
//! Owns:          read, read_from.
//! Doesn't own:   where the file comes from (the caller's door), what a group means.
//! Entry points:  read, read_from.
//! Decisions:     §40 13.

use std::io::BufRead;

use quick_xml::events::{BytesStart, Event};

use crate::pool::{PoolXmlGroup, PoolXmlItem, PoolXml};
use crate::PoolXmlError;

// ██╗ ██████╗
// ██║██╔═══██╗
// ██║██║   ██║
// ██║██║   ██║
// ██║╚██████╔╝
// ╚═╝ ╚═════╝
// PART: IO

/// The export in `bytes`.
pub fn read(bytes: &[u8]) -> Result<PoolXml, PoolXmlError> {
    read_from(std::io::Cursor::new(bytes))
}

/// The export behind `reader`, streamed.
pub fn read_from<R: BufRead>(reader: R) -> Result<PoolXml, PoolXmlError> {
    let mut xml = quick_xml::Reader::from_reader(reader);
    xml.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut pool = PoolXml::default();
    let mut depth: usize = 0;
    let mut seen_root = false;
    let mut seen_pool = false;
    // Where we are: inside Groups (depth of the Groups element), the group being read.
    let mut groups_depth: Option<usize> = None;
    let mut group: Option<PoolXmlGroup> = None;
    let mut counted = std::collections::BTreeMap::<String, usize>::new();
    // The pool child being walked (Sequences, Macros…): its members are counted.
    let mut section: Option<String> = None;
    loop {
        let event = xml.read_event_into(&mut buf).map_err(|e| PoolXmlError::BadXml(e.to_string()))?;
        match event {
            Event::Start(e) => {
                depth += 1;
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                on_start(&e, &name, depth, &mut pool, &mut seen_root, &mut seen_pool, &mut groups_depth, &mut group, &mut counted, &mut section)?;
            }
            Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                on_start(&e, &name, depth + 1, &mut pool, &mut seen_root, &mut seen_pool, &mut groups_depth, &mut group, &mut counted, &mut section)?;
                if depth + 1 == 3 && section.as_deref() == Some(name.as_str()) {
                    section = None;
                }
                if groups_depth.is_some_and(|g| depth + 1 == g + 1) && name == "Group" {
                    // An empty `<Group/>`: no items.
                    if let Some(g) = group.take() {
                        pool.groups.push(g);
                    }
                }
            }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if groups_depth.is_some_and(|g| depth == g + 1) && name == "Group" {
                    if let Some(g) = group.take() {
                        pool.groups.push(g);
                    }
                }
                if groups_depth == Some(depth) && name == "Groups" {
                    groups_depth = None;
                }
                if depth == 3 && section.as_deref() == Some(name.as_str()) {
                    section = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if !seen_root {
        return Err(PoolXmlError::NotAnExport);
    }
    if !seen_pool {
        return Err(PoolXmlError::NoPool);
    }
    pool.other = counted.into_iter().collect();
    Ok(pool)
}

#[allow(clippy::too_many_arguments)]
fn on_start(
    e: &BytesStart,
    name: &str,
    depth: usize,
    pool: &mut PoolXml,
    seen_root: &mut bool,
    seen_pool: &mut bool,
    groups_depth: &mut Option<usize>,
    group: &mut Option<PoolXmlGroup>,
    counted: &mut std::collections::BTreeMap<String, usize>,
    section: &mut Option<String>,
) -> Result<(), PoolXmlError> {
    let attr = |key: &str| -> Option<String> {
        e.attributes().flatten().find(|a| a.key.as_ref() == key.as_bytes()).map(|a| String::from_utf8_lossy(&a.value).to_string())
    };
    match (depth, name) {
        // The root's name is the desk's own; a `Pool` under it is what makes the file an export.
        (1, _) => {
            *seen_root = true;
            pool.data_version = attr("DataVersion").unwrap_or_default();
        }
        (2, "Pool") => {
            *seen_pool = true;
            pool.name = attr("Name").unwrap_or_default();
        }
        (3, "Groups") => *groups_depth = Some(3),
        (3, other) if *seen_pool => {
            // A child of the pool we don't read: its members are counted (Sequences holds
            // Sequence…), so the count is the pool's, not the file's.
            counted.entry(other.to_string()).or_insert(0);
            *section = Some(other.to_string());
        }
        (4, _) if groups_depth.is_none() && *seen_pool => {
            if let Some(parent) = section.as_ref() {
                *counted.entry(parent.clone()).or_insert(0) += 1;
            }
        }
        (4, "Group") if groups_depth.is_some() => {
            *group = Some(PoolXmlGroup { name: attr("Name").unwrap_or_default(), guid: attr("Guid").unwrap_or_default(), items: Vec::new() });
        }
        (_, "Item") if groups_depth.is_some() => {
            if let Some(g) = group.as_mut() {
                let num = |key: &str| attr(key).and_then(|v| v.trim().parse::<i32>().ok()).unwrap_or(0);
                g.items.push(PoolXmlItem {
                    id_type: attr("IDType").and_then(|v| v.parse().ok()).unwrap_or(0),
                    id: attr("ID").unwrap_or_default(),
                    x: num("X"),
                    y: num("Y"),
                    z: num("Z"),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests;
