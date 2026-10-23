# Handoff · 34g4 patch, round 3 hardening (2026-10-18)

## Status (2026-10-20)

Built and committed 2026-10-19 (the frozen log `docs/archive/decisions-2026-10.md` §34, the entries "Round 3, the parts" through "Round 3 part 4d3b built" and "4d4 built"; the roadmap's "34g4 patch, round 3" item names each part with its commit): step 0 the missing tests · part 1 the words (1a Store / Assign / Unassign `At DMX`, 1b `Fixture n DMX`, `/Swap`, Move `/Skip` / `/By`, 1c the MVR node delete answers, Multipatch) · part 2 `Import FixtureType` and the file index · part 3 the kit (3a the claim button, the ring above dialogs, slab states; 3b the one context menu; 3c the value dialog) · part 4 the patch window (4a the Live / Draft grid, 4b the function block, 4c1–4c2 Insert fixtures' data and dialog, 4d1 park on conflict, 4d2 the Map view, 4d3a Identify, 4d3b Edit Patch, 4d4 drag and drop + lasso — `docs/plans/done/2026-10-19-4d4-hardening.md`), the phone round (§37) in between, the tree in the Patch dialog after it (§40 40d, 2026-10-20).

Remains of this round (the roadmap item stays open and is the list; among them): 4c3 Place's DMX sheet · revisions and the type / revision / mode change with the red-address warning (items 5–6) · the attribute map — change map, Defined attributes, Clone with its own window, colour through CIE (item 7) · update fixture types with a diff, Probe / Find, keep spacing / bunch up, kit-wide renumber, usage bars (item 13) · 3d RunTool / EditTool (its own roadmap item) · Clean up unused on the Metadata tab · the parts / pixelmapper model (items 14–17, a future aim, hardened on its own).

Decisions: the frozen log §34 "34g4 patch, round 3 hardening", items 1–19, and the "Round 3 …" entries after it. Mockups and their artifact links: `docs/dev/ui-kit/mockups/2026-10-18-34g4-r3/README.md`. Research data: `docs/research/patch/` (attributes, community-fixtures; summarised below). The audit's findings and the research below are kept as written on 2026-10-18 — the user's next step then: **"get back to the patching"**; the fixture-parts / pixelmapper work is a starting point and a future aim, not this round's build.

## How the round started

An audit of the built patch window (two agents: decisions vs code, dialog vs locked mockups) plus
the user's own use: no way to add a fixture in the dialog; MVR import / export not built (parts 4
and 5); a universe can't be renumbered from the Universes tab; changing a patched fixture's type /
mode isn't built; the claim button's hold shows no animation (the kit's hold ring is drawn under
every dialog: `.lc-ring` z-index 100001 < `.lc-setup-scrim` 100004) and it isn't anchored
bottom-right. Found on the way: the old pre-Lum `Patch 6x "Type" At 1.001` and `Import "x.gdtf"`
still add fixtures / types (the demo builder uses them); no UI reaches them.

### The audit's other findings (still open, see "Open" below)

- Draft looks like Live: live columns never lock in Live, and Fixture type / Mode / Tags / MVR
  group stay read-only in Draft (the grid's lock only applies to prop columns, `setup.js:275`).
- Every tab's function block is empty (`objects/patch/menu.rs` `actions: &[]`).
- Slab states: New / Import greyed in Draft, Paste enabled with an empty clipboard.
- Deleting an MVR layer / group **removes** its fixtures (`lumacore-patch/src/patch.rs:221`) while
  the question says "unpatched".
- Live editing of a fixture name only works through `Set`; another user's `Label Fixture` is refused.
- Small layout differences: FID column labelled "No"; X / Y / Z split; the Note column missing; the
  title shows "In the show · every station" instead of the path; the echo line lacks counts; the
  tree has no guide lines and no "+ Layer / + Group" bar; tree counts without "fixtures"; the claim
  hint lingers; the open-lock icon isn't in the sprite.
- Issue 11's "no universe" display isn't in the grid.
- Untested built rules: the Live column lock, the Delete Universe question naming fixtures, `Assign …
  At MvrLayer`, Unassign without `At`, bare `MvrClass` / `MvrPosition`, `Move Tag … At Root`, the
  claim button's "theirs" state, the 5 s lost-holder timer, revision folder numbering, Copy / Label
  of MvrGroup.
- Dropped / contradicting decisions: Fixture.Luma extras (grid spot, encoder inverts, pan/tilt
  offsets), Metadata "Clean up unused", the other common live columns (CastShadow, beam angle, invert, P1–P4);
  hardening point 7 says MvrClass / MvrPosition can be labelled, part 3c made them read-only.
- Stale roadmap notes: "selecting by the second id comes with the dialog", "Tags as a live column
  with the Devices claim".

## Decided (summary; the full text is in decisions)

1. Words (lum.md first): `Store Fixture … /Type /Mode [/Revision] [/Address] [/Offset]` creates,
   `Patch Fixture <sel> At u.aaa [/Offset n]` addresses, `Unpatch Fixture` keeps the fixture; the
   old `Patch Nx` / `Unpatch <id>` go.
2. `Import FixtureType "file.gdtf"` from `library/gdtf/` + a picker with Upload from this device; the
   old `Import "x"` / `Import library` go.
3. One library file index behind the one suggestion engine (every file kind registers a folder +
   a reader); `/Type <Tab>` and every picker share it.
4. Insert fixtures… = the usual two steps in our kit (Choose → Place), every field one part of the one
   Store line. Show is the default scope.
5. Revisions: newest by default, `/Revision n`; importing a newer one changes no fixture (the reply
   offers the line); revision changes need the claimed patch; told apart by SHA-256 + revision text.
6. Type / revision / mode change = `Set Fixture … "FixtureType" | "Revision" | "Mode"` (claimed
   patch); a colliding / past-512 address turns red with a warning until resolved (no question).
7. The attribute map (cross-clone) — one base window, three uses: the change map, Defined
   attributes (pattern rules saved in the show on the show's attributes, built-in GDTF families at
   the bottom), Clone (outside the patch, Program rights, its own window: pairs + an object tree with
   counts). Colour converts as a whole through CIE (`palette` + an own lighting-colour module).
   Remembered maps pre-fill and still open; one Meaning / DMX output switch, Meaning default. Hand
   edits: two columns joined by lines (variant B).
8. Universes renumber by the No cell or Move to….
9. The claim button: the locked mockup done right (bottom-right, a gold fill on hold, hint only on
   a press, open-lock icon; kit-wide the hold ring goes above dialogs).
10. Live vs Draft: Live edits the live columns only (others locked, header too); Draft edits every
    column as its one line; the Note column returns; gold / violet edit dots.
11. Function block: Insert fixtures…, Patch to…, Next free address, Multipatch…, Info, Sort, Clone….
12. One DMX map element, two zooms (Universes › Map with usage bars; Edit Patch with the slot grid).
13. From other consoles: update fixture types with a diff; identify / probe / find; swap addresses;
    keep spacing / bunch up; renumber kit-wide (lasso, type the first number); park on conflict + a
    conflict list; usage bars. Later: patch report, DIP helper, address display toggle, per-fixture
    patch attributes, origin flag, RDM, drag-to-plan, MVR import modes.
14–17. Fixture parts → **Media server → canvases (Pixelmapper devices with FIDs, layers inside,
    controlled like fixtures) → mappings (Canvas pool, live data; Linked / Copied / Free group
    layout; self-expanding pixels; 2D raster vs 3D rig) → parts (a Parts pool of kinds in Devices;
    numbered instances; resample by position)**; the pixelmapper controls a mapped part completely;
    output stage (masters, blackout, Park) still applies; the encoder bar gets a Pixelmapper mode.

## Settled after the compaction (decisions item 18; mockups README)

DMX map v2 (drag and drop, lasso, both parked, Identify / Probe for Setup with a warning strip) ·
lasso in every list and grid · swap addresses (uneven counts pair up) · one context menu for every
list / grid / pool (common actions, Set for every column, the object's verbs; Edit… stays) · the
renumber value dialog (live preview, free chips, Skip taken; the same dialog for Address) · the
Live / Draft patch window approved (`patch-grid.html`: claim button, locks, slab states, function
block) · MVR node delete asks Keep (default) / Delete the fixtures · MvrClass / MvrPosition
labellable · the Fixture.Luma extras and the other common live columns built this round · Clean up
unused on the Metadata tab.

## Open

- ~~The round's words~~ — hardened and written into lum.md 2026-10-18 (decisions item 19; the lines that don't parse yet are on `tests/catalog/src/lum/tests.rs` PENDING). Item 1's `Patch` / `Unpatch` verbs are corrected to Store / Assign / Unassign.
- **Build step 0 — the missing tests: done 2026-10-18.** New tests: a fixture in a layer itself,
  bare `MvrClass` / `MvrPosition` select, Copy / Label of MvrGroup (`devices_meta/tests.rs`); the
  Delete Universe question names its fixtures (`playback_tests.rs`); the 5 s lost-holder watch
  (moved out of the server's periodic closure into `claims::HolderWatch`, `patch_draft_tests.rs`);
  a live column without a draft; revision folders numbered by place (`show_file/tests.rs`); the
  claim button's "theirs" state (`kit-34g4.spec.ts`, Bob on his own page, 4 platforms). Not
  written: Unassign without `At` — its rule changed with the words (item 19: refused, naming the
  forms), so its test comes with that build. **Two findings, tests `#[ignore = "finding: …"]`:**
  (1) `Move Tag … At Root` couldn't run — `Root` wasn't a word — **fixed**: the Root object and
  node numbers (§33 53–54) are built; (2) with no draft open, a typed `Set` on a
  non-live column (`"FixtureID"`) changes the live rig without the Devices claim — Live is
  decided to lock it ("Claim the draft to change it").
- **Approved since:** `insert-fixtures.html` v6 ("for now"), `map-clone.html` v7 (Map & clone, the
  first build; attribute model). `attribute-map.html` v3 keeps the native-values and Update fixture
  types boards (not locked separately).
- **Parts, hardened on their own:** the parts model and where parts are used (the clone menu among
  them). Input: `docs/research/patch/community-fixtures/report.md` (part-kind rules 96% on kind, shape asked
  for half; values keyed by part + instance; resampling is the normal case) and v5's part-by-part
  vs attributes-only comparison in the `map-clone.html` history.
- **Pixelmapper details** (future aim): canvas size, layer pages in the pool, 3D canvas as a plane or
  a projection, the mode's canvas list, rendering station, video sources, node wires; a Linked
  member moved in the canvas editor (refuse vs offer Copied).

## Research (condensed; agents' full reports are not kept)

- **Webview shells / Linux** (for §36, already recorded there).
- **Attribute dataset** — `docs/research/patch/attributes/`: 279 GDTF attributes from
  mvrdevelopment/spec `gdtf_attributes_with_description.json` @ 098d379, wildcard patterns, 9 feature groups / 21 features; `observed_gdtf_share.json` = 5,033
  attribute names used on GDTF Share (552 standard names cover 91.5 % of uses); some consoles fold
  ColorAdd_* / ColorSub_* into ColorRGB_*; community.smidn.com/docs is one page documenting a read-only
  JSON API over GDTF Share (10,405 fixtures) + Depence (9,685) mapping each channel to GDTF and console
  names. `scripts/build.py` rebuilds it.
- **Basic attribute rules from community.smidn.com** — an agent was still running at handoff
  (writing to a temporary folder, not saved). Rerun if needed: vendor spellings → standard names,
  console ↔ GDTF pairs, families and their fallbacks, substitutes when a target lacks an attribute.
- **What LumaCore already has about attributes** (12f-a): `standard_attributes.json` (346, features,
  main / activation), `attribute_templates.json` (89 pattern rules), the resolver
  (`attribute_resolver::resolve_in`: learned rule → exact → family → parent → context → keyword →
  guess); values keyed by (fixture GUID, attribute name) in programmer / cue parts / presets; no
  attribute mapping between types and no mode-change path today; a legacy `AttributeType`
  classifier duplicates the resolver.
- **Colour science**: `palette` 0.7.7 (MIT/Apache, light, `no_std`: XYZ / xyY / Lab / Oklab / CAM16,
  ΔE2000, Bradford) + own module (emitter vertex solver with a Quality blend, CMY optimiser,
  wheel slot by ΔE2000, mired for CTO / CTB, gamut clipping in Oklab, fallback: emitter colour →
  dominant wavelength → nominal by name → ColorSpace). `colorimetry` only for spectra later.
  Check ETC patent 8403523 before copying a least-squares mixing method.
- **Other consoles' patch features** (Eos 3.x, MagicQ 1.9.9, Titan 19, Hog 4, ONYX, QLC+, Lightkey,
  Capture): the sources behind item 13 (Eos Update Profile / Preserve Native / Flash /
  About Address; MagicQ Clone / Morph head / View DMX / `@` syntax; Titan Exchange Mapping / Range
  Mapping / Park on conflict / Swap addresses / Find Fixture; Hog Replicate / Patch @@ gap; ONYX Auto
  Patch fields, Fixture Swap stage → review → execute, Patch Exchange; QLC+ universe view and remap;
  Capture selection-order patching and reports).
- **Fixture parts** — consoles: some (Hog, ONYX among them) make parts their own fixtures (type changes break:
  Hog no compound → compound, ONYX no multipart swap, multi → single may lose presets); Eos /
  Titan a master + cells (Eos runs an effect on master and cells at once); MagicQ "element 1 recorded,
  copied to the rest unless set on its own" is closest to good. Operators' top pains (forum quotes):
  cell order / layout for effects, the master leaking into cells, an opaque parent / child structure,
  presets not carrying across cells or types. Models outside lighting: cascade (CSS), layer (pixel
  map), linked channels (stereo link), range mask — and the user's media-server model, adopted.
