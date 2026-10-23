// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `context`: every object's menu comes from its verbs, properties and `ContextDef` row.
//!
//! Owns:          the unit tests of context.rs.
//! Doesn't own:   the drawing (kit context.js, the e2e spec kit-34g4).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 round 3, 18.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn menu(word: &str) -> Value {
    object_menus_json()[word].clone()
}
fn ids(list: &Value, key: &str) -> Vec<String> {
    list.as_array().unwrap().iter().map(|i| i[key].as_str().unwrap_or_default().to_string()).collect()
}

#[test]
fn a_fixture_gets_the_common_actions_every_property_and_its_own_lines() {
    let m = menu("Fixture");
    let common = ids(&m["common"], "id");
    for id in ["edit", "renumber", "label", "delete"] {
        assert!(common.contains(&id.to_string()), "{id} in {common:?}");
    }
    // Delete needs the claim in Live; Label changes a live column (the name).
    let of = |id: &str| m["common"].as_array().unwrap().iter().find(|i| i["id"] == id).cloned().unwrap();
    assert_eq!(of("delete")["claim"], true);
    assert_eq!(of("label")["claim"], false);
    let set = m["set"].as_array().unwrap();
    let address = set.iter().find(|p| p["name"] == "Address").expect("Address is declared");
    assert_eq!(address["live"], true);
    assert!(set.iter().any(|p| p["live"] == false), "non-live properties are listed too");
    let own = ids(&m["own"], "label");
    assert_eq!(own.first().map(String::as_str), Some("Patch to…"));
    // Patch to… and Renumber… open the value dialog's kinds.
    assert_eq!(m["own"][0]["value"], "address");
    assert_eq!(of("renumber")["value"], "number");
    // Every line names its rows; an item without a line opens what its value names (Edit Patch…).
    assert!(m["own"].as_array().unwrap().iter().all(|i| {
        let line = i["line"].as_str().unwrap_or("");
        line.contains("{rows}") || (line.is_empty() && i["value"].is_string())
    }));
}

#[test]
fn an_object_without_its_own_row_gets_info_and_a_group_its_select() {
    assert_eq!(ids(&menu("Group")["own"], "label"), ["Select", "Edit members", "Info"]);
    assert_eq!(ids(&menu("Universe")["own"], "label"), ["Info"]);
}
