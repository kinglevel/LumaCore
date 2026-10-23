//! `lumacore_wire::SessionDiscoveryWire` from the network (JSON): anything that decodes
//! encodes back to the same.
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(v) = serde_json::from_slice::<lumacore_wire::SessionDiscoveryWire>(data) {
        let bytes = serde_json::to_vec(&v).expect("a decoded value encodes");
        let again: lumacore_wire::SessionDiscoveryWire = serde_json::from_slice(&bytes).expect("an encoded value decodes");
        assert_eq!(serde_json::to_value(&v).ok(), serde_json::to_value(&again).ok());
    }
});
