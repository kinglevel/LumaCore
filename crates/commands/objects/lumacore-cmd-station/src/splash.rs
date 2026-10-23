// ███████╗██████╗ ██╗      █████╗ ███████╗██╗  ██╗
// ██╔════╝██╔══██╗██║     ██╔══██╗██╔════╝██║  ██║
// ███████╗██████╔╝██║     ███████║███████╗███████║
// ╚════██║██╔═══╝ ██║     ██╔══██║╚════██║██╔══██║
// ███████║██║     ███████╗██║  ██║███████║██║  ██║
// ╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
// FILE: splash

//! The splash the terminal shows when the REPL starts, and the `banner` command.
//!
//! Owns:          the splash: LumaCore's start art and build line (the `banner` command, the REPL's start).
//! Doesn't own:   the header tool's font (lumacore-headers).
//! Entry points:  banner_string.
//! Decisions:     §34 25.7.

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// Return the LumaCore ASCII banner as a single String.
pub fn banner_string() -> String {
    let p = "\x1b[38;5;135m"; // purple
    let b = "\x1b[38;5;69m"; // cold blue
    let cy = "\x1b[38;5;117m"; // pale cyan
    let dm = "\x1b[38;5;60m"; // muted grey
    let rs = "\x1b[0m";

    format!(
        "\n\
         {dm}        .          {cy}*  {dm}.                    {cy}*{dm}       .  \n\
         {dm}    {cy}*                  _.._                          \n\
         {dm}              {cy}*  {b}    .' {cy}.-'`{b}                             \n\
         {dm}   .              {b}   /  {cy}/{dm}                                \n\
         {dm}                  {b}   |  {cy}|{dm}        {cy}*                      \n\
         {dm}    {cy}*   {dm}  .        {b}   \\  {cy}\\{dm}                               \n\
         {dm}                  {b}    '._'._ {cy}`-'{b}                         \n\
         {dm}    {p}  /\\_/\\                                            \n\
         {dm}    {p} ( o.o )   {b}  _                          ___       \n\
         {dm}    {p}  > ^ <    {b} | |   _   _ _ __ ___   __ / __|___  _ __ ___  \n\
         {dm}    {p} /|   |\\   {b} | |  | | | | '_ ` _ \\ / _` | |/ _ \\| '__/ _ \\ \n\
         {dm}    {p}(_|   |_)  {b} | |__| |_| | | | | | | (_| | | (_) | | |  __/ \n\
         {dm}    {p}           {b} |_____|\\__,_|_| |_| |_|\\__,_|\\_\\___/|_|  \\___| \n\
         {dm}   ─────────────────────────────────────────────────────────── \n\
         {dm}    v0.1.0                    terminal-first DMX console      {rs}\n"
    )
}
