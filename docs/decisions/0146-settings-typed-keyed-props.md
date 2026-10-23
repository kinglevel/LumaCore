---
id: 0146
title: Station, Interface, Network and Show settings are typed keyed properties
area: settings
status: built
date: 2026-10-13
built_in: 12d part 3
supersedes: []
superseded_by: —
confirmed_by: [kit-12d.spec.ts]
source: [archive §30 L4715–4721, archive §30 L4899–4903, archive §30 L5047–5062, archive §30 L5150–5153]
---
# Station, Interface, Network and Show settings are typed keyed properties

## Context
The setup menus needed cells that say what a value is and where it lives. `Set Station`, the interface services and the asset folders each had a hand parser (`set_station`, `set_interface*`, `set_asset_folder`), and the user asked that text, number, switch and pick list be told apart at a glance without colour per kind.

## Decision
The station's settings (name, location, LumaSession, startup show, priority, theme override, multicast base, Online, Invite, HoldOutput, OutputWhenAlone), an interface's services (LumaSession / Shell / HTTP), the network-to-NIC map, the show's settings (description, autosave, merge default and timeout), the asset folder per drive (`Set AssetRoot "Video" Folder …`) and About (read-only, no home mark) are typed keyed properties (`Access::Keyed`) of the one registry — declared today in `lumacore-cmd-station`'s `settings` / `wiring` and `core::settings_props`, each with home, unit, section and rights. `Set`, `EditSetting` and the menus' grid columns read them; one `Set` line writes many rows (`Set <Object> 1 Thru 3 + 5 "<Prop>" <value>`: `a Thru b` spreads, relative per row, refused rows skipped and named with codes, one Oops step). `SettingField` carries home, unit, rights and readonly.

Every settings cell and edit-card field shows its kind badge in one neutral outline: Aa text (an underlined field), 123 number (right-aligned with unit, steppers, keypad on tap), I/O switch, ≡ list (a pick list on tap; grid columns of kind list show ▾); read-only cells grey the badge. Dangerous settings are station asks (`Set Station Online Off`, `HoldOutput On`).

## Consequences
A new station or show setting is a keyed `Property` on its object (see one-property-registry); never a hand parser. The words on every cell ("This station · never in the show") decided here are superseded by an icon, see settings-home-icons.
