// ██╗     ██╗███╗   ██╗███████╗███████╗
// ██║     ██║████╗  ██║██╔════╝██╔════╝
// ██║     ██║██╔██╗ ██║█████╗  ███████╗
// ██║     ██║██║╚██╗██║██╔══╝  ╚════██║
// ███████╗██║██║ ╚████║███████╗███████║
// ╚══════╝╚═╝╚═╝  ╚═══╝╚══════╝╚══════╝
// FILE: lines

//! The one set of line handlers every protocol's object plugs into (decisions §34 "34f7c
//! hardened" 1, 9): `Store Sacn 1 /Network "Rig" /LocalUniverse 1 /Amount 32 /SacnUniverse 101`,
//! `Assign Sacn 2 At Universe 40 /Universe 9`, `Unassign Sacn 1`, the door's verbs, List, Info
//! and the write checks. Each protocol's crate declares its object with [`line_object!`]; nothing
//! here names a protocol — "never duplicate per protocol".
//!
//! Owns:          the lines' shared handlers (Store, the write checks, the tree views) and `line_object!`, `dmx_line_verbs!`.
//! Doesn't own:   the protocols' objects (their crates under commands/objects/protocols), their properties (props.rs), the Network (lumacore-cmd-station), the drivers (lumacore-lines).
//! Entry points:  line_object!, dmx_line_verbs!, line_store, LineSpec, line_write, lines_children, line_view, any_dmx_line, clear_runs.
//! Decisions:     §11, §34 "34f7c hardened", §35.

use lumacore_objects::AppState;
use lumacore_objects::DispatchResult;
use lumacore_objects::errors::render_code;
use lumacore_objects::objects::out;
use lumacore_objects::props;
use lumacore_objects::text::render_success;
use lumacore_objects::vfs::{DynChild, NodeView, btree_to_children, object_view};
use lumacore_protocols::protocol::{self, Kind};
use lumacore_protocols::{DEFAULT_NETWORK, Direction};
use lumacore_tree::{LineOps, Root};

// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

type Edit<'a> = &'a mut dyn FnMut(&mut serde_json::Value) -> Result<(), String>;

/// What a line's Store names (decisions §33 3d: from the typed line).
pub struct LineSpec {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub mode: Option<Direction>,
    pub network: Option<String>,
    pub user: Option<String>,
    pub port: Option<u32>,
    pub local: Option<u32>,
    pub amount: Option<u32>,
    pub universe: Option<String>,
}

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// `Store <Protocol> [<n>] ["name"] /Network … [/Mode …] [/LocalUniverse … /Amount … /<Universe> …]
/// [/Port … /User …]`: a new line of `protocol`.
pub fn line_store(protocol: &str, state: &mut AppState, a: LineSpec) -> DispatchResult {
    let Some(row) = protocol::find(protocol) else {
        return out(render_code(format!("This build has no protocol {protocol}."), "E-CMD-007"));
    };
    let mode = a.mode.unwrap_or(if row.outputs { Direction::Out } else { Direction::In });
    if !takes(row, mode) {
        return out(render_code(format!("{} lines are {}: Store {} … /Mode {}", row.name, only(row), row.name, only(row)), "E-CMD-008"));
    }
    let data = row.kind == Kind::Data;
    // `/Port` is a setting only some protocols declare (refused, naming the protocol).
    if a.port.is_some() && !(row.settings)().iter().any(|d| d.key == "Port") {
        return out(render_code(format!("{} declares no Port (its port is {}).", row.name, row.port), "E-CMD-008"));
    }
    if data && (a.local.is_some() || a.amount.is_some() || a.universe.is_some()) {
        return out(render_code(format!("{} carries commands, not universes.", row.name), "E-OUTPUT-006"));
    }
    let network = a.network.clone().unwrap_or_else(|| DEFAULT_NETWORK.to_string());
    // The line runs as its user: an unknown one is refused, and a Data line without one too —
    // it never runs as Admin (decisions §30 "12d"). A line may run as a user up to your own
    // level (changing it later is Admin's).
    let user_guid = match &a.user {
        Some(u) => match state.root.show_data.user(u) {
            Some(found) => {
                if let Err(refused) = lumacore_objects::rights::require(
                    state,
                    found.rights.into(),
                    &format!("{} running as {} ({})", row.name, found.login, found.rights.label()),
                ) {
                    return out(refused);
                }
                Some(found.meta.identity.guid)
            }
            None => return out(render_code(format!("No user \"{u}\" (Store User first)."), "E-CMD-011")),
        },
        None if data => {
            return out(render_code(
                format!("A {} line runs its commands as a user: add /User \"name\" (Store {} … /User \"LD\").", row.name, row.name),
                "E-RIGHTS-003",
            ));
        }
        None => None,
    };
    let wire = match (&a.universe, row.universes) {
        (Some(w), Some(u)) => match (u.parse)(w.trim()) {
            Some(w) => Some(w),
            None => return out(render_code(format!("/{} {w}: a {} universe ({}).", u.property, row.name, u.range_text()), "E-CMD-008")),
        },
        _ => None,
    };
    let show = &mut state.root.show_data;
    if let Some(id) = a.id
        && show.protocols.line(row.name, id).is_some()
    {
        return out(render_code(format!("{} {id} already exists.", row.name), "E-CMD-010"));
    }
    let Some(id) = LineOps::create_line(show, row.name, mode, a.id, a.name.clone(), &network) else {
        return out(render_code(format!("No {} number is left: give the new one a number.", row.name), "E-CMD-010"));
    };
    let line = show.protocols.lines_mut(row.name).and_then(|m| m.get_mut(&id)).expect("just created");
    if let Some(port) = a.port.and_then(|p| u16::try_from(p).ok()) {
        line.settings.insert("Port".to_string(), port.into());
    }
    line.user = user_guid;
    if let Some(l) = a.local {
        line.local_universe = l.max(1);
    }
    if let Some(n) = a.amount {
        line.amount = n;
    }
    if let Some(w) = wire {
        line.universe = w;
    }
    if let Some(e) = LineOps::check_range(show, row.name, id).err().or_else(|| LineOps::wire_clash(show, row.name, id)) {
        let _ = LineOps::delete_line(show, row.name, id);
        show.rebuild_id_pools();
        return out(lumacore_objects::errors::render(&e));
    }
    let line = show.protocols.line(row.name, id).expect("just created");
    let next = if data || line.amount > 0 { String::new() } else { format!(". Assign Universe 1 At {} {id}", row.name) };
    let run = if line.amount > 0 { format!(" {}", line.run_label()) } else { String::new() };
    out(render_success(&format!("{} {id} ({}){run} on Network \"{network}\"{next}", row.name, mode.label())))
}

fn takes(row: &protocol::ProtocolDef, mode: Direction) -> bool {
    match mode {
        Direction::Out => row.outputs,
        Direction::In => row.inputs,
    }
}

/// The mode a protocol's lines take when they take only one.
fn only(row: &protocol::ProtocolDef) -> &'static str {
    if row.outputs { "Output" } else { "Input" }
}

/// An option's value as text: quoted text, a declared word or a number (`/Network "Rig"`).
pub fn option_text(v: &Option<lumacore_syntax::ast::Value>) -> Result<Option<String>, String> {
    use lumacore_syntax::ast::Value;
    Ok(match v {
        None => None,
        Some(Value::Text(t)) => Some(t.node.clone()),
        Some(Value::Word(w)) => Some(lumacore_objects::lum::vocabulary().word(w.node).name.clone()),
        Some(Value::Number(n)) => Some(n.node.clone()),
        Some(_) => return Err(render_code("A name in quotes here.", "E-CMD-008")),
    })
}

/// `/Mode Output|Input`.
pub fn mode_option(v: &Option<lumacore_syntax::ast::Value>) -> Result<Option<Direction>, String> {
    match option_text(v)? {
        None => Ok(None),
        Some(t) => Direction::parse(&t).map(Some).ok_or_else(|| render_code(format!("/Mode {t}: Output or Input."), "E-CMD-008")),
    }
}

/// A line may change only within what its protocol takes (Osc lines are inputs), its run must
/// fit the protocol's range and an output must not send a protocol universe another output of
/// its protocol sends to the same destination (E-OUTPUT-003, 005): checked after every write,
/// the line put back when refused.
pub fn line_write(root: &mut Root, protocol: &str, id: u32, edit: Edit) -> Result<(), String> {
    let lines = root.show_data.protocols.lines_mut(protocol).ok_or_else(|| format!("no protocol {protocol}"))?;
    let line = lines.get_mut(&id).ok_or_else(|| format!("no {protocol} {id}"))?;
    let before = line.clone();
    props::edit_json(line, edit)?;
    line.protocol.clone_from(&before.protocol);
    let row = protocol::find(protocol);
    let refused = if let Some(r) = row.filter(|r| !takes(r, line.mode)) {
        Some(render_code(format!("{protocol} lines are {}.", only(r)), "E-CMD-008"))
    } else {
        LineOps::check_range(&root.show_data, protocol, id)
            .err()
            .or_else(|| LineOps::wire_clash(&root.show_data, protocol, id))
            .map(|e| lumacore_objects::errors::with_code(&e, lumacore_tree::Coded::code(&e)))
    };
    if let Some(e) = refused {
        if let Some(m) = root.show_data.protocols.lines_mut(protocol) {
            m.insert(id, before);
        }
        return Err(e);
    }
    Ok(())
}

/// A protocol's lines as tree children (`Root.ShowData.Protocols.DMXProtocols.Sacn`).
pub fn lines_children(root: &Root, protocol: &str) -> Vec<DynChild> {
    root.show_data.protocols.lines(protocol).map(btree_to_children).unwrap_or_default()
}

/// One line as a tree object.
pub fn line_view(root: &Root, protocol: &str, id: u32) -> Option<NodeView> {
    root.show_data.protocols.line(protocol, id).map(|t| object_view(id, t))
}

/// The first DMX protocol's line object of this build (the DMX Protocols grid's columns are the
/// ones every DMX line shares, decisions §34 "34f7c hardened" 7).
pub fn any_dmx_line() -> Option<&'static lumacore_objects::objects::ObjectDecl> {
    lumacore_objects::objects::all()
        .find(|o| o.tree.as_ref().is_some_and(|t| t.node == lumacore_objects::tree_path::Node::DmxLines))
        .map(|o| &o.decl)
}

/// `Unassign Sacn 1`: the line carries no universes (Amount 0; the line and its settings stay).
pub fn clear_runs(state: &mut AppState, protocol: &str, ids: &[u32]) -> DispatchResult {
    let mut done = Vec::new();
    for &id in ids {
        if let Some(l) = state.root.show_data.protocols.lines_mut(protocol).and_then(|m| m.get_mut(&id))
            && l.amount > 0
        {
            l.amount = 0;
            done.push(id.to_string());
        }
    }
    if done.is_empty() {
        return out(render_code(
            format!("{protocol} {} carries no universes.", ids.iter().map(u32::to_string).collect::<Vec<_>>().join(" + ")),
            "E-CMD-009",
        ));
    }
    out(render_success(&format!("{protocol} {}: no universes", done.join(" + "))))
}

/// One protocol's line object (decisions §34 "34f7c hardened" 1, 9): its keyword is the row's
/// name, its tree the protocol's node, its properties the kind's shared ones plus its row's,
/// its verbs the one set of line handlers. `extra_verbs` adds a DMX protocol's
/// [`dmx_line_verbs!`].
#[macro_export]
macro_rules! line_object {
    (@props dmx) => { &$crate::props::DMX_LINE_PROPS };
    (@props data) => { &$crate::props::DATA_LINE_PROPS };
    (@node dmx) => { lumacore_objects::tree_path::Node::DmxLines };
    (@node data) => { lumacore_objects::tree_path::Node::DataLines };
    (@store dmx $store:ident, $word:literal, $universe:literal, $path:path) => {
        /// `Store <P> [<n>] ["name"] /Network … /Mode … /LocalUniverse … /Amount … /<Universe> …`.
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "Store", object = $word)]
        pub struct $store {
            #[lum(target)]
            pub target: Option<lumacore_syntax::ast::Target>,
            #[lum(value)]
            pub name: Option<String>,
            #[lum(option = "Network")]
            pub network: Option<lumacore_syntax::ast::Value>,
            #[lum(option = "Mode")]
            pub mode: Option<lumacore_syntax::ast::Value>,
            #[lum(option = "LocalUniverse")]
            pub local: Option<u32>,
            #[lum(option = "Amount")]
            pub amount: Option<u32>,
            #[lum(option = $universe)]
            pub universe: Option<lumacore_syntax::ast::Value>,
        }

        impl lumacore_objects::commands::typed::Command for $store {
            fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                let spec = (|| {
                    let (id, name) = lumacore_objects::commands::set_rows::store_slot(state, &$path, self.target.as_ref(), self.name)?;
                    Ok::<_, String>($crate::lines::LineSpec {
                        id,
                        name,
                        mode: $crate::lines::mode_option(&self.mode)?,
                        network: $crate::lines::option_text(&self.network)?,
                        user: None,
                        port: None,
                        local: self.local,
                        amount: self.amount,
                        universe: $crate::lines::option_text(&self.universe)?,
                    })
                })();
                match spec {
                    Ok(spec) => $crate::lines::line_store($word, state, spec),
                    Err(e) => lumacore_objects::objects::out(e),
                }
            }
        }
    };
    (@store data $store:ident, $word:literal, $universe:literal, $path:path) => {
        /// `Store <P> [<n>] ["name"] /Network … /User … [/Port …] [/Mode …]`.
        #[derive(lumacore_syntax_derive::Args)]
        #[lum(verb = "Store", object = $word)]
        pub struct $store {
            #[lum(target)]
            pub target: Option<lumacore_syntax::ast::Target>,
            #[lum(value)]
            pub name: Option<String>,
            #[lum(option = "Network")]
            pub network: Option<lumacore_syntax::ast::Value>,
            #[lum(option = "Mode")]
            pub mode: Option<lumacore_syntax::ast::Value>,
            #[lum(option = "User")]
            pub user: Option<lumacore_syntax::ast::Value>,
            #[lum(option = "Port")]
            pub port: Option<u32>,
        }

        impl lumacore_objects::commands::typed::Command for $store {
            fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                let spec = (|| {
                    let (id, name) = lumacore_objects::commands::set_rows::store_slot(state, &$path, self.target.as_ref(), self.name)?;
                    Ok::<_, String>($crate::lines::LineSpec {
                        id,
                        name,
                        mode: $crate::lines::mode_option(&self.mode)?,
                        network: $crate::lines::option_text(&self.network)?,
                        user: $crate::lines::option_text(&self.user)?,
                        port: self.port,
                        local: None,
                        amount: None,
                        universe: None,
                    })
                })();
                match spec {
                    Ok(spec) => $crate::lines::line_store($word, state, spec),
                    Err(e) => lumacore_objects::objects::out(e),
                }
            }
        }
    };
    (
        $kind:ident $static:ident, $word:literal, $universe:literal, $path:path,
        door: $door:ident, store: $store:ident, flags: $flags:expr,
        doc: { syntax: $syntax:literal, description: $description:literal, examples: [$($example:literal),* $(,)?] $(,)? },
        extra_verbs: [$($extra:expr),* $(,)?] $(,)?
    ) => {
        $crate::line_object!(@store $kind $store, $word, $universe, $path);

        // The object door (decisions §35): Copy / Cut / Paste / Move / Insert / Compact / Delete.
        lumacore_objects::door_verbs!($door, $word, $path);

        pub mod lines_door {
            /// The protocol's lines on the door: a copy keeps its network (a link); the ID pools
            /// follow every write.
            pub static DOOR: lumacore_objects::pools::door::BTreePool<lumacore_protocols::Line> =
                lumacore_objects::pools::door::BTreePool {
                    object: $word,
                    in_datapool: false,
                    max: lumacore_objects::pools::MAX_NO,
                    map: |s, _| s.root.show_data.protocols.lines_mut($word),
                    map_ref: |s, _| s.root.show_data.protocols.lines($word),
                    name: |l| l.meta.identity.name.clone(),
                    identify: |l, guid| l.meta.identity.guid = guid,
                    rules: lumacore_objects::pools::door::PoolRules {
                        can_delete: |_, _| Ok(()),
                        deleted: |_, _| {},
                        changed: |state| state.root.show_data.rebuild_id_pools(),
                        merge: None,
                        orders: &[],
                    },
                };

            /// `List <P>`: the protocol's lines.
            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "List", object = $word)]
            pub struct List {
                #[lum(target)]
                pub target: Option<lumacore_syntax::ast::Target>,
            }

            impl lumacore_objects::commands::typed::Command for List {
                fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                    lumacore_objects::objects::out(lumacore_objects::commands::list::lines(state, $word))
                }
            }

            /// `Info <P> <n>`: a line; bare, the protocol itself (decisions §34 "34f7c hardened" 6).
            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Info", object = $word)]
            pub struct Info {
                #[lum(target)]
                pub target: Option<lumacore_syntax::ast::Target>,
            }

            impl lumacore_objects::commands::typed::Command for Info {
                fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                    // Without a number the word names the protocol itself.
                    let bare = self.target.as_ref().is_none_or(|t| match t.items.as_slice() {
                        [(None, lumacore_syntax::ast::Item::Path(segs))] => segs.last().is_some_and(|s| s.ids.is_none()),
                        _ => false,
                    });
                    let Some(target) = self.target.as_ref().filter(|_| !bare) else {
                        return lumacore_objects::objects::out(lumacore_objects::commands::info::info_line(state, $word, None));
                    };
                    let Some(object) = lumacore_objects::objects::find($word) else {
                        return lumacore_objects::objects::out(String::new());
                    };
                    match lumacore_objects::resolve::resolve_one(state, target, object) {
                        Ok(r) => {
                            let text: Vec<String> = r
                                .members
                                .iter()
                                .map(|m| lumacore_objects::commands::info::info_line(state, $word, Some(&m.no.to_string())))
                                .collect();
                            lumacore_objects::objects::out(text.join("\n"))
                        }
                        Err(e) => lumacore_objects::objects::out(e),
                    }
                }
            }

            /// Its own settings and protocol universe, from its row (built once).
            pub fn props() -> &'static [lumacore_objects::props::Property] {
                static ALL: std::sync::OnceLock<Vec<lumacore_objects::props::Property>> = std::sync::OnceLock::new();
                ALL.get_or_init(|| {
                    $crate::props::protocol_props(
                        $word,
                        |s, k, n| $crate::props::setting_get(s, $word, k, n),
                        |s, k, n, v| $crate::props::setting_set(s, $word, k, n, v),
                        |s, k| $crate::props::universe_get(s, $word, k),
                        |s, k, v| $crate::props::universe_set(s, $word, k, v),
                    )
                })
            }
        }

        pub static $static: lumacore_objects::objects::ObjectDef = lumacore_objects::objects::ObjectDef {
            decl: lumacore_objects::objects::ObjectDecl {
                keyword: $word,
                lum: lumacore_objects::objects::LumShape::ids(1),
                default_verb: lumacore_objects::objects::Verb::Info,
                flags: $flags,
                aliases: &[],
                doc: lumacore_objects::commands::registry::Doc {
                    display: $word,
                    category: lumacore_objects::commands::help::HelpCategory::Universe,
                    also_applies_to: &[],
                    syntax: $syntax,
                    description: $description,
                    examples: &[$($example),*],
                    related: &["network", "assign", "universe"],
                    alias_depth: lumacore_objects::commands::registry::AliasDepth::Base,
                    hidden: false,
                },
                family: None,
                call_undo: lumacore_objects::commands::registry::Undo::ReadOnly,
                call_class: lumacore_objects::commands::registry::CommandClass::Realtime,
                scope: lumacore_objects::objects::on_master,
                reach: lumacore_objects::objects::show_reach,
                rights: lumacore_objects::rights::setup,
                planned: &[],
            },
            pool: None,
            verbs: &[
                (lumacore_objects::objects::Verb::Store, lumacore_objects::commands::typed::typed::<$store>),
                (lumacore_objects::objects::Verb::Delete, lumacore_objects::commands::typed::typed::<$door::Delete>),
                (lumacore_objects::objects::Verb::Copy, lumacore_objects::commands::typed::typed::<$door::Copy>),
                (lumacore_objects::objects::Verb::Cut, lumacore_objects::commands::typed::typed::<$door::Cut>),
                (lumacore_objects::objects::Verb::Paste, lumacore_objects::commands::typed::typed::<$door::Paste>),
                (lumacore_objects::objects::Verb::Move, lumacore_objects::commands::typed::typed::<$door::Move>),
                (lumacore_objects::objects::Verb::Insert, lumacore_objects::commands::typed::typed::<$door::Insert>),
                (lumacore_objects::objects::Verb::Compact, lumacore_objects::commands::typed::typed::<$door::Compact>),
                (lumacore_objects::objects::Verb::List, lumacore_objects::commands::typed::typed::<lines_door::List>),
                (lumacore_objects::objects::Verb::Info, lumacore_objects::commands::typed::typed::<lines_door::Info>),
                $($extra,)*
            ],
            delete_ask: Some(|s, a| lumacore_objects::commands::door::delete_ask(s, &$path, a)),
            complete: None,
            props: Some(&lumacore_objects::props::Props {
                meta: lumacore_objects::props::MetaLevel::Node,
                lists: &[$crate::line_object!(@props $kind)],
                dynamic: Some(lines_door::props),
                apply: Some(|r, _, id, e| $crate::lines::line_write(r, $word, id, e)),
                ..lumacore_objects::props::Props::BASE
            }),
            tree: Some(lumacore_objects::objects::TreeDef {
                name: Some($word),
                door: Some(&lines_door::DOOR),
                datapool: None,
                node: $crate::line_object!(@node $kind),
                children: |r| $crate::lines::lines_children(r, $word),
                object: |r, id| $crate::lines::line_view(r, $word, id),
            }),
        };
    };
}

/// A DMX protocol's `Assign <P> <n> At Universe 40 Thru 43 [/Universe 9]` (an input run) and
/// `Unassign <P> <n>` (no universes): declared once, for [`line_object!`]'s extra verbs.
#[macro_export]
macro_rules! dmx_line_verbs {
    ($module:ident, $word:literal) => {
        pub mod $module {
            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Assign", object = $word)]
            pub struct Assign {
                #[lum(target)]
                pub target: lumacore_syntax::ast::Target,
                #[lum(at = "destination")]
                pub to: lumacore_syntax::ast::Value,
                #[lum(option = "Universe")]
                pub wire: Option<lumacore_syntax::ast::Value>,
            }

            impl lumacore_objects::commands::typed::Command for Assign {
                fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                    let ids = match &self.to {
                        lumacore_syntax::ast::Value::Target(t) => lumacore_objects::resolve::resolve_one(state, t, &lumacore_objects::objects::UNIVERSE)
                            .map(|r| r.members.iter().map(|m| m.no).collect::<Vec<_>>()),
                        _ => Err(lumacore_objects::errors::render_code(concat!("Assign ", $word, " <n> At Universe …."), "E-CMD-006")),
                    };
                    match (lumacore_objects::objects::line_of(state, &self.target), ids, lumacore_objects::objects::wire_text(&self.wire)) {
                        (Ok((protocol, line)), Ok(ids), Ok(wire)) => lumacore_objects::commands::universe::assign_run(
                            state,
                            protocol,
                            line,
                            &ids,
                            lumacore_protocols::Direction::In,
                            wire.as_deref(),
                        ),
                        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => lumacore_objects::objects::out(e),
                    }
                }
            }

            #[derive(lumacore_syntax_derive::Args)]
            #[lum(verb = "Unassign", object = $word)]
            pub struct Unassign {
                #[lum(target)]
                pub target: lumacore_syntax::ast::Target,
            }

            impl lumacore_objects::commands::typed::Command for Unassign {
                fn run(self, state: &mut lumacore_objects::AppState) -> lumacore_objects::DispatchResult {
                    let Some(object) = lumacore_objects::objects::find($word) else {
                        return lumacore_objects::objects::out(String::new());
                    };
                    match lumacore_objects::resolve::resolve_one(state, &self.target, object) {
                        Ok(r) => {
                            let ids: Vec<u32> = r.members.iter().map(|m| m.no).collect();
                            $crate::lines::clear_runs(state, $word, &ids)
                        }
                        Err(e) => lumacore_objects::objects::out(e),
                    }
                }
            }
        }
    };
}
