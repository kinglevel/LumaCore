// ██████╗ ███████╗███████╗███████╗
// ██╔══██╗██╔════╝██╔════╝██╔════╝
// ██║  ██║█████╗  █████╗  ███████╗
// ██║  ██║██╔══╝  ██╔══╝  ╚════██║
// ██████╔╝███████╗██║     ███████║
// ╚═════╝ ╚══════╝╚═╝     ╚══════╝
// FILE: defs

//! The registries' types (decisions §34 15): what a command and an object declare. They sit
//! with the state so everything below the registry tables can name them; the tables
//! themselves (the command families' `ROWS`, `OBJECTS`) live above, beside the objects.
//!
//! Owns:          the declaration types of commands and objects.
//! Doesn't own:   the tables, resolving a word, dispatch.
//! Entry points:  command, object.
//! Decisions:     §15, §34 15.

pub mod command;
pub mod object;
pub mod view;
