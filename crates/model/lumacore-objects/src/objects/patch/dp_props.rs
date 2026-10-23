// ██████╗ ██████╗     ██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔══██╗    ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// ██║  ██║██████╔╝    ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██║  ██║██╔═══╝     ██╔═══╝ ██╔══██╗██║   ██║██╔═══╝ ╚════██║
// ██████╔╝██║         ██║     ██║  ██║╚██████╔╝██║     ███████║
// ╚═════╝ ╚═╝         ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝
// FILE: dp_props

//! A DataPool's own properties (decisions 0294: every show object with a NodeMeta takes a Paint):
//! its Paint, keyed by number or name.
//!
//! Owns:          DATAPOOL_PROPS.
//! Doesn't own:   the DataPool object (patch.rs), the Paint parser (lumacore_base::paint).
//! Entry points:  DATAPOOL_PROPS.
//! Decisions:     docs/decisions/0294-object-paint-palette-free-theme-links.

use crate::AppState;

// ██████╗ ███████╗ ██████╗ ██╗███████╗████████╗██████╗ ██╗   ██╗
// ██╔══██╗██╔════╝██╔════╝ ██║██╔════╝╚══██╔══╝██╔══██╗╚██╗ ██╔╝
// ██████╔╝█████╗  ██║  ███╗██║███████╗   ██║   ██████╔╝ ╚████╔╝
// ██╔══██╗██╔══╝  ██║   ██║██║╚════██║   ██║   ██╔══██╗  ╚██╔╝
// ██║  ██║███████╗╚██████╔╝██║███████║   ██║   ██║  ██║   ██║
// ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝
// PART: REGISTRY

/// The DataPools by number: their Paint (decisions 0294: every show object can be painted).
pub static DATAPOOL_PROPS: crate::props::Props = crate::props::Props {
    lists: &[&DATAPOOL_PAINT],
    keyed: Some(crate::props::Keyed {
        rows: |s| s.root.show_data.datapools.keys().map(u32::to_string).collect(),
        row: |s, w| {
            let w = w.trim_matches('"');
            let pools = &s.root.show_data.datapools;
            w.parse::<u32>().ok().filter(|n| pools.contains_key(n)).or_else(|| pools.iter().find(|(_, d)| d.meta.name().is_some_and(|n| n.eq_ignore_ascii_case(w))).map(|(n, _)| *n)).map(|n| n.to_string())
        },
    }),
    label: Some(|_, k| format!("DataPool {k}")),
    ..crate::props::Props::BASE
};
static DATAPOOL_PAINT: [crate::props::Property; 1] = [crate::keyed_paint!(datapool_meta, datapool_meta_mut, "DataPool")];

fn datapool_meta<'a>(s: &'a AppState, k: &str) -> Option<&'a lumacore_base::NodeMeta> {
    s.root.show_data.datapools.get(&k.parse().ok()?).map(|d| &d.meta)
}

fn datapool_meta_mut<'a>(s: &'a mut AppState, k: &str) -> Option<&'a mut lumacore_base::NodeMeta> {
    s.root.show_data.datapools.get_mut(&k.parse().ok()?).map(|d| &mut d.meta)
}
