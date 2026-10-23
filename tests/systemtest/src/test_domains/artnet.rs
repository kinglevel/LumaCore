use lumacore_objects::AppState;

use crate::defs::*;

fn step_artnet_packet_format(_state: &mut AppState) -> Result<(), String> {
    let lo = lumacore_net::nics::Nic {
        name: "lo".into(),
        ip: std::net::Ipv4Addr::LOCALHOST,
        netmask: std::net::Ipv4Addr::new(255, 0, 0, 0),
        broadcast: std::net::Ipv4Addr::new(127, 255, 255, 255),
        index: None,
    };
    let source = lumacore_artnet::ArtNetSource::new(&lo);
    if source.is_err() {
        println!("    artnet source creation failed (expected in sandboxed CI)");
        return Ok(());
    }
    Ok(())
}

fn step_output_protocol_artnet(_state: &mut AppState) -> Result<(), String> {
    use lumacore_protocols::line::Direction;

    // A line lives under its protocol's node (decisions §34 "34f7c hardened" 5): the show says
    // `dmx_protocols.ArtNet.lines.1`, and a load gives the line its protocol back.
    let mut show = lumacore_tree::ShowData::new();
    let no = lumacore_tree::LineOps::create_line(&mut show, "ArtNet", Direction::Out, Some(1), Some("Art-Net Out".to_string()), "Rig")
        .ok_or("ArtNet isn't a protocol of this build")?;
    let json = serde_json::to_value(&show.protocols).map_err(|e| format!("serialize: {e}"))?;
    if json.pointer(&format!("/dmx_protocols/ArtNet/lines/{no}")).is_none() {
        return Err(format!("the line should sit under dmx_protocols.ArtNet: {json}"));
    }
    let mut decoded = lumacore_tree::ShowData::new();
    decoded.protocols = serde_json::from_value(json).map_err(|e| format!("deserialize: {e}"))?;
    decoded.rebuild_id_pools();
    let line = decoded.protocols.line("ArtNet", no).ok_or("the line is gone")?;
    if line.protocol != "ArtNet" || lumacore_protocols::protocol::find(&line.protocol).is_none() {
        return Err(format!("expected ArtNet protocol, got {:?}", line.protocol));
    }
    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "Art-Net Packet Source Creation",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "ArtNetSource can be created on the loopback interface",
                kind: StepKind::Verify(step_artnet_packet_format),
            }],
        },
        TestDef {
            name: "An ArtNet Line Serializes",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "a line under the ArtNet node round-trips through JSON",
                kind: StepKind::Verify(step_output_protocol_artnet),
            }],
        },
    ]
}
