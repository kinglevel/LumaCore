// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// FILE: tests

//! Tests of `listener`.
//!
//! Owns:          the unit tests of listener.rs.
//! Doesn't own:   line-in / reply-out tests (the crate's tests/it/).
//! Entry points:  the #[test] fns.
//! Decisions:     §11, §31 "OSC line users", §34 22.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

use super::*;

/// Build a minimal OSC message for testing.
fn build_osc_message(address: &str, type_tag: &str, args: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();

    // Address (null-terminated, 4-byte aligned)
    packet.extend_from_slice(address.as_bytes());
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    // Type tag string (null-terminated, 4-byte aligned)
    packet.push(b',');
    packet.extend_from_slice(type_tag.as_bytes());
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    // Arguments
    packet.extend_from_slice(args);

    packet
}

fn build_osc_string_arg(s: &str) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(s.as_bytes());
    data.push(0);
    while data.len() % 4 != 0 {
        data.push(0);
    }
    data
}

#[test]
fn parse_command_message() {
    let args = build_osc_string_arg("fixture 1 at 80");
    let packet = build_osc_message("/lumacore/cmd", "s", &args);

    let event = parse_osc_packet(&packet).expect("should parse");
    match event {
        InboundEvent::Command(cmd) => assert_eq!(cmd, "fixture 1 at 80"),
        _ => panic!("expected Command variant"),
    }
}

#[test]
fn parse_attribute_set_message() {
    let value: f32 = 0.75;
    let args = value.to_be_bytes().to_vec();
    let packet = build_osc_message("/lumacore/fixture/42/Dimmer", "f", &args);

    let event = parse_osc_packet(&packet).expect("should parse");
    match event {
        InboundEvent::AttributeSet {
            fixture_id,
            attribute,
            value,
        } => {
            assert_eq!(fixture_id, 42);
            assert_eq!(attribute, "Dimmer");
            assert!((value - 0.75).abs() < 0.001);
        }
        _ => panic!("expected AttributeSet variant"),
    }
}

#[test]
fn parse_transport_go() {
    let packet = build_osc_message("/lumacore/go", "", &[]);
    let event = parse_osc_packet(&packet).expect("should parse");
    match event {
        InboundEvent::TransportControl(TransportAction::Go) => {}
        _ => panic!("expected TransportControl::Go"),
    }
}

#[test]
fn parse_transport_back() {
    let packet = build_osc_message("/lumacore/back", "", &[]);
    let event = parse_osc_packet(&packet).expect("should parse");
    match event {
        InboundEvent::TransportControl(TransportAction::Back) => {}
        _ => panic!("expected TransportControl::Back"),
    }
}

#[test]
fn parse_transport_stop() {
    let packet = build_osc_message("/lumacore/stop", "", &[]);
    let event = parse_osc_packet(&packet).expect("should parse");
    match event {
        InboundEvent::TransportControl(TransportAction::Stop) => {}
        _ => panic!("expected TransportControl::Stop"),
    }
}

#[test]
fn parse_unknown_address_returns_none() {
    let packet = build_osc_message("/unknown/path", "", &[]);
    assert!(parse_osc_packet(&packet).is_none());
}
