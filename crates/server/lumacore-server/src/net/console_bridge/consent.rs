//  ██████╗ ██████╗ ███╗   ██╗███████╗███████╗███╗   ██╗████████╗
// ██╔════╝██╔═══██╗████╗  ██║██╔════╝██╔════╝████╗  ██║╚══██╔══╝
// ██║     ██║   ██║██╔██╗ ██║███████╗█████╗  ██╔██╗ ██║   ██║
// ██║     ██║   ██║██║╚██╗██║╚════██║██╔══╝  ██║╚██╗██║   ██║
// ╚██████╗╚██████╔╝██║ ╚████║███████║███████╗██║ ╚████║   ██║
//  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝
// FILE: consent

//! HTTP's logins and the Meow License (decisions 0290): `POST /api/login` issues a token that
//! waits for the consent, and a waiting token opens nothing (read from the published view).
//!
//! Owns:          the HTTP login job, the token checks against the view's waiting tokens.
//! Doesn't own:   the pending logins (lumacore-state consent), the endpoints (lumacore-http).
//! Entry points:  login, waits, user_of.
//! Decisions:     docs/decisions/0290-meow-license-consent-on-every-login.

use crate::state::{Published, StateHandle};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Does `token`'s login wait for the Meow License?
pub(super) fn waits(published: &Published, token: &str) -> bool {
    lumacore_objects::consent::token_pending(&published.station.load().consent_tokens, token)
}

/// The user of a valid token whose login waits for the consent.
pub(super) fn user_of(published: &Published, token: &str) -> Option<String> {
    if !waits(published, token) {
        return None;
    }
    let station = published.station.load();
    let show = published.show.load();
    lumacore_objects::passwords::verify_token_with(&station.token_secret, &station.revoked_tokens, &show, token)
}

/// `POST /api/login`: check the login on the state thread and issue its token, which waits for
/// the Meow License; it goes out once HTTP's view holds it back.
pub(super) fn login(
    state: &StateHandle,
    published: &Published,
    user: &str,
    password: Option<&str>,
    remote: bool,
    from: &str,
) -> Result<(String, String), String> {
    use lumacore_objects::passwords;
    let (user, password, from) = (user.to_string(), password.map(str::to_string), from.to_string());
    let result = state
        .call_within(crate::state::HTTP_WAIT, move |ctx| {
            let result = passwords::authenticate(&ctx.state, &user, password.as_deref(), remote).and_then(|user| {
                let token = passwords::issue_token(&ctx.state, &user).ok_or("cannot issue a login token")?;
                lumacore_objects::consent::await_token(&mut ctx.state, &token);
                Ok((user, token))
            });
            let message = match &result {
                Ok((user, _)) => format!("login '{user}' from {from} (HTTP, {})", if remote { "remote" } else { "local" }),
                Err(e) => format!("login '{user}' refused from {from} (HTTP): {e}"),
            };
            crate::context::sysmon(ctx, "Session", message);
            result
        })
        .map_err(|e| e.to_string())??;
    until_held(published, &result.1);
    Ok(result)
}

/// A token just issued goes out only once HTTP's view holds it back (the next publish, at most a
/// view period away): until the consent it opens nothing.
fn until_held(published: &Published, token: &str) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while !waits(published, token) && std::time::Instant::now() < until {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}
