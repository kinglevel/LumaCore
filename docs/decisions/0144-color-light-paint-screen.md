---
id: 0144
title: Color is light (CIE xyY); Paint is screen (sRGB hex)
area: properties
status: built
date: 2026-10-15
built_in: 3c2b
supersedes: []
superseded_by: —
confirmed_by: [NodeMeta "Paint"]
source: [archive §33 L7497–7504, archive §33 L7846–7863]
---
# Color is light (CIE xyY); Paint is screen (sRGB hex)

## Context
One word `Color` named both a fixture's light colour (the feature group) and an object's pool colour on screen, so a property could not say which it took. Two kinds by role were decided when the theme's colours became properties.

## Decision
`Color` is light: the feature group (GDTF), stored as CIE 1931 xyY (device-independent, like GDTF's wheels, emitters and filters) and converted to each fixture's emitters. `Paint` is screen: the theme's colours and an object's own pool colour, sRGB with alpha written `"#rrggbb"`. The `NodeMeta` property is `"Paint"` (`Set Group 1 "Paint" "#3c8f3c"`), a user's own colour is `"Paint"`, the pool windows' colour settings are `"EmptyPaint"` and the like, and a theme's colour group is the child object `Paint` (see theme-lockscreen-paint-objects). A property declares which kind it takes; `none` on a Paint means the base's or the pool type's colour.

## Consequences
Never name a screen colour `"Color"`, never store a light colour as sRGB. How a light colour is written and handled on the line (`At Color …`, CIE to emitters, presets, blending) is its own hardening on the roadmap, not decided here.
