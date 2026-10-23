//! Which layer an attribute board shows and the words its lines start with (decisions §30
//! "12f-a"); the board itself is `lumacore_views::attribute_board`.

use lumacore_tree::ShowData;
use serde_json::Value;

/// Which layer a board shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTarget {
    Show,
    Profile(u32),
}

impl BoardTarget {
    /// A profile's number, `"mine"` (`user`'s own profile), else the show.
    pub fn parse(show: &ShowData, user: &str, v: &Value) -> Self {
        if v.as_str().is_some_and(|s| s.eq_ignore_ascii_case("mine")) {
            return show.profile_id_for_user(user).map_or(Self::Show, Self::Profile);
        }
        match v.as_u64().and_then(|n| u32::try_from(n).ok()) {
            Some(n) if n > 0 => Self::Profile(n),
            _ => Self::Show,
        }
    }
}

/// The words a line from this board starts its object with: `""` (the
/// show's), `UserProfile` (yours) or `UserProfile "Bob"` (owner first).
pub fn owner_words(show: &ShowData, user: &str, target: BoardTarget) -> String {
    match target {
        BoardTarget::Show => String::new(),
        BoardTarget::Profile(no) if show.profile_id_for_user(user) == Some(no) => "UserProfile".into(),
        BoardTarget::Profile(no) => show
            .user_profiles
            .get(&no)
            .map_or_else(|| format!("UserProfile {no}"), |p| format!("UserProfile \"{}\"", p.meta.identity.name.clone().unwrap_or_default())),
    }
}
