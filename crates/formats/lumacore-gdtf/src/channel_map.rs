//! Flat channel map views for DMX mode display.
//!
//! Provides two complementary views of a mode's channels, both required by
//! the locked decision "Both views available: grouped by attribute type AND
//! raw DMX offset order".
//!
//! - `by_dmx_offset`: channels sorted by DMX offset — matches physical wiring
//! - `grouped_by_type`: channels grouped by `AttributeType` — matches typical
//!   programmer view (all dimmers together, all colors together, etc.)

use lumacore_fixtures::{AttributeType, ChannelFunction, Mode};
use std::collections::BTreeMap;

/// A flat channel map for display and introspection of a DMX mode.
///
/// Neither view owns the channels — both return references into the `Mode`.
pub struct ChannelMap;

impl ChannelMap {
    /// Returns all channel functions sorted by DMX offset (ascending).
    ///
    /// Virtual channels (no physical offset) are placed at the end, sorted
    /// by attribute name. This preserves physical wiring order for display
    /// while keeping virtual channels visible.
    pub fn by_dmx_offset(mode: &Mode) -> Vec<&ChannelFunction> {
        let mut physical: Vec<&ChannelFunction> = mode
            .channel_functions
            .iter()
            .filter(|cf| !cf.is_virtual)
            .collect();
        physical.sort_by_key(|cf| cf.dmx_offset);

        let mut virtual_chs: Vec<&ChannelFunction> = mode
            .channel_functions
            .iter()
            .filter(|cf| cf.is_virtual)
            .collect();
        virtual_chs.sort_by_key(|cf| cf.attribute_name.as_str());

        physical.extend(virtual_chs);
        physical
    }

    /// Returns channel functions grouped by `AttributeType`.
    ///
    /// The `BTreeMap` key is the `AttributeType` (ordered by debug string for
    /// deterministic display). Within each group, channels are sorted by DMX offset.
    ///
    /// Virtual channels are grouped with their attribute type. Unresolved virtual
    /// channels appear in the appropriate group alongside physical channels of
    /// the same type.
    pub fn grouped_by_type(mode: &Mode) -> BTreeMap<String, Vec<&ChannelFunction>> {
        let mut groups: BTreeMap<String, Vec<&ChannelFunction>> = BTreeMap::new();

        for cf in &mode.channel_functions {
            let type_key = attribute_type_key(&cf.attribute_type);
            groups.entry(type_key).or_default().push(cf);
        }

        // Sort within each group by DMX offset (physical first, then virtual)
        for group in groups.values_mut() {
            group.sort_by(|a, b| match (a.is_virtual, b.is_virtual) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.dmx_offset.cmp(&b.dmx_offset),
            });
        }

        groups
    }
}

/// Convert an `AttributeType` to a stable string key for grouping.
///
/// Using a string key rather than the enum directly allows `BTreeMap` ordering
/// and avoids needing `Ord` on `AttributeType`.
fn attribute_type_key(attr_type: &AttributeType) -> String {
    match attr_type {
        AttributeType::Dimmer => "Dimmer".to_string(),
        AttributeType::Color => "Color".to_string(),
        AttributeType::ColorRed => "Color.Red".to_string(),
        AttributeType::ColorGreen => "Color.Green".to_string(),
        AttributeType::ColorBlue => "Color.Blue".to_string(),
        AttributeType::ColorWhite => "Color.White".to_string(),
        AttributeType::ColorAmber => "Color.Amber".to_string(),
        AttributeType::Pan => "Position.Pan".to_string(),
        AttributeType::Tilt => "Position.Tilt".to_string(),
        AttributeType::Zoom => "Beam.Zoom".to_string(),
        AttributeType::Focus => "Beam.Focus".to_string(),
        AttributeType::Iris => "Beam.Iris".to_string(),
        AttributeType::Frost => "Beam.Frost".to_string(),
        AttributeType::Gobo => "Gobo".to_string(),
        AttributeType::Shutter => "Shutter".to_string(),
        AttributeType::Speed => "Speed".to_string(),
        AttributeType::Effect => "Effect".to_string(),
        AttributeType::Macro => "Macro".to_string(),
        AttributeType::Control => "Control".to_string(),
        AttributeType::NoFeature => "NoFeature".to_string(),
        AttributeType::Unknown(name) => format!("Unknown.{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumacore_base::{NodeMeta};
    use lumacore_fixtures::{AttributeType, ChannelFunction, Mode};

    fn make_mode(channels: Vec<ChannelFunction>) -> Mode {
        let footprint: u16 = channels
            .iter()
            .filter(|cf| !cf.is_virtual)
            .map(|cf| cf.byte_count as u16)
            .sum();
        Mode {
            meta: NodeMeta::new(Some("TestMode".to_string())),
            channel_functions: channels,
            dmx_footprint: footprint,
            geometry: String::new(),
            cells: Vec::new(),
            relations: Vec::new(),
        }
    }

    fn ch(name: &str, attr: AttributeType, offset: u16, is_virtual: bool) -> ChannelFunction {
        ChannelFunction {
            attribute_name: name.to_string(),
            attribute_type: attr,
            dmx_offset: offset,
            byte_count: 1,
            default_value: 0,
            default_fraction: None,
            physical_range: None,
            is_virtual,
            resolved: !is_virtual,
            ..Default::default()
        }
    }

    #[test]
    fn by_dmx_offset_sorts_physical_channels_ascending() {
        let mode = make_mode(vec![
            ch("Tilt", AttributeType::Tilt, 5, false),
            ch("Dimmer", AttributeType::Dimmer, 0, false),
            ch("Pan", AttributeType::Pan, 3, false),
        ]);

        let sorted = ChannelMap::by_dmx_offset(&mode);
        assert_eq!(sorted[0].attribute_name, "Dimmer");
        assert_eq!(sorted[1].attribute_name, "Pan");
        assert_eq!(sorted[2].attribute_name, "Tilt");
    }

    #[test]
    fn by_dmx_offset_places_virtual_channels_at_end() {
        let mode = make_mode(vec![
            ch("Dimmer", AttributeType::Dimmer, 0, false),
            ch("VirtualDimmer", AttributeType::Dimmer, 0, true),
            ch("Pan", AttributeType::Pan, 1, false),
        ]);

        let sorted = ChannelMap::by_dmx_offset(&mode);
        assert_eq!(sorted.len(), 3);
        // virtual should be last
        assert!(sorted[2].is_virtual, "virtual channel should be at end");
    }

    #[test]
    fn grouped_by_type_groups_channels_correctly() {
        let mode = make_mode(vec![
            ch("Dimmer1", AttributeType::Dimmer, 0, false),
            ch("Pan", AttributeType::Pan, 1, false),
            ch("Tilt", AttributeType::Tilt, 2, false),
            ch("Dimmer2", AttributeType::Dimmer, 3, false),
        ]);

        let grouped = ChannelMap::grouped_by_type(&mode);
        assert!(grouped.contains_key("Dimmer"), "should have Dimmer group");
        assert_eq!(grouped["Dimmer"].len(), 2, "should have 2 dimmer channels");
        assert!(
            grouped.contains_key("Position.Pan"),
            "should have Pan group"
        );
        assert!(
            grouped.contains_key("Position.Tilt"),
            "should have Tilt group"
        );
    }

    #[test]
    fn grouped_by_type_empty_mode_returns_empty_map() {
        let mode = make_mode(vec![]);
        let grouped = ChannelMap::grouped_by_type(&mode);
        assert!(grouped.is_empty());
    }

    #[test]
    fn by_dmx_offset_empty_mode_returns_empty_vec() {
        let mode = make_mode(vec![]);
        let sorted = ChannelMap::by_dmx_offset(&mode);
        assert!(sorted.is_empty());
    }
}
