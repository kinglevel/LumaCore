# 12d research: one-finger lasso vs. scrolling on touch (grids, pools, layout canvases)

Date: 2026-10-13. Question: how do large open-source, multiplatform projects let a one-finger lasso (rubber band) live next to scrolling and panning on touch, and how do they add or remove items without a keyboard? What should LumaCore do, given that the user wants a console-style always-on lasso?

Reference console baseline: a one-finger drag is a rectangle lasso, **two fingers scroll** (with inertia), Ctrl+tap adds or removes, a header tap moves the selection to that column and a second tap selects the whole column, and a lasso down the No column selects rows. "Precise Edit" exists because "a finger that slips becomes a lasso".

How the evidence was gathered: **source** = I read the project's code (tldraw `useGestureEvents.ts`, `Editor.ts`, `options.ts`, `PointingCanvas.ts`, `Brushing.ts`; Excalidraw `App.tsx`, fetched from `main`/`master` today). **issue** = I read a GitHub/GitLab issue body through the API. **doc** = official docs. *(not found)* = I searched and found nothing reliable.

---

## 1. The patterns, and who uses each

### P1. Always-on lasso, two-finger pan (console style)

One finger on empty space draws a box or lasso. Two fingers pan and pinch. The surface uses `touch-action: none` and implements its own scrolling.

- **tldraw** (source). In the select tool, a pointer-down on the canvas enters `pointing_canvas`, and once the pointer passes the drag threshold it moves to `brushing`. Two touches go to `useGestureEvents`, which starts in `pinchState = 'not sure'` and decides between "zooming" (fingers moved more than 24 px apart) and "panning" (the midpoint moved). Going from panning back to zooming needs 64 px. Add-to-selection works only with a modifier: `const additiveSelectionKey = info.shiftKey || info.accelKey`. Touch thresholds are larger: `coarseDragDistanceSquared: 36 // 6 squared` against 4 px for a mouse, `uiCoarseDragDistanceSquared: 625` ("it's really easy to accidentally drag from the toolbar on mobile"), `longPressDurationMs: 500`, edge scroll `edgeScrollDistance: 8`, `edgeScrollDelay: 200`. https://github.com/tldraw/tldraw/blob/main/packages/editor/src/lib/hooks/useGestureEvents.ts · https://github.com/tldraw/tldraw/blob/main/packages/editor/src/lib/options.ts
  - **Late second finger, approach 1 (rollback).** On `pinch_start` the editor calls `this.interrupt()` and restores the selection from before the gesture: "on touch, the first finger's pointer_down can change the selection before the second finger starts the pinch, and we want to restore the selection from before that change." (`Editor.ts`)
  - **Late second finger, approach 2 (ignore).** Open PR #9545: "once `inputs.getIsDragging()` is true (drawing, translating, resizing, brushing, etc.), a starting pinch is dropped and events from any other pointer are rejected until it lifts … A pinch that begins *before* the interaction becomes a drag still zooms exactly as before." Its motivation: "a stray palm, a partner grabbing the tablet, or just reaching to pan" wrecked the action. https://github.com/tldraw/tldraw/pull/9545
- **Excalidraw** (source). On a phone the default selection tool is the **lasso**: `formFactor === "phone" ? "lasso" : "selection"`. Pointer-down ignores selection while panning: `// don't select while panning  if (gesture.pointers.size > 1) return;`. Two pointers drive pinch and pan (`gesture.pointers.size === 2`). Alt switches a box selection into a lasso.
  - Open complaints: #7832 "having to press a button to enable the pan function feels a bit awkward … I wish I could move the canvas around with one finger". #5176 "zooming in on the canvas with two fingers triggers drawing" (19 comments, open). #9705 "Lacks standard tablet features like single-finger pan". #966 asks to "abort first touch if a second `pointerdown` (i.e. for pan/zoom) occurs before the first pointer moves too far" and to "Select on `pointerup` for `touch` events".
  - Links: https://github.com/excalidraw/excalidraw/issues/7832 · /5176 · /9705 · /966
- **Figma (iPad, not open source; shown for the complaint).** "don't touch two fingers to pan at exactly the same time I can accidentally drag something." Users ask for FigJam's model instead. https://forum.figma.com/suggest-a-feature-11/feature-request-prevent-simultaneous-tap-and-drag-on-touchscreens-39114
- **GNOME Files** (issue/MR). A touch drag on empty space draws the rubber band. Open MR !1508 (May 2024, not merged): "the long press gesture is also triggered and cancels the user's selection". The same root cause gives #2872, where a tap triggers a right-click. https://gitlab.gnome.org/GNOME/nautilus/-/merge_requests/1508
- **Godot Android editor** (doc). Two fingers rotate and zoom, double-tap-and-drag zooms, and "Tap and hold … to activate its context menu". The docs say nothing about touch box selection. https://godotengine.org/article/android_godot_editor_play_store_beta_release/
- **Krita** (doc/forum). Gestures are configurable (Canvas Input Settings). A developer MR says the defaults should be "three finger drag to pan canvas" so that one finger stays free for tools. https://krita-artists.org/t/touch-gesture-controls-not-working-properly-in-krita-6-0-nightlies/124583/3

**Pros.** No mode. Fast. Matches established consoles and drawing tools.
**Cons, with evidence.**
- Two-finger timing errors: Figma, Excalidraw #5176, tldraw #9545.
- On a phone, people expect one-finger scrolling: Excalidraw #7832 and #9705.
- `touch-action: none` disables browser pinch-zoom inside the surface (see §3).
- Two-finger pan is a multipoint gesture, so it needs a single-pointer alternative (WCAG 2.5.1).

### P2. One-finger pan, lasso only through a tool or mode

- **Excalidraw hand tool** (#794 "Add 'pan mode' … would also allow single-finger panning on mobile", closed and built), **tldraw hand tool**, and **FigJam** ("panning is performed with a single finger and an object needs to be explicitly selected"). The user switches between Select and Hand explicitly.
- **iOS UIKit** (doc). Band selection (`UIBandSelectionInteraction`) is **pointer-only**: "On devices with pointer-based input, such as an iPad with a Magic Keyboard … a visible selection rectangle … when someone clicks and drags". Shift at the start extends the selection. On touch, multi-select happens in **edit mode**: two-finger pan selects and "changes to edit mode". The "selected items don't have to be contiguous … scroll the view, and use your two fingers again". Dragging over selected items **deselects** them, and in edit mode "the user can also pan a single finger along the constrained axis to select more items". https://developer.apple.com/documentation/uikit/selecting-multiple-items-with-a-two-finger-pan-gesture · https://developer.apple.com/documentation/uikit/uibandselectioninteraction
- **Android `recyclerview-selection`** (doc). Band selection is for the mouse ("PointerSelection is best bound to TOOL_TYPE_MOUSE, and only that tool type"). The finger gets gesture selection ("GestureSelection is best bound to TOOL_TYPE_FINGER"): long-press an item, then drag. https://developer.android.com/reference/androidx/recyclerview/selection/SelectionTracker.Builder
- **Home Assistant** (doc). An explicit "Enter selection mode" button in tables. On dashboards, edit mode plus "tap and hold the Move handle". https://home-assistant.io/blog/2024/03/06/release-20243/

**Pros.** Native scrolling. Accessible. No timing races.
**Cons.** Hunting for the mode. Users forget which mode they are in. Opposite of console muscle memory.

### P3. Long-press, then drag, gives the lasso; a plain drag scrolls

- **draw.io** (doc): "Long tap on a blank area of the drawing canvas, then drag a selection box - a rubberband"; "Tap and drag on a blank area … to move to another position". https://www.drawio.com/blog/touch-screen-diagrams
- **Windows File Explorer** (third-party guides): "press and hold your finger on the screen while immediately dragging … A quick swipe won't work"; item checkboxes are on by default on touch devices. https://www.digitalcitizen.life/select-multiple-files-windows
- **Google Photos and Immich mobile**: in selection mode, long-press then drag ("press your thumb to drag and select in the selection mode", Immich v1.99, PR #8004). Google Photos **broke** this in Aug 2026 ("dragging … would only cause the screen to scroll"; cited in `12d-grid-projects.md`).
- **DayPilot scheduler**: "the time range selecting mode requires a 'tap-and-hold' gesture … Before the selection is activated you can scroll". **react-beautiful-dnd**: "If the user lifts their finger before the timer is finished, the event is released to the browser". **dnd-kit**: `delay` + `tolerance` ("particularly useful for touch input"). https://github.com/ivan-pik/react-beautiful-dnd/blob/master/docs/sensors/touch.md · https://dndkit.com/api-documentation/sensors/touch

**Pros.** Native scrolling stays. One rule for all sizes. Discoverable through habit (Android/Photos).
**Cons.**
- A 400–500 ms wait for every lasso.
- Long-press is overloaded: context menu, edit (console tap-and-hold = edit), and Nautilus's cancel bug.
- Fragile when the browser already owns the scroll (the Photos breakage).

### P4. Drag from a handle, margin strip, or constrained axis

- **Excel Android** (doc): "Tap, then drag the selection handler"; header tap selects a row or column; one-finger scroll. https://support.microsoft.com/en-US/Excel/excel-for-android-touch-guide
- **Google Sheets mobile**: the same handle model (`12d-grid-projects.md`).
- **Collabora Online / LibreOffice Android**: the same handle model, with this complaint: "Tap+drag … on a selected cell's handle is sometimes mistaken for panning … the view jumps one way while the selection jumps the other". https://forum.collaboraonline.com/t/android-annoyances-pan-instead-of-select-frequent-reloading-etc/4829
- **iOS** (doc above): one finger along the edge of the checkbox column, or along the non-scrolling axis, selects.
- **react-grid-layout / jQuery UI / SmartClient**: drag only from a handle; `touch-action: none` only on the handle, "where dragging of records might otherwise interfere with scrolling".
- **AG Grid** (doc): the touch page lists no touch range selection. Range selection is a mouse drag, and touch is an afterthought. https://www.ag-grid.com/javascript-data-grid/touch/

**Pros.** Native scrolling. Precise.
**Cons.** Small targets on 390 px. Handle-vs-pan misreads (Collabora). Rectangles only grow from an existing selection.

### Keyboard-only projects (no touch answer)

- **QLC+ v5** (doc): "Ctrl-click to add" in the Virtual Console and the 2D view. Rubber band from empty space. Clicking empty space clears. An alpha tester asked for "shift or long press (for touch screens)". https://docs.qlcplus.org/v5/fixtures-and-functions/2d-view · https://forum.qlcplus.org/viewtopic.php?p=51579
- **Inkscape**: Alt turns the rubber band into "touch" (intersect) selection; no touch-screen mode. **Kdenlive, OBS, Penpot**: *(not found)* no documented touch selection. These are desktop tools.

### Summary table

| Project | One-finger drag on empty space | Pan / scroll | Add / remove without keyboard |
|---|---|---|---|
| reference console | lasso | 2 fingers | none (Ctrl only) |
| tldraw | brush (select tool) | 2 fingers; Hand tool | none (Shift/Accel) |
| Excalidraw | lasso (phone default) / box | 2 fingers; Hand tool | none (Shift) |
| GNOME Files | rubber band | scroll bar / kinetic | none (Ctrl) |
| draw.io | pan | 1 or 2 fingers | long-press-drag box |
| Windows Explorer | scroll | 1 finger | hold-then-drag band; checkboxes |
| iOS UIKit | scroll | 1 finger | edit mode: tap toggles, 2-finger pan, cross-axis drag |
| Android selection lib | scroll | 1 finger | long-press, then tap / drag |
| Excel / Sheets / Collabora | scroll | 1 finger | handle drag; header tap |
| Home Assistant | scroll | 1 finger | "Enter selection mode" button |
| QLC+ | rubber band (mouse) | — | none (Ctrl) |

The pattern: **canvas tools** (tldraw, Excalidraw, the reference console, Nautilus icon view) take one finger for selection and pan with two. **Lists and documents** (iOS, Android, Sheets, Explorer, HA) keep one-finger scrolling and put selection behind a mode, a long-press or a handle. **No project offers a modifier-free add/remove except through a mode** (iOS edit mode, Android action mode, HA selection mode).

---

## 2. Adding, removing and toggling without Ctrl

| Technique | Who | Notes |
|---|---|---|
| **Selection mode: tap toggles** | iOS edit mode, Android action mode (long-press enters), HA, Google Photos, Immich | The universal phone answer. Exit with Done. |
| **Drag over selected items deselects** | iOS two-finger pan | The first item touched decides the direction (add or remove) for the rest of the drag. Gmail and Photos do the same. Worth copying for the lasso: start on a selected cell → remove. |
| **Modifier latch on screen** | Windows on-screen keyboard Ctrl (Explorer guide: "use the on-screen keyboard's Ctrl key"); a console's own modifier key only in its selection grid | Clumsy as a system keyboard, but a dedicated **Add / Remove latch** in the app's own bar is cheap. LumaCore already has a held Meow button that lights while held (12b2). |
| **Hold a button with one hand, tap with the other** | console hardware (modifier key + tap), tldraw/Excalidraw with Shift on a keyboard | Fine on a desk or tablet. Impossible one-handed on a phone. |
| **Tap-tap range ("Thru")** | WCAG 2.5.7 example ("tap one corner and then the opposite corner"); a console's modifier + tap on the end point in its selection grid | The accessible alternative to dragging. Matches console language. |
| **Checkbox column** | Explorer touch default, Material, Gmail | Costs width on 390 px. A grid cell **is** its own target, so no checkbox is needed. |
| **Header tap = column; gutter = rows** | the reference console, Excel/Sheets | Already console behaviour. |

Latch design notes from the evidence:
- A latch that stays on after use leads to "which mode am I in". Users of mode-based tools complain, as in Excalidraw #7832 about the pan button.
- A **one-shot** latch (applies to the next tap or lasso, then drops) avoids that but needs re-arming each time.
- Best of both, the iOS Shift-key pattern: tap once = one-shot, double-tap or hold = locked, and the bar shows the state.
- Console users know `+` and `-` from the command line (`Fixture 1 + 5 - 3`), so **`+` / `−` labels** say exactly what they do.

---

## 3. Detecting two-finger tap/pan reliably in browsers

**Mechanics (MDN, dnd-kit).**
- `touch-action` is decided **at gesture start** by intersecting the element and its ancestors: "After a gesture starts, changes to `touch-action` will not have any impact on the behavior of the current gesture." https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action
- With pointer events, the browser takes over a pan it is allowed to do and sends `pointercancel`. dnd-kit: setting `touch-action: none` is "the only way to reliably prevent scrolling for pointer events". Touch events, by contrast, can `preventDefault()` in a non-passive `touchmove` ("Touch events … do not suffer the same limitations"). https://dndkit.com/api-documentation/sensors/pointer
- Track pointers in a map by `pointerId`. Treat `pointercancel`, `pointerup` and `pointerleave` the same way (MDN multi-touch example). https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events/Multi-touch_interaction

**Recipe used by tldraw and Excalidraw, extended by #966 and #9545 (for P1 surfaces).**
1. Set `touch-action: none` on the surface only, never on `body`. Ignore `pointerType === 'mouse' | 'pen'` for the pan logic.
2. On the first pointer-down, enter a **pending** state: no selection change yet, remember the selection, record time and position.
3. If a second pointer goes down **while pending** (before 6–10 px of movement, or within about 150 ms), switch to **pan**. Restore the remembered selection (tldraw rollback).
4. Once the first pointer has moved past the slop (6 px coarse, tldraw), start the **lasso**. From then on, **reject other pointers until it lifts** (tldraw #9545). A palm or late finger cannot spoil the lasso. Lifting and putting down again is cheap.
5. For two pointers, use tldraw's classifier: compare the change in distance (more than 24 px = zoom) with the midpoint movement (pan). Panning back to zooming needs 64 px. Add inertia on release (the reference console has it).
6. Tap means pointer-up without passing the slop. Never select on pointer-down (Excalidraw #966 "Select on `pointerup` for `touch`"; Syncfusion lesson in `12d-grid-projects.md`).
7. A **two-finger tap** is both pointers down and up within about 250 ms with under 10 px movement each and no pan or zoom chosen. A reference console calls this "2 Finger Edit". It is reliable once (3) and (4) exist, but it is not discoverable, so treat it as a shortcut only.
8. Edge auto-scroll during the lasso: tldraw uses an 8 px zone with a 200 ms delay on desktop. On touch, use a 32–48 px zone with speed rising toward the edge, because fingers cover the edge.

**For P2/P3 surfaces (one-finger scroll stays native).**
- Use `touch-action: pan-x pan-y` (or `manipulation`) and native overflow scrolling. Momentum and accessibility come free.
- Long-press: start a 400–500 ms timer. Cancel it if the pointer moves more than about 8 px, or on `pointercancel`. A non-passive `touchmove` listener calls `preventDefault()` **only after the timer fired**. Movement before that stays a scroll, and movement after it becomes the lasso (dnd-kit's touch-sensor rule). This is how the Photos/Immich drag-select works, and also where it breaks: if any ancestor already started scrolling, it cannot be taken back.
- Suppress the long-press side effects: `-webkit-touch-callout: none` and `user-select: none` on the surface (iOS callout and magnifier), and `contextmenu` `preventDefault` for touch pointers (Android and Windows press-and-hold right-click).

**Platform conflicts.**
- **iOS Safari.** It ignores `user-scalable=no` (since iOS 10, https://webkit.org/?p=7367), so page pinch-zoom always exists outside surfaces that use `touch-action: none`. Inside such a surface, the surface owns pinch. Safari's left-edge swipe-back and the iOS system gestures (home bar, 3-finger undo, 4/5-finger on iPad) cannot be blocked. Keep a lasso's start zone away from about 20 px of the left screen edge, or accept it. `touch-action` is supported fully since iOS 13 (caniuse).
- **Android Chrome.** Pull-to-refresh and overscroll navigation: use `overscroll-behavior: contain` on scroll containers. System back gesture from both edges cannot be blocked. Long-press fires `contextmenu`.
- **Windows touch (Edge/Chrome).** Pointer events are native. Press-and-hold fires right-click (`contextmenu`). Edge swipes belong to the OS. `touch-action` is honoured.
- **Desktop app (WebView on macOS/Windows/Linux).** No back-swipe in a plain WKWebView/WebView2 unless enabled, so P1 behaves best there.

**Accessibility.**
- WCAG 2.5.1 (A): "All functionality that uses multipoint or path-based gestures … can be operated with a single pointer". Two-finger pan needs an alternative: visible scroll bars on touch (the reference console has 20 px bars), the one-finger scroll mode, or keyboard and encoder. https://www.w3.org/WAI/WCAG22/Understanding/pointer-gestures.html
- WCAG 2.5.7 (AA, 2.2): "All functionality that uses a dragging movement … can be achieved by a single pointer without dragging". Their example is tap one corner, then the opposite corner, which is exactly `Thru`. https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html
- MDN: "`touch-action: none;` may inhibit operating a browser's zooming capabilities". Give the window its own zoom (the reference console's font size 10–32 / LumaCore window font size) and never put `none` on the page.
- With VoiceOver or TalkBack on, custom gestures don't arrive. Every selection must also be a command line (`Select Line 3 Thru 7`), which LumaCore already guarantees.

---

## 4. Recommendation for LumaCore

**One kit behaviour, not per-window code: a "select surface" controller** used by `<luma-grid>`, pool windows and future layout canvases. Grid cells, pool cells and layout objects are all "items with rectangles". The same gesture table applies to all three, and only the hit-testing differs (cells vs. free objects). Every gesture still ends as **one command line** (`Select Line 3 Thru 7`, `Set Line 2 + 5 + 9 Rate 30`, `Fixture 1 Thru 12 + 20`).

**The gesture table, the same on every size:**

| Gesture | Lasso mode (desk, tablet; console style) | Scroll mode (phone default) |
|---|---|---|
| One-finger drag | **lasso** (rectangle; from a selected item = remove) | native scroll |
| Hold about 450 ms, then drag | lasso | **lasso** |
| Hold and release without moving | edit (console tap-and-hold) | edit |
| Two-finger drag | **pan with inertia** | (native: scroll/zoom page) |
| Pinch | layout canvas: zoom · grid/pool: nothing (window font size instead) | browser |
| Tap | select (replace); with `+` latched: toggle | same |
| Tap header / gutter | column (second tap = whole column) / row | same |
| `+` / `−` latch, then lasso or tap | union / subtract | same |
| Tap A · `Thru` · tap B | range (accessible, no drag) | same |

Supporting rules:
- **Mode is a per-device setting** (like Keyboard Auto/OS/Built-in: local storage per screen): `Drag: Auto | Lasso | Scroll`. Auto = Lasso when the window is at least 600 px wide or a pointer is fine, and Scroll on a phone.
- A **lasso toggle** in the selection bar flips the mode for the current window, for a phone user who wants the console lasso for a while. Its state is visible.
- Hold-then-drag lassos in **both** modes, so one habit works everywhere. In Lasso mode the hold is unnecessary but harmless.
- **Two-finger rules** (§3 recipe): pending until 6 px of movement or 150 ms. A second finger in that window means pan, with the selection rolled back. Once the lasso has started, extra fingers are ignored.
- **Mouse.** Left drag = lasso. Ctrl+click and Ctrl+drag = `+`. Ctrl+Alt (or Alt) = `−`. Shift+click = Thru from the anchor. Wheel and middle drag = scroll. A mouse needs no modes.
- **Lasso rectangle in grids snaps to cells**. **In layouts it hits by intersecting** (console / QLC+ behaviour); Meow while dragging switches to "fully inside".
- **Selection order** for `Thru` and patch addressing: lasso = reading order (row-major, or column-major in a column selection), taps = tap order. Show small order numbers only where the order matters (patch).
- **Visible scroll bars on touch** in Lasso mode (20 px, kit-styled) cover WCAG 2.5.1. `Thru` covers 2.5.7.
- **Edge auto-scroll** while lassoing: a 40 px zone on touch, 16 px with a mouse, speed rising toward the edge.
- `touch-action: none` only on the surface's item area in Lasso mode, `pan-x pan-y` in Scroll mode. Window chrome, bars and the page never get `none`.

**Why this fits the user's goal.**
- Programmers get the console lasso and two-finger scroll unchanged on the desk and tablet.
- On the phone the layout is the familiar one-finger scroll (avoiding Excalidraw #7832 and #9705), and the same lasso is one hold away.
- `+` / `−` replace Ctrl in console vocabulary.
- The same controller later drives layout canvases, where a pinch zoom is added.

---

## Decision questions (recommended option first)

**Q1. One-finger drag on a selection surface**
- **(a)** Per-device `Drag: Auto | Lasso | Scroll`; Auto = Lasso on desk/tablet and Scroll on phones; hold-then-drag lassos in both. *Recommended: the console lasso where there is room, native scrolling where people expect it, one shared habit.*
- (b) Lasso everywhere, including phones (pure console style; Excalidraw does this on phones). Consistent, but phone users fight it (#7832, #9705).
- (c) Scroll everywhere, lasso only on hold (draw.io/Explorer). Safe, but drops the console lasso the user asked for.

**Q2. Add/remove without a keyboard**
- **(a)** `+` / `−` latches in the selection bar: tap = one shot, double-tap = locked; with a keyboard, Ctrl = `+` and Alt = `−`. *Recommended: console vocabulary (`Fixture 1 + 5 - 3`), one-handed, works on lasso and tap.*
- (b) Hold the on-screen Meow button as Ctrl (two hands). Good on a desk, impossible one-handed on a phone.
- (c) A selection mode where every tap toggles (iOS/Android/HA). Familiar on phones, but a mode to enter and leave, and it breaks the console habit "tap replaces".

**Q3. A second finger arrives during a one-finger touch**
- **(a)** Before 6 px or 150 ms: pan, with the selection rolled back. After the lasso has started: ignore extra fingers until lift. *Recommended: tldraw's rollback plus PR #9545; fixes Figma's and Excalidraw's #5176 class of bug.*
- (b) Always cancel the lasso and pan. Simple, but palms wreck lassos.
- (c) Always ignore second fingers once one is down. Forces a perfectly simultaneous two-finger landing.

**Q4. Pinch on a surface**
- **(a)** Grids and pools: no pinch (window font size controls size); layout canvases: pinch zooms the canvas. *Recommended: grids stay console-like and predictable, canvases behave like tldraw/Excalidraw.*
- (b) Pinch scales font size in grids too. Handy, but it competes with two-finger scroll classification (24/64 px thresholds).
- (c) Let the browser zoom the page (`touch-action` with `pinch-zoom`). Gives up owning two-finger pan.

**Q5. Starting a lasso on an already selected item**
- **(a)** Removes from the selection (iOS / Photos rule: the first item decides). *Recommended: subtract without any latch, natural "erase".*
- (b) Always replaces (console style). Needs `−` latch to subtract.
- (c) Drags or moves the selection (canvas tools). Right for layout editing, wrong for grids. Possibly layout-only with Meow.

**Q6. Accessible, no-drag alternative**
- **(a)** `Thru` in the selection bar (tap A, Thru, tap B), plus touch scroll bars in Lasso mode. *Recommended: satisfies WCAG 2.5.1 and 2.5.7 and is the console's own word.*
- (b) Scroll mode only (rely on the device setting). Leaves Lasso-mode users without a one-finger scroll.
