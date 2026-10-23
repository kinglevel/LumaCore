//! An E1.31 (sACN) data packet on an input line.
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = lumacore_sacn::receiver::parse_data_packet(data);
    let _ = lumacore_sacn::receiver::parse_sync_packet(data);
});
