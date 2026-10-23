# 09 — Show files lose float precision on load

**Severity:** low · **Found:** sweep 2026-10-13 (`save-load-order` on `loadshow Network 1`,
mislabelled "same content, other order").

## Cause
Not order: the last digit of re-derived GDTF floats changes, e.g. `"dmx_from":
0.09803921568627451` → `…452`. The workspace `serde_json = "1.0"` (`Cargo.toml:~32`) lacks the
`float_roundtrip` feature, so parsing isn't exact; it surfaces after LoadShow because
`refresh_revision` recomputes exact f64s. Reproduced with `show_file::read(demo.luma)` → JSON →
`parse_show` → JSON.

## Fix
`serde_json = { version = "1.0", features = ["float_roundtrip"] }`; then drop the tolerance in
`sweep/property_tests.rs:~124`. Sweep side: `order_diff` (`world.rs:~500`) reports a wrong path —
report the first differing text line instead of "order" (issue 20). Check
`the_demo_show_builds_loads_and_is_committed` afterwards (demo.luma may need regenerating).

## Decision
As above. **Fixed** (2026-10-13, on branch `bugfixes`).
