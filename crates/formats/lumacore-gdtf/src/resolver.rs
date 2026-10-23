//! Two-pass virtual channel resolver for GDTF modes.
//!
//! GDTF fixtures can define "virtual channels" — channels with no DMX offset
//! that reference a geometry rather than a direct DMX slot. These must be
//! resolved against the physical channels in the same mode.
//!
//! The resolver uses two passes:
//! 1. Collect all physical channels (those with a DMX offset) by
//!    (geometry, attribute).
//! 2. Resolve virtual channels (those with no offset) against the physical
//!    channel of the same attribute on the same geometry.
//!
//! Single-layer limitation: if the target of a virtual channel is itself
//! virtual, resolution fails (the channel is marked unresolved and a warning
//! is logged). Multi-layer virtual channel chains are not supported in Phase 1.

use lumacore_fixtures::Mode;
use std::collections::HashMap;

/// Result of running the virtual channel resolver on a mode.
#[derive(Debug, Clone, Copy)]
pub struct ResolutionResult {
    /// Number of virtual channels that were successfully resolved.
    pub resolved: usize,
    /// Number of virtual channels that could not be resolved.
    pub unresolved: usize,
}

/// Two-pass virtual channel resolver.
///
/// Created once per import, used by the importer to resolve virtual channels
/// in each DMX mode after physical channels are processed.
pub struct VirtualChannelResolver;

impl VirtualChannelResolver {
    /// Run the two-pass resolver on a mode's channel functions.
    ///
    /// Pass 1: collect physical channel indices (attribute_name -> index).
    /// Pass 2: for each virtual channel, look up its geometry reference in
    ///         the physical map and mark it resolved.
    ///
    /// Returns a `ResolutionResult` with counts of resolved/unresolved channels.
    ///
    /// # Limitations
    ///
    /// TODO: Single-layer virtual channel limitation — if the target of a
    /// virtual channel is itself virtual, the resolution fails and the channel
    /// is marked unresolved. Multi-layer chains (virtual -> virtual -> physical)
    /// would require a recursive resolution pass and are deferred to a future
    /// version.
    pub fn resolve(mode: &mut Mode) -> ResolutionResult {
        // Pass 1: physical channels by (geometry, attribute) — every cell's
        // Dimmer is its own (decisions §29 "11b hardening").
        let physical: HashMap<(String, String), usize> = mode
            .channel_functions
            .iter()
            .enumerate()
            .filter(|(_, cf)| !cf.is_virtual)
            .map(|(idx, cf)| ((cf.geometry.clone(), cf.attribute_name.clone()), idx))
            .collect();

        // Pass 2: resolve virtual channels on the same geometry (a virtual
        // channel without a geometry matches by attribute alone).
        let mut resolved = 0usize;
        let mut unresolved = 0usize;
        for idx in 0..mode.channel_functions.len() {
            let cf = &mode.channel_functions[idx];
            if !cf.is_virtual || cf.resolved {
                continue;
            }
            let found = if cf.geometry.is_empty() {
                physical.keys().any(|(_, a)| *a == cf.attribute_name)
            } else {
                physical.contains_key(&(cf.geometry.clone(), cf.attribute_name.clone()))
            };
            if found {
                mode.channel_functions[idx].resolved = true;
                resolved += 1;
            } else {
                // Missing, or the target is itself virtual (one layer only).
                unresolved += 1;
                log::warn!(
                    "Virtual channel '{}' on '{}' could not be resolved in mode '{}'",
                    cf.attribute_name,
                    cf.geometry,
                    mode.meta.identity.name.as_deref().unwrap_or("<unnamed>")
                );
            }
        }

        ResolutionResult {
            resolved,
            unresolved,
        }
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

    fn physical(name: &str, offset: u16) -> ChannelFunction {
        ChannelFunction {
            attribute_name: name.to_string(),
            attribute_type: AttributeType::Unknown(name.to_string()),
            dmx_offset: offset,
            byte_count: 1,
            default_value: 0,
            default_fraction: None,
            physical_range: None,
            is_virtual: false,
            resolved: false,
            ..Default::default()
        }
    }

    fn virtual_ch(name: &str) -> ChannelFunction {
        ChannelFunction {
            attribute_name: name.to_string(),
            attribute_type: AttributeType::Unknown(name.to_string()),
            dmx_offset: 0,
            byte_count: 1,
            default_value: 0,
            default_fraction: None,
            physical_range: None,
            is_virtual: true,
            resolved: false,
            ..Default::default()
        }
    }

    #[test]
    fn resolves_virtual_channel_with_matching_physical() {
        let mut mode = make_mode(vec![physical("Dimmer", 0), virtual_ch("Dimmer")]);

        let result = VirtualChannelResolver::resolve(&mut mode);
        assert_eq!(result.resolved, 1);
        assert_eq!(result.unresolved, 0);
        assert!(
            mode.channel_functions[1].resolved,
            "virtual channel should be marked resolved"
        );
    }

    #[test]
    fn unresolved_virtual_channel_when_no_matching_physical() {
        let mut mode = make_mode(vec![
            physical("Pan", 0),
            virtual_ch("Tilt"), // references attribute that doesn't exist as physical
        ]);

        let result = VirtualChannelResolver::resolve(&mut mode);
        assert_eq!(result.resolved, 0);
        assert_eq!(result.unresolved, 1);
        assert!(
            !mode.channel_functions[1].resolved,
            "unresolvable virtual should stay unresolved"
        );
    }

    #[test]
    fn no_virtual_channels_returns_zeros() {
        let mut mode = make_mode(vec![physical("Dimmer", 0), physical("Pan", 1)]);

        let result = VirtualChannelResolver::resolve(&mut mode);
        assert_eq!(result.resolved, 0);
        assert_eq!(result.unresolved, 0);
    }

    #[test]
    fn multiple_virtual_channels_resolved() {
        let mut mode = make_mode(vec![
            physical("Dimmer", 0),
            physical("Pan", 1),
            physical("Tilt", 2),
            virtual_ch("Dimmer"),
            virtual_ch("Pan"),
            virtual_ch("Missing"), // not resolvable
        ]);

        let result = VirtualChannelResolver::resolve(&mut mode);
        assert_eq!(result.resolved, 2);
        assert_eq!(result.unresolved, 1);
    }
}
