# Decisions — the index

Built by `scripts/unix/docs_check --index` from every record's frontmatter; do not edit by hand.
One file per binding decision; a reversal is a new record that supersedes the old one.
The log these came from is frozen at `docs/archive/decisions-2026-10.md`; `source` cites its section and lines.

Records: 217.

## attributes

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0150](0150-attributes-two-layers.md) | "Attributes: a show layer and a user layer, never mixed" | built | [FORMAT_VERSION bump, E-USER-002] | [archive §30 L5920–5922 "EncoderBarPool on UserProfile"] | [archive §30 L5920–5969, archive §30 L6083–6113, archive §30 L6585–6592] |
| [0151](0151-attribute-resolver-data-driven.md) | "The attribute resolver is data: table, family rules, templates" | built | none | [] | [archive §30 L5987–5991, archive §30 L6017–6026, archive §30 L6574–6579] |
| [0152](0152-attr-board.md) | "One attribute board: show, profile, new attributes" | built | [attributes feed, Status.attributes_waiting] | [] | [archive §30 L6048–6063, archive §30 L6072–6082, archive §30 L6097–6100, archive §30 L6114–6120, archive §30 L6141–6150, archive §30 L6243–6245, archive §30 L6593–6608] |
| [0153](0153-featuregroup-attribute-syntax.md) | FeatureGroup, Attribute, AttributeRule, EncoderBar words; scope options are owner keywords | built | [E-CMD-007, the 12f-a flags test (no E-CMD-018)] | [archive §30 L6154–6156 "Attribute is a command"] | [archive §30 L6154–6224, archive §30 L6580–6584, archive §30 L6672–6674] |

## backup

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0083](0083-autosave-recovery-versions.md) | Every save keeps a version; autosave writes recovery copies | built | [backups_tests, plan_cleanup unit tests, E-CMD-012, E-SHOW-006] | [archive §9 L417–419 "master-initiated network-synchronized save", archive §9 L422–430 "backups newest-10 scheme"] | [archive §9 L415–430, archive §21 L1654–1657, archive §23 L2147–2150, archive §30 L4722–4737, archive §30 L5242–5276, archive §30 L5338–5346, archive §30 L5361–5366] |
| [0084](0084-load-show-dialog-history.md) | Load show is one dialog; Import Show by GUID; history matrix | built | [backups_tests, core::show_import, GET /api/show-parts] | [archive §30 L4737 "Partial load later", archive §30 L4864–4872 "separate load parts"] | [archive §30 L4864–4903, archive §30 L5277–5309] |

## build

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0236](0236-lumacore-root-env.md) | Code reaches the repo root through LUMACORE_ROOT | built | none | [] | [archive §34 L8294–8296 (23.6), archive §34 L8786–8797] |
| [0250](0250-build-times-stamp-profile-jobs.md) | Git stamp in the station, local profile, jobs, bench rows | built | [`lumacore_station::stamp()`, `docs/dev/build-times.md`] | [archive §31 L6854–6857 "jobs = 2", archive §34 L8929–8931 "lumacore-luma build.rs stamp"] | [archive §34 L10721–10774] |
| [0251](0251-build-profiles-local-checks-hygiene.md) | Build profiles, pinned toolchain, local checks, git hygiene | built | [`./scripts/unix/test check`, `rust-toolchain.toml`, `[workspace.lints]`] | [] | [archive §31 L6854–6868] |

## catalog

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0130](0130-dispatch-special-cases-are-hooks.md) | Dispatch's special cases are Hook rows families push | built | [the objects crate's tests, `tests/catalog`] | [archive §34 L8974–8985 "dispatch naming families"] | [archive §34 L8974–8985, archive §34 L8997–9017, archive §34 L9097–9113] |
| [0241](0241-catalog-one-registry.md) | The catalog: every crate pushes its rows at startup, frozen once | built | [`freeze()`, tidy `crates_on_the_registry_register_and_install`, tidy `the_station_only_assembles`] | [archive §34 L8156–8161 "f before g"] | [archive §34 L8127–8161 (items 21.1–21.7), archive §34 L8173–8176 (21.9), archive §34 L8640–8730, archive §34 L8862–8868] |
| [0242](0242-test-binary-installs-catalog-via-ctor.md) | A test binary installs its catalog before its first test | built | [tidy `no_unsafe_code`, tidy `crates_on_the_registry_register_and_install`] | [] | [archive §34 L8162–8172 (item 21.8), archive §34 L8652–8660] |
| [0243](0243-properties-stay-beside-row.md) | Properties stay on the ObjectDef, not in catalog rows | built | [`lumacore_views::column_json`, catalog test `every_column_prop_resolves_on_its_object`] | [archive §34 L8134–8137 "menu column names"] | [archive §34 L8177–8184 (item 21.10), archive §34 L8718–8720] |
| [0244](0244-rows-typed-above-registry.md) | Rows whose type lives above the registry are handed back by type | built | [`Registry::check` at freeze] | [] | [archive §34 L8253–8259 (item 22.7), archive §34 L8768–8770] |

## claims

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0004](0004-claims-by-tree-path.md) | Claims by GUID: editors claim, cd never, owner asked | built | [E-CLAIM-001] | [archive §4 L250–253 "bare keyword quick claim", archive §4 L245–249 "claim Patch by word", archive §21 L1670–1671 "ListClaimed replaced", archive §21 L1665–1668 "Allow Deny buttons"] | [archive §4 L243–260, archive §10 L476–481, archive §21 L1497–1500, archive §21 L1658–1672, archive §21 L1712–1720] |

## commands

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0009](0009-selections-one-only.md) | Select one DataPool, Sequence, Timecode; SelectFixtures selects fixtures | built | [E-PLAY-003, E-CMD-014] | [archive §29 L3202–3203 "selected_datapool on profile"] | [archive §29 L3194–3215, archive §29 L3281–3284, archive §33 L7182–7188, archive §33 L7244–7247, archive §33 L7279–7290] |
| [0010](0010-selection-ordered-one-range-rule.md) | Selection is ordered; the one range rule | built | [parser::number_list] | [] | [archive §2 L194–212] |
| [0012](0012-variables-three-tiers.md) | User, Show and Global variables | decided | none | [] | [archive §5 L281–291, archive §15 L1189–1191] |
| [0103](0103-owner-first-syntax.md) | Nested objects are addressed owner first | built | [E-SYN-003, E-SYN-004, `Props::within` / `scoped`, `resolve::resolve_owned` / `slot_owned`] | [archive §33 L7013–7020 "dotted Theme.AccentHue", archive §30 L3843–3845 "Set UserProfile Theme.Preset", archive §33 L7669–7670 "child-first cue timing"] | [archive §30 L3801–3804, archive §30 L3839–3846, archive §33 L7027–7038 (item 9), archive §33 L7307–7325 (item 49), archive §33 L7479–7487 (items 59, 60), archive §33 L7664–7671 (item 87), archive §33 L7745–7790 (3d1–3d8)] |
| [0106](0106-tree-paths-typed-node.md) | Tree paths are typed; code names a Node, never a string | built | [tidy `tree_paths_are_typed`, golden `tests/catalog/testdata/tree_paths.golden.txt`, `Node::no` / `nth_child`, E-SYN-003] | [archive §31 L6815–6817 "string compares ~10 files", archive §31 L6827–6828 "cd enter lenient", archive §33 L7307–7325 "numeric positions refused"] | [archive §31 L6815–6837, archive §33 L7210–7215 (item 31), archive §33 L7307–7336 (items 49, 50), archive §33 L7388–7419 (items 53, 54, 55, 56)] |
| [0112](0112-typed-handlers-derive-args.md) | Handlers are derived Args structs bound from the right | built | [`compile_fail` tests of `lumacore-syntax-derive`, `Signature::check`, E-SYN-001, E-SYN-014, the ratchet `the_shim_count_only_goes_down`] | [archive §31 L6803–6808 "no signature change"] | [archive §33 L6996–6999 (item 4), archive §33 L7116–7123 (item 17), archive §33 L7362–7376] |
| [0121](0121-bare-object-default-verb.md) | A bare object runs its declared default verb | built | [`ObjectDef::default_verb`, `ObjectDef::bare_verb`, the generated object reference] | [archive §33 L7924–7942 "old catch-all Call", archive §33 L7291–7299 "Show action options"] | [archive §33 L7008–7012, archive §33 L7189–7194, archive §33 L7240–7271, archive §33 L7291–7299, archive §33 L7505–7509, archive §33 L7924–7942] |
| [0122](0122-at-verb-on-target.md) | Target At value is the At verb; levels only with At | built | [`Props::hint`, E-SYN-004, E-SYN-001] | [archive §33 L7778–7790 "child-first Master forms"] | [archive §33 L7021–7026, archive §33 L7272–7278, archive §33 L7690–7697, archive §33 L7778–7790] |
| [0125](0125-watch-is-a-verb.md) | Watching is the Watch verb only | decided | none | [archive §33 L7200–7204 "global /Watch option"] | [archive §33 L7200–7204, archive §33 L7924–7942] |
| [0126](0126-list-info-dump-one-command.md) | List, Info and Dump are one command each | decided | none | [] | [archive §33 L7709–7744] |
| [0127](0127-assign-places-objects-actions-are-verbs.md) | Assign places objects; Set never acts; actions are verbs | built | [the typed `Sort` / `Split` rows with `/By`, `Assign` / `Call` in the generated reference] | [archive §33 L7656–7663 "Set Attribute FeatureGroup", archive §33 L7672–7689 "Set FeatureGroup Sort|Merge|Split"] | [archive §33 L7656–7663, archive §33 L7672–7689] |
| [0128](0128-relative-numbers-on-station.md) | Relative numbers are parsed on the station | built | [E-CMD-008, `relative_tests`] | [archive §30 L4161–4164 "unparseable was 0 %"] | [archive §30 L4082–4084, archive §30 L4161–4164, archive §30 L4538] |
| [0129](0129-errors-coded.md) | Every error reply carries a code | built | [tidy `error_replies_carry_a_code`, the sweep's `uncoded-error` finding, the error catalog's format and uniqueness test] | [archive §26 L2514–2522 "handler string errors"] | [archive §26 L2514–2522, archive §29 L3112–3116, archive §29 L3262–3265] |
| [0293](0293-presets-link-into-cues-update-recast-extract.md) | Presets link into cues; Update, Recast, Extract | built | [objects::preset::links tests, a_cue_value_from_a_preset_keeps_only_its_link, an_older_show_upgrades_step_by_step_to_today, a_linked_value_shows_its_preset_by_the_show_setting, a_linked_preset_is_used_by_its_cues, kit-update.spec.ts] | [one-preset-pool-follows-the-programmer] | [session 2026-10-23, preset links rounds 1–3 — docs/dev/ui-kit/mockups/2026-10-23-preset-links/ (03 approved: "approved"), how a reference console updates, recasts and extracts presets (session research)] |

## crates

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0046](0046-protocol-contract-below-the-tree.md) | The protocol contract sits in model, below the tree | built | [tidy `crate_edges_point_down`, `EDGES`] | [archive §34 L8185–8252 "only the interface", archive §34 L8758–8767 "net on tree"] | [archive §34 L8462–8503, archive §34 L8936–8946, archive §34 L8758–8767] |
| [0051](0051-render-engine-is-the-renderer.md) | lumacore-render holds the plan and the tick | built | [the budget tests in `engine/tests.rs`, `benches/render.rs`] | [archive §34 L8091–8095 "lumacore-dmx keeps wire"] | [archive §34 L8091–8099, archive §34 L8740–8757] |
| [0227](0227-layers-point-down.md) | Crates sit in domain folders and their edges point down | built | [crate_edges_point_down, crates_sit_in_their_group] | [code-structure-later-items, archive §23 L2029–2033 "Modules now; separate", archive §34 L7992–7998 "cut in three", archive §34 L8266–8273 "Eight groups under", archive §34 L8278 "control/ holds osc"] | [archive §23 L2007–2049, archive §23 L2071–2105, archive §34 L7992–7998 (item 7), archive §34 L8021–8040 (items 15–16), archive §34 L8593–8620, archive §34 L8260–8302 (23.1–23.7), archive §34 L8369–8376, archive §34 L8386–8389, archive §34 L8456–8461, archive §34 L8786–8797, archive §34 L9270–9276] |
| [0228](0228-crate-naming-lumacore-category.md) | Every crate is lumacore-category; folder equals package | built | [crates_sit_in_their_group] | [archive §34 L8043–8046 "the two that"] | [archive §34 L8041–8047 (item 17), archive §34 L8287–8288 (23.3), archive §34 L8390–8395 (25.6)] |
| [0229](0229-crate-readme.md) | Every crate has a README naming its checks | built | [crates_have_readmes] | [] | [archive §34 L8048–8063 (item 18)] |
| [0231](0231-station-code-becomes-crates.md) | The station's modules become crates in their groups | built | [crate_edges_point_down, the_station_only_assembles] | [archive §34 L8091–8094 "lumacore-dmx splits along", archive §34 L8320–8321 "net/sync.rs"] | [archive §34 L8091–8118 (items 20.3–20.6, 20.10), archive §34 L8315–8331 (24.2), archive §34 L8803–8891] |
| [0232](0232-views-one-source-of-ui.md) | lumacore-views is the one source of what any UI shows | built | none | [] | [archive §34 L8104–8107 (20.7), archive §34 L8731–8739] |
| [0234](0234-trace-macros-one-door.md) | The trace macros have one door, lumacore-trace | built | [shared_doors_have_no_copies] | [] | [archive §34 L8113–8116 (20.9), archive §34 L8630–8639] |
| [0235](0235-dev-tools-never-ship.md) | No product crate depends on the developer tools | built | [dev_tools_never_ship] | [archive §34 L8273 "tools/ tidy banner"] | [archive §34 L8117–8118 (20.10), archive §34 L8426–8432 (25.8), archive §34 L8878–8891] |
| [0245](0245-objects-split-kernel-and-domain-crates.md) | A command kernel plus one cmd- crate per domain | built | [tidy `crate_edges_point_down`, tidy `crates_sit_in_their_group`, tidy `every_command_states_what_decides_it`] | [archive §34 L8355–8359 "one OBJECTS list"] | [archive §34 L8346–8368 (item 25.1), archive §34 L8377–8385 (25.3–25.4), archive §34 L8396–8425 (25.7), archive §34 L8433–8437 (25.9), archive §34 L8568–8587 (25.13), archive §34 L8955–9127, archive §34 L9284–9290] |
| [0246](0246-cuts-below-the-commands.md) | sysmon, the luma disk side and fixtures are crates of their own | built | [tidy `crate_edges_point_down`, tidy `crates_sit_in_their_group`] | [] | [archive §34 L8438–8451 (item 25.10), archive §34 L8917–8935, archive §34 L8947–8954] |

## demo

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0094](0094-demo-show-test.md) | The demo is the rig; every test runs on it | built | [the_demo_show_builds_loads_and_matches_its_golden, demo::ensure, demo::fresh_state, tests/catalog/testdata/demo.golden.txt] | [archive §23 L2169–2179 "committed demo.luma matches", archive §23 L2195–2203 "builds loads committed", archive §29 L3299–3300 "carries AuraXB file"] | [archive §23 L2169–2179, archive §23 L2195–2203, archive §40 L11589, archive §40 L11641–11663, archive §40 L11745–11836] |

## devices

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0166](0166-device-lock-theme-state.md) | A device is one app install or browser tab | built | [`Hello.device`, `Status.device`, `lumacore-cmd-devices` tests] | [archive §30 L3653–3657 "this station's screens", archive §30 L3658–3666 "every screen of the station"] | [archive §30 L3653–3657, archive §30 L3893–3897, archive §30 L3920–3922, archive §30 L3986–3999, archive §30 L4005–4006] |
| [0167](0167-desklock.md) | DeskLock locks the device it is pressed on | built | [E-LOCK-001, E-LOCK-002, `lumacore_cmd_devices::allowed_while_locked`, `devices::may_unlock`] | [archive §30 L3658–3666 "scope every screen", archive §30 L3660–3661 "F9 cd fixture"] | [archive §30 L3658–3666, archive §30 L3810–3824, archive §30 L3893–3897, archive §30 L3911–3915, archive §30 L3991–3999] |
| [0168](0168-one-surface-login-lock-connect.md) | One surface for connect, login and lock | built | [e2e `kit-surface.spec.ts`, e2e `connect.spec.ts`] | [archive §30 L3674–3681 "lock-screen image on the User", archive §30 L3688–3689 "old home.html", archive §30 L3901–3903 "open event stream", archive §30 L4010–4011 "SSE station events"] | [archive §30 L3667–3681, archive §30 L3688–3689, archive §30 L3810–3824, archive §30 L3901–3910, archive §30 L3949–3955, archive §30 L4005–4017] |
| [0169](0169-keyboard-per-device.md) | Built-in keyboard; the Keyboard mode lives on the device | built | [tidy `keyboard_words_are_registry_words`] | [archive §30 L3916–3919 "Kiosk opens it at once"] | [archive §30 L3566–3571, archive §30 L3916–3919, archive §30 L4096–4102, archive §30 L4183–4190] |

## dialogs

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0186](0186-one-server-held-dialog.md) | Every question the station asks is one server-held dialog | built | [tidy frontend_asks_only_through_the_kit_dialog, E-CMD-008 (template fill), e2e kit-dialogs.spec.ts] | [archive §30 L3595–3598 "One standard dialog", archive §30 L4053–4058 "answer packet UiEvent"] | [archive §30 L3595–3598, archive §30 L3956–3958, archive §30 L4041–4064, archive §30 L4108–4110, archive §30 L4128–4160, archive §30 L4599–4600, archive §30 L5124–5129, archive §30 L5380–5383] |

## dispatch

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0031](0031-typed-replies-text-at-edge.md) | Replies typed inside, rendered to text at the edge | decided | none | [archive §23 L2012–2013 "Command output stays text" (amended)] | [archive §31 L6776–6782] |
| [0123](0123-default-object-per-user.md) | A counted keyword alone sets the user's default object | built | [E-SYN-008, the prompt `Root [Group]>`] | [archive §33 L7910–7923 "context_number shortcut gone"] | [archive §33 L7145–7153, archive §33 L7177–7181, archive §33 L7189–7194, archive §33 L7250–7256, archive §33 L7396–7400, archive §33 L7910–7923] |

## docs

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0252](0252-docs-one-plan-archive.md) | One roadmap, archive never delete, generated reference | built | [catalog test `the_reference_is_current`, `docs/archive/README.md`] | [] | [archive §29 L3086–3103, archive §29 L3158–3163, archive §29 L3180–3189] |
| [0282](0282-docs-checked-in-the-tidy-gate.md) | The docs are checked in the tidy gate | built | [tidy `claude_md_stays_short`, tidy `the_docs_pass_docs_check`] | [] | [docs/plans/active/md-upgrade.md decision 10 and its findings, 2026-10-20] |

## door

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0133](0133-one-object-door.md) | One pool door for every write; next free first unused | built | [E-CMD-020, E-USER-002, `pools::door`, `PoolRules`] | [store-cue-default-object, archive §34 L8017–8020 "after the last", archive §35 L10919–10985 "per-object insert/remove copies", archive §35 L11027–11040 "E-GROUP-002 /Force", archive §35 L11078–11087 "move_cmd gone"] | [archive §35 L10919–10985, archive §35 L11027–11040, archive §35 L11060–11087, archive §34 L8017–8020, archive §35 L10963–10966] |
| [0134](0134-clipboard-of-live-guids.md) | Copy and Cut fill a clipboard of live GUIDs | built | [`Runtime.Users.{User}.clipboard`, `SessionRuntime.clipboards`] | [archive §35 L11070–11077 "client clipboard of the slab"] | [archive §35 L10967–10975, archive §35 L10991–11000, archive §35 L11070–11077] |
| [0135](0135-door-words-insert-compact-patterns.md) | Insert shifts, Compact renumbers, quoted star patterns, no Find | built | [`resolve::name_fits`, `resolve::check_name`, E-CMD-008] | [] | [archive §35 L11001–11017, archive §35 L11041–11047] |
| [0136](0136-tree-shell-mv-cp-rm-through-doors.md) | mv, cp and rm run through each provider's door | built | [`door::shell::write_out`, `Hook::Rewrite`, the tree-shell golden record] | [archive §35 L11048–11059 "Move's alias mv"] | [archive §35 L11018–11026, archive §35 L11048–11059] |
| [0270](0270-renumber-is-move-skip-by.md) | Renumber is Move: /Skip, /By column, a previewing dialog | built | [`<luma-grid>.setPreview`, `kit/value.js`] | [] | [archive §34 L9953–9974, archive §34 L10122–10133, archive §34 L10603–10618] |

## drives

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0085](0085-asset-drives-media-sync.md) | Media in two tiers; asset drives; files travel by hash | built | [E-MEDIA-001…004, media_tests, the link / stream file-exchange tests, e2e media.spec.ts] | [archive §8 L380–389 "paths/references only", archive §8 L391–412 "auto-discovery sync GUI"] | [archive §8 L380–412, archive §11 L618–620, archive §30 L3770–3782, archive §30 L3834–3838, archive §30 L3853–3865, archive §30 L3869–3882] |
| [0086](0086-drives-are-the-os-volumes-probed.md) | Root.Drives is the station's drive list, ids remembered | built | [cargo test -p lumacore-drives, tests/e2e/tests/kit-drives.spec.ts, LUMACORE_DRIVES_PROBE] | [archive §38 L11334–11337 "Root.Drives = /"] | [archive §38 L11334–11355, archive §38 L11429–11451] |
| [0087](0087-drive-object-and-slash-drive-option.md) | The Drive object and /Drive on every file word | built | [lum.md "### Drives", AppState::library_folder, tests/catalog/testdata/lum_words.golden.txt, E-CMD-003 (Eject planned)] | [] | [archive §38 L11356–11384, archive §38 L11429–11451, archive §38 L11472–11490] |
| [0089](0089-drive-service-and-network-drives.md) | A drives watcher; library on first write; own network drives | built | [E-CMD-012, the sysmon "Drives:" lines, tests/e2e/tests/kit-drives.spec.ts] | [] | [archive §38 L11401–11417, archive §38 L11426–11428] |
| [0090](0090-reach-drive-door.md) | Every command declares reach; the disk has one door | built | [tidy core_reaches_the_disk_only_through_drive, registry::line_reach, the sweep's report] | [] | [archive §25 L2788–2798, archive §25 L2836–2841, archive §29 L3122–3130, archive §29 L3272–3276] |

## effects

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0014](0014-pawtern-pool.md) | Pawtern replaces Distribute; slot reserved | decided | none | [archive §3 L214–217 "the Distribute verb"] | [archive §3 L214–241, archive §22 L1739–1740] |
| [0071](0071-effects-model-and-control.md) | Effects are DataPool objects with lines, forms, calls, stomp | decided | [`status: Planned` words, `E-EFX` catalog] | [archive §27 L2562 "phaser steps in Value", archive §28 L2888–2889 "stores no fixture list"] | [archive §28 L2881–2908, archive §28 L2952–2976, archive §28 L2993–3015, archive §28 L3016–3073, archive §32 L6956–6972] |
| [0072](0072-effects-deterministic-anchors.md) | The cue is the effect sync point; handover slides phase | decided | none | [] | [archive §28 L2909–2951] |

## engine

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0026](0026-one-state-thread.md) | One state thread, two lanes, no waiting inside a job | built | [E-INTERNAL-001, E-CMD-017, CallError::Full, E-WIRE-002, Info Health Threads queue / refused, spawn_named, StateHandle::on_busy_change] | [archive §26 L2454–2458 "global Mutex ServerContext", archive §26 L2487–2490 "one queue ten seconds"] | [archive §26 L2452–2465, archive §26 L2480–2500, archive §31 L6753–6773, archive §30 L5370–5379] |
| [0028](0028-session-clock-rate-overlay.md) | Session clock: a rate overlay that never runs backwards | built | [R1-04 Q1 property test, Runtime.Health.Clock] | [archive §24 L2318–2324 "clock over the stream heartbeat"] | [archive §24 L2318–2353, archive §25 L2770–2772] |
| [0029](0029-shutdown-signals-local-server.md) | Clean shutdown on signals; the app never attaches blindly | built | [SHUTDOWN_DEADLINE, watch_signals] | [archive §26 L2511–2513 "inputs state thread outputs"] | [archive §23 L2121–2161, archive §23 L2180–2219, archive §26 L2509–2513, archive §26 L2523–2531] |
| [0030](0030-show-snapshot-arc-parts.md) | A publish copies only what changed | built | none | [archive §26 L2490–2492 "programmer snapshot not built"] | [archive §31 L6726–6737] |
| [0052](0052-one-line-costs-two-fingerprints.md) | A line costs two fingerprints, never a second walk | built | [`lumacore_base::fingerprint`, `FidIndex`, `the_demo_show_builds_loads_and_matches_its_golden`] | [archive §40 L11790–11796 "sysmon's JSON walks"] | [archive §40 L11773–11800] |

## feeds

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0185](0185-client-feeds-one-socket.md) | Everything live rides one feeds socket | built | [tidy frontend_feeds_ride_one_socket, tidy frontend_has_no_event_stream_or_fetch_poll, E-FEED-001…007, the router test (only /ws/feeds and /ws/shell), e2e feeds.spec.ts] | [archive §30 L3901–3904 "open event stream", archive §30 L3591–3594 "existing SSE WS"] | [archive §30 L3901–3904, archive §30 L4340–4353, archive §30 L4409–4448, archive §30 L4738–4752, archive §30 L4918–4955] |

## files

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0088](0088-one-files-element-on-one-feed.md) | One files element on one feed is every file dialog | built | [InputKind::File, Revisions.library, tests/e2e/tests/kit-drives.spec.ts, the Showroom] | [archive §34 L9709–9713 "picker of those files"] | [archive §38 L11385–11400, archive §34 L10789–10812, archive §38 L11452–11490] |
| [0092](0092-gdtf-import-revisions-and-file-index.md) | Import FixtureType from the library; revisions by hash | built | [LinkOut::Upload, GET /api/fixture-types, the Import FixtureType reply] | [archive §34 L9712–9713 "old Import go"] | [archive §34 L9709–9728, archive §34 L10648–10657, archive §34 L10560–10574, archive §34 L10379–10387, archive §34 L10416–10419] |
| [0093](0093-poolxml-reader-import-groups.md) | A small pool XML reader; Import PoolXml Groups | built | [the Import PoolXml reply's counts, tests/catalog/testdata/demo.golden.txt] | [] | [archive §40 L11636–11640, archive §40 L11708–11718] |

## flags

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0113](0113-flags-and-options-declared.md) | Options are declared; an unknown flag is refused | built | [E-CMD-018, E-SYN-006, `core::commands::flags`, `FlagDef::choices`, the sweep, help Options and the reference] | [archive §31 L6805–6808 "flags::has from args", archive §31 L6809–6810 "every /x swallows", archive §33 L7136–7137 "the Rest form", archive §33 L7138–7144 "quoted-line examples"] | [archive §31 L6783–6814, archive §33 L7134–7144 (items 19, 20)] |

## groups

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0011](0011-groups-in-datapool-selection-rule.md) | Groups only in the selected DataPool; the add-or-replace selection rule | built | [E-GROUP-001, E-GROUP-002, E-GROUP-003, groups_tests.rs] | [archive §22 L1733–1735 "FixtureGroup fixture_groups removed", archive §22 L1787–1788 "recall searching every datapool"] | [archive §22 L1733–1800, archive §22 L1815–1822, archive §22 L1863–1882] |

## headers

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0221](0221-file-headers-and-parts.md) | Every .rs file opens with a header and Parts in one order | built | [banners_match_their_tag, headers_stand_in_order, files_have_headers] | [archive §34 L7986–7989 "one font table"] | [archive §34 L7973–7989 (items 1–6), archive §34 L8452–8455 (25.11), archive §34 L8910–8916] |

## ids

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0271](0271-patch-slot-key-and-three-ids.md) | The patch is keyed by a Slot; FID, IDType, CID | built | [`Slot` has no `Display`, `FidIndex`, `PatchData`'s door, the tidy ratchet on `patch.by_id`, E-CMD-006] | [archive §34 L9414–9417 "selectable numbers are MVR's", archive §34 L10388–10401 "No CIDs: tags instead", archive §34 L10682–10686 "several second ids", archive §34 L10368–10373 "the second id dropped"] | [archive §34 L9414–9417, archive §34 L10158–10165, archive §34 L10388–10409, archive §40 L11597–11627, archive §40 L11672–11681] |

## keys

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0170](0170-keys-uikey-gestures.md) | Keyboard to UiKey to command line; core keys; Meow is Shift | built | [E-KEY-005, `keybindings::CORE_KEYS`, `ui_keys::CORE_MEOW`, e2e `kit-12b2.spec.ts`] | [archive §17 L1242–1245 "KeyBinding key→command map", archive §17 L1247–1249 "Meow default Option/Alt", archive §30 L3923–3948 "F12 hides sidebars only", archive §30 L3945–3946 "F1–F5 F9 F10 defaults"] | [archive §17 L1230–1257, archive §29 L3110–3111, archive §30 L3923–3948, archive §30 L3973–3975, archive §30 L4002–4004, archive §30 L4230–4252, archive §30 L4297–4305] |
| [0295](0295-keys-in-shortcut-layers-and-press-cycles.md) | Keys bind per shortcut mode; a press can cycle | built | [E-KEY-006, E-KEY-007, a_mode_binding_wins_over_all_and_an_empty_one_turns_a_key_off, a_letter_binds_in_a_mode_as_a_press_cycle, prg_keys_cycle_their_word_in_place_and_a_run_completes_the_line, every_built_in_keys_lines_are_the_catalogs_words, lines_in_a_row_are_one_value, every_format_from_the_oldest_has_its_step] | [0170 "letters are never bound" — that sentence only: letters bind in PRG and NAV] | [docs/plans/active/2026-10-23-profile-and-key-editor.md, the user in session 2026-10-23 ("looks good, implement" on mockups docs/dev/ui-kit/mockups/2026-10-23-profile-editor/02-keys.html, 03-cycles.html), docs/research/keys/2026-10-23-key-editor.md] |

## lines

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0042](0042-lines-one-type-runs.md) | A protocol is an object, a line is one row with one run | built | [E-OUTPUT-001, E-OUTPUT-003, E-OUTPUT-005, E-OUTPUT-006, E-OUTPUT-007, E-SHOW-007, E-SHOW-008, `LineOps::check_protocols`] | [archive §11 L552–602 "per-universe routes, Output", archive §11 L627–639 "Output n Input n", archive §33 L7154–7160 "At Output 1", archive §34 L8221–8236 "kept, never sends", archive §34 L9128–9178 "34f7b dormant settings"] | [archive §11 L552–602, archive §11 L627–639, archive §11 L736–770, archive §33 L7154–7160, archive §33 L7745–7757, archive §34 L8221–8236, archive §34 L9128–9224, archive §34 L9225–9333] |
| [0043](0043-dmx-input-merge.md) | DMX input merges per channel in the tick | built | [`InputMerger`, the sACN fuzz target] | [archive §27 L2549–2599 "external input on top"] | [archive §11 L640–683, archive §27 L2662–2683, archive §29 L3231–3233, archive §29 L3291–3293] |
| [0044](0044-line-rates-send-pacing.md) | Lines sample the engine's frame at their own Rate | built | [R1-54 Q21, `LineStats.doubled` / `skipped`, `Info Health Lines`] | [archive §24 L2277–2287 "has its own timer"] | [archive §24 L2277–2287, archive §11 L573–574, archive §11 L605–610, archive §27 L2637–2645, archive §31 L6699–6708] |

## links

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0148](0148-links-live-property-links.md) | "Live links: Wire, typed nodes, computed where used" | decided | none | [archive §33 L7527–7533 "Assign makes Link", archive §33 L7607–7614 "Assign reads source"] | [archive §33 L7519–7555, archive §33 L7556–7562, archive §33 L7563–7579, archive §33 L7580–7585, archive §33 L7586–7606, archive §33 L7607–7621, archive §33 L7622–7648, archive §33 L7649–7655] |

## lumasession

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0021](0021-master-executes-results-flow.md) | The master executes as the sender; results flow in order | built | [E-LUMA-001, E-LUMA-002, E-LUMA-003] | [archive §10 L470–481 "programmer instant, corrected", archive §21 L1426–1428 "Old UDP pieces"] | [archive §10 L438–449, archive §10 L470–481, archive §21 L1394–1476, archive §21 L1500–1510] |
| [0022](0022-master-loss-election.md) | Heartbeat, election, IdleMaster only when alone, fencing | built | [every_order_elects_the_same_winner] | [archive §10 L482–502 "IdleMaster after timeout", archive §10 L497–498 "Standalone Connected IdleMaster GlobalMaster"] | [archive §10 L482–513, archive §21 L1477–1492, archive §21 L1511–1606, archive §29 L3108–3109, archive §29 L3259–3260] |
| [0023](0023-merge-on-rejoin.md) | Three-way merge by GUID on rejoin | built | [E-LUMA-005, lumacore_tree::merge] | [archive §10 L514–526 "Merge All Keep Master", archive §21 L1598–1600 "Merge keeps both copies"] | [archive §10 L514–526, archive §21 L1626–1657, archive §21 L1681–1692, archive §21 L1721–1726] |
| [0024](0024-lumasession-membership-services.md) | Invite, Dismiss, Leave and per-NIC services | built | [ServiceAccess] | [archive §21 L1475–1476 "UDP I/O inside"] | [archive §11 L700–721, archive §21 L1429–1433, archive §27 L2690–2718] |
| [0027](0027-session-stream-json-zstd-journal-cap.md) | The stream stays JSON plus zstd; journal capped | built | [lumacore_base::embedded_json, StreamMsg::Encoded] | [archive §21 L1398–1400 "length-prefixed JSON frames" (refined)] | [archive §31 L6738–6752] |
| [0032](0032-same-build-across-a-lumasession.md) | Every station of a LumaSession runs the same build | built | [E-LUMA-006, Hello.version + commit] | [archive §21 L1402–1404 "build commit only warns"] | [archive §34 L9136–9140, archive §34 L9204–9206] |

## menus

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0165](0165-users-profiles-menu.md) | Users & Profiles follows storage: grid, slab, drill-in | built | [`tree_paths.golden.txt`, tidy `frontend_actions_only_in_the_slab`, tidy `frontend_tables_only_in_the_grid`] | [archive §30 L4328–4336 "floating luma-people window", archive §30 L6593 "top-level Encoder bar tab"] | [archive §30 L4328–4336, archive §30 L4396–4405, archive §30 L4841–4850, archive §30 L6609–6664, archive §30 L6668–6671] |
| [0187](0187-menudef-luma-setup.md) | Menus are MenuDefs drawn by the setup frame | built | [GET /api/menu-types, List Menu / Info Menu, e2e kit-12d.spec.ts] | [archive §30 L3591–3594 "over existing SSE"] | [archive §30 L3591–3594, archive §30 L4337–4339, archive §30 L4406–4408, archive §30 L4684–4721, archive §30 L4904–4907, archive §30 L5047–5068, archive §30 L5130–5142] |

## modes

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0016](0016-freeze-lifts-programmer.md) | Freeze lifts the LTP programmer above playbacks | built | none | [archive §7 L338–347 "Freeze Toggle flips", archive §7 L325–328 "masters as a layer"] | [archive §7 L317–347, archive §27 L2620–2621] |
| [0017](0017-mode-objects-verb-first.md) | Modes are objects: On, Off, Toggle verb-first, per user | built | [E-CMD-005] | [archive §16 L1195–1197 "postfix Blind Off", archive §16 L1205–1206 "Black command", archive §16 L1213–1214 "global Highlight Solo overlays"] | [archive §16 L1193–1229, archive §27 L2624–2626, archive §29 L3222–3230, archive §29 L3288–3290] |

## mvr

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0267](0267-import-mvr-reviews-against-the-draft.md) | Import MVR keeps a review on the draft; Apply writes | built | [`PatchDraft.review`, `Action.when`, E-CMD-010, lum.md "### MVR import" (its PENDING shrank), `tests/fixtures/rig.mvr`] | [archive §34 L9452–9460 "File | Xchange switch", archive §34 L9504–9506 "write straight into the draft", archive §34 L10833–10836 "/Missing Recreate | Parent"] | [archive §34 L9452–9472, archive §34 L10775–10918, archive §40 L11690–11699] |
| [0268](0268-export-mvr-and-xchange.md) | Export MVR writes what the dialog shows; Xchange per NIC | decided | none | [archive §34 L9489–9491 "By MVR layer | By Luma layer"] | [archive §34 L9485–9498, archive §34 L9519–9520, archive §40 L11695] |

## naming

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0025](0025-naming-vocabulary.md) | LumaSession, Station, Shell, Client, Screen, CommandLine | built | none | [archive §18 L1259–1260 "Session means mesh", archive §11 L684–691 "JoinSession LeaveSession go", archive §18 L1264–1266 "Runtime.Sessions ViewSession"] | [archive §11 L684–699, archive §18 L1258–1268, archive §23 L1964–1970, archive §29 L3305–3307] |

## other

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0149](0149-notifications-teams.md) | Notify reaches the LumaSession; profile category grid; Team object | decided | none | [] | [archive §33 L7803–7845] |

## patch

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0091](0091-gdtf-native.md) | The .gdtf is the source; the reader is the vendored fork | built | [refresh_revision, Info FixtureType, vendor/README.md, the "// LumaCore:" marks] | [] | [archive §29 L3234–3243, archive §29 L3294–3300, archive §40 L11551–11560, archive §40 L11588] |
| [0261](0261-patch-faithful-to-mvr-and-guids.md) | The patch is faithful to MVR and to GUIDs | built | [the exhaustive `Walk` destructure, E-SHOW-003, `tests/catalog/testdata/tree_paths.golden.txt`] | [archive §34 L9499–9508 "separate Luma layer tree"] | [archive §34 L9395–9434, archive §34 L9522–9541] |
| [0262](0262-staged-patch-draft.md) | Claiming Devices edits a draft; live columns stay live | built | [E-PATCH-008, E-CMD-010, `Props::live`, `commands::live_draft::runs_live`, `patch_draft::rebase`, `claims::HolderWatch`, `tests/e2e/tests/kit-34g4.spec.ts`] | [archive §11 L621–626 "Unclaim Patch still clears", archive §11 L730–735 "claiming Patch takes a draft"] | [archive §11 L621–626, archive §11 L727–770, archive §24 L2684–2690, archive §34 L9435–9441, archive §34 L9542–9555, archive §34 L9668–9672, archive §34 L9683–9688, archive §34 L10402–10404, archive §34 L10420–10427, archive §34 L10575–10587, archive §34 L10705–10712] |
| [0263](0263-universe-move-delete-ask.md) | Universe moves and deletes ask; fixtures lose the address | built | [`objects/patch/universe_ask.rs`] | [] | [archive §11 L757–766, archive §34 L9556–9563, archive §34 L9746–9747] |
| [0265](0265-mvr-tree-words-and-one-nesting-rule.md) | MVR tree words; Assign puts in, Move At a node nests | built | [E-PATCH-011, `commands::door::nest_ask`, lum.md's MVR examples, `tests/e2e/tests/kit-34g4.spec.ts`, the 40d e2e on the rig] | [archive §34 L9483–9484 "Delete deletes the node's fixtures", archive §34 L9634–9636 "then unpatches them", archive §34 L9597–9599 "read only: they come by import"] | [archive §34 L9473–9484, archive §34 L9595–9649, archive §34 L9995–10003, archive §34 L10153–10157, archive §34 L10658–10671, archive §40 L11700–11744] |
| [0266](0266-mvr-metadata-pools-by-uuid.md) | Devices.Metadata.MVR holds the MVR pools by UUID | built | none | [] | [archive §34 L9426–9434, archive §34 L10004–10008, archive §34 L10020–10022] |
| [0269](0269-patch-words-store-assign-at-dmx.md) | Store creates, Assign addresses, Unassign removes; DMX is slots | built | [E-PATCH-001, E-CMD-006, E-CMD-008, E-CMD-010, lum.md's patch examples] | [archive §34 L9705–9718 "Patch Fixture <sel> At", archive §34 L10189–10193 "DMX In 1"] | [archive §34 L10071–10120, archive §34 L10189–10193, archive §34 L10648–10657, archive §34 L10672–10689, archive §34 L10690–10704] |
| [0272](0272-nested-fixtures-folders.md) | A fixture may hold fixtures; a folder keeps an order | built | [`next_order`, `nodes::fixtures_under`, E-PATCH-011, `fixture::delete_ask`, the 40d e2e on the rig] | [archive §28 L2982 "room for Fixture 3.4"] | [archive §40 L11590–11596, archive §40 L11603–11607, archive §40 L11627–11635, archive §40 L11682–11744] |
| [0273](0273-multipatch-inside-the-fixture.md) | Multipatch copies live inside their fixture, owner first | built | none | [] | [archive §34 L10166–10172, archive §34 L10351–10357, archive §34 L10410–10415, archive §34 L10658–10671] |
| [0275](0275-type-revision-mode-change-native-values.md) | Type, revision and mode changes keep native values | decided | none | [] | [archive §34 L9725–9732, archive §34 L10040–10049] |
| [0276](0276-map-and-clone-attribute-maps.md) | Map & clone: AttributeMap pool, AttributeRule, Clone verb | decided | none | [archive §34 L9813–9818 "scoped to all or to a pair"] | [archive §34 L9733–9745, archive §34 L9803–9836, archive §34 L10033–10070, archive §34 L10173–10184] |
| [0277](0277-cells-on-fixture-type.md) | No sub-fixtures: cells, then parts and the pixelmapper | decided | none | [archive §28 L2977–2989 "cell layout is defined per mode"] | [archive §28 L2977–2989, archive §28 L3294–3296, archive §34 L9837–9908, archive §34 L10185–10188] |
| [0279](0279-identify-probe-find.md) | Identify is one object above everything but Park | built | [`EngineInput::identify`, `identify::IdentifyTier`, `/As`, lum.md "Identify", `identify::tick`] | [] | [archive §34 L9914–9916, archive §34 L10465–10515] |
| [0281](0281-fixture-luma-extras-live-columns.md) | Fixture.Luma extras and the other live columns | decided | none | [] | [archive §34 L10010–10019] |

## playback

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0013](0013-scenes-stateful-logic.md) | Scenes are state configurations, not timelines | decided | none | [] | [archive §6 L293–315] |
| [0015](0015-priority-pool.md) | A user-defined Priority pool replaces fixed bands | decided | none | [] | [archive §7 L349–363, archive §27 L2586–2589, archive §11 L648–652] |
| [0061](0061-playback-state-saved.md) | Playback state lives in ShowData.Playbacks, never undone | built | [`ShowSubtree::Playbacks`, test "no master move files an Oops step"] | [archive §13 L1108–1115 "Playbacks.Sequences, Priority pool", archive §27 L2613–2614 "ShowData.Playbacks.Sequences resumed", archive §32 L6913–6915 "global PlaybackData Playback.json"] | [archive §13 L1100–1115, archive §27 L2613–2614, archive §32 L6911–6928, archive §30 L6553–6562] |
| [0062](0062-sequence-cuezero-offcue.md) | Every sequence has CueZero and OffCue; Top starts echoes | built | [E-CMD-010, `CommandDef::hold`, "load refuses a sequence without them"] | [archive §30 L6234–6235 "Top sends Goto"] | [archive §32 L6930–6955, archive §30 L6259–6278, archive §30 L6529–6552, archive §33 L7105–7110] |
| [0063](0063-group-masters.md) | Masters live in Playbacks.Live.Masters, never undone | built | [test "no master move files an Oops step", E-CMD-003] | [archive §22 L1801–1804 "ShowData.Playbacks.group_masters GUID", archive §27 L2616 "Master 1 grand", archive §30 L5731 "ShowData.Playbacks.Masters"] | [archive §22 L1801–1812, archive §22 L1883–1887, archive §30 L5726–5738, archive §30 L6249–6258, archive §30 L6553–6562] |
| [0064](0064-tracking-cues-parts.md) | Tracking cues with parts; timing per part, group, value | built | [E-PLAY-003, E-CMD-006, E-CMD-009] | [archive §29 L3221 "No per-value timing"] | [archive §27 L2567–2582, archive §27 L2610–2616, archive §29 L3216–3221, archive §29 L3285–3287, archive §30 L5918–5919, archive §30 L6393–6400, archive §30 L6501–6518] |
| [0065](0065-store-cue-default-object.md) | Store Cue between CueZero and OffCue; Cue is Store's default object | built | [E-PLAY-004, E-CMD-008, E-CMD-009, E-CMD-010] | [archive §30 L6280–6281 "Store Sequence did nothing"] | [archive §30 L6279–6342, archive §30 L6362–6373] |
| [0066](0066-sequences-in-datapools.md) | Sequences, Timing and Curve presets live in DataPools | built | [`FORMAT_VERSION` bump] | [archive §32 L6913–6915 "global PlaybackData Playback.json"] | [archive §30 L5828–5832, archive §30 L6246–6248, archive §30 L6265–6267, archive §30 L6413–6426] |
| [0068](0068-gesture-channel.md) | Continuous controls ride the gesture channel, one Oops step | built | [test "one entry per gesture", e2e `kit-12f.spec.ts`, `CLIENT_PROTOCOL` / `SESSION_PROTOCOL` bumps] | [archive §30 L5739 "a /Quiet option"] | [archive §30 L5739–5746, archive §30 L6453–6458, archive §30 L6501–6512, archive §30 L6563–6572] |

## pointer

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0194](0194-pointer-core-one-lift-gesture.md) | Every drag is the pointer core's one lift gesture | built | [e2e kit-4d4.spec.ts, tidy frontend_gestures_only_in_the_kit] | [] | [archive §34 L9917–9931, archive §34 L10194–10275, archive §40 L11719–11728] |

## pools

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0205](0205-nodemeta-colour-icon.md) | Colour and icon live only in NodeMeta | built | [a_group_takes_a_colour_and_an_icon] | [] | [archive §30 L5490–5496, archive §30 L5619–5625] |
| [0206](0206-grid-tools-registry.md) | Grid tools are one registry; the list button marks a menu | built | [grid_tools_change_the_group_layout_one_step_each, E-CMD-003] | [archive §30 L5476 "the tool row"] | [archive §30 L5442–5447, archive §30 L5501–5507, archive §30 L5519–5524, archive §30 L5528–5530, archive §30 L5553–5567, archive §30 L5572–5576, archive §30 L5653–5683] |

## programmer

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0002](0002-programmer-owned-by-user.md) | Programmer and selection belong to the User | built | [ShowSubtree::Programmer, ShowSubtree::Selection, ShowData::programmer_of] | [archive §2 L171–176 "Arc-wrapped RwLock Programmer", archive §10 L459–461 "programmer on UserProfile"] | [archive §2 L171–192, archive §10 L450–469, archive §21 L1360–1362, archive §21 L1372–1380] |
| [0003](0003-programmers-ltp-fixture-claims.md) | Programmers are LTP; selecting claims nothing | built | none | [archive §4 L268–269 "derived fixture-selection claim"] | [archive §4 L261–279, archive §27 L2578–2581, archive §27 L2590–2593, archive §27 L2620–2623, archive §27 L2633–2636] |

## properties

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0141](0141-property-kinds-values-reset.md) | The property's kind parses the value once; Reset restores defaults | built | [E-SYN-014, E-CMD-008, E-SET-003, Property::default] | [archive §33 L7098–7104 "handler-local bool lists"] | [archive §33 L7098–7104, archive §33 L7488–7496, archive §33 L7510–7518] |
| [0143](0143-one-property-registry.md) | Every object declares its properties once on ObjectDef::props | built | [E-SET-001, E-SET-002, the_shim_count_only_goes_down] | [archive §33 L7451–7458 "three property systems", archive §30 L4719–4721 "TreeDef::write kinds rights"] | [archive §33 L7451–7472, archive §33 L7864–7909] |
| [0144](0144-color-light-paint-screen.md) | Color is light (CIE xyY); Paint is screen (sRGB hex) | built | [NodeMeta "Paint"] | [] | [archive §33 L7497–7504, archive §33 L7846–7863] |
| [0145](0145-theme-lockscreen-paint-objects.md) | Theme, Paint, LockScreen are one-of-a-kind profile children | built | [Verb::EditSetting, E-SYN-007] | [archive §33 L7473–7478 "dotted Theme.AccentHue property"] | [archive §33 L7479–7487, archive §33 L7505–7509, archive §33 L7846–7863] |

## protocols

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0041](0041-protocol-runtime-lifecycle.md) | Protocol runtimes start from show lines; only the master sends | built | [`protocol_plan::desired`, `ProtocolRuntimes::reconcile`] | [archive §27 L2645–2660 "the dmx_output_enabled hack"] | [archive §11 L541–551, archive §11 L603–604, archive §11 L662–665, archive §27 L2645–2660] |
| [0045](0045-protocol-crates-declare-themselves.md) | Each protocol is a crate adding its ProtocolDef row | built | [`protocol::find`, the catalog freeze check, the E-DMX codes] | [archive §34 L8091–8099 "lines go up"] | [archive §34 L8185–8252, archive §34 L8768–8785] |
| [0047](0047-osc-line-user-required.md) | Every OSC line names its user explicitly | built | [E-RIGHTS-003, `RowDelete::affects`, `List Reference /Broken`] | [archive §11 L549–550 "names a user"] | [archive §31 L6884–6897] |

## registry

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0111](0111-one-registry-one-namespace.md) | One registry, one namespace, one name per word | built | [tidy keyword rule (a match naming two registered words outside the registry), test `planned_words_reply_planned_and_change_nothing`, E-CMD-003, E-SYN-008, E-SYN-010, golden `tests/catalog/testdata/lum_words.golden.txt`, tidy `every_command_states_what_decides_it`] | [archive §14 L1128–1131 "Abbreviation System", archive §15 L1159–1166 "short forms aliases prefix", archive §15 L1151–1152 "not yet implemented", archive §33 L7300–7306 "Patch DMX verbs", archive §33 L7793 "Create alias Store"] | [archive §33 L7057–7076 (item 12), archive §33 L7300–7306 (item 48), archive §33 L7340–7342 (item 52), archive §15 L1143–1170, archive §21 L3244–3247, archive §21 L3266–3271, archive §31 L6838–6842] |

## render

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0049](0049-render-tick-rules.md) | The render tick walks arrays only | built | [`assert_no_alloc`, the tidy render regions, the budget test, `cargo bench -p lumacore-render --bench render`, `Info Health Threads` `scheduling`] | [archive §24 L2244–2250 "the first tick rate", archive §24 L2354–2357 "LumaRuntime, HealthReport"] | [archive §24 L2244–2250, archive §24 L2274–2276, archive §24 L2288–2317, archive §24 L2354–2381, archive §24 L2382–2399, archive §31 L6709–6713, archive §31 L6714–6723] |
| [0050](0050-playback-merge-order.md) | One merge, one Value type, DMX bytes only in the tick | built | [`merge::resolve`] | [archive §27 L2549–2560 "the three renderers", archive §27 L2558–2562 "phaser steps in Value"] | [archive §27 L2549–2599, archive §27 L2601–2635, archive §28 L2901–2904, archive §11 L614–617] |

## resolve

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0124](0124-one-resolver.md) | Every target becomes objects only through core::resolve | built | [tidy `old_resolvers_only_go_down`, E-CMD-007, E-CMD-008, E-CMD-020] | [archive §33 L7421–7427 "five resolver families"] | [archive §33 L7421–7450, archive §33 L7745–7777] |

## rights

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0161](0161-rights-one-gate.md) | One rights gate, a six-level ladder, Admin's fixed password | built | [E-RIGHTS-001, E-RIGHTS-002, E-RIGHTS-003, E-USER-001, E-USER-004, E-USER-005, `passwords::mask_secrets`, `rights_tests`, the sweep's one user per level] | [archive §19 L1270–1278 "token on every request", archive §19 L1288–1290 "Guest Programmer Admin", archive §19 L1305–1306 "OSC runs as Admin"] | [archive §19 L1270–1318, archive §23 L2220–2227, archive §29 L3105–3107, archive §29 L3259, archive §30 L3815–3817, archive §30 L3898–3900, archive §30 L4000–4001, archive §30 L4597–4598, archive §30 L4753–4768, archive §30 L4956–4998, archive §30 L5388–5390] |

## set

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0142](0142-set-one-typed-handler.md) | Set names its target; one typed handler on the registry | built | [E-CMD-006, E-CMD-015, E-SET-001, E-SET-002, E-SYN-007, the_shim_count_only_goes_down] | [archive §30 L5059–5061 "set_rows one Set"] | [archive §33 L7111–7115, archive §33 L7791–7802, archive §33 L7889–7909] |

## settings

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0146](0146-settings-typed-keyed-props.md) | Station, Interface, Network and Show settings are typed keyed properties | built | [kit-12d.spec.ts] | [] | [archive §30 L4715–4721, archive §30 L4899–4903, archive §30 L5047–5062, archive §30 L5150–5153] |
| [0147](0147-settings-home-icons.md) | Three homes, one icon per cell, gold where it differs | built | [kit-12d.spec.ts data-home] | [archive §30 L4715–4718 "every cell says"] | [archive §39 L11491–11533] |

## showfile

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0081](0081-luma-show-file-container.md) | A .luma is a zip of the ShowData tree | built | [show_file::FILES, show_file::FOLDERS, E-SHOW-005, E-SHOW-002, the show_file tests (round trip, pictures, damaged part, old formats, folder names)] | [archive §21 L1676–1677 "plain JSON file", archive §29 L3234–3243 "zip show.json gdtf"] | [archive §8 L365–379, archive §21 L1676–1677, archive §29 L3234–3243, archive §29 L3294–3300, archive §30 L3762–3769, archive §30 L3783–3788, archive §30 L3827–3833, archive §30 L3850–3852] |
| [0082](0082-show-format-forward-only.md) | Show files only move forward, no migrations | superseded | [E-SHOW-001, E-SHOW-002, E-SHOW-004, the_demo_show_builds_loads_and_matches_its_golden] | [archive §20 L1321–1322 "runs migration steps"] | [archive §20 L1319–1330, archive §21 L1359–1366, archive §21 L1385–1391, archive §21 L1623–1625, archive §23 L2162–2168, archive §23 L2190–2194] |
| [0289](0289-show-files-upgrade-step-by-step.md) | Older show files upgrade step by step | built | [every_format_from_the_oldest_has_its_step, an_older_show_upgrades_step_by_step_to_today, E-SHOW-001, E-SHOW-002] | [show-format-forward-only] | [session 2026-10-23, "it must be possible to load older showfiles … do the recommended"] |

## station

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0033](0033-station-table.md) | Rig wiring in the show's station table, identity on the station | built | [stations_tests, AppState::follow_renames (R1-69 Q41)] | [archive §10 L459–462 "station-local interfaces subtree", archive §11 L583–585 "saved in station config", archive §30 L4859 "edited remotely through master"] | [archive §11 L581–586, archive §11 L618–620, archive §30 L4712–4714, archive §30 L4856–4863, archive §30 L5098–5123] |
| [0034](0034-remote-station-edits.md) | Another station's identity edit goes to its command port | built | [E-CMD-010, E-RIGHTS-001, remote_edit_tests, kit-12d "pending until it reports it"] | [archive §30 L4859 "edited remotely through the master"] | [archive §30 L5161–5167, archive §30 L5180–5236] |
| [0035](0035-app-thin-client.md) | Apps are basic clients that open the station's page | built | [SESSION_PROTOCOL bump for leaving, e2e app-mode] | [archive §30 L4224–4226 "entry alive while mDNS lives"] | [archive §30 L3752–3761, archive §30 L3962–3972, archive §30 L4018–4022, archive §30 L4224–4229, archive §30 L4291–4296] |
| [0036](0036-clients-app-navigation-cli-disconnect.md) | App webview locked to the station; CLI ends with the stream | decided | none | [] | [archive §31 L6871–6879] |
| [0230](0230-station-only-assembles.md) | The station only assembles: the catalog and thin binaries | built | [the_station_only_assembles, crates_sit_in_their_group] | [archive §34 L8064–8079 "station splits into", archive §34 L8080–8083 "Binaries live with", archive §34 L8084–8087 "station moves to"] | [archive §34 L8064–8090 (items 19, 20.1, 20.2), archive §34 L8303–8314 (24.1), archive §34 L8332–8345 (24.3, 24.4), archive §34 L8798–8802, archive §34 L8866–8877, archive §34 L8892–8909] |
| [0247](0247-rig-wiring-in-cmd-station.md) | Station, Interface, Network in cmd-station; DMX lines on it | built | [`objects::object_of(Node)`, tidy `crate_edges_point_down`] | [archive §34 L8411–8414 "cmd-protocols holding Network"] | [archive §34 L8542–8567 (item 25.17), archive §34 L9084–9127, archive §34 L9284–9290] |

## structure

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0037](0037-wire-crate-clients.md) | Clients compile only the wire and the WebUI | built | [E-WIRE-001, the_webui_is_only_files, os_code_lives_in_platform, default-members] | [archive §23 L1973 "lumacore-proto", archive §23 L2003–2006 "lumacore-app --server", archive §23 L1998–2001 "clients only wire" (amended), archive §36 L11113 "web/dist default"] | [archive §23 L1971–2006, archive §23 L2052–2070, archive §36 L11105–11129, archive §36 L11161–11185] |
| [0222](0222-one-folder-per-object-one-file-per-verb.md) | One folder per object, one file per verb | built | none | [] | [archive §34 L7999–8002 (item 8)] |
| [0223](0223-file-size-limits.md) | Files stay short; the hard limit needs a listed reason | built | [files_stay_short] | [] | [archive §34 L8003–8005 (item 9)] |
| [0224](0224-no-mod-rs.md) | Modules are foo.rs plus foo/, never mod.rs | built | [modules_are_foo_rs_not_mod_rs] | [] | [archive §34 L8006–8007 (item 10)] |
| [0225](0225-tests-in-foo-tests-rs.md) | A module's tests live in foo/tests.rs; three homes for a test | built | [tests_live_in_foo_tests_rs] | [archive §34 L8008–8010 "tests/it/ binary"] | [archive §34 L8008–8010 (item 11), archive §34 L8516–8541 (25.16)] |
| [0226](0226-shared-code-one-door.md) | Shared code has one door each; copies are ratchets | decided | [shared_doors_have_no_copies, old_resolvers_only_go_down] | [] | [archive §34 L8011–8020 (items 12–13)] |
| [0237](0237-code-structure-later-items.md) | Crate split after measuring; AppState grouped gradually | superseded | none | [] | [archive §31 L6843–6851] |

## suggest

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0173](0173-one-suggestion-engine.md) | One suggestion engine; Enter runs the line as typed | built | [`the_terminal_and_the_bar_show_the_same_suggestions`, e2e `suggest-parity.spec.ts`, e2e `kit-suggest.spec.ts`, `ShowSubtree::Usage`] | [archive §30 L4254 "completer core in terminal/"] | [archive §30 L4239–4282, archive §30 L4354–4368] |

## syntax

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0048](0048-declared-words-written-as-declared.md) | Declared values and words run in their declared form | decided | [E-CMD-008, E-SYN-014] | [archive §34 L9145–9160 "lowercase protocol names"] | [archive §34 L9145–9170, archive §34 L9218–9222] |
| [0101](0101-lum-grammar-one-parser.md) | lum.md is the binding spec, one total parser | built | [the grammar-listing test against lum.md's `grammar` block, every ```lum / ```lum-error example a test (tests/catalog lum/tests.rs, `PENDING`), the E-SYN catalog, fuzz target `syntax_parse`, tidy `the_shim_count_only_goes_down`] | [archive §14 L1128–1131 "Abbreviation System", archive §15 L1159–1166 "prefix tie rules", archive §30 L4268 "Enter runs typed", archive §33 L7952 "issue 02 Please"] | [archive §33 L6976–7007 (items 1, 2, 3, 5), archive §33 L7226–7239 (items 33, 34), archive §33 L7343–7346, archive §33 L7943–7953] |
| [0102](0102-quoting-values-options.md) | A quoted word is a value, a bare /word an option | built | [E-CMD-006, E-CMD-008, E-SYN-005, E-SYN-014, `tokens::unquote`, `drive::library_file`, `Value::Line` / `Kind::Line`] | [archive §15 L1180–1181 "one word unquoted", archive §33 L7138–7144 "quoted-line examples", archive §33 L7200–7204 "Watch quoted line"] | [archive §15 L1171–1191, archive §33 L7013–7020 (item 7), archive §33 L7698–7708 (item 91)] |
| [0104](0104-ids-operators-case-lexing.md) | Ids are numbers or quoted text; glued signs; case | built | [E-SYN-007, E-SYN-012, E-SYN-001, the `NUMBER` / `WORD` tokens of lum.md's grammar block] | [archive §33 L7337–7339 "goback go- names", archive §33 L7786–7787 ".5 numbers", archive §33 L7788–7790 "bare Master Highlight"] | [archive §33 L7039–7056 (items 10, 11), archive §33 L7171–7176 (item 24), archive §33 L7337–7339 (item 51), archive §33 L7475–7478 (item 59)] |
| [0105](0105-dotted-ids-per-object.md) | Each object declares how it reads a dotted id | built | [catalog test `an_object_that_nests_itself_reads_a_path`, `IdReading` on `LumShape`, the reference's `ids decimal` / `ids 2 parts` lines, the golden record] | [archive §12 L866–868 "path by shape"] | [archive §12 L862–874, archive §12 L910–918] |
| [0107](0107-tree-shell-unix-fence.md) | The tree shell is a fenced Unix sub-grammar | built | [E-SYN-002, E-SYN-003, E-SYN-009, `commands::typed::FenceCommand`, `CommandDef::fence`, lum.md's `fence` rule] | [archive §33 L7128–7129 "root rm whoami", archive §33 L7945–7946 "ls /r"] | [archive §33 L7124–7133 (item 18), archive §33 L7205–7209 (item 30), archive §33 L7377–7387 (3a)] |
| [0108](0108-user-aliases-bang.md) | Personal aliases on the profile, typed with ! | decided | none | [archive §15 L1159–1161 "short forms registry aliases"] | [archive §33 L7077–7097 (item 13), archive §33 L7195–7199 (item 28)] |
| [0109](0109-if-filter-three-shapes.md) | If and EndIf in three shapes, no comparisons; planned | decided | [E-CMD-003, test `planned_words_reply_planned_and_change_nothing`] | [archive §33 L7169–7170 "half-built comparison filter"] | [archive §33 L7161–7170 (item 23), archive §21 L3266–3271] |
| [0110](0110-self-reads-in-values.md) | Self reads fill a value slot; Not in conditions | decided | none | [archive §33 L7222–7223 "!Self.X conditions"] | [archive §33 L7216–7225 (item 32)] |

## tags

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0207](0207-tags-pool-and-links.md) | Tags are a tree pool with GUID links; verbs reach members | built | [E-TAG-001…007, an_object_that_nests_itself_reads_a_path] | [archive §12 L808–829 "deep, Assign nests"] | [archive §12 L783–890, archive §12 L891–958] |

## tests

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0131](0131-command-sweep-dynamic.md) | The command sweep is generated from the registries | built | [`lumacore-sweep`, the `run.json` resume, the sweep report's `unreachable` list] | [archive §25 L2842–2846 "sweep in the station"] | [archive §25 L2781–2833, archive §25 L2842–2846, archive §29 L3122–3130, archive §29 L3272–3276] |
| [0132](0132-fuzz-long-run-only.md) | Fuzzing is a long run; property tests are fast tests | built | [`scripts/unix/fuzz_long`, the proptest suite in `lumacore-sweep`] | [archive §25 L2847–2854 "fuzz/ over lumacore_station::fuzzing"] | [archive §25 L2731–2762, archive §25 L2847–2854] |
| [0248](0248-catalog-tests-crate.md) | The whole catalog's tests are the crate tests/catalog | built | [`cargo test -p lumacore-catalog-tests`: `the_reference_is_current`, `settled_clashes`, `the_demo_show_builds_loads_and_matches_its_golden`] | [] | [archive §34 L8504–8516 (item 25.15), archive §34 L8997–9016] |
| [0249](0249-three-homes-for-a-test.md) | A test's home: catalog, stand-in words, or beside its code | built | [the stand-in words in `commands/test_words.rs` (TestAsk, TestEcho, TestInternal)] | [archive §34 L8010 "tests/it/ binary per crate"] | [archive §34 L8517–8541 (item 25.16), archive §34 L9066–9083] |

## tidy

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0233](0233-tidy-test-only-crate-with-rulebook.md) | Tidy is a test-only crate with one rulebook | built | [the_rulebook_matches_the_rules] | [] | [archive §34 L8108–8112 (20.8), archive §34 L8621–8629] |

## tree

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0001](0001-object-tree-shape.md) | The object tree: ShowData, Runtime, station-local subtrees | superseded | none | [] | [archive §1 L63–169] |
| [0005](0005-state-homes-by-type.md) | Every field has one home: show, runtime, station-local | built | none | [object-tree-shape] | [archive §10 L450–469, archive §21 L1411–1418] |
| [0006](0006-node-meta-levels.md) | Identity and Annotation: meta in levels on every node | built | [every_object_with_a_node_meta_has_its_level, every_resource_has_a_derived_guid] | [archive §12 L1038–1040 "per-object META lists", archive §12 L1052–1054 "guid_identified identified macros", archive §12 L1033–1035 "NodeMeta.id number"] | [archive §12 L1000–1098, archive §21 L1613–1618] |
| [0007](0007-guids-everywhere-links.md) | Every object a GUID, every stored link a GUID | built | [E-SHOW-003, ShowData::identity(), List Reference /Broken] | [archive §21 L1619–1623 "numbers names as links", archive §21 L1690–1691 "tombstones of deleted"] | [archive §21 L1493–1496, archive §21 L1613–1628, archive §21 L1650–1653, archive §21 L1679–1711, archive §29 L3101–3103] |

## ui

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0174](0174-screenshots-client-capture.md) | Screenshots are captured in the client, asked by the server | built | [e2e `screenshot.spec.ts`, e2e `screenshots-all.spec.ts`, E-CMD-010] | [archive §29 L3407–3412 "/api/events viewer stream", archive §29 L3442–3446 "vendor under apps/lumacore-app/frontend", archive §29 L3452–3454 "F11 cd output"] | [archive §29 L3164–3179, archive §29 L3407–3479] |
| [0204](0204-pool-window-unified.md) | One pool window from PoolDef; hold opens the object sheet | built | [pools::tests, kit-pool.spec.ts] | [archive §22 L1844–1853 "full-screen selector", archive §30 L3550–3556 "PoolHold board A", archive §30 L5430–5437 "candidates B C"] | [archive §22 L1824–1862, archive §22 L1888–1900, archive §30 L3550–3556, archive §30 L3695, archive §30 L5421–5437, archive §30 L5463–5468, archive §30 L5512–5513, archive §30 L5626–5650] |
| [0208](0208-tags-ui.md) | Tags in EditSetting, the pool window and the sidebar | built | [E-TAG-003] | [] | [archive §12 L838–861, archive §12 L959–999] |

## ui-kit

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0069](0069-fader.md) | The fader drags relative only, no tap jump, no hold | built | [e2e `kit-12f.spec.ts`] | [archive §30 L3559–3561 "double-tap or hold keypad", archive §30 L5747–5748 "hold = master's sheet"] | [archive §30 L3557–3561, archive §30 L5747–5748, archive §30 L5766, archive §30 L5783–5784, archive §30 L5798–5799, archive §30 L6519–6528] |
| [0070](0070-encoder-bar.md) | The encoder bar: concept A, Mode above Layer, per screen | built | [e2e `kit-12f.spec.ts`, `Revisions.modes`] | [archive §30 L3560–3561 "encoders are drag", archive §30 L5767–5769 "hold spawns slider", archive §30 L5886–5888 "horizontal indicators option"] | [archive §30 L3557–3561, archive §30 L5767–5769, archive §30 L5800–5801, archive §30 L5810–5812, archive §30 L5833–5835, archive §30 L5859–5866, archive §30 L5877–5917, archive §30 L5954–5969, archive §30 L6195–6207, archive §30 L6443–6500] |
| [0181](0181-ui-kit-only.md) | Every UI surface is built only from kit elements | built | [tidy frontend_colours_only_in_the_theme, tidy frontend_icons_come_from_the_kit, tidy frontend_font_sizes_are_tokens, tidy frontend_gestures_only_in_the_kit, e2e kit.spec.ts] | [] | [archive §30 L3521–3526, archive §30 L3609–3612, archive §30 L3624–3632, archive §30 L3636–3642, archive §30 L3696–3700, archive §30 L3701–3717] |
| [0182](0182-descriptors-from-registries.md) | Typed descriptors from the registries; every gesture a command line | built | [CLIENT_PROTOCOL bump on every wire change, e2e kit-12b2.spec.ts, e2e kit-12c.spec.ts] | [] | [archive §30 L3527–3533, archive §30 L3564–3565, archive §30 L4085–4095, archive §30 L4165–4169, archive §30 L4312–4327, archive §30 L4385–4395, archive §30 L4514–4515, archive §30 L4553, archive §30 L5452–5455, archive §30 L5606–5611] |
| [0183](0183-look-tokens-fonts-icons.md) | The look: palette as tokens, Lucide icons, vendored fonts | built | [tidy frontend_colours_only_in_the_theme, tidy frontend_icons_come_from_the_kit] | [] | [archive §30 L3534–3549, archive §30 L3643–3646, archive §30 L3682–3687, archive §30 L3701–3717, archive §30 L3727–3732] |
| [0184](0184-text-selection-rule.md) | Text is selectable only where you read or type | built | [tidy text_selection_only_in_the_kit_rule] | [] | [archive §30 L4308–4311, archive §30 L4381–4384] |
| [0188](0188-luma-grid.md) | Every list of objects is the kit's one grid | built | [tidy frontend_tables_only_in_the_grid, e2e kit-12d.spec.ts] | [] | [archive §30 L4769–4822, archive §30 L4835–4840, archive §30 L5003–5046] |
| [0189](0189-slab.md) | The slab is the kit's one action block | built | [tidy frontend_actions_only_in_the_slab, e2e kit-12d.spec.ts] | [] | [archive §30 L4823–4834, archive §30 L4841–4850, archive §30 L5003–5046] |
| [0190](0190-multiplatform-contracts.md) | The kit's multiplatform contracts | built | [tidy frontend_font_sizes_are_tokens, tidy font_and_grid_scanners, e2e multiplatform.spec.ts, e2e kit-12c2.spec.ts] | [] | [archive §30 L4601–4616, archive §30 L4664–4672] |
| [0191](0191-phone-desk-miniature.md) | Same view everywhere: miniature panes, hold full screen, the phone rule | built | [e2e kit-12c2.spec.ts, e2e kit-12e.spec.ts, e2e phone.spec.ts, e2e audit-phone.spec.ts] | [archive §30 L4631 "hold 0.4 s", archive §30 L5448–5451 "at 600 px"] | [archive §30 L4617–4639, archive §30 L4659–4663, archive §30 L5448–5451, archive §30 L5472–5475, archive §30 L5494–5500, archive §30 L5516–5518, archive §30 L5531–5549, archive §30 L5552, archive §30 L5690–5713, archive §37 L11186–11319] |
| [0192](0192-narrow-command-line.md) | The prompt shrinks on a narrow bar, fixed on the station | built | [repl test the_prompt_shrinks_on_a_narrow_line] | [] | [archive §30 L5406–5411, archive §30 L5588–5592] |
| [0193](0193-one-hold-one-owner.md) | One hold, one owner, the profile's HoldTime | built | [tidy frontend_gestures_only_in_the_kit, e2e kit.spec.ts (split), e2e kit-12e.spec.ts] | [archive §30 L5438–5441 "hairline hit band", archive §30 L3553–3557 "hold show me"] | [archive §30 L3550–3563, archive §30 L3647–3648, archive §30 L5412–5420, archive §30 L5438–5441, archive §30 L5469–5471, archive §30 L5514–5515, archive §30 L5593–5605, archive §30 L5684–5686] |
| [0283](0283-device-grid-and-value-marks.md) | The device grid and one set of value marks | built | [lumacore-views devices::tests, tests/e2e/tests/kit-devices.spec.ts] | [] | [session 2026-10-22, device sheet round 1 — docs/dev/ui-kit/mockups/2026-10-22-device-sheet/02-device-grid.html, approved] |
| [0284](0284-device-sheet-window.md) | The Device Sheet window | built | [lumacore-views devices::tests, tests/e2e/tests/kit-devices.spec.ts] | [] | [session 2026-10-22, device sheet round 1 — docs/dev/ui-kit/mockups/2026-10-22-device-sheet/03-device-sheet.html, approved] |
| [0285](0285-top-bar-status-strip.md) | The top bar is a status strip with the modes | built | [tests/e2e/tests/kit-topbar.spec.ts] | [] | [session 2026-10-22, top bar round 1 — docs/dev/ui-kit/mockups/2026-10-22-top-bar (01-top-bar, 02-status-mark), approved] |
| [0286](0286-theme-editor-background-adjusters.md) | The theme editor; background hue and saturation replace Tint | built | [lumacore-state look_tests, tests/e2e/tests/kit-theme.spec.ts] | [] | [archive §30 "12g" L4199–4212, session 2026-10-22 — docs/dev/ui-kit/mockups/2026-10-12-12g-test (ThemeBg, option A)] |
| [0287](0287-view-bar-any-object-pool-windows.md) | The view bar holds any object; one pool window type per pool | built | [views::buttons tests, pools_every_pool_has_a_window_type, kit-viewbar.spec.ts] | [] | [session 2026-10-22, view bar round 1 — docs/dev/ui-kit/mockups/2026-10-22-view-bar/01-view-bar.html, approved] |
| [0288](0288-one-preset-pool-follows-the-programmer.md) | One preset pool that follows the programmer | built | [objects::preset tests, a_preset_carries_the_tag_assigned_at_it, kit-presets.spec.ts] | [visions/presets.md "feature group pools", record 0105 Preset Parts(2)] | [session 2026-10-23, preset pool round 1 — docs/dev/ui-kit/mockups/2026-10-23-presets/01-presets.html, approved] |
| [0290](0290-meow-license-consent-on-every-login.md) | Meow License consent on every login | built | [E-USER-010, a_login_waits_for_the_meow_license_and_consent_finishes_it, each_holder_waits_on_its_own_and_its_token_with_it, login-sysmon.spec.ts] | [] | [session 2026-10-23, "Meow consent A is good" — docs/dev/ui-kit/mockups/2026-10-23-meow-consent/01-consent.html, approved] |
| [0291](0291-nav-mode-ls-rows-and-an-inspector.md) | NAV mode is ls's rows and an inspector | built | [nav_rows_are_ls_rows, every_object_opens, every_row_opens_by_its_path, fixture_101_has_identity_facts_properties_and_links, a_bookmark_survives_renumbering, ls_draws_the_last_glyph_on_the_last_row, copy.spec.ts] | [] | [session 2026-10-23, "all looks good i think" — docs/dev/ui-kit/mockups/2026-10-23-nav-mode/01-nav.html (round 1, layout A) and 02-inspector.html (round 2); research into four families of tools: file managers, dashboards, data explorers, inspectors] |
| [0292](0292-tree-window-nav-in-the-web-and-app.md) | The Tree window is NAV's tree and inspector | built | [the_tree_opens_down_to_the_place, a_place_is_a_path_or_a_guid, group_1s_inspector_has_settings_cells, follow_goes_to_the_newest_pick, the_tree_window_follows_the_selection, kit-tree-window.spec.ts] | [] | [session 2026-10-23, "looks good, implement" — docs/dev/ui-kit/mockups/2026-10-23-tree-window/01-tree-window.html (round 1, every question as recommended)] |
| [0294](0294-object-paint-palette-free-theme-links.md) | "Object paint: palette, free and theme links; colour from tags" | built | [lumacore-base paint tests, paints_become_hex_and_feature_groups_follow_their_slots, commands::paint_slots tests, the_colour_order_is_own_then_tag_priority_then_parent_then_pool, lumacore-views paint and rules tests, a_one_of_a_kind_object_as_a_value_takes_a_name, kit-paint.spec.ts, kit-rules.spec.ts] | [nodemeta-colour-icon] | [session 2026-10-23, object paint rounds 1–4 — docs/dev/ui-kit/mockups/2026-10-23-paint/ (04 approved: "looks good, implement")] |

## undo

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0008](0008-change-path-undo-subtrees.md) | The change path diffs subtrees; undo per ShowData field | built | [change::subtrees] | [archive §26 L2470–2471 "hand-written affected_subtrees list", archive §26 L2469 "root.change door"] | [archive §21 L1331–1393, archive §26 L2466–2479, archive §26 L2480–2495] |
| [0171](0171-oops-categories-statements.md) | Oops: one list, category a label, statement a step, Delete asks once | built | [`oops_tests.rs`, `a_members_station_step_is_undone_there_by_an_oops_on_the_master`, `a_grid_delete_asks_once_naming_its_rows`, E-CLAIM-001] | [archive §29 L3142–3146 "Programmer Selection ShowData Create", archive §29 L3357–3359 "Reset and Meow ReadOnly", archive §29 L3388–3392 "target-less Set CreateOops", archive §30 L4103–4107 "Delete without use stays", archive §30 L4589–4596 "handler asks for Delete User"] | [archive §29 L3138–3157, archive §29 L3319–3406, archive §30 L4147–4150, archive §30 L4589–4596, archive §30 L5347–5360] |
| [0172](0172-oops-window.md) | The Oops window: select, then Oops n | built | [the `oops` State feed, e2e on desktop / tablet / phone / WebKit] | [archive §30 L6236–6238 "round 7 chips filter the view"] | [archive §30 L5749–5752, archive §30 L5770, archive §30 L6236–6238, archive §30 L6427–6442] |

## users

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0162](0162-default-profile-and-user-asks.md) | Default profile is a locked template; user asks are station asks | built | [E-USER-002, E-USER-003, E-USER-004, E-USER-005, E-USER-006, E-USER-007, E-USER-008, E-USER-009] | [] | [archive §30 L4578–4600, archive §30 L4648–4658] |
| [0163](0163-profile-carries-look.md) | The profile carries the look and travels as a file | built | [E-SHOW-003, E-MEDIA-001, E-MEDIA-002, E-MEDIA-003, E-MEDIA-004] | [archive §30 L3667–3681 "LockScreen on the User", archive §30 L3805–3809 "needs /Overwrite", archive §30 L3847–3849 "needs /Overwrite"] | [archive §30 L3789–3800, archive §30 L3805–3809, archive §30 L3834–3849, archive §30 L4103–4107, archive §30 L4151–4160] |
| [0164](0164-theme-on-profile.md) | Theme on the UserProfile; the TUI wears it | built | [tidy `terminal_colours_only_in_the_look`, tidy `frontend_colours_only_in_the_theme`, `look_tests`] | [archive §30 L3604–3608 "stored on the user", archive §30 L3649–3652 "ShowData.Users.n.Theme", archive §30 L3839 "Tint adjuster"] | [archive §30 L3599–3608, archive §30 L3649–3652, archive §30 L3789–3795, archive §30 L3839–3846, archive §30 L4072–4081, archive §30 L4170–4176, archive §30 L4199–4210] |

## webui

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0038](0038-3d-one-renderer-on-the-station.md) | Clients never draw 3D; one renderer on the station | decided | none | [] | [archive §36 L11130–11160] |
| [0195](0195-one-context-menu-from-the-registry.md) | One context menu for every list, built from the registry | built | [GET /api/object-menus, ContextDef rows, lumacore_views::context tests, e2e phone.spec.ts] | [archive §30 L3553–3557 "hold show me"] | [archive §34 L9935–9952, archive §34 L10619–10635, archive §37 L11256–11265] |
| [0196](0196-runtool-edittool-tool-object.md) | Two user tools, Run and Edit; the Tool object | decided | none | [archive §34 L9921–9925 "lasso in every"] | [archive §34 L10276–10335] |
| [0264](0264-patch-dialog-one-tile-claim-button.md) | One Patch tile, Live by default, a hold-to-claim button | built | [`tests/e2e/tests/kit-34g4.spec.ts`, `MenuDef.area`, the `patch` feed] | [archive §34 L9442–9451 "bottom-right button is Exit", archive §34 L9581–9584 "Exit in the power slot"] | [archive §34 L9442–9451, archive §34 L9564–9594, archive §34 L9650–9691, archive §34 L9748–9756, archive §34 L9975–9994, archive §34 L10588–10602, archive §34 L10636–10647] |
| [0274](0274-insert-fixtures-dialog.md) | Insert fixtures: Choose, then Place, one Store line | built | [`kit/insert.js`, `tests/e2e/tests/phone.spec.ts`] | [] | [archive §34 L9719–9724, archive §34 L9790–9802, archive §34 L10023–10032, archive §34 L10368–10378, archive §34 L10543–10559, archive §37 L11234–11242] |
| [0278](0278-dmx-map-two-zooms-park-on-conflict.md) | One DMX map, two zooms; an overlap parks both | built | [`lumacore_tree::dmx_slots::SlotMap`, the `patch` feed's `runs` / `conflicts`] | [archive §34 L9759–9766 "Address cell or Patch to… opens Edit Patch"] | [archive §34 L9759–9766, archive §34 L9909–9916, archive §34 L10428–10464, archive §34 L10516–10542, archive §37 L11243–11255] |
| [0280](0280-patch-function-block-sort-offset.md) | The function block: Sort is grid order, Offset spare slots | built | [`Set GridColumns "patch.fixtures" "Offset" n`, `GridState.offset`, `<luma-grid>.sortBy`] | [] | [archive §34 L9757–9758, archive §34 L10358–10367, archive §34 L10410–10419, archive §34 L10575–10587] |

## windows

| id | title | status | confirmed by | supersedes | source |
|---|---|---|---|---|---|
| [0067](0067-sequence-pool-and-window.md) | Sequence pool in executor style; sequence window sheet and timeline | built | [e2e `kit-12f.spec.ts`] | [] | [archive §30 L5757–5763, archive §30 L5771–5780, archive §30 L5786–5795, archive §30 L5802–5807, archive §30 L5813–5827, archive §30 L6343–6361, archive §30 L6374–6392, archive §30 L6401–6412] |
| [0201](0201-window-bar-and-self.md) | Window bar items are lines; Self is the line's owner | built | [E-CMD-016, window_bar tests] | [archive §30 L3586 "its window or the key"] | [archive §30 L3575–3590, archive §30 L4461–4479, archive §30 L4492–4498, archive §30 L4532–4542, archive §30 L4551–4559] |
| [0202](0202-window-types-rust-json.md) | Window types declared in Rust, sent as JSON | built | [GET /api/window-types, data-id hooks] | [archive §30 L3733–3740 "KDL on disk"] | [archive §30 L3733–3740, archive §30 L4499–4509, archive §30 L4532–4537] |
| [0203](0203-window-presets-resets.md) | WindowPreset on the profile; layered resets; UI JSON export | built | none | [] | [archive §30 L3572–3574, archive §30 L4480–4491, archive §30 L4510–4513, archive §30 L4543–4550] |

## Citation map (archive § → records)

| § | records |
|---|---|
| §1 | 0001 |
| §2 | 0002, 0010 |
| §3 | 0014 |
| §4 | 0003, 0004 |
| §5 | 0012 |
| §6 | 0013 |
| §7 | 0015, 0016 |
| §8 | 0081, 0085 |
| §9 | 0083 |
| §10 | 0002, 0004, 0005, 0021, 0022, 0023 |
| §11 | 0015, 0024, 0025, 0033, 0041, 0042, 0043, 0044, 0050, 0085, 0262, 0263 |
| §12 | 0006, 0105, 0207, 0208 |
| §13 | 0061 |
| §15 | 0012, 0102, 0111 |
| §16 | 0017 |
| §17 | 0170 |
| §18 | 0025 |
| §19 | 0161 |
| §20 | 0082 |
| §21 | 0002, 0004, 0005, 0006, 0007, 0008, 0021, 0022, 0023, 0024, 0081, 0082, 0083, 0109, 0111 |
| §22 | 0011, 0014, 0063, 0204 |
| §23 | 0025, 0029, 0037, 0082, 0083, 0094, 0161, 0227 |
| §24 | 0028, 0044, 0049, 0262 |
| §25 | 0028, 0090, 0131, 0132 |
| §26 | 0008, 0026, 0029, 0129 |
| §27 | 0003, 0015, 0016, 0017, 0024, 0041, 0043, 0044, 0050, 0061, 0064 |
| §28 | 0050, 0071, 0072, 0277 |
| §29 | 0007, 0009, 0017, 0022, 0025, 0043, 0064, 0081, 0090, 0091, 0129, 0131, 0161, 0170, 0171, 0174, 0252 |
| §30 | 0026, 0033, 0034, 0035, 0061, 0062, 0063, 0064, 0065, 0066, 0067, 0068, 0069, 0070, 0081, 0083, 0084, 0085, 0103, 0128, 0146, 0150, 0151, 0152, 0153, 0161, 0162, 0163, 0164, 0165, 0166, 0167, 0168, 0169, 0170, 0171, 0172, 0173, 0181, 0182, 0183, 0184, 0185, 0186, 0187, 0188, 0189, 0190, 0191, 0192, 0193, 0201, 0202, 0203, 0204, 0205, 0206, 0286 |
| §31 | 0026, 0027, 0030, 0031, 0036, 0044, 0047, 0049, 0106, 0111, 0113, 0237, 0251 |
| §32 | 0061, 0062, 0071 |
| §33 | 0009, 0042, 0062, 0101, 0102, 0103, 0104, 0106, 0107, 0108, 0109, 0110, 0111, 0112, 0113, 0121, 0122, 0123, 0124, 0125, 0126, 0127, 0141, 0142, 0143, 0144, 0145, 0148, 0149 |
| §34 | 0032, 0042, 0045, 0046, 0048, 0051, 0088, 0092, 0130, 0133, 0194, 0195, 0196, 0221, 0222, 0223, 0224, 0225, 0226, 0227, 0228, 0229, 0230, 0231, 0232, 0233, 0234, 0235, 0236, 0241, 0242, 0243, 0244, 0245, 0246, 0247, 0248, 0249, 0250, 0261, 0262, 0263, 0264, 0265, 0266, 0267, 0268, 0269, 0270, 0271, 0273, 0274, 0275, 0276, 0277, 0278, 0279, 0280, 0281 |
| §35 | 0133, 0134, 0135, 0136 |
| §36 | 0037, 0038 |
| §37 | 0191, 0195, 0274, 0278 |
| §38 | 0086, 0087, 0088, 0089 |
| §39 | 0147 |
| §40 | 0052, 0091, 0093, 0094, 0194, 0265, 0267, 0268, 0271, 0272 |
