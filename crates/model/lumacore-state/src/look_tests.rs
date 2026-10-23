use super::*;
use lumacore_tree::media::ThemeAdjust;

fn kit(file: &str) -> String {
    let path = format!(
        "{}/crates/ui/lumacore-webui/web/kit/{file}",
        env!("LUMACORE_ROOT")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn rgb_hex(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

/// The presets here are the ones in tokens.css (the 14 colours + on-accent).
#[test]
fn the_presets_match_tokens_css() {
    let css = kit("tokens.css");
    let names = [
        "bg", "surface", "raised", "line", "text", "text-dim", "accent", "info", "ok", "warn",
        "alert", "programmer", "blind", "park", "on-accent",
    ];
    for (preset, colours) in PRESETS {
        let block_start = match preset {
            "Dark" => css.find(":root {").unwrap(),
            other => css.find(&format!(":root[data-theme=\"{other}\"]")).unwrap(),
        };
        let block = &css[block_start..];
        let block = &block[..block.find('}').unwrap()];
        for (name, colour) in names.iter().zip(colours) {
            let key = format!("--lc-{name}:");
            let at = block.find(&key).unwrap_or_else(|| panic!("{preset} {key}"));
            let value = block[at + key.len()..].trim_start();
            let value: String = value.chars().take_while(|c| *c == '#' || c.is_ascii_hexdigit()).collect();
            let want = parse_hex(&value).unwrap();
            assert_eq!(hex(colour), want, "{preset} {name}");
        }
    }
}

/// `adjust()` gives what kit/theme.js gives (values taken from its code,
/// run in node).
#[test]
fn the_adjusters_match_theme_js() {
    let a = |brightness, contrast, saturation, accent_hue, background_hue, background_saturation| ThemeAdjust {
        brightness,
        contrast,
        saturation,
        accent_hue,
        background_hue,
        background_saturation,
    };
    // (colour, adjusters, accent, neutral, what theme.js gives)
    let cases = [
        ("#9a7a08", a(20, -10, 30, 40, 0, 0), true, false, "#73d000"),
        ("#121212", a(20, -10, 30, 40, 0, 0), false, true, "#2f2f2f"),
        ("#d1d1d1", a(-30, 25, 0, 0, 0, 0), false, false, "#bfbfbf"),
        ("#e10000", a(0, 0, -50, 0, 0, 0), false, false, "#a93838"),
        ("#3f9a90", a(10, 10, 10, -90, 0, 0), true, false, "#82ac3f"),
        // The background adjusters move the neutrals toward the hue, never text (archive §30 "12g").
        ("#121212", a(0, 0, 0, 0, 210, 60), false, true, "#0c1218"),
        ("#303030", a(10, 0, 0, 0, 30, 100), false, true, "#5a3a1b"),
        ("#d1d1d1", a(0, 0, 0, 0, 210, 60), false, false, "#d1d1d1"),
    ];
    for (from, adj, accent, neutral, want) in cases {
        let got = adjust(parse_hex(from).unwrap(), &adj, accent, neutral);
        assert_eq!(rgb_hex(got), want, "{from} {adj:?}");
    }
    // The adjuster names are the ones theme.js reads.
    let js = kit("theme.js");
    for field in ["brightness", "contrast", "saturation", "accent_hue", "background_hue", "background_saturation"] {
        assert!(js.contains(&format!("a.{field}")), "theme.js reads {field}");
    }
}

#[test]
fn a_profile_palette_colour_and_the_preset_resolve() {
    let mut theme = Theme::default();
    assert_eq!(resolve(&theme), preset("Dark"));
    theme.preset = "Night".into();
    theme.palette.accent = Some("#00ff00".into());
    let p = resolve(&theme);
    assert_eq!(p.accent, (0, 255, 0));
    assert_eq!(p.bg, preset("Night").bg);
}
