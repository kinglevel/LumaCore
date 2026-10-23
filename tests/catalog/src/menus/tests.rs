// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Every menu's JSON on the whole catalog (moved from lumacore-views when the Backup menu went
//! to its crate): tabs, columns, the slab, the contract's menus, every column's property.
//!
//! Owns:          the menu checks over every menu.
//! Doesn't own:   the JSON (lumacore-views `menus`), the menus (each crate's register).
//! Entry points:  the #[test] fns.
//! Decisions:     §30 "12d", §34 20.7, 21.10, 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use lumacore_objects::menu_defs::{Content, MenuDef, MenuKind};
use lumacore_views::menus::{column_json, menu_types_json};

const SLAB: [&str; 10] = [
    "select", "new", "copy", "cut", "paste", "delete", "oops", "import", "export", "edit",
];

#[test]
fn every_menu_publishes_its_tabs_columns_and_slab() {
    let json = menu_types_json();
    let menus = json.as_array().unwrap();
    let defs: Vec<&MenuDef> = lumacore_registry::get().menus().collect();
    assert_eq!(menus.len(), defs.len());
    for (m, def) in menus.iter().zip(defs) {
        assert!(m.get("error").is_none(), "{}: {m}", def.name);
        assert_eq!(m["name"], def.name);
        assert_eq!(
            m["kind"],
            if def.kind == MenuKind::Setup {
                "setup"
            } else {
                "page"
            }
        );
        for t in m["tabs"].as_array().unwrap() {
            for id in SLAB {
                let c = &t["slab"][id];
                assert!(
                    c.is_object() && c["rights"].is_string(),
                    "{} {} slab {id}",
                    def.name,
                    t["name"]
                );
                // A greyed cell always says why.
                if c["line"].is_null() && !["select", "edit"].contains(&id) {
                    assert!(
                        c["off"].is_string(),
                        "{} {} {id} greyed without a reason",
                        def.name,
                        t["name"]
                    );
                }
            }
            assert!(t["bound"].as_array().unwrap().len() <= 4);
            if t["content"]["kind"] == "grid" {
                let g = &t["content"]["grid"];
                for c in g["columns"].as_array().unwrap() {
                    assert!(
                        ["text", "int", "bool", "list"].contains(&c["kind"].as_str().unwrap()),
                        "{c}"
                    );
                    // A per-network column (`expand`) is the station row's NIC.
                    let writable = !c["prop"].is_null() || c["expand"].is_string();
                    assert_eq!(c["readonly"], !writable || g["object"].is_null(), "{c}");
                }
                // Rows from the hardware: no new-object line; New, Copy,
                // Cut, Paste, Delete and Import greyed with the reason (the
                // part 3 Chrome finding: "Delete 2" on interfaces).
                if g["hardware"].is_string() {
                    assert!(g["new_line"].is_null());
                    for id in ["new", "copy", "cut", "paste", "delete", "import"] {
                        assert!(t["slab"][id]["line"].is_null(), "{} {id}", t["name"]);
                        assert_eq!(t["slab"][id]["off"], g["hardware"], "{} {id}", t["name"]);
                    }
                }
            }
        }
    }
    // Every page menu and every setup menu of the contract is there.
    for name in [
        "Menu",
        "Shutdown",
        "Settings",
        "Network",
        "DMXProtocols",
        "LumaSessions",
        "Backup",
        "Users",
    ] {
        assert!(lumacore_objects::menu_defs::find(name).is_some(), "{name}");
    }
    // The columns read kinds and ranges from the declared props.
    let net = menus.iter().find(|m| m["name"] == "DMXProtocols").unwrap();
    let rate = net["tabs"][0]["content"]["grid"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "rate")
        .unwrap();
    assert_eq!(
        (
            rate["kind"].as_str(),
            rate["min"].as_i64(),
            rate["max"].as_i64()
        ),
        (Some("int"), Some(1), Some(60))
    );
    assert_eq!(rate["rights"], "Setup");
    let net = menus.iter().find(|m| m["name"] == "Network").unwrap();
    let shell = net["tabs"][0]["content"]["grid"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "shell")
        .unwrap();
    assert_eq!(shell["options"], serde_json::json!(["Yes", "No"]));
    assert_eq!(
        net["tabs"][0]["slab"]["delete"]["off"],
        "Interfaces come from the hardware"
    );
    assert_eq!(net["power"]["on"]["line"], "Set Station \"Online\" Off");
    let settings = menus.iter().find(|m| m["name"] == "Settings").unwrap();
    assert_eq!(settings["tabs"][0]["content"]["settings"], "Station");
}

#[test]
fn every_column_prop_resolves_on_its_object() {
    for m in lumacore_registry::get().menus() {
        for t in m.tabs {
            if let Content::Grid(g) = t.content {
                for c in g.columns {
                    column_json(g.object.map(lumacore_objects::objects::def_of), c)
                        .unwrap_or_else(|e| panic!("{} {}: {e}", m.name, g.key));
                }
            }
        }
    }
}
