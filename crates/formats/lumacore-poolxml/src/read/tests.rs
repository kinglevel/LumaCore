// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `read`: a small export's groups and items, the counts of what is not read, the
//! refusals.
//!
//! Owns:          the unit tests of read.rs.
//! Doesn't own:   the import's tests (the objects crate).
//! Entry points:  the #[test] fns.
//! Decisions:     §40 13.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

const POOL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<PoolExport DataVersion="2.2.5.2">
  <Pool Name="Default" Guid="01 74 C4 91">
    <Worlds><World Name="Full"/></Worlds>
    <Groups>
      <Group Name="Photon" Guid="01 74 C4 91 BA F9 20 1A">
        <SelectionData>
          <Item IDType="0" ID="840" X="87" Y="98" Z="0"/>
          <Item IDType="0" ID="845" X="81" Y="140" Z="0"/>
        </SelectionData>
      </Group>
      <Group Name="Icarus LEDs">
        <SelectionData>
          <Item IDType="0" ID="849.1" X="0" Y="0" Z="0"/>
          <Item IDType="0" ID="849.2" X="1" Y="0" Z="0"/>
        </SelectionData>
      </Group>
      <Group Name="Empty"/>
    </Groups>
    <Sequences><Sequence Name="A"/><Sequence Name="B"/></Sequences>
    <Macros><Macro Name="M"/></Macros>
  </Pool>
</PoolExport>"#;

#[test]
fn the_groups_and_their_items_are_read_and_the_rest_counted() {
    let pool = read(POOL.as_bytes()).unwrap();
    assert_eq!((pool.name.as_str(), pool.data_version.as_str()), ("Default", "2.2.5.2"));
    assert_eq!(pool.groups.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), ["Photon", "Icarus LEDs", "Empty"]);
    let photon = &pool.groups[0];
    assert_eq!(photon.guid, "01 74 C4 91 BA F9 20 1A");
    assert_eq!(photon.items.len(), 2);
    assert_eq!((photon.items[0].fid(), photon.items[0].x, photon.items[0].y), (Some(840), 87, 98));
    let leds = &pool.groups[1];
    assert!(leds.sub_fixtures_only() && leds.items[0].fid().is_none());
    assert!(pool.groups[2].items.is_empty() && !pool.groups[2].sub_fixtures_only());
    assert_eq!(pool.other, [("Macros".to_string(), 1), ("Sequences".to_string(), 2), ("Worlds".to_string(), 1)]);
}

#[test]
fn a_file_that_is_not_an_export_is_refused() {
    assert!(matches!(read(b""), Err(PoolXmlError::NotAnExport)));
    assert!(matches!(read(b"<Scene/>"), Err(PoolXmlError::NoPool)));
    assert!(matches!(read(b"<PoolExport DataVersion=\"2\"></PoolExport>"), Err(PoolXmlError::NoPool)));
    assert!(matches!(read(b"<PoolExport><Pool Name=\"x\"><Groups><Group"), Err(PoolXmlError::BadXml(_))));
}
