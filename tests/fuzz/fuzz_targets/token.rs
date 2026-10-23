//! A login token (HTTP Authorization).
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| lumacore_sweep::fuzzing::token(data));
