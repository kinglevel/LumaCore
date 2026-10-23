---
name: regen-reference
description: Regenerate the generated docs and golden files after a registry, tree-path, Lum-vocabulary or demo-show change. Use when the_reference_is_current, a golden test or the demo golden fails.
---

# Regenerate the reference and the goldens

The command and object reference is generated; never hand-edit it:

```bash
cargo run -p lumacore-station --bin gen-docs      # docs/user/reference/commands.md + objects.md
```

The golden files are rewritten by their own tests in bless mode. Read the diff before committing; a golden change is a behaviour change:

```bash
LUMACORE_BLESS=1 cargo test -p lumacore-catalog-tests tree_paths     # tests/catalog/testdata/tree_paths.golden.txt
LUMACORE_BLESS=1 cargo test -p lumacore-catalog-tests lum             # tests/catalog/testdata/lum_words.golden.txt
LUMACORE_BLESS=1 cargo test -p lumacore-catalog-tests demo            # tests/catalog/testdata/demo.golden.txt (counts + SHA-256)
LUMACORE_BLESS=1 cargo test -p lumacore-suggest tab_completion        # the suggestion golden
```

The demo show itself is built from its sources in git when stale (`cargo run -p lumacore-station --bin gen-demo -- --if-stale`; plain `gen-demo` always). `library/shows/demo.luma` is gitignored.

Which test fails tells you which to run: `the_reference_is_current` → gen-docs; `tree_paths_are_unchanged` → tree paths; `the_words_are_the_golden_record` → lum words; `the_demo_show_builds_loads_and_matches_its_golden` → demo.

Never bless to make a failure go away: a golden diff you did not intend is a bug.
