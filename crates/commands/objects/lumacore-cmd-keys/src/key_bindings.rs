// ██╗  ██╗███████╗██╗   ██╗    ██████╗ ██╗███╗   ██╗██████╗ ██╗███╗   ██╗ ██████╗ ███████╗
// ██║ ██╔╝██╔════╝╚██╗ ██╔╝    ██╔══██╗██║████╗  ██║██╔══██╗██║████╗  ██║██╔════╝ ██╔════╝
// █████╔╝ █████╗   ╚████╔╝     ██████╔╝██║██╔██╗ ██║██║  ██║██║██╔██╗ ██║██║  ███╗███████╗
// ██╔═██╗ ██╔══╝    ╚██╔╝      ██╔══██╗██║██║╚██╗██║██║  ██║██║██║╚██╗██║██║   ██║╚════██║
// ██║  ██╗███████╗   ██║       ██████╔╝██║██║ ╚████║██████╔╝██║██║ ╚████║╚██████╔╝███████║
// ╚═╝  ╚═╝╚══════╝   ╚═╝       ╚═════╝ ╚═╝╚═╝  ╚═══╝╚═════╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
// FILE: key bindings

//! Key bindings: what each keyboard key names, debug > profile > vanilla.
//!
//! Owns:          the KeyBindings object: List KeyBindings by layer.
//! Doesn't own:   the bindings' data and layers (lumacore-tree keybindings).
//! Entry points:  KEY_BINDINGS.
//! Decisions:     §17.

use nu_ansi_term::{Color, Style};

use lumacore_objects::AppState;
use lumacore_objects::objects::{ObjectDef, out};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// List the keybindings of the screen's shortcut mode (decisions 0295), grouped by source:
/// Debug > User > Vanilla; each says the mode it belongs to, a turned-off key says so.
pub fn list_keybindings(state: &AppState) -> String {
    use lumacore_tree::{BindingSource, resolve_bindings_with_source};

    let user_bindings = state
        .root
        .show_data
        .profile_for_user(state.effective_user_name())
        .map(|p| &p.content.keybindings)
        .cloned()
        .unwrap_or_default();

    let resolved = resolve_bindings_with_source(
        &state.root.default_keybindings,
        &user_bindings,
        &state.root.debug_keybindings,
        state.debug_mode,
        state.shortcut_mode.layer(),
    );

    if resolved.is_empty() {
        return format!("{}", Color::Yellow.paint("No key bindings configured."));
    }

    // A binding names a UiKey (§17): show it with its Press line.
    let shown: std::collections::BTreeMap<&str, String> = resolved
        .values()
        .map(|(b, _)| {
            if b.ui_key.is_empty() {
                return (b.key.as_str(), "off (LumaCore's turned off here)".to_string());
            }
            let press = crate::lookup::find(state, &b.ui_key)
                .map(|k| k.press_rows(false).iter().map(|r| r.line.clone()).collect::<Vec<_>>().join(" › "))
                .unwrap_or_else(|| "⚠ no such UIKey".into());
            (b.key.as_str(), format!("{} ({press})", b.ui_key))
        })
        .collect();

    // Group by source
    let mut debug_rows: Vec<(String, &str, &str)> = Vec::new();
    let mut user_rows: Vec<(String, &str, &str)> = Vec::new();
    let mut vanilla_rows: Vec<(String, &str, &str)> = Vec::new();

    for (binding, source) in resolved.values() {
        let row = (
            lumacore_tree::keybindings::slot(binding.layer, &binding.key),
            shown[binding.key.as_str()].as_str(),
            binding.description.as_str(),
        );
        match source {
            BindingSource::Debug => debug_rows.push(row),
            BindingSource::User => user_rows.push(row),
            BindingSource::Vanilla => vanilla_rows.push(row),
        }
    }

    let bold = |s: &str| Style::new().bold().paint(s).to_string();
    let _green = |s: &str| Color::Green.paint(s).to_string();
    let cyan = |s: &str| Color::Cyan.paint(s).to_string();
    let dim = |s: &str| Style::new().dimmed().paint(s).to_string();

    let key_col: usize = 16;
    let cmd_col: usize = 40;
    let format_row = |key: &str, cmd: &str, desc: &str, key_color: &dyn Fn(&str) -> String| {
        let key_str = key_color(key);
        let key_visible = key.len();
        let key_pad = if key_visible < key_col {
            key_col - key_visible
        } else {
            1
        };
        let cmd_visible = cmd.len();
        let cmd_pad = if cmd_visible < cmd_col {
            cmd_col - cmd_visible
        } else {
            1
        };
        format!(
            "  {}{}{}{}{}",
            key_str,
            " ".repeat(key_pad),
            cmd,
            " ".repeat(cmd_pad),
            dim(desc)
        )
    };

    let mut text = format!(
        "{}  {}\n",
        bold("Key Bindings"),
        dim(&format!("({} · {})", state.effective_user_name(), state.shortcut_mode.label())),
    );

    // Debug section
    if !debug_rows.is_empty() {
        let red = |s: &str| nu_ansi_term::Color::Red.paint(s).to_string();
        text.push_str(&format!("\n{}\n", bold("-- Debug Binds --")));
        for (key, cmd, desc) in &debug_rows {
            text.push_str(&format_row(key, cmd, desc, &red));
            text.push('\n');
        }
    }

    // User section
    if !user_rows.is_empty() {
        text.push_str(&format!("\n{}\n", bold("-- User Binds --")));
        for (key, cmd, desc) in &user_rows {
            text.push_str(&format_row(key, cmd, desc, &cyan));
            text.push('\n');
        }
    }

    // Vanilla section
    if !vanilla_rows.is_empty() {
        let green = |s: &str| Color::Green.paint(s).to_string();
        text.push_str(&format!("\n{}\n", bold("-- Vanilla Binds --")));
        for (key, cmd, desc) in &vanilla_rows {
            text.push_str(&format_row(key, cmd, desc, &green));
            text.push('\n');
        }
    }

    text.push_str(&format!(
        "\n{}",
        dim("Use 'Bind <key> `<line>` | <UIKey> [/Shortcuts PRG]' to add (takes effect at once). 'Unbind <key> [/Shortcuts PRG]' to remove.")
    ));

    text
}

/// `Edit KeyBindings` (decisions 0295): open the key editor of your profile; a terminal gets the
/// list.
#[derive(lumacore_syntax_derive::Args)]
#[lum(verb = "Edit", object = "KeyBindings")]
pub struct EditKeyBindings {
    #[lum(target)]
    pub target: Option<lumacore_syntax::ast::Target>,
}

impl lumacore_objects::commands::typed::Command for EditKeyBindings {
    fn run(self, state: &mut AppState) -> lumacore_objects::DispatchResult {
        // Always your own profile's keys (Bind's): the target names nothing more.
        let _ = self.target;
        state.ui_requests.push(lumacore_wire::UiEvent::KeyEditor);
        out(list_keybindings(state))
    }
}

lumacore_objects::listed_object!(
default List,
@rights lumacore_objects::rights::content, lumacore_objects::objects::on_client, lumacore_objects::objects::show_reach;
KEY_BINDINGS,
"KeyBindings",
["keys", "bindings"],
"Key bindings of your profile; Edit opens the key editor",
[
    List => |s, _| out(list_keybindings(s)),
    Edit => lumacore_objects::commands::typed::typed::<EditKeyBindings>,
]);
