use lumacore_objects::AppState;

use crate::defs::*;

/// An OSC input line listens as its user (decisions §11).
fn step_osc_listener_binds(_state: &mut AppState) -> Result<(), String> {
    let (tx, _rx) = crossbeam_channel::bounded::<lumacore_base::InboundEvent>(16);
    let mut l = lumacore_osc::OscListener::start(
        std::net::Ipv4Addr::LOCALHOST,
        0,
        "Admin",
        tx,
        None,
    )?;
    let t = l.stop().ok_or("listener thread missing")?;
    t.join().map_err(|_| "listener thread panicked".to_string())
}

fn step_osc_parse_command(_state: &mut AppState) -> Result<(), String> {
    use lumacore_base::InboundEvent;

    let packet = build_osc_test_message("/lumacore/cmd", "s", "fixture 1 at 80");
    let event = parse_osc_test_packet(&packet).ok_or("should parse /lumacore/cmd message")?;

    match event {
        InboundEvent::Command(cmd) => {
            if cmd != "fixture 1 at 80" {
                return Err(format!("wrong command: {cmd}"));
            }
        }
        _ => return Err("expected Command variant".into()),
    }

    Ok(())
}

fn step_osc_parse_transport(_state: &mut AppState) -> Result<(), String> {
    use lumacore_base::{InboundEvent, TransportAction};

    for (addr, expected) in [
        ("/lumacore/go", TransportAction::Go),
        ("/lumacore/back", TransportAction::Back),
        ("/lumacore/stop", TransportAction::Stop),
    ] {
        let packet = build_osc_test_message(addr, "", "");
        let event = parse_osc_test_packet(&packet).ok_or_else(|| format!("should parse {addr}"))?;

        match event {
            InboundEvent::TransportControl(action) => {
                if action != expected {
                    return Err(format!("wrong action for {addr}: {action:?}"));
                }
            }
            _ => return Err(format!("expected TransportControl for {addr}")),
        }
    }

    Ok(())
}

fn step_osc_parse_attribute_set(_state: &mut AppState) -> Result<(), String> {
    use lumacore_base::InboundEvent;

    let value: f32 = 0.5;
    let mut packet = Vec::new();

    let addr = "/lumacore/fixture/7/Dimmer";
    packet.extend_from_slice(addr.as_bytes());
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    packet.push(b',');
    packet.push(b'f');
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    packet.extend_from_slice(&value.to_be_bytes());

    let event = parse_osc_test_packet(&packet).ok_or("should parse fixture attribute message")?;

    match event {
        InboundEvent::AttributeSet {
            fixture_id,
            attribute,
            value: v,
        } => {
            if fixture_id != 7 {
                return Err(format!("wrong fixture_id: {fixture_id}"));
            }
            if attribute != "Dimmer" {
                return Err(format!("wrong attribute: {attribute}"));
            }
            if (v - 0.5).abs() > 0.001 {
                return Err(format!("wrong value: {v}"));
            }
        }
        _ => return Err("expected AttributeSet variant".into()),
    }

    Ok(())
}

fn build_osc_test_message(address: &str, type_tag: &str, string_arg: &str) -> Vec<u8> {
    let mut packet = Vec::new();

    packet.extend_from_slice(address.as_bytes());
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    packet.push(b',');
    packet.extend_from_slice(type_tag.as_bytes());
    packet.push(0);
    while packet.len() % 4 != 0 {
        packet.push(0);
    }

    if type_tag.starts_with('s') && !string_arg.is_empty() {
        packet.extend_from_slice(string_arg.as_bytes());
        packet.push(0);
        while packet.len() % 4 != 0 {
            packet.push(0);
        }
    }

    packet
}

fn parse_osc_test_packet(data: &[u8]) -> Option<lumacore_base::InboundEvent> {
    use lumacore_base::{InboundEvent, TransportAction};

    let addr_end = data.iter().position(|&b| b == 0)?;
    let address = std::str::from_utf8(&data[..addr_end]).ok()?;
    let addr_padded = (addr_end + 4) & !3;

    if addr_padded >= data.len() {
        return match address {
            "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
            "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
            "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
            _ => None,
        };
    }

    let type_tag_start = addr_padded;
    if data[type_tag_start] != b',' {
        return match address {
            "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
            "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
            "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
            _ => None,
        };
    }

    let type_tag_end = data[type_tag_start..].iter().position(|&b| b == 0)? + type_tag_start;
    let type_tags = std::str::from_utf8(&data[type_tag_start + 1..type_tag_end]).ok()?;
    let args_start = (type_tag_end + 4) & !3;

    if address == "/lumacore/cmd" && type_tags.starts_with('s') {
        let str_end = data[args_start..].iter().position(|&b| b == 0)? + args_start;
        let s = std::str::from_utf8(&data[args_start..str_end]).ok()?;
        return Some(InboundEvent::Command(s.to_string()));
    }

    if let Some(rest) = address.strip_prefix("/lumacore/fixture/") {
        let (id_str, attr) = rest.split_once('/')?;

        let fixture_id: u32 = id_str.parse().ok()?;
        if type_tags.starts_with('f') && args_start + 4 <= data.len() {
            let bytes: [u8; 4] = data[args_start..args_start + 4].try_into().ok()?;
            let value = f32::from_be_bytes(bytes);
            return Some(InboundEvent::AttributeSet {
                fixture_id,
                attribute: attr.to_string(),
                value: value as f64,
            });
        }
    }

    match address {
        "/lumacore/go" => Some(InboundEvent::TransportControl(TransportAction::Go)),
        "/lumacore/back" => Some(InboundEvent::TransportControl(TransportAction::Back)),
        "/lumacore/stop" => Some(InboundEvent::TransportControl(TransportAction::Stop)),
        _ => None,
    }
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "OSC Input Line Listens",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "OscListener binds, runs and stops",
                kind: StepKind::Verify(step_osc_listener_binds),
            }],
        },
        TestDef {
            name: "OSC Parse Command Message",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "/lumacore/cmd <string> parses to InboundEvent::Command",
                kind: StepKind::Verify(step_osc_parse_command),
            }],
        },
        TestDef {
            name: "OSC Parse Transport Controls",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "/lumacore/go|back|stop parse to TransportControl events",
                kind: StepKind::Verify(step_osc_parse_transport),
            }],
        },
        TestDef {
            name: "OSC Parse Attribute Set",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "/lumacore/fixture/<id>/<attr> <float> parses to AttributeSet",
                kind: StepKind::Verify(step_osc_parse_attribute_set),
            }],
        },
    ]
}
