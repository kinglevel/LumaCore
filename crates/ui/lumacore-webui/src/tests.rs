// ██╗    ██╗███████╗██████╗ ██╗   ██╗██╗    ████████╗███████╗███████╗████████╗███████╗
// ██║    ██║██╔════╝██╔══██╗██║   ██║██║    ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
// ██║ █╗ ██║█████╗  ██████╔╝██║   ██║██║       ██║   █████╗  ███████╗   ██║   ███████╗
// ██║███╗██║██╔══╝  ██╔══██╗██║   ██║██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
// ╚███╔███╔╝███████╗██████╔╝╚██████╔╝██║       ██║   ███████╗███████║   ██║   ███████║
//  ╚══╝╚══╝ ╚══════╝╚═════╝  ╚═════╝ ╚═╝       ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: webui tests

//! Tests of the embedded WebUI.
//!
//! Owns:          the unit tests of lib.rs.
//! Doesn't own:   serving (lumacore-http's static tests, the app's assets test).
//! Entry points:  the #[test] fns.
//! Decisions:     §36 1–3.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::{CONNECT, FILES, INDEX, get};

fn text(path: &str) -> &'static str {
    std::str::from_utf8(get(path).unwrap_or_else(|| panic!("{path} is embedded")).0).expect("utf-8")
}

#[test]
fn both_pages_and_everything_they_load_are_embedded() {
    assert_eq!(get("/").map(|f| f.0), get(INDEX).map(|f| f.0), "/ is the station's page");
    for page in [INDEX, CONNECT] {
        let html = text(page);
        for part in html.split(['"', '\'']).filter(|p| p.starts_with("/kit/") || p.starts_with("/vendor/")) {
            assert!(get(part).is_some(), "{page} loads {part}, which isn't embedded");
        }
    }
}

#[test]
fn every_module_a_kit_file_imports_is_embedded() {
    for (path, bytes, _) in FILES.iter().filter(|(p, _, _)| p.starts_with("/kit/") && p.ends_with(".js")) {
        let source = std::str::from_utf8(bytes).expect("utf-8");
        let dir = &path[..path.rfind('/').expect("a folder") + 1];
        for import in source.split("from './").skip(1).chain(source.split("import './").skip(1)) {
            let file = import.split('\'').next().expect("a quoted path");
            assert!(get(&format!("{dir}{file}")).is_some(), "{path} imports {file}");
        }
    }
}

#[test]
fn types_are_known_and_dot_files_stay_out() {
    for (path, _, mime) in FILES {
        assert!(!path.split('/').any(|p| p.starts_with('.')), "{path}");
        if !path.ends_with("LICENSE") && !path.ends_with("README.md") {
            assert_ne!(*mime, "application/octet-stream", "{path}: add its type to build.rs");
        }
    }
    assert_eq!(get("/kit/kit.js").map(|f| f.1), Some("text/javascript; charset=utf-8"));
    assert!(get("/kit/kit.js?v=2").is_some(), "a query is ignored");
}
