// ██╗     ██╗██████╗ ██████╗  █████╗ ██████╗ ██╗   ██╗
// ██║     ██║██╔══██╗██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝
// ██║     ██║██████╔╝██████╔╝███████║██████╔╝ ╚████╔╝
// ██║     ██║██╔══██╗██╔══██╗██╔══██║██╔══██╗  ╚██╔╝
// ███████╗██║██████╔╝██║  ██║██║  ██║██║  ██║   ██║
// ╚══════╝╚═╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝
// FILE: library

//! The library on this station's disk; never swept (decisions §25).
//!
//! Owns:          the Library object: List Library (GDTF files, shows).
//! Doesn't own:   the listings (the kernel's list), the disk (lumacore-luma drive).
//! Entry points:  LIBRARY.
//! Decisions:     §25, §30 "12d".

use lumacore_objects::commands::list;
use lumacore_objects::objects::{ObjectDef, out};

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

// The library: GDTF files and shows on this station's disk (never swept, decisions §25).
lumacore_objects::listed_object!(
default List,
drive station LIBRARY,
"Library",
["lib"],
"The library (GDTF files, shows)",
[
    List => |s, a| out(list::library(s, a)),
]);
