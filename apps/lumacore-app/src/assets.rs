//! The files the app ships (decisions §30 "12a3", §36 3): its connect page and the kit it is
//! built from — the same files the station serves, from `lumacore-webui` — so the app shows
//! something before any server is found (and a mobile port can, too). Served on
//! `lumacore://localhost/<path>`; `/` is the connect page.

/// The file at `path` (`/` = the connect page).
pub fn get(path: &str) -> Option<(&'static [u8], &'static str)> {
    let path = path.split(['?', '#']).next().unwrap_or("/");
    lumacore_webui::get(if path == "/" { lumacore_webui::CONNECT } else { path })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_connect_page_and_what_it_loads_ship_with_the_app() {
        let page = std::str::from_utf8(super::get("/").unwrap().0).unwrap();
        for href in ["/vendor/fonts/fonts.css", "/kit/tokens.css", "/kit/kit.css", "/kit/kit.js"] {
            assert!(page.contains(href) && super::get(href).is_some(), "{href}");
        }
        assert!(super::get("/kit/lockbg.jpg").is_some());
        assert!(super::get("/vendor/lucide/sprite.svg").is_some());
    }
}
