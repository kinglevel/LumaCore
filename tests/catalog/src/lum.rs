// ██╗     ██╗   ██╗███╗   ███╗
// ██║     ██║   ██║████╗ ████║
// ██║     ██║   ██║██╔████╔██║
// ██║     ██║   ██║██║╚██╔╝██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝
// FILE: lum

//! lum.md against every word of the catalog (decisions §33 decision 3, part 2): every ` ```lum `
//! example parses with the station's words, every ` ```lum-error CODE ` example fails with its code,
//! and the words are the golden record `testdata/lum_words.golden.txt`.
//!
//! Owns:          nothing but its tests (lum/tests.rs).
//! Doesn't own:   the grammar (lumacore-syntax), the vocabulary (the kernel's lum).
//! Entry points:  the #[test] fns.
//! Decisions:     §33 3, §34 25.15.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[cfg(test)]
mod tests;
