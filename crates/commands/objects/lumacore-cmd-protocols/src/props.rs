// ██████╗ ██████╗  ██████╗ ██████╗ ███████╗
// ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██╔════╝
// ██████╔╝██████╔╝██║   ██║██████╔╝███████╗
// ██╔═══╝ ██╔══██╗██║   ██║██╔═══╝ ╚════██║
// ██║     ██║  ██║╚██████╔╝██║     ███████║
// ╚═╝     ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝
// FILE: props

//! A line's settings, declared once in the one property registry (decisions §33 58, §34 "34f7c
//! hardened" 7): what every line of a kind has (DMX: Mode, LocalUniverse, Amount, the timing,
//! InputPriority …; Data: the user) once here, and what only one protocol has from its row —
//! its protocol universe (`SacnUniverse`, `PortAddress`) and its settings (`Priority`, `Port`).
//!
//! Owns:          the lines' shared properties per kind and the builder of a protocol's own (from its row).
//! Doesn't own:   Set's handler (the kernel's set), the plan that uses them (the kernel's protocol_plan).
//! Entry points:  DMX_LINE_PROPS, DATA_LINE_PROPS, protocol_props, setting_get, setting_set, universe_get, universe_set.
//! Decisions:     §11, §33 58, §34 22.4, "34f7b hardening", "34f7c hardened".

use lumacore_objects::props::{PropKind, Property};
use lumacore_objects::AppState;
use lumacore_objects::errors::render_code;
use lumacore_protocols::protocol;
use lumacore_protocols::{Direction, InputPriority, LineSend};
use lumacore_tree::{LineOps, Root};

use lumacore_objects::commands::registry::Rights;

use lumacore_objects::objects::protocols::network_id;

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// A protocol's own setting, read and written by line (`Access::Named`).
pub type NamedGet = fn(&AppState, &str, &str) -> String;
pub type NamedSet = fn(&mut AppState, &str, &str, &str) -> Result<String, String>;
/// A protocol's universe property, read and written by line (`Access::Keyed`).
pub type KeyedGet = fn(&AppState, &str) -> String;
pub type KeyedSet = fn(&mut AppState, &str, &str) -> Result<String, String>;

pub(crate) const LINE_NETWORK: Property = Property {
    name: "Network",
    access: lumacore_objects::props::Access::Field("network"),
    kind: PropKind::Link {
        kind: "Network",
        resolve: network_link,
        items: network_items,
        optional: false,
        add: Some("Network"),
    },
    rights: Rights::Setup,
    doc: "the network the line uses (created if the show has none by that name)",
    ..lumacore_objects::props::Property::BASE
};

pub(crate) const LINE_ENABLED: Property = Property {
    name: "Enabled",
    access: lumacore_objects::props::Access::Field("enabled"),
    kind: PropKind::OnOff,
    rights: Rights::Setup,
    doc: "the line runs",
    ..lumacore_objects::props::Property::BASE
};

const LINE_MODE: Property = Property {
    name: "Mode",
    access: lumacore_objects::props::Access::Field("mode"),
    kind: PropKind::Choice(&Direction::LABELS),
    rights: Rights::Setup,
    doc: "Output (its local universes go out) or Input (the protocol's universes come in); Assign's order sets it too",
    ..lumacore_objects::props::Property::BASE
};

const LINE_TARGET: Property = Property {
    name: "Target",
    access: lumacore_objects::props::Access::Field("target"),
    kind: PropKind::Text,
    rights: Rights::Setup,
    doc: "Output: unicast IPv4 address, none = multicast (sACN) / ArtPoll + broadcast (Art-Net). Input: accept only this source, none = any",
    ..lumacore_objects::props::Property::BASE
};

/// What every DMX line has (decisions §34 "34f7c hardened" 7), whatever its protocol.
pub const DMX_LINE_PROPS: [Property; 11] = [
    LINE_NETWORK,
    LINE_MODE,
    Property {
        name: "LocalUniverse",
        access: lumacore_objects::props::Access::Field("local_universe"),
        kind: PropKind::Int { min: 1, max: 65_535 },
        rights: Rights::Setup,
        section: "Universes",
        doc: "the first local universe the line carries (a universe by number: moving a universe never moves the line)",
        ..lumacore_objects::props::Property::BASE
    },
    Property {
        name: "Amount",
        access: lumacore_objects::props::Access::Field("amount"),
        kind: PropKind::Int { min: 0, max: 4_096 },
        rights: Rights::Setup,
        section: "Universes",
        doc: "how many local universes from LocalUniverse (0 = none)",
        ..lumacore_objects::props::Property::BASE
    },
    LINE_TARGET,
    Property {
        name: "Rate",
        access: lumacore_objects::props::Access::Field("rate"),
        kind: PropKind::Int { min: 1, max: 60 },
        rights: Rights::Setup,
        section: "Timing",
        doc: "Output: frames per second (1–60, default 30): the line samples the engine's newest 60 Hz frame; never above what an Art-Net node reports",
        ..lumacore_objects::props::Property::BASE
    },
    Property {
        name: "Send",
        access: lumacore_objects::props::Access::Field("send"),
        kind: PropKind::Choice(&LineSend::LABELS),
        rights: Rights::Setup,
        section: "Timing",
        doc: "Output: Full (every universe every time) or Changes (a change at once and 3 repeats, then a keep-alive every second)",
        ..lumacore_objects::props::Property::BASE
    },
    Property {
        name: "PacketGap",
        access: lumacore_objects::props::Access::Field("packet_gap"),
        kind: PropKind::Int { min: 0, max: 2_000 },
        rights: Rights::Setup,
        section: "Timing",
        doc: "Output: µs between universes, for old nodes that drop bursts (0–2000; never more than half of one send period in all)",
        ..lumacore_objects::props::Property::BASE
    },
    Property {
        name: "Sync",
        access: lumacore_objects::props::Access::Field("sync"),
        kind: PropKind::OnOff,
        rights: Rights::Setup,
        section: "Timing",
        doc: "Output: send ArtSync / an E1.31 sync packet after the line's universes (pixel and video nodes)",
        ..lumacore_objects::props::Property::BASE
    },
    Property {
        name: "InputPriority",
        access: lumacore_objects::props::Access::Field("input_priority"),
        kind: PropKind::Choice(&InputPriority::LABELS),
        rights: Rights::Setup,
        section: "Input",
        doc: "Input: the band its DMX competes at in Prio universes (Super … Lowest)",
        ..lumacore_objects::props::Property::BASE
    },
    LINE_ENABLED,
];

/// What every Data line has (decisions §34 "34f7c hardened" 7): commands run as its user.
pub const DATA_LINE_PROPS: [Property; 5] = [
    LINE_NETWORK,
    LINE_MODE,
    LINE_TARGET,
    LINE_ENABLED,
    Property {
        name: "User",
        access: lumacore_objects::props::Access::Field("user"),
        kind: PropKind::Link {
            kind: "User",
            resolve: user_link,
            items: user_items,
            optional: true,
            add: None,
        },
        rights: Rights::Admin,
        doc: "commands from this line run as this user, with their rights",
        ..lumacore_objects::props::Property::BASE
    },
];

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// What only `protocol` has, from its row: its protocol universe (`SacnUniverse`, keyed by line)
/// and its settings (`Priority`, `Port`, by name). Built once per protocol object (its `dynamic`
/// props), with the object's own readers and writers.
pub fn protocol_props(protocol: &str, get: NamedGet, set: NamedSet, uget: KeyedGet, uset: KeyedSet) -> Vec<Property> {
    use lumacore_registry::setting::SettingKind;
    let leak = |s: &str| -> &'static str { Box::leak(s.to_string().into_boxed_str()) };
    let Some(row) = protocol::find(protocol) else { return Vec::new() };
    let mut out = Vec::new();
    if let Some(u) = row.universes {
        out.push(Property {
            name: u.property,
            kind: PropKind::Text,
            rights: Rights::Setup,
            section: "Universes",
            doc: leak(&format!("the {} universe LocalUniverse is on the wire; the rest follow on ({})", row.name, u.range_text())),
            access: lumacore_objects::props::Access::Keyed { get: uget, set: uset },
            ..Property::BASE
        });
    }
    for d in (row.settings)() {
        let kind = match &d.kind {
            SettingKind::Int { min, max, .. } => PropKind::Int { min: *min, max: *max },
            SettingKind::Bool => PropKind::OnOff,
            SettingKind::Enum { options } => {
                let labels: Vec<&'static str> = options.iter().map(|(_, l)| leak(l)).collect();
                PropKind::Choice(Box::leak(labels.into_boxed_slice()))
            }
            _ => PropKind::Text,
        };
        let only = if d.section.is_empty() { String::new() } else { format!("{} lines only; ", d.section) };
        let doc = format!("{only}{} ({})", d.hint.as_deref().unwrap_or(&d.label), row.name);
        out.push(Property {
            name: leak(&d.key),
            kind,
            rights: Rights::Setup,
            section: "Protocol",
            doc: leak(&doc),
            default: Some(leak(&d.default.to_string())),
            access: lumacore_objects::props::Access::Named { get, set },
            ..Property::BASE
        });
    }
    out
}

/// A protocol setting of line `key` as text.
pub fn setting_get(state: &AppState, protocol: &str, key: &str, name: &str) -> String {
    let Some(line) = key.parse().ok().and_then(|no: u32| state.root.show_data.protocols.line(protocol, no)) else {
        return String::new();
    };
    line.declared()
        .into_iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str().map_or_else(|| v.to_string(), str::to_string))
        .unwrap_or_default()
}

/// `Set Sacn 1 "Priority" 150`: into the line's settings; a setting of one mode only (sACN's
/// Priority is an output's) is refused on a line of the other.
pub fn setting_set(state: &mut AppState, protocol: &str, key: &str, name: &str, value: &str) -> Result<String, String> {
    use lumacore_registry::setting::SettingKind;
    let Some(line) = key
        .parse()
        .ok()
        .and_then(|no: u32| state.root.show_data.protocols.lines_mut(protocol)?.get_mut(&no))
    else {
        return Err(render_code(format!("No {protocol} {key}."), "E-CMD-007"));
    };
    let Some(def) = protocol::find(protocol).and_then(|p| (p.settings)().into_iter().find(|d| d.key.eq_ignore_ascii_case(name))) else {
        return Err(render_code(format!("{protocol} declares no {name}."), "E-SET-001"));
    };
    if !def.section.is_empty() && def.section != line.mode.label() {
        return Err(render_code(
            format!("{protocol} {key} is an {} line: {} is an {} line's (Set {protocol} {key} \"Mode\" {}).", line.mode.label(), def.key, def.section, def.section),
            "E-SET-001",
        ));
    }
    let json = match &def.kind {
        SettingKind::Int { .. } => value.parse::<i64>().map(serde_json::Value::from).map_err(|_| render_code(format!("{protocol} {key} \"{name}\": '{value}' isn't a number."), "E-CMD-008"))?,
        SettingKind::Bool => serde_json::Value::Bool(lumacore_objects::props::on_off(value).unwrap_or(false)),
        SettingKind::Enum { options } => options
            .iter()
            .find(|(_, l)| l.eq_ignore_ascii_case(value))
            .map(|(v, _)| v.clone())
            .unwrap_or_else(|| value.into()),
        _ => value.into(),
    };
    let before = line.setting(&def.key).map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string)).unwrap_or_default();
    line.settings.insert(def.key.clone(), json);
    Ok(format!("{protocol} {key} \"{}\": {before} → {value}", def.key))
}

/// A line's first protocol universe as its protocol spells it (`101`, `0.0.5`).
pub fn universe_get(state: &AppState, protocol: &str, key: &str) -> String {
    key.parse()
        .ok()
        .and_then(|no: u32| state.root.show_data.protocols.line(protocol, no))
        .map(|l| lumacore_protocols::wire_label(protocol, l.universe))
        .unwrap_or_default()
}

/// `Set Sacn 1 "SacnUniverse" 101` / `Set ArtNet 1 "PortAddress" 0.1.0`: the run moves on the
/// wire, refused when it would leave the protocol's range or clash with another output line.
pub fn universe_set(state: &mut AppState, protocol: &str, key: &str, value: &str) -> Result<String, String> {
    let Some(u) = protocol::find(protocol).and_then(|p| p.universes) else {
        return Err(render_code(format!("{protocol} carries commands, not universes."), "E-OUTPUT-006"));
    };
    let Ok(no) = key.parse::<u32>() else {
        return Err(render_code(format!("No {protocol} {key}."), "E-CMD-007"));
    };
    let wire = (u.parse)(value.trim()).ok_or_else(|| {
        render_code(format!("{protocol} {key} \"{}\": '{value}' isn't a {protocol} universe ({}).", u.property, u.range_text()), "E-CMD-008")
    })?;
    let show = &mut state.root.show_data;
    let Some(line) = show.protocols.lines_mut(protocol).and_then(|m| m.get_mut(&no)) else {
        return Err(render_code(format!("No {protocol} {key}."), "E-CMD-007"));
    };
    let before = line.universe;
    line.universe = wire;
    if let Some(e) = LineOps::check_range(show, protocol, no).err().or_else(|| LineOps::wire_clash(show, protocol, no)) {
        if let Some(l) = show.protocols.lines_mut(protocol).and_then(|m| m.get_mut(&no)) {
            l.universe = before;
        }
        return Err(lumacore_objects::errors::render(&e));
    }
    Ok(format!(
        "{protocol} {key} \"{}\": {} → {}",
        u.property,
        (u.label)(before),
        (u.label)(wire)
    ))
}

/// `Set Sacn n "Network" "Rig"`: the network by name (made if missing).
pub(crate) fn network_link(root: &mut Root, raw: &str) -> Result<Option<uuid::Uuid>, String> {
    if raw.is_empty() || raw.eq_ignore_ascii_case("none") {
        return Err("a line always has a network".into());
    }
    let id = match network_id(root, raw) {
        Some(id) => id,
        None => LineOps::ensure_network(&mut root.show_data, raw),
    };
    Ok(Some(root.show_data.protocols.networks[&id].meta.identity.guid))
}

/// The show's networks, with this station's interface for each.
pub(crate) fn network_items(root: &Root) -> Vec<lumacore_wire::PickItem> {
    root.show_data
        .protocols
        .networks
        .iter()
        .map(|(&no, n)| {
            let name = n.meta.name().unwrap_or_default().to_string();
            let note = root
                .station_settings
                .networks
                .get(&name.to_lowercase())
                .cloned()
                .unwrap_or_else(|| "unmapped".to_string());
            lumacore_wire::PickItem {
                no,
                name,
                note,
                guid: n.meta.identity.guid.to_string(),
            }
        })
        .collect()
}

/// The show's users (none = Admin).
pub(crate) fn user_items(root: &Root) -> Vec<lumacore_wire::PickItem> {
    root.show_data
        .users
        .iter()
        .map(|(&no, u)| lumacore_wire::PickItem {
            no,
            name: u.login.clone(),
            note: u.rights.label().to_string(),
            guid: u.meta.identity.guid.to_string(),
        })
        .collect()
}

/// `Set Osc n "User" "Busk"`: the user by name.
pub(crate) fn user_link(root: &mut Root, raw: &str) -> Result<Option<uuid::Uuid>, String> {
    if raw.is_empty() || raw.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    root.show_data
        .user(raw)
        .map(|u| Some(u.meta.identity.guid))
        .ok_or_else(|| format!("no user \"{raw}\""))
}
