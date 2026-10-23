---
id: 0294
title: "Object paint: palette, free and theme links; colour from tags"
area: ui-kit
status: built
date: 2026-10-23
built_in: object paint (2026-10-23)
supersedes: [nodemeta-colour-icon]
superseded_by: —
confirmed_by: [lumacore-base paint tests, paints_become_hex_and_feature_groups_follow_their_slots, commands::paint_slots tests, the_colour_order_is_own_then_tag_priority_then_parent_then_pool, lumacore-views paint and rules tests, a_one_of_a_kind_object_as_a_value_takes_a_name, kit-paint.spec.ts, kit-rules.spec.ts]
source: [session 2026-10-23, object paint rounds 1–4 — docs/dev/ui-kit/mockups/2026-10-23-paint/ (04 approved: "looks good, implement")]
---
# Object paint: palette, free and theme links; colour from tags

## Context
A Paint was RGBA bytes typed as hex, drawn only in pool cells; tags, feature groups, users and window settings each parsed colours their own way, and the settings dialog sent a Theme word object Paint refused. Four mockup rounds settled it. This replaces 0205's bytes and its "none = the pool's colour"; colour stays in NodeMeta. Not taken: alpha, a per-show palette, a light-base colour per slot.

## Decision
A Paint is `#rrggbb`, a palette word (12 hues, Black, White; stored as hex) or a theme slot link `Theme "Position"` (grammar `named-one`), saved as `"#rrggbb"` / `{"Theme": …}`; `none` clears. `lumacore_base::Paint::parse` is the one parser, `paint::hex_rgb` the one hex reading. Each profile's theme has paint slots — LumaCore's (`media::PAINT_SLOTS`) and its own: `Set … Theme Paint "<slot>"`, `Store …` adds, `Reset …` puts back. A feature group's Paint defaults to the slot of its name. The colour an object shows is resolved per viewer on the station (`lumacore_state::paint::Painter`): own Paint → the painted tag with the highest `Priority` (ties: tag tree order; a tag without Paint takes its parent's) → the pool's colour; a link to a slot the viewer lacks is kept, named and falls back. Feeds send the colour and where it came from. `<luma-paint>` picks (Palette, Free, Theme; recent colours and mode on the profile); every pick is one `Set … "Paint"` line. Every show object with a NodeMeta takes a Paint. The Rules window lists the AttributeRules.

## Consequences
A place that shows an object's colour reads a `Painter`, never the meta; clients never resolve a Paint. A wrong value is E-SET-003. The show format and both protocols bumped. Gradient and by-fixture-type painting are planned.
