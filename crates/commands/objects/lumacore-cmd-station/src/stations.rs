// ███████╗████████╗ █████╗ ████████╗██╗ ██████╗ ███╗   ██╗███████╗
// ██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██║██╔═══██╗████╗  ██║██╔════╝
// ███████╗   ██║   ███████║   ██║   ██║██║   ██║██╔██╗ ██║███████╗
// ╚════██║   ██║   ██╔══██║   ██║   ██║██║   ██║██║╚██╗██║╚════██║
// ███████║   ██║   ██║  ██║   ██║   ██║╚██████╔╝██║ ╚████║███████║
// ╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝
// FILE: stations

//! Stations: this one, another (its own settings are sent to it), the table's wiring rows and the
//! LumaSession's; Interface: this station's NICs and what each serves.
//!
//! Owns:          the Station and Interface objects: the station table's rows, their levels and scope.
//! Doesn't own:   their settings (settings.rs, wiring.rs), the table's door (lumacore-state stations).
//! Entry points:  STATION, INTERFACE, station_rights, station_scope.
//! Decisions:     §30 "12d part 4", "Remote station edits", §34 25.17.

use lumacore_objects::objects::{ObjectDef, Verb, out};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// Deleting stations: dispatch's one ask names them; this station, or one
/// that is online in the LumaSession, is refused before it.
static STATION_ROWS: lumacore_objects::commands::set_rows::RowDelete =
    lumacore_objects::commands::set_rows::RowDelete {
        keyword: "Station",
        noun: "station",
        numbers: |s| s.root.show_data.stations.keys().copied().collect(),
        refused: crate::table::station_not_deletable,
        affects: |_, _| None,
        own_ask: false,
    };

lumacore_objects::listed_object!(
props &crate::settings::STATION_SETTINGS,
lum lumacore_objects::objects::LumShape::ids(1).children(&[&crate::network::NETWORK.decl, &INTERFACE.decl]),
default Info,
@ask Some(|s, a| lumacore_objects::commands::set_rows::delete_ask(s, &STATION_ROWS, a));
@rights station_rights, station_scope, lumacore_objects::objects::show_reach; STATION,
"Station",
["stations"],
"Stations: this one (Set Station \"Name\"|\"Location\"|\"Priority\"|\"ThemeOverride\"|\"MulticastBase\"|\"StartupShow\"|\"LumaSession\"|\"Online\"|\"Invite\"|\"HoldOutput\"|\"OutputWhenAlone\" …, EditSetting Station), another (Set Station \"Tower\" \"Priority\" High — sent to it), the station table's wiring rows (Store Station \"Tower\", Set Station \"FOH\" + \"Tower\" Network \"Main\" \"Interface\" \"en1\", Set Station \"Tower\" Interface \"en0\" \"Shell\" No) and the ones in its LumaSession; Invite / Dismiss them; Delete Station forgets a station and its row",
[
    List => |s, a| out(lumacore_objects::commands::list::stations(s, a)),
    Info => |s, a| out(lumacore_objects::commands::info::info_station(s, a.first().copied())),
    Invite => |s, a| out(lumacore_objects::commands::session::cmd_invite_station(s, a)),
    Dismiss => |s, a| out(lumacore_objects::commands::session::cmd_dismiss_station(s, a)),
    Store => |s, a| out(crate::table::cmd_store_station(s, a)),
    Delete => |s, a| lumacore_objects::commands::set_rows::delete_rows(s, &STATION_ROWS, a, |s, a| {
        out(crate::table::cmd_delete_station(s, &a.iter().copied().filter(|w| !w.starts_with('/')).collect::<Vec<_>>()))
    }),
]);

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Station levels: Setup, except `Set` naming a station (`Set Station
/// "Tower" Priority High`), which passes the gate here — another station
/// checks its own declared level when the line arrives there (decisions §30
/// "Remote station edits"), and this station's settings and wiring rows
/// check theirs where they are written (`settings_props::apply`).
pub(crate) fn station_rights(
    verb: Verb,
    args: &[&str],
) -> lumacore_objects::commands::registry::Rights {
    if verb == Verb::Set && crate::wiring::names_a_station(args) {
        lumacore_objects::commands::registry::Rights::Guest
    } else {
        lumacore_objects::rights::setup(verb, args)
    }
}

/// A station's own settings stay here (and its wiring row is written here,
/// going to the master as a change); a row made in advance (`Store
/// Station`) and forgetting a station (`Delete Station`) change the show
/// only, so they go to the master.
pub(crate) fn station_scope(verb: Verb, _: &[&str]) -> lumacore_objects::objects::Scope {
    if matches!(verb, Verb::Delete | Verb::Store) {
        lumacore_objects::objects::Scope::Master
    } else {
        lumacore_objects::objects::Scope::Station
    }
}
lumacore_objects::listed_object!(
props &crate::wiring::INTERFACE_SETTINGS,
lum lumacore_objects::objects::LumShape::ids(1),
default Info,
station INTERFACE,
"Interface",
["interfaces", "nic", "nics"],
"This station's network interfaces and what each serves: Set Interface \"en1\" [+ \"en2\"] \"LumaSession\"|\"Shell\"|\"HTTP\" Yes|No, EditSetting Interface \"en1\" (loopback always serves Shell and HTTP)",
[
    List => |s, a| out(lumacore_objects::commands::list::interfaces(s, a)),
    Info => |s, a| out(lumacore_objects::commands::list::interfaces(s, a)),
]);
