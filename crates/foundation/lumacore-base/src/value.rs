//! The one attribute value (decisions §27): presets, programmer and cues all
//! store a `Value`. Values are physical or normalized (percent) — DMX bytes
//! exist only at patch output.
//!
//! A value has steps; only step 1 renders. The per-step phaser fields
//! (relative, width, transition, accel, decel) and speed / phase / measure
//! are inert until Effects (decisions §28) decides them (R1 Q43). Old value
//! forms are not read: show files only move forward (§20).

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// An absolute value: percent of the attribute's range (0–100) or a value in
/// the attribute's physical unit (degrees, Kelvin, …).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Absolute {
    Normalized(f64),
    Physical(f64),
}

/// One step of a value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Step {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absolute: Option<Absolute>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accel: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decel: Option<f64>,
}

/// One attribute value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub steps: Vec<Step>,
    /// The preset the value came from (a GUID link, decisions §21 "6d").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<uuid::Uuid>,
    /// A release: the attribute goes back to lower sources (cue tracking).
    #[serde(default, skip_serializing_if = "is_false")]
    pub release: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measure: Option<f64>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Value {
    fn with(absolute: Absolute) -> Self {
        Self {
            steps: vec![Step {
                absolute: Some(absolute),
                ..Step::default()
            }],
            preset: None,
            release: false,
            speed: None,
            phase: None,
            measure: None,
        }
    }

    /// A static value in percent of the attribute's range.
    pub fn normalized(percent: f64) -> Self {
        Self::with(Absolute::Normalized(percent))
    }

    /// A static value in the attribute's physical unit.
    pub fn physical(value: f64) -> Self {
        Self::with(Absolute::Physical(value))
    }

    /// A release (`At Release`).
    pub fn release() -> Self {
        Self {
            steps: Vec::new(),
            preset: None,
            release: true,
            speed: None,
            phase: None,
            measure: None,
        }
    }

    /// The absolute value of step 1 (what renders today).
    pub fn absolute(&self) -> Option<Absolute> {
        self.steps.first().and_then(|s| s.absolute)
    }

    /// Step 1 as percent, if it is normalized.
    pub fn percent(&self) -> Option<f64> {
        match self.absolute() {
            Some(Absolute::Normalized(p)) => Some(p),
            _ => None,
        }
    }

    /// Short readout: `50%`, `45.0 phys`, `Release`.
    pub fn readout(&self) -> String {
        if self.release {
            return "Release".to_string();
        }
        match self.absolute() {
            Some(Absolute::Normalized(p)) => format!("{p:.0}%"),
            Some(Absolute::Physical(v)) => format!("{v:.1}"),
            None => "-".to_string(),
        }
    }
}

/// The highest LTP stamp handed out or seen (`next_stamp`, `observe_stamp`).
static LAST: AtomicU64 = AtomicU64::new(0);

/// Raise the stamp floor to `stamp`: every later `next_stamp` is larger.
/// Called for every stamp that arrives from elsewhere (a loaded show, a
/// replicated op, a merge), so a station whose wall clock is behind still
/// makes new touches win over old values (R1-01).
pub fn observe_stamp(stamp: u64) {
    LAST.fetch_max(stamp, Ordering::Relaxed);
}

/// A value's own timing (individual timing; decisions §30 "12f", value timing): Fade and
/// Delay in seconds, `None` = the cue's (its feature group's, then its part's). Carried by a
/// programmer value into the cue it is stored in; used for both In and Out.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValueTiming {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
}

impl ValueTiming {
    /// Nothing of its own: the cue's timing.
    pub fn is_cue(&self) -> bool {
        self.fade.is_none() && self.delay.is_none()
    }
}

/// A strictly increasing time stamp (µs since the epoch) for LTP: the latest
/// touch of an attribute — by a programmer or a playback — wins (§4, §7).
pub fn next_stamp() -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0);
    let mut last = LAST.load(Ordering::Relaxed);
    loop {
        let next = now.max(last + 1);
        match LAST.compare_exchange_weak(last, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(seen) => last = seen,
        }
    }
}

impl crate::Walk for Value {
    fn walk(&self, w: &mut crate::Walker) {
        let Self {
            steps: _,
            preset,
            release: _,
            speed: _,
            phase: _,
            measure: _,
        } = self;
        // An embedded preset: a preset must never end up containing itself.
        w.contains("Preset", "Preset", *preset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_observed_stamp_floors_the_next_one() {
        let ahead = next_stamp() + 3_600_000_000; // one hour in the future
        observe_stamp(ahead);
        assert!(next_stamp() > ahead);
        // A lower stamp never lowers the floor.
        observe_stamp(1);
        assert!(next_stamp() > ahead);
    }

    #[test]
    fn value_round_trips_and_refuses_old_forms() {
        let v = Value::normalized(50.0);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&json).unwrap(), v);
        // Old forms (a DMX byte, the preset enum) aren't read (§20, R1-D5).
        for old in ["255", r#"{"Percent":25.0}"#, r#""Release""#] {
            assert!(serde_json::from_str::<Value>(old).is_err(), "{old}");
        }
    }

    #[test]
    fn stamps_only_go_up() {
        let a = next_stamp();
        let b = next_stamp();
        assert!(b > a);
    }
}
