//! The typed boundary (decisions §33 decision 4, part 2): a handler takes its verb's typed
//! arguments — a `#[derive(Args)]` struct, which is the signature — never the raw words.
//!
//! `run: typed::<C>` (or `(Verb::Store, typed::<C>)` on an object) fits the registries' `Run` /
//! `Handler` type, so the static tables stay as they are. Until dispatch hands over the parsed
//! statement (§33 part 3), [`typed`] is also the shim the other way round: it writes the words
//! dispatch gives it back into a line, parses it with the registries' vocabulary
//! ([`crate::lum`]) and binds it. Handlers not moved yet still take `&[&str]`; the test
//! `the_shim_count_only_goes_down` counts them, and the branch merges at 0 (decision 33).

use lumacore_syntax::{Args, FenceLine, Origin, bind, parse};

use crate::{AppState, DispatchResult, Reply};

/// A verb's handler on its typed arguments.
pub trait Command: Args {
    fn run(self, state: &mut AppState) -> DispatchResult;
}

/// The registry entry for a typed handler: `run: typed::<Redo>`.
pub fn typed<C: Command>(state: &mut AppState, args: &[&str]) -> DispatchResult {
    let sig = C::SIGNATURE;
    // `<target> At <value>` has At as its verb in the middle (decision 43): the object and its
    // words as typed (`Master 2.1 At 80`).
    let verb = (sig.verb != "At").then_some(sig.verb);
    let line: Vec<&str> = verb.into_iter().chain(sig.object).chain(args.iter().copied()).collect();
    let line = line.join(" ");
    let vocab = crate::lum::vocabulary();
    let parsed = parse(&line, vocab, Origin::Typed);
    if let Some(d) = parsed.diags.first() {
        return DispatchResult::Output(Reply::error(&d.message, d.code));
    }
    let Some(statement) = parsed.statements.first() else {
        return DispatchResult::Output(Reply::error(format!("{line}: nothing to run"), lumacore_syntax::codes::MISSING));
    };
    let globals: Vec<&str> = lumacore_registry::get().global_flags().map(|f| f.name).collect();
    match bind::<C>(statement, vocab, &globals) {
        Ok(command) => command.run(state),
        Err(d) => DispatchResult::Output(Reply::error(&d.message, d.code)),
    }
}

/// A line bound to `C`'s signature without running it — for a verb whose route picks the
/// handler from what the line names (`Set`: the one property registry, decisions §33 58).
pub fn bound<C: Args>(line: &str) -> Result<C, Reply> {
    let vocab = crate::lum::vocabulary();
    let parsed = parse(line, vocab, Origin::Typed);
    if let Some(d) = parsed.diags.first() {
        return Err(Reply::error(&d.message, d.code));
    }
    let statement = parsed
        .statements
        .first()
        .ok_or_else(|| Reply::error(format!("{line}: nothing to run"), lumacore_syntax::codes::MISSING))?;
    let globals: Vec<&str> = lumacore_registry::get().global_flags().map(|f| f.name).collect();
    bind::<C>(statement, vocab, &globals).map_err(|d| Reply::error(&d.message, d.code))
}

/// A tree-shell word's handler (decision 18): the fence statement's paths and flags.
pub trait FenceCommand {
    /// The word, lowercase (`cd`, `ls`).
    const WORD: &'static str;
    /// How many paths / texts it takes at most (`cd` 1, `diff` 2).
    const PATHS: usize;
    fn run(state: &mut AppState, line: &FenceLine) -> DispatchResult;
}

/// The registry entry for a tree-shell word: `run: fenced::<Ls>`.
pub fn fenced<C: FenceCommand>(state: &mut AppState, args: &[&str]) -> DispatchResult {
    match fence_line(&std::iter::once(C::WORD).chain(args.iter().copied()).collect::<Vec<_>>().join(" ")) {
        Ok(line) if line.positional.len() > C::PATHS => {
            let extra = &line.positional[C::PATHS];
            let hint = if extra.starts_with('/') { " (a path is written with dots: ShowData.Devices)" } else { " (quote text with spaces or signs)" };
            DispatchResult::Output(Reply::error(
                format!("{} takes {} path{} — '{extra}' is one too many{hint}", C::WORD, C::PATHS, if C::PATHS == 1 { "" } else { "s" }),
                lumacore_syntax::codes::UNEXPECTED,
            ))
        }
        Ok(line) => C::run(state, &line),
        Err(reply) => DispatchResult::Output(reply),
    }
}

/// A fence line read with the registries' vocabulary (`ls -r Root.ShowData`).
pub fn fence_line(src: &str) -> Result<FenceLine, Reply> {
    let parsed = parse(src, crate::lum::vocabulary(), Origin::Typed);
    if let Some(d) = parsed.diags.first() {
        return Err(Reply::error(&d.message, d.code));
    }
    parsed
        .statements
        .first()
        .and_then(FenceLine::of)
        .ok_or_else(|| Reply::error(format!("'{src}' isn't a tree-shell line"), lumacore_syntax::codes::UNEXPECTED))
}

/// A fence line in a test: `fl("head Fixtures -n 2")`.
#[cfg(test)]
pub fn fl(src: &str) -> FenceLine {
    fence_line(src).unwrap_or_else(|r| panic!("{src}: {r:?}"))
}
