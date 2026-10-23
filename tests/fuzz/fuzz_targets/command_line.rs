//! A command line from any client: lexer, parsers, masking, dispatch.
#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| lumacore_sweep::fuzzing::command_line(data));
