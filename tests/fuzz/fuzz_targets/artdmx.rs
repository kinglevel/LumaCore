//! An Art-Net ArtDmx packet; one that parses encodes back to the same.
#![no_main]
use lumacore_artnet::source::{artdmx_packet, parse_artdmx};
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Some((port, values)) = parse_artdmx(data) {
        let again = parse_artdmx(&artdmx_packet(port, 0, &values)).expect("an encoded ArtDmx parses");
        assert_eq!(again, (port, values));
    }
});
