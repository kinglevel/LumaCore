//! An Art-Net ArtPollReply from a node on the LAN.
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let _ = lumacore_artnet::source::parse_poll_reply(data);
});
