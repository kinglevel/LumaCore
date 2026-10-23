// ██████╗  █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗     ███╗   ███╗ █████╗ ██╗███╗   ██╗
// ██╔══██╗██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗    ████╗ ████║██╔══██╗██║████╗  ██║
// ██████╔╝███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝    ██╔████╔██║███████║██║██╔██╗ ██║
// ██╔══██╗██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗    ██║╚██╔╝██║██╔══██║██║██║╚██╗██║
// ██████╔╝██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║    ██║ ╚═╝ ██║██║  ██║██║██║ ╚████║
// ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝    ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝
// FILE: banner main

//! `lumacore-headers [--file|--part] <title>` / `--section <title> [purpose…]`: prints a header
//! for an `.rs` file (decisions §34). `./scripts/unix/headers` runs it.
//!
//! Owns:          the command line of the banner tool.
//! Doesn't own:   the font and the rendering (lib.rs).
//! Entry points:  main.
//! Decisions:     §34.

// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (kind, rest) = match args.first().map(String::as_str) {
        Some("--file") => (Some(lumacore_headers::Kind::File), &args[1..]),
        Some("--part") => (Some(lumacore_headers::Kind::Part), &args[1..]),
        Some("--section") => (None, &args[1..]),
        _ => (Some(lumacore_headers::Kind::Part), &args[..]),
    };
    let Some(title) = rest.first() else {
        eprintln!("usage: banner [--file|--part] <title>  |  banner --section <title> [purpose …]");
        std::process::exit(2);
    };
    let lines = match kind {
        Some(kind) => match lumacore_headers::banner(kind, title) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        None => lumacore_headers::section_box(title, &rest[1..].iter().map(String::as_str).collect::<Vec<_>>()),
    };
    for l in lines {
        println!("{l}");
    }
}
