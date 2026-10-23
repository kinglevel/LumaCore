//! A LumaSession stream frame (u32 length + zstd + JSON).
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| lumacore_sweep::fuzzing::stream_frame(data));
