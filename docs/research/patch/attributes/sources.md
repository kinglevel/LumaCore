# Sources: the attribute dataset

All fetched on 2026-10-18. Raw copies are in `raw/`. The build script is `scripts/build.py`. Rebuild with:
`python3 -I scripts/build.py raw .`

| # | Source | Used for | Raw copy |
|---|---|---|---|
| 1 | https://community.smidn.com/docs (served as `text/markdown`, 42 KB) | What the site holds, and its attribute naming notes | `raw/smidn/docs.md` |
| 1a | https://community.smidn.com/api/attributes?sources=all | The GDTF Share facets (5,033 attribute names actually used, with fixture counts) | `raw/smidn/api_attributes_all.json` (`api_attributes.json` = `?sources=gdtf`) |
| 1b | https://community.smidn.com/api/sources | Library sizes: GDTF Share has 10,405 fixtures and 13,621 revisions, updated 2026-10-05. Depence "Fusion" has 9,685 fixtures, imported 2026-07-24 | `raw/smidn/api_sources.json` |
| 1c | https://community.smidn.com/api/stats | **Not fetched**: it timed out after 30 s | — |
| 2 | https://raw.githubusercontent.com/mvrdevelopment/spec/main/gdtf_attributes_with_description.json (repo HEAD 098d3791, 2026-04-09) | **The main GDTF source** (machine-readable): 279 attributes with Feature, PhysicalUnit, MainAttribute, ActivationGroup, Pretty, CIE Color, subPhysicalUnits and definition | `raw/gdtf/gdtf_attributes_with_description.json` |
| 2a | https://raw.githubusercontent.com/mvrdevelopment/spec/main/gdtf-spec.md (DataVersion 1.2, DIN SPEC 15800) Annex A (table A1) and Annex B (XML listing) | A cross-check. Feature groups, features and activation groups come from Annex B | `raw/gdtf/gdtf-spec.md`, `raw/gdtf/annexB.txt` |

The following were not used:
- https://github.com/smidn/community: returns 404. It is private or does not exist. The docs link to `docs/RDM-SHARE.md` and `docs/RDM-SHARE-CODEC.md` there, and these could not be read.
- gdtf-share.com: not needed, because the spec repo JSON is the authoritative machine-readable list.

## What community.smidn.com/docs contains

It is one Markdown page, "Community Fixture Search API". It is a read-only JSON API over two fixture libraries: GDTF Share (`gdtf:<rid>`, crawled daily) and the Depence Factory Library (`depence:<GUID>`, imported by hand). It serves no `.gdtf` or `.dfix` files.

| Section | Endpoints / content |
|---|---|
| Concepts | Namespaced fixture ids and `?sources=`. **A shared attribute vocabulary**: every channel carries a GDTF attribute name, an alternative console spelling and a feature group (9 groups, plus `Other`). Queries accept either spelling (`ColorAdd_R` ≡ `ColorRGB_R`). Also defines revision, mode, footprint and offset |
| Browse | `/api/sources`, `/api/manufacturers[/:id/fixtures]`, `/api/fixtures`, `/api/fixtures/:id` and its `modes`, `physical`, `photometry`, `spectral` and `thumbnail` sub-paths |
| Search | `/api/search?q=`, `POST /api/search/layout` (by channel layout), **`/api/attributes`** (an attribute definitions list plus per-library facets), `/api/stats` |
| Sync status | `/api/sync/status`, `POST /api/sync/trigger` (secret-gated) |
| RDM Share | `/api/rdm`, `rdm/lookup`, `manufacturers`, `models`, `firmwares`, `profile`, `search`, `units[/:uid]`, `stats`, `export[/file]`, `POST /api/rdm/submissions` (Bearer key) |
| Errors | `{ok:false, error}`, 400 / 404 / 429 with Retry-After. The rate limit is about 120 requests per minute per IP |

The site publishes no separate GDTF attribute definitions; the GDTF list comes from the spec repo (source 2).

## Known defects in the sources

- **GDTF Annex B XML has typos.** These include `BladeSoft(n)A Pretty=` with a missing quote, `XYZ_Z ... /` and `AnimationSystem(n)Audio ... Pretty="Anim System Audio/>`.
  - Because of these, a naive parse finds 275 rows instead of 279.
  - The JSON has all of them. Apart from that, the JSON and Annex B differ only in 3 pretty names (`&lt;&gt;` escaping, and "Pulse Open").
- **GDTF inconsistency:** GDTF has `VideoKeyColor_R/G` but `VideoColorKey_B`.
