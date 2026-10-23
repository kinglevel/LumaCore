# Phone hardening mockups (2026-10-19)

Decisions: `docs/archive/decisions-2026-10.md` §37 "Phone hardening". The audit: `tests/e2e/tests/audit-phone.spec.ts`
(run with `AUDIT_OUT=<dir>`; 33 surfaces × phone 390 × 844, phone 360 × 740, phone landscape
844 × 390, tablet 768 × 1024 and 1024 × 768, each with what the page measures about itself).
One artifact per problem, with the real captures and the options side by side.

| File | Artifact | Status |
|---|---|---|
| `phone-1-patch.html` | https://claude.ai/artifact/EC6GaY6kGvbMsSwwCTzSCu (v1) | Problem 1, the patch window gives the list one row: A fold the chrome · B slim the slab · C both · D focus the list. **C chosen.** |
| `phone-2-landscape.html` | https://claude.ai/artifact/NSiD1NHDPZ72ytbUjEeWgR (v1) | Problem 2, a phone on its side draws the desktop layout in 390 px: A upright rules on its side · B the bar on the side · C tree beside the list. **B chosen.** |
| `phone-3-labels.html` | https://claude.ai/artifact/6LV78SxxKdXqR4oCwYNqSR (v1) | Problem 3, slab labels run into each other in a narrow dialog: A two lines · B short labels · C icons only · D clip with …. **B chosen.** |
| `phone-4-insert.html` | https://claude.ai/artifact/3Mm7y954PC5XYQH5119dVa (v1) | Problem 4, Insert fixtures stacks three columns on a phone: A one step a screen · B one page, folding sections · C keep the columns, compress. **A chosen.** |
| `phone-5-map.html` | https://claude.ai/artifact/44GSJ5rzhSyMpQZ5u6TbrH (v1) | Problem 5, the Universes map's bars are unreadable on a phone: A wrap the bar · B a list, tap to zoom · C one bar, sideways. **B chosen** ("there can be a lot of universes"). |
| `phone-6-slots.html` | https://claude.ai/artifact/SiePFgeba2gguhpUWgueVV (v1) | Problem 6, the slot grid's 24 × 32 px slots on a phone: A rows of 10 · B rows of 20, scroll sideways · C rows of 20, a loupe. **A chosen.** |
| `phone-7-context.html` | https://claude.ai/artifact/WYmG35vrRajpneHB2E1e9K (v1) | Problem 7, the context menu hangs from the tree tabs on a phone: A a bottom sheet · B a page · C quick row + sheet. **A chosen.** |
