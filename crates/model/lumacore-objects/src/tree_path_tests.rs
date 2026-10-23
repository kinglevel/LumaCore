use crate::tree_path::*;

#[test]
fn every_name_is_one_child_of_its_parent() {
    for &node in Node::ALL {
        // A named node's shape is reached by an entry's name, never by its own.
        if node.is_named_shape() {
            continue;
        }
        if let Some(parent) = node.parent() {
            assert_eq!(
                parent.child(node.name()),
                Some(node),
                "{} is ambiguous",
                node.path()
            );
        }
    }
}

#[test]
fn a_protocol_is_a_named_entry_of_its_group() {
    let p = TreePath::parse_dot("Root.ShowData.Protocols.DMXProtocols.Sacn.1").unwrap();
    assert_eq!(p.to_string(), "Root.ShowData.Protocols.DMXProtocols.Sacn.1");
    assert_eq!(p.named(), Some("Sacn"));
    assert_eq!(p.node(), Node::DmxLines);
    assert_eq!(p.entry(), Some(1));
    // The shape isn't a child: `Lines` names a protocol of that name, which none is.
    assert_eq!(Node::DmxProtocols.children().count(), 0);
    assert_eq!(TreePath::parse_dot("Root.ShowData.Protocols.DMXProtocols.1"), None, "a protocol has a name, not a number");
}

#[test]
fn paths_parse_and_print() {
    let p = TreePath::parse_dot("Root.ShowData.DataPools.2.Groups.3").unwrap();
    assert_eq!(p.to_string(), "Root.ShowData.DataPools.2.Groups.3");
    assert_eq!(
        p.schema_string(),
        "Root.ShowData.DataPools.DataPool.Groups.3"
    );
    assert_eq!(p.node(), Node::Groups);
    assert_eq!(p.entry(), Some(3));
    assert_eq!(p.datapool(), Some(2));

    let p = TreePath::parse_dot("root.showdata.devices.dmx.patch").unwrap();
    assert_eq!(p.to_string(), "Root.ShowData.Devices.Dmx.Patch");
    assert_eq!(p.node(), Node::Patch);
    assert_eq!(
        TreePath::parse_dot("ShowData.Devices").unwrap().node(),
        Node::Devices
    );
    assert_eq!(TreePath::parse_dot("Root").unwrap(), TreePath::root());

    // The reference's schema spelling names the same node.
    let schema = TreePath::parse_dot("Root.ShowData.DataPools.DataPool.Groups").unwrap();
    assert_eq!(schema.node(), Node::Groups);
    assert_eq!(schema.datapool(), None);
    assert_eq!(Node::Groups.tree_path(), schema);
    assert_eq!(
        Node::Groups.path(),
        "Root.ShowData.DataPools.DataPool.Groups"
    );

    let user = TreePath::parse_dot("Root.Runtime.Users.1.Screens.2.Windows.3").unwrap();
    assert_eq!(user.node(), Node::ScreenWindow);
    assert_eq!(user.numbered(Node::LoggedIn), Some(1));
    assert_eq!(user.numbered(Node::UserScreens), Some(2));
    assert_eq!(user.numbered(Node::ScreenWindows), Some(3));

    let drive = TreePath::parse_dot("Root.Drives.home.show.luma").unwrap();
    assert_eq!(drive.node(), Node::Drives);
    assert_eq!(drive.to_string(), "Root.Drives.home.show.luma");

    for bad in [
        "Root.Nothing",
        "Root.ShowData.Devices.Dmx.Patch.Fixtures.1.2",
        "Root.ShowData.Devices.Dmx.Patch.Fixtures.1.Fixtures",
        "Root.ShowData.99",
    ] {
        assert_eq!(TreePath::parse_dot(bad), None, "{bad}");
    }
    assert_eq!(
        TreePath::parse_dot("Root.ShowData.DataPools.1")
            .unwrap()
            .parent()
            .node(),
        Node::DataPools
    );
}

/// Where each piece of show data lives in the tree. Exhaustive: a new ShowData field (a new
/// `ShowSubtree`, decisions §21) doesn't compile here until it has a node or a reason.
fn subtree_node(subtree: &lumacore_base::undo::ShowSubtree) -> Option<Node> {
    use lumacore_base::undo::ShowSubtree as S;
    match subtree {
        S::Devices => Some(Node::Devices),
        S::DataPool(_) => Some(Node::DataPool),
        S::DatapoolPriorityOrders => Some(Node::DatapoolPriorityOrders),
        S::ShowSettings => Some(Node::ShowSettings),
        S::Spatial3D => Some(Node::Spatial3D),
        S::Playback => Some(Node::Playback),
        S::Playbacks => Some(Node::Playbacks),
        S::Profile(_) => Some(Node::UserProfiles),
        S::User(_) | S::Programmer(_) | S::Selection(_) | S::Usage(_) | S::Bookmarks(_) => Some(Node::Users),
        S::Protocols => Some(Node::Protocols),
        S::EffectEngines => Some(Node::EffectEngines),
        S::Variables => Some(Node::Variables),
        S::Images => Some(Node::Images),
        S::Assets => Some(Node::Assets),
        // Not browsable in the tree yet (roadmap 0b, typed TreePath): the LumaSession record,
        // the attribute tables (GDTF and 12f-a rules) and the station table have no node.
        S::Tags => Some(Node::Tags),
        S::Session | S::AttributeDefs | S::FeatureGroups | S::ActivationGroups | S::AttributeRules | S::Stations => {
            None
        }
    }
}

#[test]
fn every_placed_subtree_is_under_show_data() {
    use lumacore_base::undo::ShowSubtree as S;
    let g = uuid::Uuid::nil();
    for subtree in [
        S::Devices,
        S::DataPool(g),
        S::DatapoolPriorityOrders,
        S::ShowSettings,
        S::Spatial3D,
        S::Playback,
        S::Playbacks,
        S::Session,
        S::Profile(g),
        S::User(g),
        S::Programmer(g),
        S::Selection(g),
        S::Usage(g),
        S::Bookmarks(g),
        S::Protocols,
        S::EffectEngines,
        S::Variables,
        S::Images,
        S::Assets,
        S::AttributeDefs,
        S::FeatureGroups,
        S::ActivationGroups,
        S::Stations,
    ] {
        if let Some(node) = subtree_node(&subtree) {
            assert!(
                node.is_under(Node::ShowData),
                "{subtree:?} → {}",
                node.path()
            );
            assert_eq!(
                TreePath::parse_dot(&node.path()).map(|p| p.node()),
                Some(node)
            );
        }
    }
}

/// Only `Root.Drives` reaches the disk (decisions §25), and the syntactic check reach uses
/// agrees with the table.
#[test]
fn only_drives_reaches_the_disk() {
    let disk: Vec<Node> = Node::ALL
        .iter()
        .copied()
        .filter(|n| n.kind() == Kind::Disk)
        .collect();
    assert_eq!(disk, vec![Node::Drives]);
    assert!(names_drive(&Node::Drives.path()));
    assert!(names_drive("root.drives.home.x.luma"));
    for &node in Node::ALL {
        assert_eq!(
            names_drive(&node.path()),
            node.is_under(Node::Drives),
            "{}",
            node.path()
        );
    }
}

#[test]
fn aliases_name_declared_nodes() {
    for (alias, node) in ALIASES {
        assert_eq!(crate::tree_path::alias(alias), Some(*node));
        assert_eq!(
            TreePath::parse_dot(&node.path()).map(|p| p.node()),
            Some(*node)
        );
    }
    assert_eq!(slashed("Root.ShowData.Devices"), "/ShowData/Devices");
    assert_eq!(slashed("Root"), "/");
    assert_eq!(dot(&[]), "Root");
    assert_eq!(strip_root("root.ShowData"), Some("ShowData"));
    assert_eq!(strip_root("ShowData.Devices"), None);
}

/// The tree follows the profile's split (§30 "Users & Profiles follows
/// storage"): `….UserProfiles.<n>.Settings | Content`.
#[test]
fn a_profiles_tree_has_settings_and_content() {
    let mut state = crate::AppState::new();
    let mut out = |line: &str| match crate::commands::dispatch(&mut state, line) {
        crate::DispatchResult::Output(s) => crate::text::strip_ansi(&s),
        _ => String::new(),
    };
    let s = out("List Root.ShowData.UserProfiles.2.Content");
    assert!(s.contains("Attributes") || s.contains("attributes"), "{s}");
    let s = out("List Root.ShowData.UserProfiles.2.Settings");
    assert!(s.contains("Preferences") || s.contains("preferences"), "{s}");
}

/// Every node has a number, its place among its parent's declared children (decisions §33 54);
/// a number in a path is that child, entries of a collection keep their member numbers.
#[test]
fn a_number_in_a_path_is_the_childs_no() {
    for node in Node::ALL.iter().copied().filter(|n| n.no().is_some()) {
        let parent = node.parent().unwrap();
        assert_eq!(parent.nth_child(node.no().unwrap()), Some(node), "{}", node.path());
    }
    let show = Node::ShowData.no().unwrap();
    let by_no = TreePath::parse_dot(&format!("Root.{show}")).unwrap();
    assert_eq!(by_no.node(), Node::ShowData);
    let devices = Node::Devices.no().unwrap();
    assert_eq!(TreePath::parse_dot(&format!("Root.{show}.{devices}")).unwrap().node(), Node::Devices);
    // A collection's numbers stay its members'.
    assert_eq!(TreePath::parse_dot("Root.ShowData.Devices.Dmx.Patch.Fixtures.7").unwrap().entry(), Some(7));
    // A numbered entry's declared children too (`DataPools.1.2`).
    let groups = Node::Groups.no().unwrap();
    let p = TreePath::parse_dot(&format!("Root.ShowData.DataPools.1.{groups}")).unwrap();
    assert_eq!((p.node(), p.datapool()), (Node::Groups, Some(1)));
    let past = Node::ShowData.children().count() + 1;
    assert_eq!(TreePath::parse_dot(&format!("Root.ShowData.{past}")), None);
}
