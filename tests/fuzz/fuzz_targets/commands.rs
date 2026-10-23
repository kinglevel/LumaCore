//! Command sequences grown from completion by the bytes, through the sweep's
//! checks (coverage-guided twin of the overnight command sweep).
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| lumacore_sweep::fuzzing::commands(data));
