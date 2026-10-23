// ████████╗ █████╗  ██████╗     ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔══██╗██╔════╝     ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   ███████║██║  ███╗       ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══██║██║   ██║       ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ██║  ██║╚██████╔╝       ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚═╝  ╚═╝ ╚═════╝        ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tag tests

//! Tests of `tag`.
//!
//! Owns:          the unit tests of tag.rs.
//! Doesn't own:   the words' tests (lumacore-cmd-tags, tests/catalog).
//! Entry points:  the #[test] fns.
//! Decisions:     §12 "Tags hardened".

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn p(text: &str) -> TagPath {
    TagPath::parse(text).unwrap()
}

/// All › Lx1 › (Right, Left), All › Lx2 › Right, Busking.
fn rig() -> TagTree {
    let mut t = TagTree::new();
    insert(&mut t, &p("1"), Tag::new(Some("All".into()))).unwrap();
    insert(&mut t, &p("1.1"), Tag::new(Some("Lx1".into()))).unwrap();
    insert(&mut t, &p("1.1.1"), Tag::new(Some("Right".into()))).unwrap();
    insert(&mut t, &p("1.1.2"), Tag::new(Some("Left".into()))).unwrap();
    insert(&mut t, &p("1.2"), Tag::new(Some("Lx2".into()))).unwrap();
    insert(&mut t, &p("1.2.1"), Tag::new(Some("Right".into()))).unwrap();
    insert(&mut t, &p("2"), Tag::new(Some("Busking".into()))).unwrap();
    t
}

#[test]
fn a_path_is_parts_never_a_decimal() {
    assert_eq!(p("1.10").0, vec![1, 10]);
    assert_ne!(p("1.10"), p("1.1"));
    assert_eq!(p("1.2.1").to_string(), "1.2.1");
    for bad in ["", "1.", ".1", "1..2", "0", "1.0", "1.01", "a", "1.-2"] {
        assert!(TagPath::parse(bad).is_err(), "{bad:?} parsed");
    }
}

#[test]
fn a_tag_is_found_by_path_guid_and_name() {
    let t = rig();
    assert_eq!(find(&t, &p("1.1.2")).unwrap().name(), "Left");
    assert!(find(&t, &p("1.3")).is_none());
    let g = find(&t, &p("1.2.1")).unwrap().meta.identity.guid;
    assert_eq!(path_of(&t, g), Some(p("1.2.1")));
    // Names are unique among siblings only: "Right" fits two tags, owner first narrows it.
    assert_eq!(named(&t, None, "right").len(), 2);
    assert!(matches!(named_one(&t, None, "Right"), Err(TagError::Ambiguous { count: 2, .. })));
    assert_eq!(named_one(&t, Some(&p("1.1")), "Right").unwrap(), p("1.1.1"));
    assert_eq!(all(&t).first().map(|(p, _)| p.to_string()), Some("1".into()));
}

#[test]
fn names_are_unique_among_siblings_and_numbers_once() {
    let mut t = rig();
    assert!(matches!(insert(&mut t, &p("1.1.3"), Tag::new(Some("left".into()))), Err(TagError::NameTaken { .. })));
    assert!(matches!(insert(&mut t, &p("1.1.1"), Tag::new(Some("Centre".into()))), Err(TagError::SlotTaken(_))));
    assert!(matches!(insert(&mut t, &p("9.1"), Tag::new(Some("X".into()))), Err(TagError::NoSuchTag(_))));
    insert(&mut t, &p("1.1.3"), Tag::new(Some("Centre".into()))).unwrap();
}

#[test]
fn thru_walks_the_children_of_one_parent() {
    let mut t = rig();
    insert(&mut t, &p("1.4"), Tag::new(Some("Floor".into()))).unwrap();
    let names = |v: Vec<TagPath>| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert_eq!(names(siblings(&t, &p("1.2"), None).unwrap()), ["1.2", "1.4"]);
    assert_eq!(names(siblings(&t, &p("1.1"), Some(&p("1.4"))).unwrap()), ["1.1", "1.2", "1.4"]);
    assert_eq!(names(siblings(&t, &p("1.4"), Some(&p("1.1"))).unwrap()), ["1.4", "1.2", "1.1"]);
    assert!(matches!(siblings(&t, &p("1.2"), Some(&p("2"))), Err(TagError::AcrossParents { .. })));
    assert_eq!(names(siblings(&t, &p("1"), None).unwrap()), ["1", "2"]);
}

#[test]
fn a_move_keeps_the_guid_and_takes_the_next_free_number() {
    let mut t = rig();
    let right = find(&t, &p("1.2.1")).unwrap().meta.identity.guid;
    let lx2 = find(&t, &p("1.2")).unwrap().meta.identity.guid;
    // Lx2 (with its Right) goes under Busking.
    let to = move_under(&mut t, &p("1.2"), Some(&p("2"))).unwrap();
    assert_eq!(to, p("2.1"));
    assert_eq!(path_of(&t, lx2), Some(p("2.1")));
    assert_eq!(path_of(&t, right), Some(p("2.1.1")));
    // To the top.
    assert_eq!(move_under(&mut t, &p("2.1"), None).unwrap(), p("3"));
    // Never under itself.
    assert!(matches!(move_under(&mut t, &p("1"), Some(&p("1.1.1"))), Err(TagError::IntoItself(_))));
    // A sibling with its name is refused: Lx1's Right can't join Lx2's Right at the top level.
    assert!(matches!(move_under(&mut t, &p("1.1.1"), Some(&p("3"))), Err(TagError::NameTaken { .. })));
}

#[test]
fn keep_children_lifts_them_one_level() {
    let mut t = rig();
    let kids = lift_children(&mut t, &p("1.1"));
    assert_eq!(kids.iter().map(|k| k.to_string()).collect::<Vec<_>>(), ["1.3", "1.4"]);
    assert!(find(&t, &p("1.1")).unwrap().children.is_empty());
    // A lifted child whose name is taken at the new level keeps it with a number.
    let mut t = rig();
    insert(&mut t, &p("1.3"), Tag::new(Some("Right".into()))).unwrap();
    lift_children(&mut t, &p("1.1"));
    let names: Vec<&str> = find(&t, &p("1")).unwrap().children.values().map(|c| c.name()).collect();
    assert!(names.contains(&"Right 2"), "{names:?}");
}

#[test]
fn members_come_from_the_identity_walk() {
    let t = rig();
    let lx1 = find(&t, &p("1.1")).unwrap().meta.identity.guid;
    let right = find(&t, &p("1.1.1")).unwrap().meta.identity.guid;
    // A tag carrying a tag is a member like any object.
    let mut tree = t.clone();
    find_mut(&mut tree, &p("2")).unwrap().meta.annotation.tags.push(lx1);
    let mut w = Walker::new();
    w.pool("Tags", &tree);
    let busking = find(&tree, &p("2")).unwrap().meta.identity.guid;
    let found = members(&w, &[lx1]);
    assert_eq!(found, vec![Member { guid: busking, kind: "Tag", via: lx1 }]);
    assert!(members(&w, &[right]).is_empty());
    // Subtree order: the tag first, then what is under it.
    let sub = subtree(find(&tree, &p("1.1")).unwrap());
    assert_eq!(sub.first(), Some(&lx1));
    assert_eq!(sub.len(), 3);
}
