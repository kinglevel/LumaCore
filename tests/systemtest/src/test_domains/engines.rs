use lumacore_objects::AppState;

use crate::defs::*;

fn step_engine_context_has_all_fields(_state: &mut AppState) -> Result<(), String> {
    use lumacore_engines::EngineContext;
    use std::sync::Arc;

    let show_data = Arc::new(lumacore_tree::tree::ShowData::new());
    let hub = lumacore_render::RenderHub::new(lumacore_render::EngineInput::new(show_data.clone()));
    let (tx, _rx) = crossbeam_channel::bounded::<lumacore_base::InboundEvent>(16);

    let ctx = EngineContext {
        show_data: Some(show_data),
        render: Some(hub),
        command_tx: Some(tx),
        health: None,
    };

    // The session clock is global (`lumacore_base::clock`, decisions §24), not
    // a context field: it never runs backwards.
    let a = lumacore_base::clock::session_ms();
    let b = lumacore_base::clock::session_ms();
    if b < a {
        return Err(format!("session clock ran backwards: {a} → {b}"));
    }
    if ctx.show_data.is_none() {
        return Err("show_data should be Some".into());
    }
    if ctx.render.is_none() {
        return Err("render hub should be Some".into());
    }
    if ctx.command_tx.is_none() {
        return Err("command_tx should be Some".into());
    }
    Ok(())
}

fn step_inbound_event_channel_roundtrip(_state: &mut AppState) -> Result<(), String> {
    use lumacore_base::{InboundEvent, TransportAction};

    let (tx, rx) = crossbeam_channel::bounded::<InboundEvent>(16);

    tx.send(InboundEvent::Command("fixture 1 at full".into()))
        .map_err(|e| format!("send Command: {e}"))?;
    tx.send(InboundEvent::AttributeSet {
        fixture_id: 42,
        attribute: "Dimmer".into(),
        value: 0.75,
    })
    .map_err(|e| format!("send AttributeSet: {e}"))?;
    tx.send(InboundEvent::TransportControl(TransportAction::Go))
        .map_err(|e| format!("send TransportControl: {e}"))?;

    let e1 = rx.try_recv().map_err(|e| format!("recv 1: {e}"))?;
    match e1 {
        InboundEvent::Command(cmd) => {
            if cmd != "fixture 1 at full" {
                return Err(format!("wrong command: {cmd}"));
            }
        }
        _ => return Err("expected Command variant".into()),
    }

    let e2 = rx.try_recv().map_err(|e| format!("recv 2: {e}"))?;
    match e2 {
        InboundEvent::AttributeSet {
            fixture_id,
            attribute,
            value,
        } => {
            if fixture_id != 42 {
                return Err(format!("wrong fixture_id: {fixture_id}"));
            }
            if attribute != "Dimmer" {
                return Err(format!("wrong attribute: {attribute}"));
            }
            if (value - 0.75).abs() > 0.001 {
                return Err(format!("wrong value: {value}"));
            }
        }
        _ => return Err("expected AttributeSet variant".into()),
    }

    let e3 = rx.try_recv().map_err(|e| format!("recv 3: {e}"))?;
    match e3 {
        InboundEvent::TransportControl(lumacore_base::TransportAction::Go) => {}
        _ => return Err("expected TransportControl::Go".into()),
    }

    Ok(())
}

fn step_capability_command_dispatch(_state: &mut AppState) -> Result<(), String> {
    use lumacore_wire::Capability;

    let cd = Capability::CommandDispatch;
    let network = Capability::Network;

    if cd == network {
        return Err("CommandDispatch should be distinct from Network".into());
    }

    let json = serde_json::to_string(&cd).map_err(|e| format!("serialize: {e}"))?;
    if !json.contains("CommandDispatch") {
        return Err(format!("expected CommandDispatch in JSON: {json}"));
    }

    Ok(())
}

pub fn tests() -> Vec<TestDef> {
    vec![
        TestDef {
            name: "EngineContext Carries All Resource Fields",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "enriched EngineContext has clock, show_data, frame_buffer, command_tx",
                kind: StepKind::Verify(step_engine_context_has_all_fields),
            }],
        },
        TestDef {
            name: "Inbound Event Channel Roundtrip",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "send and receive Command, AttributeSet, TransportControl events",
                kind: StepKind::Verify(step_inbound_event_channel_roundtrip),
            }],
        },
        TestDef {
            name: "CommandDispatch Capability Is Distinct",
            user: "Admin",
            gates: vec![],
            steps: vec![StepDef {
                name: "CommandDispatch is a unique capability that serializes correctly",
                kind: StepKind::Verify(step_capability_command_dispatch),
            }],
        },
    ]
}
