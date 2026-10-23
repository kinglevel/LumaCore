// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `fixture_mvr`.
//!
//! Owns:          the unit tests of fixture_mvr.rs: the matrix and its angles.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §34 "34g4 patch hardened" 4, 5.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

fn close(a: [f64; 3], b: [f64; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
}

#[test]
fn the_identity_matrix_reads_zero() {
    let m = Matrix::default();
    assert_eq!(m.rotation_deg(), [0.0; 3]);
    assert_eq!(m.position_m(), [0.0; 3]);
}

#[test]
fn angles_written_read_back_and_keep_the_others() {
    let mut m = Matrix::default();
    m.set_rotation_deg(0, 30.0);
    m.set_rotation_deg(1, -20.0);
    m.set_rotation_deg(2, 135.0);
    assert!(close(m.rotation_deg(), [30.0, -20.0, 135.0]), "{:?}", m.rotation_deg());
    m.set_rotation_deg(1, 10.0);
    assert!(close(m.rotation_deg(), [30.0, 10.0, 135.0]), "{:?}", m.rotation_deg());
}

#[test]
fn a_turn_about_z_moves_the_x_axis_to_y() {
    let mut m = Matrix::default();
    m.set_rotation_deg(2, 90.0);
    assert!(close(m.u, [0.0, 1.0, 0.0]), "{:?}", m.u);
    assert!(close(m.w, [0.0, 0.0, 1.0]), "{:?}", m.w);
}

#[test]
fn reading_never_changes_the_matrix_and_scale_is_kept() {
    // A hang straight down (y = -90°) with a scale: reading is exact, writing keeps the scale.
    let mut m = Matrix { u: [0.0, 0.0, 2.0], v: [0.0, 2.0, 0.0], w: [-2.0, 0.0, 0.0], o: [1500.0, -250.0, 6000.0] };
    let before = m;
    let _ = m.rotation_deg();
    assert_eq!(m, before);
    assert_eq!(m.position_m(), [1.5, -0.25, 6.0]);
    m.set_rotation_deg(2, 0.0);
    assert!((m.u.iter().map(|e| e * e).sum::<f64>().sqrt() - 2.0).abs() < 1e-9);
    m.set_position_m(2, 7.25);
    assert_eq!(m.o, [1500.0, -250.0, 7250.0]);
}

#[test]
fn custom_id_types_are_mvrs_numbers() {
    assert_eq!(IdType::Houselights.number(), 4);
    assert_eq!(IdType::from_number(10), IdType::Marker);
    assert_eq!(IdType::from_number(3), IdType::Generic); // the spec skips 3; Generic takes it (decisions §40 10)
    assert_eq!(IdType::from_number(11), IdType::None);
    assert_eq!(IdType::NAMES.len(), 11);
}
