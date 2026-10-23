# Map & clone and the parts model, tested against real fixture data

Research for patch round 3 (2026-10-18). Source: the Community Fixture Search API (community.smidn.com), GDTF Share library. Read-only. Every number below comes from `sample.json` (made by `build.py`) or from the two library-wide endpoints named in Method.

## Summary

- **Real fixtures are messy exactly where Map & clone matters.** Of 53 multi-mode heavy fixtures, 46 lose attributes between some modes, 36 change the number of instances (pixels, zones, heads), and 37 put the same attribute on a differently named geometry in another mode.
- **Geometry names are not stable between modes of the same fixture.** GLP prefixes every geometry with the mode (`Norm Head`, `SPix LED 7`, `DPix LED 3`); Ayrton, Chauvet and Solaris do similar (`Cell 3 Mode120 RGBW`). Matching parts by geometry name fails; match by part + instance number.
- **Instance counts change all the time.** X4 Bar 20 has 20, 10, 3 or 1 colour cells depending on the mode; Solaris Flare XL 120 has 120, 20, 12, 6 or 1. Resampling by position is the normal case.
- **Same feature group is a weak suggestion.** In 12 type swaps only about 2 of ~20 "same group" suggestions were sensible (`Color1WheelIndex -> Color1`, `StrobeRate -> Shutter1Strobe`); most were wrong (`Gobo3 -> AnimationWheel1Pos`, `Blade1Rot -> Blade1B`, `PositionEffect -> Pan`). Keep them behind "advanced".
- **Colour is the biggest mapping block, and it converts well as a whole.** 2–6 attributes per swap were colour attributes with no exact twin; CIE conversion covers all of them.
- **Placeholders are noise.** `NoFeature1…10`, `*MSpeed`, `Control1`, `*Reset` made up 1–6 unmatched attributes per pair. A beginner should never see them.
- **The part-kind rules get the kind right, not always the shape.** 54 hand-labelled parts: kind right 52 (96%); shape right 37 of 44, the other 7 asked, never guessed wrong. 730 broad parts: a shape guessed for 49%, asked for 51%; a spot check of 20 guesses found 17 plausible, 2 wrong, 1 unsure.

## Method

- **Date:** 2026-10-18. GDTF Share index updated 2026-10-05: 13,621 revisions (13,609 parsed) of 10,405 fixtures from 1,260 manufacturers; footprint average 30.2, maximum 1,813 (`/api/stats`).
- **Endpoints:** `/docs` (read in full), `/api/sources`, `/api/stats`, `/api/attributes?sources=gdtf` (library-wide naming facets), `/api/fixtures?sort=channels|rating|date`, `/api/search`, `/api/fixtures/:id`, `/api/fixtures/:id/modes` (channel sheets with per-channel `geometry`, `dmx_break`, `virtual`, plus `wheels`), `/api/fixtures/:id/spectral`.
- **Source:** GDTF only — Depence has no geometry instances and no spectral data.
- **Rate:** one call at a time, ≥ 0.65 s apart, 429 / Retry-After back-off; about 560 calls, all cached on disk.
- **Heavy sample (57 revisions):** 55 hand-picked fixtures + 2 older revisions (X4 Bar 20 `gdtf:59461`, HalupixDUO `gdtf:145478`): pixel battens, ring and aura washes, multi-cell washes, tilting multi-head bars, hybrid strobes, spots, profiles, beams; the newest manufacturer upload where one exists.
- **Broad sample (316 fixtures with channel data):** distinct manufacturer + fixture from the 160 top-rated, the 100 with the most channels (97 distinct) and the 60 newest; "User Test" uploads excluded. With the heavy sample: **366 revisions**.
- **Library-wide naming:** `/api/attributes` facets, 183,732 attribute × revision uses.
- **"Instances":** in one mode, geometries with the same attribute set and name family (digits → `#`); `SPix LED 1…20` = one part, 20 instances. A lone geometry with the same attributes but an odd name is an "odd sibling", not merged (Robe's `Lens1` next to `P2 Zone2…`).
- **"Standard" attribute:** matches GDTF 1.2 Annex A names (regex list in `build.py`; an approximation).
- **Limits:** the API doesn't say which wheel a channel uses, gives no geometry type (Beam, Reference, Axis) or positions; gobo `media` was empty; the §5 hand labels come from product knowledge; the rules were tuned on the same labels, so the heavy accuracy is optimistic — the broad spot check is the fairer number.

## 1. The heaviest fixtures

Modes = number of modes · Ch = channel count min–max over modes · Fp = largest footprint · Attrs = most distinct attributes in one mode · Part max = instances in the largest repeated part · Dimmer inst. = Dimmer instances min–max (virtual included) · Virt = most virtual channels in one mode · Breaks = DMX breaks · Wheels · Colour = colour system of the largest mode.

| Fixture | id | Modes | Ch | Fp | Attrs | Part max | Dimmer inst. | Virt | Breaks | Wheels | Colour (largest mode) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Ayrton Zonda 9 FX | gdtf:160435 | 7 | 17–1954 | 1770 | 29 | 588 | 1–44 | 40 | 3 | Color Preset 94 | RGBW · wheel · CTC |
| Prolights HalupixDUO | gdtf:162855 | 27 | 12–1813 | 1813 | 18 | 441 | 2–490 | 441 | 3 | ColorMacro 90 | R G B WW |
| Ayrton Argo 6 FX | gdtf:160431 | 7 | 18–1072 | 956 | 29 | 318 | 1–26 | 22 | 3 | Color Preset 94 | RGBW · wheel · CTC |
| MAC Aura PXL | gdtf:159073 | 5 | 10–505 | 512 | 18 | 141 | 1–163 | 161 | 1 | Beam/Aura wheel 49 | ColorRGB (virtual mix) · 2 wheels |
| Solaris Flare XL 120 | gdtf:74383 | 17 | 3–488 | 488 | 7 | 120 | 1–121 | 120 | 1 | – | RGBW |
| Brighter X-DOT Max | gdtf:147669 | 12 | 25–318 | 322 | 19 | 75 | 2–97 | 95 | 1 | 2 macro wheels | R G B Lime · CTO · Tint |
| VDO Sceptron 10 1000 | gdtf:139321 | 9 | 1–300 | 300 | 4 | 100 | 0–20 | 20 | 1 | – | RGB |
| COLORado PXL Curve 5 | gdtf:147974 | 5 | 100–235 | 355 | 44 | 40 | 10–85 | 80 | 1 | macro 20 | RGBW+WW+CW · CTC |
| CP Tambora Stormy Linear | gdtf:113709 | 8 | 15–190 | 191 | 19 | 50 | 2–127 | 125 | 1 | – | RGB+WW+CW · CTO |
| Robin Tarrantula | gdtf:119741 | 6 | 26–187 | 196 | 24 | 36 | 3–41 | 37 | 1 | virtual 67 | RGBW · CTO · macro |
| Maverick MK3 Wash | gdtf:89324 | 10 | 7–154 | 297 | 21 | 27 | 1–28 | 27 | 1 | macro 17 | RGBW |
| CP B-EYE K20 | gdtf:74459 | 4 | 14–148 | 148 | 18 | 37 | 0–3 | 0 | 1 | macro 66 | RGBW |
| GLP JDC2 IP V2 | gdtf:150958 | 5 | 15–128 | 141 | 35 | 24 | 2–40 | 24 | 1 | – | RGB · CTC |
| Astera AX1 PixelTube | gdtf:62991 | **165** | 3–128 | 160 | 12 | 16 | 1–16 | 16 | 1 | virtual 151 | RGB / HSB / CIE by mode |
| Robin Tetra2 | gdtf:119736 | 6 | 33–118 | 128 | 23 | 18 | 4–23 | 18 | 1 | virtual 67 | RGBW · CTO · macro |
| Ayrton MagicPanel FX | gdtf:149953 | 3 | 19–115 | 118 | 18 | 25 | 1–26 | 25 | 1 | preset 25 | RGBW · macro |
| Robin Spiider | gdtf:119729 | 10 | 25–114 | 123 | 23 | 17 (+2 odd) | 2–23 | 19 | 1 | virtual 67 | RGBW · CTO · macro |
| COLORado PXL Curve 12 | gdtf:142714 | 13 | 19–101 | 179 | 23 | 12 heads | 1–13 | 12 | 1 | macro 18 | RGBW · CTC |
| COLORado PXL Bar 16 | gdtf:146713 | 13 | 6–101 | 186 | 22 | 16 | 1–17 | 16 | 1 | macro 17 | RGBW |
| GLP X4 Bar 20 | gdtf:162563 | 7 | 17–87 | 89 | 20 | 20 | 1–21 | 20 | 1 | Color Wheel 15 | RGBW · CTO |
| GLP impression X5 | gdtf:150111 | 10 | 17–86 | 89 | 22 | 19 | 1–21 | 19 | 1 | Color Wheel 65 | R G B Lime · CTC |
| MAC Aura XIP | gdtf:159072 | 5 | 12–85 | 93 | 20 | 12 (+7) | 1–22 | 20 | 1 | Beam/Aura 49 | ColorRGB · 2 wheels · Tint |
| Elation SixBar 1000 | gdtf:56156 | 8 | 6–79 | 79 | 13 | 12 | **0**–1 | 0 | 1 | macro 64 | RGBWA+UV |
| Elation Chorus Line 16 | gdtf:144065 | 3 | 15–76 | 78 | 16 | 16 | 1–17 | 16 | 1 | macro 65 | RGBW · CTC |
| Robin Tetra1 | gdtf:119735 | 6 | 24–73 | 81 | 23 | 9 | 3–13 | 9 | 1 | virtual 67 | RGBW · CTO |
| GLP JDC1 | gdtf:103537 | 6 | 10–67 | 68 | 12 | 12 + 12 | 2–27 | 12 | 1 | – | RGB |
| COLORado PXL Bar 8 | gdtf:89307 | 10 | 6–58 | 105 | 22 | 8 | 1–9 | 8 | 1 | macro 17 | RGBW · CTC |
| GLP X4 Bar 10 | gdtf:22993 | 7 | 16–47 | 49 | 19 | 10 | 1–11 | 10 | 1 | Color Wheel 15 | RGBW · CTO |
| Robin Esprite | gdtf:116608 | 5 | 40–41 | 51 | 41 | – | 1 | 0 | 1 | Col 6+6, Gobo 10+9, virtual 67 | CMY · 2 wheels · CTO · Tint |
| Ayrton Khamsin | gdtf:149945 | 3 | 36 | 64 | 36 | – | 1 | 0 | 1 | Col 7, Gobo 7+7 | CMY · 2 wheels · CTO |
| Maverick Force 2 Profile | gdtf:78256 | 2 | 31–34 | 50 | 34 | – | 1 | 0 | 1 | Col 8, Gobo 8+8 | CMY · wheel · CTO |
| Ayrton Perseo Profile | gdtf:159118 | 4 | 32–34 | 61 | 32 | – | 1 | 0 | 1 | Col 8, Gobo 8+12 | CMY · wheel · CTO |
| Rogue R2X Wash | gdtf:91587 | 8 | 12–33 | 56 | 17 | 5 | 1–6 | 5 | 1 | Colour 35 | RGBW · wheel |
| Robin MegaPointe | gdtf:150913 | 2 | 32 | 39 | 33 | – | 1 | 1 | 1 | Col 14, Gobo 10+15, macro 67 | CMY · 2 wheels |
| Robin BMFL Spot | gdtf:135703 | 2 | 31 | 41 | 31 | – | 1 | 0 | 1 | Col 7+7, Gobo 7+7, macro 97 | CMY · 2 wheels · CTO |
| Ayrton Diablo | gdtf:159108 | 3 | 30 | 56 | 30 | – | 1 | 0 | 1 | Col 8, Gobo 8 | CMY · wheel · CTO |
| MAC Quantum Wash | gdtf:37128 | 2 | 11–29 | 33 | 18 | 3 rings | 1–5 | 3 | 1 | Beam/Aura 37 | RGB · 2 wheels · CTC |
| MAC Viper Profile | gdtf:146138 | 2 | 22–27 | 34 | 26 | – | 1 | 0 | 1 | Col 9, Gobo 6+6+10 | CMY · wheel · CTO |
| CP Sharpy Plus | gdtf:159992 | 1 | 26 | 31 | 26 | – | 1 | 0 | 1 | Col 6×3, Gobo 19+9 | CMY · 3 wheels |
| Robin 1200 LEDWash | gdtf:113182 | 6 | 8–25 | 45 | 13 | 4 zones | 1–5 | 4 | 1 | virtual 248 | RGBW · CTO |
| MAC Aura XB | gdtf:40784 | 2 | 12–23 | 25 | 18 | – (Beams/Auras) | 1–2 | 0 | 1 | Beam/Aura 34 | RGBW · 2 wheels · CTC |
| Robin 600 LEDWash | gdtf:60186 | 6 | 8–21 | 37 | 13 | 3 zones | 1–4 | 3 | 1 | virtual 249 | RGBW · CTO |
| Robin Spikie | gdtf:99663 | 2 | 19 | 27 | 19 | – | 1 | 0 | 1 | macro 67 | RGBW · CTO |
| Martin Atomic 3000 LED | gdtf:103011 | 3 | 3–14 | 14 | 12 | – | 1–2 | 0 | 1 | Aura 37 | RGB · wheel |
| SGM P-5 | gdtf:14093 | 8 | 3–6 | 10 | 6 | – | 1 | 1 | 1 | – | RGBW / RGB+CTC by mode |

The rest of the heavy sample (Proteus Hybrid, Mythos, Pointe, Encore Performance WRM, Ghibli, Viper Wash, LEDBeam 150, impression X4) is in `sample.json`.

- **Scale:** up to 1,954 channel rows (Zonda 9 FX "Liquid", 3 breaks) and 441–588 instances of one part; elsewhere MTS LED Strip Stufe 5 (405), AS360 Flex Neon (400), DJ MATRIX BEAM (441).
- **One attribute on many instances:** Dimmer 1–490 instances in one fixture (HalupixDUO); Aura PXL 1–163.
- **Virtual dimmers are common:** 32 of 53 heavy fixtures, in 30 as per-pixel virtual dimmers beside a real master; broad set 195 of 366 (53%).
- **Some modes have no Dimmer at all** (SixBar 1000, B-EYE K20 pixel modes): a stored Dimmer must become RGB brightness or stay unused.
- **Footprint smaller than the channel count** in 4 fixtures (breaks / shared channels).
- **Multi-break:** 3 heavy fixtures use 3 breaks; broad 20 of 366 (5%).
- **Big "colour wheels" are mostly virtual macro wheels:** 25 of 60 in the heavy set have 43–249 slots (Robe "Virtual Color Wheel", Ayrton "Color Preset", Elation "Color Macro"). Slot colours almost always present (only 8 of 204 broad colour wheels lack them).
- **Mode counts:** median 5 in the heavy set, Astera AX1 165; 69% of broad revisions have more than one mode.

## 2. Modes of the same fixture (the common case)

All ordered mode pairs in the 53 multi-mode heavy revisions (30,448 pairs; AX1 alone 27,060), each fixture weighted equally.

| Problem on a mode change | Fixtures hit (of 53) | Average share of a fixture's mode changes |
|---|---|---|
| An attribute disappears | 46 | 40% |
| 8-bit ↔ 16-bit on the same attribute | 46 | 55% |
| The same attribute sits on a differently named geometry | 37 | 47% |
| Instance count goes down (collapse) | 36 | 31% |
| Instance count goes up (expand) | 36 | 31% |
| Virtual ↔ real on the same attribute (mostly Dimmer) | 32 | 32% |
| Colour system family changes (emitters ↔ virtual RGB ↔ wheel-only ↔ CMY) | 18 | 14% |

Broad set (366 revisions, first 30 modes each): 182 move an attribute to another geometry name between modes; 189 change an attribute's instance count; 107 have Pan, Tilt or Dimmer in both 8- and 16-bit.

| Fixture | From → to | What happens | Mapping problem |
|---|---|---|---|
| GLP X4 Bar 20 | Single Pixel → Dual Pixel | RGBW + Dimmer 20 → 10 instances; all 11 geometry names change (`SPix LED n` → `DPix LED n`) | Two pixels → one; name matching finds nothing |
| GLP X4 Bar 20 | Single Pixel → Comp | 20 instances → 1; wheel and gobo-spin appear | Average, first, or master? |
| GLP X4 Bar 20 | Norm → Single Pixel | 3 "Sets" → 20 LEDs; Dimmer becomes per-pixel virtual; 9 attributes lost | 3 zones resampled onto 20 pixels |
| Robin Spiider | Full RGBW → Zones | R/G/B 21 → 4 instances; White 2 → 4 | Pixels become rings (another part kind) |
| Robin Spiider | Basic → Pixel RGBW | Dimmer 8 → 16-bit and 2 → 22 instances; Effects1 lost | 19 pixels filled from one value |
| MAC Aura PXL | Ludicrous → Extended | 160 parts → 19; aura pixels vanish | The aura part has no target |
| MAC Aura PXL | Extended → Compact Direct | `ColorRGB_*` → `ColorAdd_R/G/B/W`; both wheels lost | Colour model changes inside one fixture |
| MAC Aura XB | Standard → Extended | Colour on `Head` splits into `Beams` + `Auras` | One value feeds two parts |
| Robin MegaPointe | 16-bit → 8-bit | 5 attributes lose their fine byte | Harmless in native units; fatal as DMX |
| Solaris Flare XL 120 | Mode 120 → Mode 20 | 121 instances → 21; names carry the mode | Six cells → one |
| COLORado PXL Bar 16 | Single Standard → Dual Pixels | 10 attributes lost (CTC, macros, effects, `Background Dimmer`, `B_Overwrite`) | The other half of a dual patch must carry them |
| COLORado PXL Curve 5 | Advanced → Basic | WW/CW lost; Effects11–16 → Effects6; 16 → 8-bit; Dimmer 85 → 10 instances | Same effect, different number |
| Astera AX1 | HUE SAT → CRO RGB | HSB ↔ RGB; other modes CIE x/y | Colour model by mode |
| GLP impression X5 | RGBL → RGB | Lime lost; `NoFeature` takes its slot | Lime folded into RGB through CIE |
| Ayrton Zonda 9 FX | Extended Pixel → Extended Ring | 37 LEDs → 3 rings + centre; all 24 geometry names change | Pixels become rings |

## 3. Type changes: eight swaps plus four controls

Tiers in order: **exact** (same GDTF name) · **colour** (converted as a whole through CIE) · **sibling** (same name, other number) · **noise** (placeholder / control, never carried) · **group** (an unmatched target attribute in the same feature group; suggestion only) · **none** (kept unused). Mode: "std" = richest mode without repeated parts; "max" = richest overall.

| Swap | Mode | Attrs (src → dst) | exact | colour | sibling | noise | group | none | Target attrs left empty | Parts (src → dst) |
|---|---|---|---|---|---|---|---|---|---|---|
| MAC Viper Profile → Robin BMFL Spot | std | 26 → 31 | 19 | 0 | 0 | 6 | 1 | 0 | 12 | – |
| Robin Esprite → Maverick Force 2 Profile | std | 41 → 34 | 26 | 2 | 0 | 6 | 7 | 0 | 8 | – |
| Robin MegaPointe → Proteus Hybrid | std | 33 → 23 | 16 | 1 | 1 | 3 | 4 | 8 | 7 | – |
| CP Sharpy Plus → Robin Pointe | std | 26 → 23 | 13 | 5 | 0 | 3 | 3 | 2 | 10 | – |
| MAC Aura XB → Rogue R2X Wash | max | 18 → 17 | 15 | 2 | 0 | 0 | 0 | 1 | 2 | Beams + Auras → 5 zones |
| Robin Spiider → MAC Aura PXL | max | 23 → 18 | 7 | 6 | 0 | 3 | 0 | 7 | 11 | 17 (+2) px → 19 beam + 141 aura |
| MAC Aura XIP → Robin 600 LEDWash | max | 20 → 13 | 7 | 5 | 0 | 3 | 0 | 5 | 6 | 7 + 12 → 3 zones |
| SixBar 1000 → COLORado PXL Bar 16 | max | 13 → 10 | 6 | 3 | 0 | 1 | 0 | 3 | 4 | 12 → 16 cells |
| GLP X4 Bar 20 → Robin Tetra2 | max | 11 → 23 | 9 | 0 | 0 | 2 | 0 | 0 | 14 | 20 → 18 px (+2 flower px, 2 zoom) |
| GLP JDC1 → Martin Atomic 3000 LED | std | 12 → 12 | 6 | 0 | 0 | 0 | 1 | 5 | 6 | 12 + 12 → none |
| PXL Curve 12 → PXL Curve 5 | max | 23 → 44 | 13 | 2 | 0 | 1 | 2 | 5 | 31 | 12 heads → 5 heads (3 geometries each + 2 × 40 nested beams) |
| PXL Curve 12 → Robin Tetra2 | max | 23 → 23 | 10 | 2 | 0 | 1 | 0 | 10 | 13 | 12 tilting heads → 18 cells on one tilt |
| Control: X4 Bar 20, revision change | both | 11 → 11, 16 → 17 | all | – | – | – | – | 0 | 0–1 | same |
| Control: HalupixDUO, revision change | both | 18 → 18, 5 → 5 | all | – | – | – | – | 0 | 0 | same |

- **Exact names carry most of a spot / profile (50–75%), much less of a pixel wash (~30%).**
- **Colour covers most of the rest** (2–6 per pair: Sharpy Plus's 3 wheels + CMY onto a wheel-only Pointe; Spiider RGBW + CTO + macro onto virtual RGB; SixBar's Amber + UV onto RGBW).
- **Placeholders swell "unmatched"** (Viper → BMFL: all 6 non-exact are `NoFeature…NoFeature5`; Esprite → Force 2: `*MSpeed`, `LEDFrequency*`, `IntensityIndication`).
- **Naive "same group" suggestions are mostly wrong** (Viper `Gobo3` → `AnimationWheel1Macro`, `Blade1Rot` → `Blade1B`, MegaPointe `Gobo1` → `AnimationSystem1`, `Prism2` → `Effects1Rate`, `PositionEffect` → `Pan`); plausible: `Color1WheelIndex` → `Color1`, `StrobeRate` → `Shutter1Strobe` (3rd candidate).
- **Instance mismatches are routine** in washes and battens (12 → 16, 20 → 18, 12 heads → 5, 2 parts → 5 zones).
- **Revision changes were clean** (both controls: same names and counts).
- **Heads can become not-heads** (Curve 12's per-head Tilt × 12 → Tetra2's single tilt).
- **Wheel slot counts never match across makers** (Viper colour 9 vs BMFL 7 + 7; Sharpy 6 + 6 + 6 vs Pointe 14; Force 2 8 vs Esprite 6 + 6): the only usable signal for colour slots is ΔE.

## 4. Attribute naming in the wild

Library-wide (13,609 revisions, 183,732 uses):
- **Long tail, thin:** 5,033 distinct names, 530 (10.5%) GDTF-standard, yet standard names cover 91.4% of uses.
- **Non-standard share by group:** Other 86.6% · Video 15.5% · Color 5.5% · Control 1.5% · Dimmer 1.4% · Focus 1.4% · Gobo 1.2% · Position 1.0% · Beam 0.5% · Shapers 0.2%.
- **Color, 1,211 non-standard names:** `Color XF` (113), `ColorMacro` (101), `ColorXFade` / `ColorXfade` (59 / 50), `Green / Magenta` (42), `CTC<>RGB` (37), `ColorAdd_DR` (35), `ColorAdd_I` (33), `Warm/Cool` (28).
- **Other, 3,147 names (12,812 uses):** `PT Speed` (204), `Crossfade` (130), `Background Dimmer` (78) / `BackgroundDimmer` (63).
- **Names with spaces:** 2,216 distinct (6,583 uses); variants pile up (`Zoom Rota`, `ZoomRot`, `Zoom  Rt`, `ZoomRotation`).
- **Placeholders:** `NoFeature` 2,212 uses, plus `NoFeature1…10`, `Dummy2…4`.
- **Numbered families:** `Effects<n>*` 84 names / 6,880 uses; `Control<n>` 24 / 4,105; `Shutter2+` 345; `Color2+` 910.
- **Dimmer-like custom names:** 388 distinct (1,092 uses) — part intensities dressed as attributes.
- **Two RGB vocabularies:** `ColorAdd_*` 33,488 uses, `ColorRGB_*` 3,287; 21 broad revisions use only `ColorRGB_*` (the Martin Aura family), 34 use both.

In the samples: 171 of 366 revisions (47%) have a non-standard attribute, 160 (44%) one in Other; no null names in GDTF (Depence would add them). **Gobo roles are not fixed by number** — of 56 with both Gobo1 and Gobo2: Gobo1 static / Gobo2 rotating 34 (61%), the reverse 5, both rotating 15, both static 2; Martin's Viper calls its animation wheel `Gobo3`. **Effects numbers change between modes of one fixture** (Curve 5: Effects11–16 vs Effects6).

## 5. Geometry and part naming

**Common part-name families** (3,003 repeated parts in 366 revisions): `Pixel-#-#-#` (578, Quasar) · `#/# Pixel #` (265, Astera) · `LE_#_#` (140, Aputure) · `Segment#of#` (81, Martin VDO / Exterior) · `LED #` · `Pixel #` · `Beam_#` (Chauvet) · `HPX#` / `B##` / `Pix##` (Prolights) · `Head #` · `Zone #` (Robe washes) · `Dot #` · `Ring #` · `Reference #` (Ayrton Liquid) · `Cell # Mode# RGB` (Solaris).

**What breaks name-based grouping:**
- **Mode prefixes / suffixes:** GLP `Norm/Comp/SPix/DPix/Hires`, `M3 Single Plate 4`; Solaris `Cell 3 Mode120 RGBW`; Ayrton `LED 4 Ext`.
- **Odd siblings:** 549 of 3,003 parts (18%) have a lone same-attribute geometry with another name (Spiider `Lens1` and `P2 Zone 2` beside `P3 Zone2…P19 Zone2`: name rules find 17 of 19 pixels; Tarrantula `Lens1`; Zonda / Argo `LENS 1` beside `LED n Ext`; X-DOT Max `Do 70`; Stormy Linear `Sy 88`, `S 100`; JDC1 `M34Single Beam`).
- **One head of several geometries (PXL Curve 5):** Pan on `Head n`, Tilt on `Body(Head n)`, colour on `Center(Head n)`, plus 8 nested `CW Beam(Head n) m` — the parts model needs nesting and a shared instance key.
- **Special instances:** Tetra2's `P5 Flower`, `P14 Flower` carry extra attributes.
- **Two parts with identical attributes:** Aura PXL's Beam Pixel (19) and Aura Pixel (141); JDC1's plate and beam segments — grouping by attributes alone would wrongly merge them.
- **Coordinates inside names:** PixelBrute `H__3x4_1`, Quasar `Pixel-2-1-5` — a shape hint in each maker's own format.

### Proposed part-kind rules (first hit wins)

| # | Rule | Gives |
|---|---|---|
| K1 | Part has Pan / Tilt / PanRotate / TiltRotate, or its name says "head" and it isn't nested below a head | **Heads** |
| K2 | Part has no light attribute (no Dimmer, Color\*, CTx, Shutter\*) | **Mechanical** |
| K3 | Name says ring / aura / halo / outer / background / backlight / filament / surround / corona, or zone / segment / section / set with ≤ 6 instances on a non-bar fixture | **Ring** |
| K4 | Otherwise | **Cells** |
| S1 | Fixture name says matrix / panel / grid / tile / cube / AxB | Grid |
| S2 | Fixture name says bar / batten / strip / tube / line / linear / curve / NNNNmm / sixbar | Linear |
| S3 | Part is a Ring, or count (+ odd siblings) is 7 / 19 / 37 / 61 / 91 / 127, or fixture name says aura / ring / eye / halo / circle / round / spiider | Circle |
| S4 | Count is a square ≥ 25 | Grid |
| S5 | Part is Heads | Linear |
| — | None of the above | **? (ask)** |

**54 labelled parts** (richest mode of the 34 heavy fixtures with parts): kind 52 of 54 (96%; misses: R2X Wash's 5 concentric `Beam_n` zones and X-DOT Max's aura `Dot n`, both rings called Cells); shape 37 right, 0 wrong, 7 asked of 44 scored (Tetra1/2, JDC1/2 — names don't say "bar"); both right 46 of 54 (85%).

**Broad set** (730 parts in 187 revisions): Cells / ? 374 · Cells / Linear 168 · Cells / Grid 84 · Ring / Circle 47 · Cells / Circle 34 · Heads / Linear 9 · Mechanical 6 · Ring / Grid 5 · Heads / Grid 2 · Ring / Linear 1. A shape guessed for 49%. Spot check of 45: 25 asks, 20 guesses — 17 plausible, 2 wrong (Universe25 PL1 `Mode#-Plate-Zone #` → Ring/Circle; "BLITZBARFX" 64 cells → Grid since "BAR" is glued in and 64 = 8²), 1 unsure. Fixes to try: match "bar" without a word boundary; never S4 when the name contains "bar". The count alone can't tell Linear from Grid (9, 16, 25, 100 occur as both).

## 6. Predicted problems, ranked by impact

**P1. Instance counts differ — resampling is the normal case.** 36 of 53 heavy fixtures between their own modes; 189 of 366 broad; 7 of 8 wash / batten swaps (X4 Bar 20 20 → 10 → 3 → 1; SixBar 12 → PXL Bar 16; Aura PXL 160 → 19 → 1). Resample by normalised position along the part's shape; n → 1 = average (or the master); 1 → n = copy. Beginner: "Pixels 20 → 10, values blended". Advanced: average / first / brightest, a per-instance preview.

**P2. Geometry names are not identities.** 37 of 53 heavy; 182 of 366 broad (`SPix LED 7` vs `DPix LED 7`; `Head` vs `Beams` + `Auras`; `Lens1` vs `P2 Zone2`). Key values by (part, instance index), the geometry → part link stored once per fixture type; detect instances by attribute set + name family; offer odd siblings (18%) as "1 more instance with an unusual name — include?". Beginner: nothing. Advanced: the link table.

**P3. Colour changes model, not just names.** The largest non-exact block in 9 of 12 swaps; 18 of 53 heavy change colour family between their own modes; 21 broad use only `ColorRGB_*` (Spiider → Aura PXL; AX1 RGB / HSB / CIE; X5 RGBL → RGB; Sharpy Plus → Pointe). Convert each part's colour as a whole through CIE; accept ColorRGB, HSB and CIE as inputs; treat virtual macro wheels (25 of 60, 43–249 slots) as presets, not glass. Beginner: one "Colour converted" row with a swatch pair, a warning only above a ΔE threshold. Advanced: per-emitter values, wheel slot choice.

**P4. Placeholders flood "unmatched".** 1–6 per swap in 11 of 12; `NoFeature` 2,212 uses (Viper → BMFL "6 unmatched" = `NoFeature…5`). A fixed noise class, never carried, never counted as a loss: `NoFeature*`, `Dummy*`, `*MSpeed`, `*MTime`, `*Reset`, `Control<n>`, `*Mode`, `LampControl`, `LEDFrequency*`, `IntensityIndication`. Beginner: one grey line "6 settings channels ignored", or nothing.

**P5. "Same feature group" suggestions mislead** (~2 of ~20 right). Never pre-select; rank number-siblings, then same-stem names, then group. Beginner: "Not used on the new fixture" with a "Use for…" button. Advanced: the candidate list.

**P6. Numbered roles are unstable** (Gobo1 static 61% / rotating 36%; Effects numbers change between modes). Pair gobo wheels by role first (rotating = has `GoboNPos` / `PosRotate`), then number; `Effects<n>` across types unmatched unless the channel-function names match (LumaCore has them from the GDTF; this API doesn't).

**P7. Wheel slots never match across makers** (4 of 4 cross-maker spot / beam swaps; gobo slot names generic, media empty). Colour slots by ΔE; gobos default to open, say so, ask. Beginner: "Gobos: 4 values need a choice".

**P8. One value feeds several parts, or several parts one value.** 37 heavy fixtures move attributes between geometries; every beam + aura fixture (Aura XB / XIP / PXL, X-DOT Max, Atomic). A part-to-part table; the main emitter part takes the whole-fixture value; a ring / aura part with no match stays at its default unless "copy main colour to aura" is ticked.

**P9. Virtual vs real intensity changes meaning** (32 heavy; 195 of 366 broad; some modes have no Dimmer). Master → master, pixel → pixel; with no Dimmer in the target, fold intensity into RGB through the colour conversion and say so; never drop it silently.

**P10. Heads and nesting** (rare: 2 heavy, 11 of 730 broad parts, but the hardest: Curve 12 → Curve 5 12 heads → 5, each 3 geometries + 8 + 8 nested beams; Curve 12 → Tetra2 12 tilts → 1). The parts model needs a part inside a part and instances that span several geometries.

**P11. Shape is often unknown** (51% of broad parts "?"). Ask once per fixture type with pictures of Linear / Grid / Circle and the count, pre-filled when a rule fired; remember it on the type.

**P12. 8 ↔ 16-bit and footprint quirks** (46 of 53 heavy; 107 broad; footprint < channel count in 4). Nothing needed when values are native (decided). Never shown to a beginner.

**P13. Very large fixtures** (165 modes, 1,954 channel rows, 588 instances). Summarise per part, never list instances; the mode picker needs search; compute mappings off the state thread.

### Beginner view sketch

One row per part (Main, Aura, Pixels × 20, Heads × 12); inside each, one row per topic (Intensity, Colour, Position, Gobo, Beam, Focus, Effects) in one of four states: ✓ kept · ↻ converted / resampled · ? needs a choice · – not used on the new fixture. The noise class hidden; group suggestions, geometry links and per-instance values behind "advanced". From the sample, a typical cross-maker spot swap shows about 3 "?" rows (gobo wheels, animation wheel, prism 2) and 1–2 "–"; a pixel wash swap shows "Pixels 17 → 19 resampled", "Aura: no match", "Colour converted".

## Files

- `report.md` — this file.
- `sample.json` — per-fixture / per-mode summaries: `heavy`, `mode_diffs` + `mode_diff_totals`, `type_pairs`, `broad`, `naming`, `parts` (labels + rule output).
- `build.py` — the analysis; imports `fetch.py` (download) and `api.py` (cached, rate-limited client).
- Raw JSON stays in the session scratchpad (`raw/`, about 560 responses). Rebuild: `python3 -I fetch.py`, then `python3 -I build.py`.
