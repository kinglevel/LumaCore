// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `render tick`.
//!
//! Owns:          the unit tests of render tick.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §24 "Step 9 hardening", §27.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;
use lumacore_protocols::input::InputLayer;

fn layer(line: u32, coarse: u8, fine: u8) -> InputLayer {
    let mut data = [0u8; 512];
    data[0] = coarse;
    data[1] = fine;
    InputLayer {
        line,
        universe: 1,
        band: InputPriority::Ltp,
        data,
        stamps: std::sync::Arc::new([line as u64; 512]),
    }
}

/// R1-51: a 16-bit channel takes both bytes from one source — HTP on
/// the 16-bit value, never coarse from one line and fine from another.
#[test]
fn a_16_bit_channel_merges_whole() {
    let mut spans = [1u8; 512];
    spans[0] = 2;
    spans[1] = 0;
    let (a, b) = (layer(1, 0x10, 0xFF), layer(2, 0x11, 0x00));
    for mode in [MergeMode::Htp, MergeMode::Prio] {
        let mut out = [0u8; 512];
        let mut won = [Winner::Default; 512];
        merge_universe(&mut out, &mut won, &[Owner::Default; 512], &spans, mode, &[&a, &b]);
        assert_eq!(&out[..2], &[0x11, 0x00], "{mode:?}: B's 0x1100 beats A's 0x10FF");
        assert_eq!((won[0], won[1]), (Winner::Input(2), Winner::Input(2)));
    }
    let mut out = [0u8; 512];
    let mut won = [Winner::Default; 512];
    merge_universe(&mut out, &mut won, &[Owner::Default; 512], &spans, MergeMode::LowTp, &[&a, &b]);
    assert_eq!(&out[..2], &[0, 0], "LoTP keeps our 0x0000");
}

/// Park on the fine byte alone keeps the whole channel from input.
#[test]
fn a_parked_fine_byte_keeps_the_channel() {
    let mut spans = [1u8; 512];
    spans[0] = 2;
    spans[1] = 0;
    let mut owners = [Owner::Default; 512];
    owners[1] = Owner::Park;
    let a = layer(1, 0x10, 0xFF);
    for mode in [MergeMode::Htp, MergeMode::Prio] {
        let mut out = [0u8; 512];
        let mut won = [Winner::Default; 512];
        merge_universe(&mut out, &mut won, &owners, &spans, mode, &[&a]);
        assert_eq!(&out[..2], &[0, 0], "{mode:?}: parked");
    }
}
