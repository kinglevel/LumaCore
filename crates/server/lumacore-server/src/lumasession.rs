// ██╗     ██╗   ██╗███╗   ███╗ █████╗ ███████╗███████╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
// ██║     ██║   ██║████╗ ████║██╔══██╗██╔════╝██╔════╝██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
// ██║     ██║   ██║██╔████╔██║███████║███████╗█████╗  ███████╗███████╗██║██║   ██║██╔██╗ ██║
// ██║     ██║   ██║██║╚██╔╝██║██╔══██║╚════██║██╔══╝  ╚════██║╚════██║██║██║   ██║██║╚██╗██║
// ███████╗╚██████╔╝██║ ╚═╝ ██║██║  ██║███████║███████╗███████║███████║██║╚██████╔╝██║ ╚████║
// ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// FILE: lumasession

//! LumaSession's tests on a whole server (`ServerContext`): the link, the election and remote
//! station edits. The code is `lumacore-lumasession`; these need the server it runs on.
//!
//! Owns:          the whole-server LumaSession tests.
//! Doesn't own:   LumaSession's own unit tests (lumacore-lumasession's foo/tests.rs).
//! Entry points:  the #[test] fns of link_tests, election_tests, remote_edit_tests.
//! Decisions:     §21 "6b", "6c", "6d", §30 "Remote station edits", §34 24.2.

// ████████╗███████╗███████╗████████╗███████╗
// ╚══██╔══╝██╔════╝██╔════╝╚══██╔══╝██╔════╝
//    ██║   █████╗  ███████╗   ██║   ███████╗
//    ██║   ██╔══╝  ╚════██║   ██║   ╚════██║
//    ██║   ███████╗███████║   ██║   ███████║
//    ╚═╝   ╚══════╝╚══════╝   ╚═╝   ╚══════╝
// PART: TESTS

#[path = "lumasession/link_tests.rs"]
mod link_tests;
#[path = "lumasession/election_tests.rs"]
mod election_tests;
#[path = "lumasession/remote_edit_tests.rs"]
mod remote_edit_tests;
