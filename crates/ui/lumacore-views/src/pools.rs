// ██████╗  ██████╗  ██████╗ ██╗     ███████╗
// ██╔══██╗██╔═══██╗██╔═══██╗██║     ██╔════╝
// ██████╔╝██║   ██║██║   ██║██║     ███████╗
// ██╔═══╝ ██║   ██║██║   ██║██║     ╚════██║
// ██║     ╚██████╔╝╚██████╔╝███████╗███████║
// ╚═╝      ╚═════╝  ╚═════╝ ╚══════╝╚══════╝
// FILE: pools

//! A pool window's content (decisions §22 "Unified pool window"): every pool object's cells,
//! built from the object's `PoolDef`, and one object's editor content.
//!
//! Owns:          the pool objects list, `view` (the `pools` feed), `detail` (one object).
//! Doesn't own:   the pool types and DataPool helpers (`lumacore_objects::pools`), each pool's cells (its object).
//! Entry points:  pools, view, detail.
//! Decisions:     §22, §30 "12d", §34 15.

use std::collections::{BTreeMap, HashSet};

use lumacore_tree::ShowData;
use serde_json::{Value, json};
use uuid::Uuid;

use lumacore_objects::pools::{PoolContext, PoolDef, referenced, user_datapool};

// ██╗   ██╗██╗███████╗██╗    ██╗███████╗
// ██║   ██║██║██╔════╝██║    ██║██╔════╝
// ██║   ██║██║█████╗  ██║ █╗ ██║███████╗
// ╚██╗ ██╔╝██║██╔══╝  ██║███╗██║╚════██║
//  ╚████╔╝ ██║███████╗╚███╔███╔╝███████║
//   ╚═══╝  ╚═╝╚══════╝ ╚══╝╚══╝ ╚══════╝
// PART: VIEWS

// *************************************************************************************************
// The pools
// Every object that declares a pool, and its window's content.
// *************************************************************************************************

/// The pool objects (keyword, def).
pub fn pools() -> impl Iterator<Item = (&'static str, &'static PoolDef)> {
    lumacore_objects::objects::all()
        .filter_map(|o| o.pool.map(|p| (o.keyword, p)))
}

/// A pool window's content: `{pool, window, actions, datapool, profile, moving,
/// cells}` (`moving`: something in it changes with time alone).
/// `pool` is the object keyword or window type name; `datapool` None =
/// follow the user's; `profile` the profile a profile's pool shows (None = the
/// user's); `filter` what the window narrows it to. None when there is no such pool.
pub fn view(
    show: &ShowData,
    claims: &BTreeMap<Uuid, String>,
    user: &str,
    pool: &str,
    datapool: Option<u32>,
    profile: Option<u32>,
    filter: lumacore_objects::pools::PoolFilter,
) -> Option<Value> {
    let (keyword, def) = pools()
        .find(|(k, d)| k.eq_ignore_ascii_case(pool) || d.window.eq_ignore_ascii_case(pool))?;
    let datapool = datapool
        .filter(|n| *n > 0)
        .unwrap_or_else(|| user_datapool(show, user));
    let referenced = referenced(show);
    let ctx = PoolContext {
        show,
        user,
        datapool,
        claims,
        referenced: &referenced,
        now_ms: lumacore_tree::now_ms(),
        profile,
        filter,
    };
    // A group master fading: the `pools` feed builds again until it rests
    // (decisions §30 "12d": builds follow counters, a fade has none).
    let moving = show.playbacks.live.masters.groups.values().any(|l| l.fading(ctx.now_ms));
    Some(json!({
        "pool": keyword,
        "window": def.window,
        "actions": def.actions,
        "datapool": datapool,
        "profile": profile.or_else(|| show.profile_id_for_user(user))
            .and_then(|n| show.user_profiles.get(&n))
            .map(|p| p.meta.identity.name.clone().unwrap_or_default()),
        "mine": profile.is_none_or(|p| show.profile_id_for_user(user) == Some(p)),
        "moving": moving,
        "cells": (def.cells)(&ctx),
        "head": def.head.map(|h| h(&ctx)),
    }))
}

/// One pool object's editor content (`GET /api/pools/{pool}/{no}`).
pub fn detail(
    show: &ShowData,
    claims: &BTreeMap<Uuid, String>,
    user: &str,
    pool: &str,
    datapool: Option<u32>,
    no: u32,
) -> Option<Value> {
    let (_, def) = pools()
        .find(|(k, d)| k.eq_ignore_ascii_case(pool) || d.window.eq_ignore_ascii_case(pool))?;
    let datapool = datapool
        .filter(|n| *n > 0)
        .unwrap_or_else(|| user_datapool(show, user));
    let referenced = HashSet::new();
    let ctx = PoolContext {
        show,
        user,
        datapool,
        claims,
        referenced: &referenced,
        now_ms: lumacore_tree::now_ms(),
        profile: None,
        filter: Default::default(),
    };
    (def.detail?)(&ctx, no)
}

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
