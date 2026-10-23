//! An OSC packet on an input line (UDP, anyone on the LAN).
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| lumacore_sweep::fuzzing::osc(data));
