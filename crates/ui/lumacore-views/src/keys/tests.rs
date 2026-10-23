// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! The `keys` feed on the rig (decisions 0295): each layer's keys with where they come from and
//! their lines; a profile's own binding, a turned-off key, the command line's and NAV's own keys.
//!
//! Owns:          the tests of keys.rs.
//! Doesn't own:   the order of the bindings (lumacore-base keybindings tests).
//! Entry points:  the #[test] fns.
//! Decisions:     docs/decisions/0295-keys-in-shortcut-layers-and-press-cycles.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_tree::keybindings::Layer;

fn layer<'a>(v: &'a Value, name: &str) -> &'a Vec<Value> {
    v["layers"].as_array().unwrap().iter().find(|l| l["layer"] == name).unwrap()["keys"].as_array().unwrap()
}

fn key<'a>(v: &'a Value, l: &str, k: &str) -> &'a Value {
    layer(v, l).iter().find(|x| x["key"] == k).unwrap_or_else(|| panic!("{l} {k}"))
}

#[test]
fn each_layer_says_what_every_key_does_and_where_it_comes_from() {
    let mut state = lumacore_objects::demo::fresh_state();
    let user = state.effective_user_name().to_string();
    let v = view(&state.root.show_data, &user);
    let e = key(&v, "PRG", "E");
    assert_eq!((e["source"].as_str(), e["ui_key"].as_str(), e["locked"].as_bool()), (Some("lumacore"), Some("prg.e"), Some(true)));
    assert_eq!(e["lines"]["press"], json!([{ "line": "Edit", "types": true }, { "line": "EditSetting", "types": true }]));
    assert_eq!(key(&v, "All", "F10")["source"], "core");
    assert_eq!(key(&v, "NAV", "Y")["source"], "nav");
    assert_eq!(key(&v, "NAV", "Up")["text"], "Move", "NAV's own over the line's History");
    assert_eq!(key(&v, "PRG", "Backspace")["text"], "Reset (empty line)");
    assert!(!layer(&v, "KBD").iter().any(|k| k["key"] == "E"), "letters type in KBD");
    // Your own binding in PRG, and LumaCore's L turned off there.
    let profile = state.current_user_profile_mut().unwrap();
    profile.bind_rows(Layer::Programmer, "E", &["Edit".into(), "EditSetting".into(), "EditRecipe".into()], &[1, 2, 3], "");
    profile.bind_ui_key_in(Layer::Programmer, "L", "", "");
    let v = view(&state.root.show_data, &user);
    let e = key(&v, "PRG", "E");
    assert_eq!((e["source"].as_str(), e["ui_key"].as_str(), e["locked"].as_bool()), (Some("profile"), Some("user.prg.e"), Some(false)));
    assert_eq!(e["lines"]["press"].as_array().unwrap().len(), 3);
    assert_eq!(key(&v, "PRG", "L")["source"], "off");
    assert_eq!(v["lines"]["bind"], "Bind \"{key}\" {lines}{mode}");
}
