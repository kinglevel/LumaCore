# lumacore-tidy

Every rule here has one entry in `docs/meta/tidy.md`, and a test (`the_rulebook_matches_the_rules`) checks the two against each other both ways, ratchet values included. Add the entry with the rule, never after.

Ratchets (`LONG_FILES`, `HEADERLESS_AT_MOST`, `MOD_RS_AT_MOST`, `OLD_TEST_FILES_AT_MOST`, `READMELESS_AT_MOST`, `OLD_RESOLVER_USES_AT_MOST`, `COPIES`) only go down: lower the constant in the same change that removes the debt; never raise one to make a test pass. An exception to a rule is a marker in the source with a reason (`// tidy: <rule>-ok (<reason>)`), never a silent allowlist.

This crate is a dev tool: no product crate depends on it (`dev_tools_never_ship`). It reads the source as text; it never compiles or runs the code it checks. The one thing it runs is `scripts/unix/docs_check` (`the_docs_pass_docs_check`), so the docs are gated with the code.
