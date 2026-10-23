//! What a page tells the app (`window.ipc.postMessage(JSON)`), as one typed enum (decisions
//! §31): the connect page (`lumacore-webui`'s `web/connect.html`, `send`), a station screen (`LumaHost` from
//! `app.rs`, `toApp` in the station page) and the Screens overview. A message whose type or
//! fields don't fit is ignored, as an unknown `type` always was.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Ipc {
    // ── Connect page ──
    /// The page has loaded: send it the state.
    Ready,
    AutostartGo,
    AutostartCancel,
    StartLocal,
    /// A server picked from the list or typed in.
    Open { host: Option<String>, port: Option<u16> },
    /// Back to the server list (also the station surface's Servers… button).
    Servers,
    Cancel,
    AutoStart { on: Option<bool> },
    Kiosk { on: Option<bool> },
    /// Quit: stop the local server (`save` None = ask nothing, the server decides).
    StopLocal { save: Option<bool> },
    /// Another version of the local server is running (§23): replace it, use it or cancel.
    OtherReplace { save: Option<bool> },
    OtherUse,
    OtherCancel,
    /// A stop that hangs: Force stop (on this click only), keep waiting or cancel.
    ForceStop,
    KeepWaiting,
    HungCancel,
    CancelQuit,
    CleanStart,
    CleanStartGo { save: Option<bool> },

    // ── Station screen ──
    /// Logout → this server's users; `disconnect` → the server list.
    LoggedOut {
        message: Option<String>,
        #[serde(default)]
        disconnect: bool,
    },
    Disconnect { message: Option<String> },
    UserChanged { user: Option<String>, token: Option<String> },
    /// The station surface logged in (§30 "12a3").
    LoggedIn { user: Option<String>, token: Option<String> },
    ScreenConfig { no: Option<u32> },
    /// Connect to another station (`target`: a station name or `host[:port]`).
    Connect { target: Option<String>, user: Option<String> },
    /// One page preference of this screen (kept per user and screen config).
    Pref {
        key: Option<String>,
        #[serde(default)]
        value: Value,
    },
    /// The page confirmed a power line (`Shutdown …`).
    Power { command: Option<String> },
    PowerCancel,
    /// Native full screen (None = toggle).
    Fullscreen { on: Option<bool> },
    FocusSurface,
    OpenOverview,

    // ── Screens overview ──
    OverviewData,
    OverviewAction {
        act: Option<OverviewAct>,
        no: Option<u32>,
        display: Option<String>,
    },

    /// Any other type: ignored.
    #[serde(other)]
    Unknown,
}

/// What the overview asks for one screen window.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverviewAct {
    Bring,
    Toggle,
    Popout,
    Send,
    #[serde(other)]
    Other,
}

impl Ipc {
    /// A page's message; None when it isn't one the app knows the shape of.
    pub fn parse(body: &str) -> Option<Self> {
        match serde_json::from_str(body) {
            Ok(msg) => Some(msg),
            Err(e) => {
                log::debug!("ignored a page message ({e}): {body}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every message the pages send today, as they send it.
    #[test]
    fn the_pages_messages_parse() {
        let cases = [
            (r#"{"type":"ready"}"#, Ipc::Ready),
            (r#"{"type":"open","host":"10.0.0.5","port":47656}"#, Ipc::Open { host: Some("10.0.0.5".into()), port: Some(47656) }),
            (r#"{"type":"auto_start","on":false}"#, Ipc::AutoStart { on: Some(false) }),
            (r#"{"type":"stop_local","save":null}"#, Ipc::StopLocal { save: None }),
            (r#"{"type":"clean_start_go","save":true}"#, Ipc::CleanStartGo { save: Some(true) }),
            (r#"{"type":"logged_out","message":"bye"}"#, Ipc::LoggedOut { message: Some("bye".into()), disconnect: false }),
            (r#"{"type":"logged_out","disconnect":true}"#, Ipc::LoggedOut { message: None, disconnect: true }),
            (r#"{"type":"disconnect"}"#, Ipc::Disconnect { message: None }),
            (r#"{"type":"user_changed","user":"LD","token":"t"}"#, Ipc::UserChanged { user: Some("LD".into()), token: Some("t".into()) }),
            (
                r#"{"type":"connect","target":"FOH","user":"LD","password":"x"}"#,
                Ipc::Connect { target: Some("FOH".into()), user: Some("LD".into()) },
            ),
            (r#"{"type":"screen_config","no":2}"#, Ipc::ScreenConfig { no: Some(2) }),
            (r#"{"type":"pref","key":"sidebars","value":{"left":true}}"#, Ipc::Pref { key: Some("sidebars".into()), value: serde_json::json!({"left": true}) }),
            (r#"{"type":"power","command":"Shutdown"}"#, Ipc::Power { command: Some("Shutdown".into()) }),
            (r#"{"type":"fullscreen","on":null}"#, Ipc::Fullscreen { on: None }),
            (r#"{"type":"servers"}"#, Ipc::Servers),
            (
                r#"{"type":"overview_action","act":"send","no":3,"display":"HDMI-1"}"#,
                Ipc::OverviewAction { act: Some(OverviewAct::Send), no: Some(3), display: Some("HDMI-1".into()) },
            ),
            (r#"{"type":"overview_action","act":"wave","no":1,"display":null}"#, Ipc::OverviewAction { act: Some(OverviewAct::Other), no: Some(1), display: None }),
            (r#"{"type":"something_new","x":1}"#, Ipc::Unknown),
        ];
        for (body, want) in cases {
            assert_eq!(Ipc::parse(body), Some(want), "{body}");
        }
        // Not a message: ignored.
        assert_eq!(Ipc::parse("not json"), None);
        assert_eq!(Ipc::parse(r#"{"type":"screen_config","no":"two"}"#), None);
    }
}
