//! User passwords: PBKDF2-HMAC-SHA256 with a random salt, stored as
//! `pbkdf2-sha256$<iterations>$<salt hex>$<hash hex>` in `User.password_hash`.

use sha2::Sha256;

const ITERATIONS: u32 = 100_000;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

fn derive(password: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut out);
    out
}

pub fn hash_password(password: &str) -> String {
    let salt = *uuid::Uuid::new_v4().as_bytes();
    format!(
        "pbkdf2-sha256${ITERATIONS}${}${}",
        hex(&salt),
        hex(&derive(password, &salt, ITERATIONS))
    )
}

pub fn verify_password(stored: &str, password: &str) -> bool {
    let mut parts = stored.split('$');
    let (Some("pbkdf2-sha256"), Some(iter), Some(salt), Some(hash), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return false;
    };
    let (Ok(iter), Some(salt), Some(hash)) = (iter.parse::<u32>(), unhex(salt), unhex(hash)) else {
        return false;
    };
    let got = derive(password, &salt, iter);
    // Constant-time compare.
    hash.len() == got.len()
        && hash
            .iter()
            .zip(got.iter())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

// ── Login tokens ─────────────────────────────────────────────────────────────
//
// `v2.<user hex>.<expiry unix s>.<password fingerprint>.<nonce>.<hmac hex>`,
// signed with the station's token secret (persisted, so tokens survive a
// restart). The random nonce makes every login's token its own: without it a
// login in the same second as a logout got the token just revoked.
// The fingerprint of the user's password hash is part of the signed payload:
// changing or removing the password revokes the user's tokens. Logout revokes
// one token: its signature is remembered until it expires.

const TOKEN_DAYS: u64 = 30;

fn hmac_hex(secret: &str, payload: &str) -> String {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac accepts any key");
    mac.update(payload.as_bytes());
    hex(&mac.finalize().into_bytes())
}

fn password_fingerprint(user: &lumacore_tree::User) -> String {
    use sha2::Digest;
    let digest = Sha256::digest(user.password_hash.as_deref().unwrap_or("none").as_bytes());
    hex(&digest[..8])
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// A login token for `user` (which must exist).
pub fn issue_token(state: &crate::AppState, user: &str) -> Option<String> {
    let u = state.root.show_data.user(user)?;
    let payload = format!(
        "v2.{}.{}.{}.{}",
        hex(u.login.as_bytes()),
        now_secs() + TOKEN_DAYS * 86_400,
        password_fingerprint(u),
        uuid::Uuid::new_v4().simple()
    );
    let sig = hmac_hex(&state.local_settings.token_secret, &payload);
    Some(format!("{payload}.{sig}"))
}

/// The user a token belongs to, if it is valid, unexpired and the user (and
/// their password) unchanged.
pub fn verify_token(state: &crate::AppState, token: &str) -> Option<String> {
    verify_token_with(
        &state.local_settings.token_secret,
        &state.local_settings.revoked_tokens,
        &state.root.show_data,
        token,
    )
}

/// End a token (logout): it stays refused until it would have expired.
/// Returns the user it belonged to; an invalid token changes nothing.
pub fn revoke_token(state: &mut crate::AppState, token: &str) -> Option<String> {
    // A login that never got its consent ends too (decisions 0290).
    crate::consent::forget_token(state, token);
    let user = verify_token(state, token)?;
    let (sig, expiry) = token_signature(token)?;
    let now = now_secs();
    let revoked = &mut state.local_settings.revoked_tokens;
    revoked.retain(|_, until| *until >= now);
    revoked.insert(sig.to_string(), expiry);
    state.local_settings.save();
    Some(user)
}

/// A token's signature (what a revocation or a waiting consent is kept under) and its expiry
/// (unix s); None when it isn't a token's shape. Nothing is verified here.
pub fn token_signature(token: &str) -> Option<(&str, u64)> {
    let (payload, sig) = token.rsplit_once('.')?;
    let expiry = payload.split('.').nth(2)?.parse::<u64>().ok()?;
    Some((sig, expiry))
}

/// The key a user's revocation is kept under in `revoked_tokens`.
fn user_key(name: &str) -> String {
    format!("user:{}", name.to_lowercase())
}

/// End every token `user` has now (the user was deleted, decisions §30
/// "12c2"): tokens issued up to this second stay refused until they would
/// have expired — an Oops that brings the user back doesn't bring them back.
pub fn revoke_user(state: &mut crate::AppState, user: &str) {
    let now = now_secs();
    let revoked = &mut state.local_settings.revoked_tokens;
    revoked.retain(|_, until| *until >= now);
    revoked.insert(user_key(user), now + TOKEN_DAYS * 86_400);
    state.local_settings.save();
}

/// `verify_token` from the parts it needs — for readers of the published
/// show (HTTP) that don't hold the server state.
pub fn verify_token_with(
    secret: &str,
    revoked: &std::collections::BTreeMap<String, u64>,
    show: &lumacore_tree::ShowData,
    token: &str,
) -> Option<String> {
    let (payload, sig) = token.rsplit_once('.')?;
    if secret.is_empty() || revoked.contains_key(sig) {
        return None;
    }
    let expected = hmac_hex(secret, payload);
    if expected.len() != sig.len()
        || expected
            .bytes()
            .zip(sig.bytes())
            .fold(0u8, |a, (x, y)| a | (x ^ y))
            != 0
    {
        return None;
    }
    let mut parts = payload.split('.');
    let (Some("v2"), Some(user), Some(expiry), Some(fp), Some(_nonce), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return None;
    };
    let name = String::from_utf8(unhex(user)?).ok()?;
    let expiry = expiry.parse::<u64>().ok()?;
    if expiry < now_secs() {
        return None;
    }
    // Every token of a deleted user issued before the deletion is ended.
    if let Some(until) = revoked.get(&user_key(&name))
        && expiry <= *until
    {
        return None;
    }
    let u = show.user(&name)?;
    (password_fingerprint(u) == fp).then(|| u.login.clone())
}

/// Check a login: the user exists, may log in from where they are (`remote`
/// = not this machine) and gave the right password. Returns the user's name.
/// Admin's password is always `admin` and can't be changed; it is always
/// asked for (decisions §30 "12a3": "if the LD/operator leaves, anyone needs
/// to be able to login that is a poweruser").
pub const ADMIN_PASSWORD: &str = "admin";

/// The show's Admin user (by name).
pub fn is_admin_user(name: &str) -> bool {
    name.eq_ignore_ascii_case("Admin")
}

/// Whether logging in as `user` needs a password.
pub fn has_password(state: &crate::AppState, user: &str) -> bool {
    is_admin_user(user) || state.root.show_data.user(user).is_some_and(|u| u.password_hash.is_some())
}

/// Whether `password` is `user`'s (a user without a password takes any).
pub fn check_password(user: &lumacore_tree::User, password: Option<&str>) -> bool {
    if is_admin_user(&user.login) {
        return password == Some(ADMIN_PASSWORD);
    }
    match &user.password_hash {
        Some(hash) => password.is_some_and(|p| verify_password(hash, p)),
        None => true,
    }
}

pub fn authenticate(
    state: &crate::AppState,
    user: &str,
    password: Option<&str>,
    remote: bool,
) -> Result<String, String> {
    if user.eq_ignore_ascii_case(crate::INTERNAL_ROOT_USER) {
        return Err("That user cannot log in".into());
    }
    let u = state
        .root
        .show_data
        .user(user)
        .ok_or_else(|| format!("No user \"{user}\""))?;
    may_log_in_from(state, &u.login, remote)?;
    if !check_password(u, password) {
        return Err(format!("Wrong password for {}", u.login));
    }
    Ok(u.login.clone())
}

/// May `user` log in from where they are (`remote` = another machine)?
/// Refused when the user's Remote Login is off.
pub fn may_log_in_from(
    state: &crate::AppState,
    user: &str,
    remote: bool,
) -> Result<(), String> {
    match state.root.show_data.user(user) {
        Some(u) if remote && !u.remote_login => Err(crate::errors::with_code(
            format!("{} may not log in remotely (Remote Login is off)", u.login),
            "E-RIGHTS-002",
        )),
        _ => Ok(()),
    }
}

/// Mask passwords in a command line — `Login "user" "pw"`, `Connect … User
/// "name" "pw"`, `… Password "pw"` / `/Password pw` — in every `;` statement.
/// Only the secret is replaced (by `"***"`); the rest of the line keeps its
/// spelling and quotes. Dispatch masks a line once before it reaches the
/// journal, undo, replication and sysmon (decisions §19); the Shell masks
/// echo and history.
pub fn mask_secrets(line: &str) -> String {
    use crate::tokens::{Kind, Token, lex_lenient};
    let tokens = lex_lenient(line);
    let mut secrets: Vec<&Token> = Vec::new();
    for statement in tokens.split(|t| t.kind == Kind::Semicolon) {
        let words: Vec<&Token> = statement.iter().filter(|t| t.kind != Kind::Plus).collect();
        let Some(verb) = words.first() else {
            continue;
        };
        let is = |t: &Token, w: &str| t.text.trim_matches('"').eq_ignore_ascii_case(w);
        if let Some(i) = words.iter().position(|t| is(t, "password") || is(t, "/password")) {
            // `Set User "LD" "Password" Ask` opens the dialog: nothing secret yet.
            secrets.extend(words.get(i + 1).filter(|t| !t.text.eq_ignore_ascii_case("ask")));
            continue;
        }
        let login = is(verb, "login");
        if !login && !is(verb, "connect") {
            continue;
        }
        // Login [User] "name" "password" … / Connect … User "name" "password"
        let mut seen_user_kw = false;
        let mut after_name = false;
        for t in words.iter().skip(1) {
            if t.text.starts_with('/') {
                continue;
            }
            if !seen_user_kw && is(t, "user") {
                seen_user_kw = true;
            } else if (login || seen_user_kw) && !after_name {
                after_name = true;
            } else if after_name {
                secrets.push(t);
                break;
            }
        }
    }
    let mut out = line.to_string();
    for t in secrets.iter().rev() {
        out.replace_range(t.start..t.end, "\"***\"");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_roundtrip() {
        let h = hash_password("mac");
        assert!(verify_password(&h, "mac"));
        assert!(!verify_password(&h, "Mac"));
        assert!(!verify_password("garbage", "mac"));
    }

    #[test]
    fn masks_login_passwords() {
        assert_eq!(
            mask_secrets("Login \"LD\" \"secret\""),
            "Login \"LD\" \"***\""
        );
        assert_eq!(mask_secrets("login user LD pw"), "login user LD \"***\"");
        assert_eq!(mask_secrets("Login \"LD\""), "Login \"LD\"");
        assert_eq!(
            mask_secrets("Connect IP 10.0.0.5 User \"LD\" \"pw\""),
            "Connect IP 10.0.0.5 User \"LD\" \"***\""
        );
        assert_eq!(mask_secrets("Store Group 1"), "Store Group 1");
        assert_eq!(
            mask_secrets("Set User \"LD\" \"Password\" \"x\""),
            "Set User \"LD\" \"Password\" \"***\""
        );
        assert_eq!(
            mask_secrets("Store User \"LD\" /Password x /Rights Admin"),
            "Store User \"LD\" /Password \"***\" /Rights Admin"
        );
        assert_eq!(
            mask_secrets("List; Login \"LD\" \"a b\"; Login Admin"),
            "List; Login \"LD\" \"***\"; Login Admin"
        );
    }
}
