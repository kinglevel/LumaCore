# Touch-first UI element kit — online research (2026-10-12)

Context: LumaCore frontend today is one `apps/lumacore-app/frontend/index.html` (~4600 lines, plain JS, vendored xterm.js + modern-screenshot, 6 `pointerdown`, 1 `touch-action`, 0 custom elements, 4 `mousedown`). Goal: a touch-first element kit (buttons, faders, encoders, lists, grids, sheets, pop-ups, keypads/calculator, virtual keyboard, swipe menus, settings editors) that becomes the standard for plugins and future windows, where the server owns state and every gesture is a command line.

Confidence markers: **[doc]** = read from the project's own docs/source this session; **[sec]** = secondary source (forum, search summary, aggregator); **[know]** = general knowledge, not re-verified this session.

---

## 1. Touch lighting / show-control / audio UIs

### 1.1 Open Stage Control (GPLv3, Node/Electron server + browser clients) — the most complete open reference
Docs: https://openstagecontrol.ammd.net/docs/widgets/properties-reference/ , https://openstagecontrol.ammd.net/docs/widgets/general-mechanics/ , themes https://openstagecontrol.ammd.net/docs/customization/themes/ (GitHub mirror `jean-emmanuel/open-stage-control` is now near-empty; code moved to framagit/codeberg).

- **Element set [doc]** (35 types): Basics — `button`, `switch`, `dropdown`, `menu` (drag-and-drop circular/grid menu), `input`, `textarea`, `file`; Containers — `panel` (layout default/vertical/horizontal/grid, tabs), `modal` (button that opens a popup panel), `clone`, `fragment`, `matrix` (rows/columns of widgets generated from one template), `keyboard` (piano), `patchbay`, `folder`, `tab`; Frames — `frame`, `svg`, `html`, `image`; Graphs — `plot`, `eq`, `visualizer`; Indicators — `led`, `text`; Pads — `xy`, `rgb`, `multixy`, `canvas` (multitouch canvas with user drawing + touch functions); Sliders — `fader`, `knob`, `encoder` (sends relative direction, not a value), `range` (two-headed fader), `scrollbar`; Scripts — `script`.
- **Declaration [doc]**: a session is one JSON tree of widget objects. Every widget has common groups `widget / geometry / style / value / osc / scripting` (`type`, `id`, `lock`, `address`, `preArgs`, `targets`, `linkId`, `bypass`, `css`, `onValue`, `onTouch`, `onCreate`…). The editor is a property inspector generated from the same schema. Properties can be expressions ("advanced syntaxes": `@{otherWidget}`, `#{js}`, `${}` variables) and scripts. Theming is CSS variables (`--color-accent`, `--color-widget`, `--color-fill`, `--color-stroke`, `--color-text`, `--widget-padding`), overridable per container (`.panel-container { --color-accent: blue }`); themes are tiny CSS files that override `:root` vars.
- **Fader touch properties [doc]**: `snap` (false = drag from last value — relative; true = jump to touch position — absolute), `touchZone` (`all` / `knob` / `gauge` — where a touch may start), `spring` (return to default on release), `doubleTap` (true = reset to default; or an OSC address; or `"script"` → `onTouch` with `event.type = doubleTap`), `sensitivity` (relative drag ratio), `steps` (n evenly spaced / array of values / `auto` from range breakpoints), `range` (object of breakpoints, i.e. non-linear piecewise scale), `origin` (gauge start, for bipolar), `logScale`, `gradient`, `pips`, `design` (default/round/compact), `horizontal`.
- **Knob [doc]**: `mode` = `vertical` (relative vertical drag), `horizontal`, `circular` (relative rotation), `snap` (jump to touched angle), `snap-alt`; `angle` (sweep, default 270), plus `spring`, `doubleTap`, `sensitivity`, `steps`.
- **Encoder [doc]**: knob that emits +/- direction (like an endless hardware encoder) — exactly the model of a console encoder sending "increment by n".
- **Button [doc]**: `mode` = `toggle` / `push` (press & release) / `momentary` (no release, no value) / `tap` (no release, sends `on`); `doubleTap` (require double tap — protects against accidental fire); `click` (fire on release instead of press); `decoupled` (local state only follows feedback from the server — i.e. "server owns state").
- **Switch/menu [doc]**: menu `mode` = `default` / `toggle` / `swipe` (opens on touch, value follows the finger over items, commits on release) / `doubleTap` / `doubleTap-toggle`. The `swipe` mode is the same idea as a console's swipe-out pool commands.
- **Gestures [doc]**: tap is handled at press time; click at release; double-tap; drag is 1:1; **fine mode = Ctrl+drag (mouse) or two-finger drag on a single-touch widget → precision ×10**. XY `axisLock` (Shift locks to first movement direction).
- **Multitouch / traversing [doc + sec]**: every widget gets its own pointer → multiple faders at once. Containers have `traversing: false | true | smart` — one finger sliding across many widgets drives each as it passes (like "painting" a bank of faders/buttons); `smart` limits it to widgets of the same type as the first touched. Since v1.29 the gesture may start on empty space inside a traversing container [sec: https://openstagecontrol.discourse.group/t/traversing-sliding-finger-from-blank-area-onto-a-button/3199]. In traversing containers `touchZone` is ignored and faders behave as snap (a known interaction [sec: https://openstagecontrol.discourse.group/t/fader-snap-feature-problem/448]).
- **Sync [doc]**: widgets with the same address/preArgs/targets are synced across clients automatically; `bypass` stops sync; `decoupled` makes feedback authoritative.
- Pitfalls reported [sec]: property names are case-sensitive (`doubleTap` not `DoubleTap`); `spring` has no speed and `spring`/`default` changes rebuild the widget [https://openstagecontrol.discourse.group/t/spring-mode-on-fader/2501].

**Take-away for LumaCore**: copy the vocabulary (`snap`/`touchZone`/`spring`/`doubleTap`/`steps`/`origin`/`sensitivity`, button `mode`s, menu `swipe`, container `traversing`) and the "one JSON tree, one schema drives editor + renderer + docs" idea. Do **not** copy OSC address-based widget identity, user JS expressions/scripts in descriptors, or client-to-client sync — LumaCore's server is the only state owner.

### 1.2 QLC+ (Apache-2.0, C++/Qt; v5 is QML)
Docs: https://docs.qlcplus.org/v5/advanced/web-interface , https://www.qlcplus.org/docs/html_en_EN/webinterface.html , widget overview https://mintlify.wiki/mcallegari/qlcplus/concepts/virtual-console
- **Virtual Console widgets [sec]**: Button (toggle/flash), Slider (Playback = submaster of a function, Level = direct channel, plus Grand Master mode), XY Pad (pan/tilt), Frame, Solo Frame (one child running at a time — radio), Speed Dial (tap tempo, ×/÷, BPM), Cue List (crossfade, next/prev), Label, Clock, Audio Triggers, Animation (RGB matrix). Widgets are C++ objects saved in the workspace XML with geometry, page, caption, colours and an input binding (`QLCInputSource`: universe/channel/feedback values).
- **Web interface [doc]**: the browser page mirrors the VC over WebSocket (buttons, sliders, cue lists, frames, clocks, XY pads, speed dials, animations, audio triggers, labels); auth only with `--web-auth`. It is a server-rendered HTML page + small JS — a "render what the server describes" model.
- **Multitouch [sec]**: forum reports desktop QLC+ (Qt5) not handling two faders at once; maintainer says it's up to Qt (https://forum.qlcplus.org/viewtopic.php?p=33151). No statement for the web page. Forum advice: make widgets bigger for touch.
- **Take-away**: the *kinds* (Solo Frame, Speed Dial with tap tempo, Cue List widget, slider modes Playback/Level/GM) are the lighting-domain catalogue to cover; the web page shows a server→client widget mirror works, but QLC+ is not a touch-engineering reference.

### 1.3 Chataigne (GPLv3, JUCE/C++) + chataigne-web-dashboard
https://github.com/benkuper/Chataigne , https://github.com/benkuper/chataigne-web-dashboard
- Dashboard [sec]: user builds dashboards from parameters (sliders, buttons, enums, colours, labels, groups, comments); "Dashboard Server" in project settings serves it on **WebSocket/HTTP port 9999**; the web client (Ember.js) renders the same dashboard from the parameter descriptions the server sends and writes values back. Customisable colours/labels.
- **Take-away**: closest in spirit to "server sends descriptors, browser renders, every change goes back to the server". Parameters carry their own type/range, so the dashboard item is generic over the parameter — the same as rendering a LumaCore `TreeDef::write` property.

### 1.4 Bitfocus Companion (MIT) — buttons as server-rendered state
Docs: Web Buttons https://companion.free/user-guide/sidebar/web_buttons , Emulator https://companion.free/user-guide/v4.3/interactive-buttons/emulator , Satellite API https://companion.free/for-developers/Satellite-API
- **Satellite API [doc]**: line-based protocol (TCP 16622 / WS 16623). A client registers a surface (`ADD-DEVICE`) either as a simple grid (`KEYS_TOTAL`, `KEYS_PER_ROW`, bitmaps/colours/text flags) or an advanced `LAYOUT_MANIFEST` JSON: `stylePresets` (named render styles; `default` required) + `controls` (id → row, column, preset). Input up: `KEY-PRESS` (pressed true/false), `KEY-ROTATE` (signed direction; magnitude with `ROTARY_AMOUNT` capability). Output down: `KEY-STATE` with `BITMAP` (raw RGB or png/webp data URL), `COLOR`, `TEXTCOLOR`, `TEXT`, `FONT_SIZE`, `LEDS` (encoder rings). `CAPS` capability negotiation; `PING` every 2 s. Subscriptions (`ADD-SUB` by page/row/col) let a client show individual buttons without owning a surface.
- **Web Buttons [doc]**: one page showing every button of every page; settings saved in the URL (bookmark = configuration); not a Surface, so it doesn't follow page changes. Emulator = Stream Deck grid in the browser, several emulators can view different pages.
- **Take-away**: the cleanest example of a **dumb, stateless surface**: client sends press/release/rotate, server sends the complete appearance. The style-preset + control-manifest split is a good model for LumaCore pool cells and key grids; the capability handshake is a good model for client↔server kit versioning (pairs with `CLIENT_PROTOCOL`).

### 1.5 g200kg webaudio-controls (Apache-2.0) — custom elements for knobs/sliders
https://github.com/g200kg/webaudio-controls , docs https://g200kg.github.io/webaudio-controls/docs/index.html , tracking https://g200kg.github.io/webaudio-controls/docs/tracking.html
- **[doc]** Custom elements `<webaudio-knob>`, `<webaudio-slider>`, `<webaudio-switch>`, `<webaudio-param>`, `<webaudio-keyboard>`; configured entirely by HTML attributes (`value`, `min`, `max`, `step`, `width`, `height`, `tracking`, sprite `src`…). Slider `tracking="rel"` (default; drag changes relative; tap without drag changes nothing) vs `tracking="abs"` (value jumps to pointer; fine mode ignored). Shift+drag fine by `step`; wheel (+Shift fine); Ctrl/Cmd+click reset to default; arrow keys when focused; right-click = MIDI learn; multitouch on the keyboard element; works on iOS/Android. Images = film-strip sprites (knob galleries).
- **Take-away**: proves "plain custom elements configured by attributes" is enough for a control kit; the rel/abs naming, tap-without-drag-is-no-op rule, and keyboard/wheel equivalents are worth copying.

### 1.6 NexusUI (MIT)
API: https://nexus-js.github.io/ui/api/ , NIME paper https://nime.org/proc/btaylor2014
- **[doc]** Core: Button, Dial, Number, Position, Slider, Toggle; General: Envelope, Multislider, Piano, RadioButton, Select, Sequencer, TextButton; Mobile: Tilt; Spatial: Pan, Pan2D; Viz: Meter, Oscilloscope, Spectrogram. `new Nexus.Dial('#el', {size, interaction, mode, min, max, step, value})`; `Nexus.Rack` builds many from `nexus-ui="dial"` attributes; events `change`, `click`, `release`; dial/slider `mode: 'relative' | 'absolute'`, dial `interaction: 'radial' | 'vertical' | 'horizontal'`. Theming: `Nexus.colors` {accent, fill, dark, light, mediumDark, mediumLight} + `.colorize()`. SVG rendering. Docs say interfaces are not responsive.
- **Take-away**: the "interaction × mode" matrix for dials (radial / vertical / horizontal × relative / absolute) is a tidy way to describe encoder behaviour; a 6-token palette is enough for controls.

### 1.7 remote-mixer (kryops, MIT) — open web mixer
https://github.com/kryops/remote-mixer
- **[doc]** Node backend with `DeviceController`s (X32/M32 via OSC, Yamaha 01v96 via MIDI, dummy); each device declares `deviceConfig.categories` (key, label, count, meters, `faderProperties`) → the frontend renders tabs of fader strips (fader, on/off, name, meters, colour); responsive, light/dark, mouse or touch, multiple clients at once. **Server declares the shape, the client renders strips** — again a descriptor model. Author: "don't rely on it as your only means of mixing".
- Closed mixer apps for reference only: X32-Edit, Mixing Station (https://apps.apple.com/us/app/mixing-station/id1438352631).

### 1.8 Luminosus (ETCLabs, Eos edition; GPL, Qt/QML) — not maintained
https://github.com/ETCLabs/LuminosusEosEdition , https://build.opensuse.org/package/show/home:edogawa/luminosus
- **[doc]** Touch-first node/"block" canvas (blocks: faders, buttons, cue list, MIDI→OSC, Sound2Light) for Windows/Mac/Android/iOS; cue list display/edit, fader banks with labels; "not maintained anymore". Good inspiration for a patchable canvas, not for a widget kit.

### 1.9 DMXControl 3 (.NET, GPL)
No useful public touch/web widget documentation found this session. DMXControl 3 has a separate GUI client talking to a kernel over network (same split as LumaCore), but its widgets are WPF; skip as a reference.

### 1.10 Closed products — conventions only
- **A reference console [doc]** gestures: Tap; Tap & Hold ≥ 1 s; Zoom (2 fingers); Resize (hold title bar + tap elsewhere with 2nd finger); 1-finger swipe; **2-finger scroll** (so 1-finger drag stays free for controls); 3+-finger scroll of the user-defined area; Drag & Drop = hold then move; **2 Finger Edit** in sheets (tap cell, then tap with 2 fingers; "Precise Edit" variant: hold in cell + tap elsewhere); **encoders: Rotate Style** (hold and draw circles, clockwise = up, **larger circles = finer**) and **Drag Style** (hold, drag outside the encoder, release → a fader pop-up; holding the fader > 1 s locks the value). **Swipe commands**: press-hold a pool object, drag outside it, move to Assign / Edit / Edit Setting / Store / Label / Delete, release → command runs.
- **ETC Eos [sec]**: virtual encoders live in ML Controls; ETC staff say there is no virtual circular encoder because touch lacks precision — they recommend XY grid for coarse + linear encoders (tap/drag) for fine; users note Eos expects circular drags while other apps expect vertical drags (confusion → be explicit) (https://community.etcconnect.com/control_consoles/eos-family-consoles/f/eos-family/58853/how-to-add-encoders-to-etc-eos). Feature request for touch-drag on Encoder pages (https://community.etcconnect.com/control_consoles/eos-family-consoles/i/feature-requests/mouse-touch-driven-interactive-control-on-encoder-pages).
- **ChamSys MagicQ [sec]**: touch mode with large buttons for touch screens; tapping the playback legend above a fader = pressing its select key (https://docs.chamsys.co.uk/magicq/manual/console_layout.html); MagicQ Remote app added multitouch intensity/playback faders (https://apps.apple.com/app/id896639862).
- TouchOSC, Lemur, Onyx, Lightkey: closed; Lemur/TouchOSC popularised per-widget "pad" vs "fader" modes, physics (friction/attraction), and layouts as XML/JSON documents. No further detail gathered.

### 1.11 Cross-project comparison

| | Declaration | Fader drag | Fine | Reset | Long press | Multitouch | Theming |
|---|---|---|---|---|---|---|---|
| Open Stage Control | JSON tree + schema'd props, expressions, scripts | `snap` false=relative / true=absolute; `touchZone` | Ctrl+drag / 2-finger drag ×10 | `doubleTap` | — (modal/menu) | per widget + `traversing` | CSS vars, per-container override |
| webaudio-controls | HTML attributes on custom elements | `tracking` rel/abs | Shift+drag (`step`) | Ctrl/Cmd+click | right-click = MIDI learn | keyboard only | sprites |
| NexusUI | JS constructor options / `nexus-ui` attrs | `mode` relative/absolute; dial `interaction` radial/vert/horiz | — | — | — | yes (claimed) | 6-colour palette |
| Companion | Satellite manifest (style presets + controls) | n/a (buttons, rotary) | rotary amount | — | press/release | per key | server-rendered bitmaps |
| QLC+ | C++ objects / XML | absolute | — | — | — | not in Qt5 desktop | per-widget colours |
| reference console | (closed) | encoder rotate / drag-out fader pop-up | larger circle = finer | — | hold ≥1 s; swipe menu | yes | colour themes |

---

## 2. Web touch engineering (2025-2026)

### 2.1 Input model
- **Pointer Events only** (not mouse + touch pairs). One handler set covers mouse, pen, touch; mouse is `pointerId` 1-ish and constant; each finger gets its own `pointerId`. MDN https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events , capture https://developer.mozilla.org/en-US/docs/Web/API/Element/setPointerCapture .
- **Per-control capture**: in `pointerdown` call `el.setPointerCapture(e.pointerId)`; keep a `Map<pointerId, DragState>`; end the drag on `pointerup`, `pointercancel` **and** `lostpointercapture` (capture also ends when the element is removed — matters when a server update re-renders a fader mid-drag). Capture is per pointer, so two fingers on two faders work independently; touch already has implicit capture but call it explicitly for mouse/pen parity [sec: https://javascript.info/pointer-events].
- `getCoalescedEvents()` for smooth high-rate input and `getPredictedEvents()` for latency hiding on canvases [know] (https://developer.mozilla.org/en-US/docs/Web/API/PointerEvent/getCoalescedEvents). Throttle *sending* (rAF or ~30–60 Hz) independently of input.
- **Don't trust `pointerType` for logic**; use it only for affordances (e.g. bigger hit slop for `touch`).
- Fine-mode triggers that work everywhere: a second pointer on the same control (OSC style), a modifier key, or a "Fine" latch button in the kit; plus encoder "rotate in bigger circles = finer" (reference console).

### 2.2 Browser gesture suppression
- `touch-action` (Baseline since 2019; https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action): `none` on faders/encoders/XY/canvas; `pan-y` on a horizontal fader inside a vertically scrolling list (and vice versa) so the list still scrolls; `manipulation` on the whole app root (keeps pan/pinch, kills double-tap-zoom and its click delay — the "300 ms delay"). Not inherited; the effective value is the intersection along ancestors up to the scroll container; changing it mid-gesture has no effect. Without it, the browser steals the gesture and fires `pointercancel`.
- iOS ignores `user-scalable=no` / `maximum-scale` for accessibility since iOS 10 [know]; rely on `touch-action` and, in WKWebView, native settings. Cancel `gesturestart` (WebKit-only) to stop pinch-zoom where needed [know].
- Rubber-band / pull-to-refresh: `overscroll-behavior: none` on `html, body` and `contain` on inner scrollers. Safari supports it from 16, but WebKit bugs remain: no effect when content doesn't overflow (bug 244232), pull-to-refresh not disabled (bug 275947) [sec: https://bugs.webkit.org/show_bug.cgi?id=244232 , https://bugs.webkit.org/show_bug.cgi?id=275947]. Best fix: make the document itself non-scrolling (`html, body { height: 100%; overflow: hidden }`) and scroll only inside panes; in WKWebView set `scrollView.bounces = false` and `alwaysBounceVertical = false` [sec: https://lightrun.com/answers/ionic-team-capacitor-bug-wkwebview-bouncing-issue-on-ios-16].
- Text selection / callouts during drags: `user-select: none; -webkit-user-select: none; -webkit-touch-callout: none; -webkit-tap-highlight-color: transparent` on controls (re-enable on inputs/terminal). Reported that `-webkit-touch-callout: none` doesn't fully suppress the system menu on iOS 26.1 [sec: https://developer.apple.com/forums/thread/808606] — call `preventDefault()` on `contextmenu` and avoid long-press on images/links in controls.
- **Passive listeners**: `touchstart`/`touchmove`/`wheel` on window/document are passive by default in Chromium/Firefox/Safari, so `preventDefault()` silently fails there; register with `{passive: false}` only where you must prevent, and prefer `touch-action` [know] (https://developer.chrome.com/docs/lighthouse/best-practices/uses-passive-event-listeners).

### 2.3 Long press vs context menu
- **iOS Safari / WKWebView never fire `contextmenu` on long press** (WebKit bug 213953 open since 2020; W3C pointer-events list 2024) [sec: https://bugs.webkit.org/show_bug.cgi?id=213953 , https://lists.w3.org/Archives/Public/public-pointer-events/2024AprJun/0007.html]. Android Chrome does fire it (after ~500 ms) along with `pointercancel` if text/links are involved.
- ⇒ Implement long-press in the kit: timer on `pointerdown` (the reference console uses ≥ 1 s for Tap & Hold; most platforms ~500 ms), cancel if moved > slop (≈ 8–10 px) or released; then *also* handle `contextmenu` for mouse right-click and Android, de-duplicated, and `preventDefault()` it. Give a visual progress ring so hold isn't a hidden gesture.

### 2.4 Capability media queries
- `@media (hover: none)` / `(pointer: coarse)` describe the **primary** input; `any-hover` / `any-pointer: coarse` tell whether *any* input can hover / is coarse (a touchscreen laptop) [know] (https://developer.mozilla.org/en-US/docs/Web/CSS/@media/pointer). Use `any-pointer: coarse` to enlarge targets and never hide functions behind hover. Never branch behaviour on UA strings.

### 2.5 Target sizes
- WCAG 2.2 **2.5.8 Target Size (Minimum), AA: 24×24 CSS px** or 24 px spacing circles [doc: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html]; **2.5.5 (Enhanced), AAA: 44×44** (https://www.w3.org/WAI/WCAG22/Understanding/target-size-enhanced.html).
- Apple HIG: **44×44 pt** minimum hit target (https://developer.apple.com/design/human-interface-guidelines/accessibility) [know]. Material: **48×48 dp** touch target, 8 dp spacing (https://m3.material.io/foundations/designing/structure — "Touch targets") [know].
- Show consoles in practice use bigger: pool cells, executor buttons ~ 60–90 px. Suggest kit tokens: `--hit-min: 44px` (coarse) / `32px` (fine, mouse) with an invisible hit-slop ring on small handles, and a user "touch size" scale.

### 2.6 Safe areas, viewport, on-screen keyboard
- `viewport-fit=cover` + `env(safe-area-inset-*)` for notches/home indicator [know] (https://developer.mozilla.org/en-US/docs/Web/CSS/env).
- Use `dvh`/`svh`/`lvh` instead of `100vh` [know].
- **visualViewport** (all engines): `resize`/`scroll` events give the visible area while the soft keyboard is up — the portable way to keep a focused field or a docked keypad visible [know] (https://developer.mozilla.org/en-US/docs/Web/API/Visual_Viewport_API).
- **VirtualKeyboard API** (`navigator.virtualKeyboard.overlaysContent`, `geometrychange`, `boundingRect`, `env(keyboard-inset-*)`, `virtualkeyboardpolicy="manual"` + `show()/hide()`): experimental, Chromium only, secure contexts only [doc: https://developer.mozilla.org/en-US/docs/Web/API/VirtualKeyboard_API]. LumaCore serves plain HTTP on a LAN → **unavailable in browsers there**; WebView2 loads from localhost (secure context) and could use it.
- Viewport meta `interactive-widget=resizes-content|overlays-content`: Chromium/Android only; WebKit bug 259770 open (all iOS browsers affected) [sec: https://bugs.webkit.org/show_bug.cgi?id=259770 , https://htmhell.dev/adventcalendar/2024/4/].
- **`inputmode`** (`numeric`, `decimal`, `tel`, `search`, `none`) + `enterkeyhint` pick the OS keypad [know] (https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/inputmode). For LumaCore's own calculator/keypad and virtual keyboard: use `inputmode="none"` on the field so the OS keyboard stays down but the caret/selection still work; offer a toggle to use the system keyboard.

### 2.7 Webview quirks
- **WKWebView (macOS/iOS)**: no `contextmenu` on long press (above); rubber-band (native `bounces`); pinch-zoom unless `touch-action`/`gesturestart` cancel; link previews/drag-image on long press (`-webkit-user-drag: none`, `allowsLinkPreview = false`); text selection via `user-select`; magnification `allowsMagnification` on macOS. Double-tap delay is gone when the page is `width=device-width` and/or `touch-action: manipulation` [know].
- **WebView2 (Windows touch)**: `CoreWebView2Settings.IsPinchZoomEnabled = false`, `IsSwipeNavigationEnabled = false` (stops back/forward swipe and pull-to-refresh); both apply after next navigation [doc: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2settings.ispinchzoomenabled , https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings]. Also `AreDefaultContextMenusEnabled = false`, `IsZoomControlEnabled = false` [know]. Windows touch keyboard auto-shows on focus of editable fields in tablet mode.
- **WebKitGTK (Linux)**: supports touch + pointer events, but on X11 touch screens often arrive as emulated mouse unless the GTK app enables touch events; Wayland is better; kinetic scrolling and pinch handled by WebKit settings (`enable-smooth-scrolling`) [know, not verified this session — test on target hardware].
- High-DPI canvas: size the backing store to `clientWidth × devicePixelRatio`, scale the context, redraw on `ResizeObserver` and on `matchMedia('(resolution: Xdppx)')` change (moving between monitors) [know] (https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio). Prefer DOM/SVG+CSS for controls (crisp, themeable, accessible) and canvas only for meters/DMX grids/visualizers.

---

## 3. Libraries worth vendoring or learning from

| Library | What | Touch | Deps / size | Verdict |
|---|---|---|---|---|
| **dockview / dockview-core** (MIT) https://dockview.dev | Tabs + split + floating + popout docking; core is vanilla TS, no runtime deps | Site claims "first-class touch & pen: pointer-events DnD backend, long-press to start drag so taps and scroll still work" [doc]; older HN comment said touch not supported (older versions) [sec: https://news.ycombinator.com/threads?id=mathuo] | moderate (~100 kB min) | Best candidate if replacing the split tree; otherwise **learn** its long-press-to-drag pattern. Verify in the current version on an iPad first. |
| **golden-layout 2.x** (MIT) https://github.com/golden-layout/golden-layout | Classic docking, popouts | README lists touch support; officially tested Firefox/Chrome only [sec: https://cdn.jsdelivr.net/npm/golden-layout@2.6.0/README.md]; 3.0 dev branch unstable | moderate | Skip. |
| **Lumino** (BSD, JupyterLab) https://github.com/jupyterlab/lumino | DockPanel, SplitPanel, DataGrid, CommandRegistry, keymaps | No touch evidence found | large, widget-class framework | Skip as dependency; its **CommandRegistry** (commands with label/icon/isEnabled/isToggled, keybindings, palette) mirrors LumaCore's registry approach — learn. |
| **interact.js** (MIT) https://interactjs.io | drag/resize/gesture with inertia, snapping, modifiers; multi-touch simultaneous interactions | yes | ~ 30 kB gz; 1.10.x; maintenance cadence unclear [sec] | Don't vendor; Pointer Events + capture is enough for faders. Learn snapping/modifier design. |
| **@use-gesture** (MIT) https://use-gesture.netlify.app | Gesture state machine (drag/pinch/wheel/move) with `@use-gesture/vanilla` | yes | small | Learn the **state model**: `first/last/active/movement/offset/velocity/direction/tap/swipe/cancel`, thresholds, `filterTaps`, `axis` lock, `bounds`, `rubberband` [know]. |
| **hammer.js** | Old gesture lib | — | — | Unmaintained since ~2016 ([sec: https://github.com/hammerjs/hammer.js/issues/1278]); fork @egjs/hammerjs exists. Don't use. |
| **TanStack Virtual (@tanstack/virtual-core)** (MIT) https://tanstack.com/virtual | Headless virtualizer: vertical, horizontal, **grid**, dynamic sizes, sticky items; ~10–15 kB; vanilla usable via core [sec] | n/a (uses native scroll → native touch momentum) | small, zero deps | Vendor if pool/sheet windows need > a few thousand cells; otherwise a 100-line fixed-row virtualizer. Native scroll is the key benefit (momentum on touch for free). |
| **Lit 3** (BSD) https://lit.dev | Reactive custom elements, `html` tagged templates; ~5 kB gz [know] | n/a | small, single file ESM vendorable | Optional. Plain custom elements suffice for a kit whose state comes from the server; Lit only helps with templating ergonomics. |
| **Design tokens (DTCG 2025.10)** https://www.designtokens.org/tr/2025.10/ | First stable spec (Oct 28, 2025): JSON tokens `$type`/`$value`, aliases `{group.token}`, colour module (P3, OKLCH), **Resolver module** for themes/modes [doc: https://www.w3.org/community/design-tokens/2025/10/28/design-tokens-specification-reaches-first-stable-version/] | — | spec only | Keep tokens as CSS custom properties at runtime; if tokens are ever authored as files, use DTCG JSON and generate the CSS (in Rust, at build time — no JS build step). |

Also: `xterm.js` already vendored (has its own touch issues — needs `touch-action` and a custom scroll for touch). `modern-screenshot` vendored.

---

## 4. Recommendation — architecture for a declarative element kit

### 4.1 Principles mapped to LumaCore rules
1. **Server owns state; the kit owns only the gesture in flight.** Every control is *decoupled* (Open Stage Control's term): it renders the value the server last published, plus a local "ghost" value while a finger is down. On release (or for continuous controls, throttled during the drag) it emits a **command line**, never a value write. When the server's new value arrives, the ghost is dropped. No client-only state, no per-control endpoints (already a LumaCore rule for pool windows).
2. **Descriptors come from the registries.** The server already knows each writable property (`TreeDef::write`: name, type, range, rights) and each `ObjectDef.pool` (cells, tap actions, detail). Extend that to an **element descriptor** = `{ kind, id (GUID/path), label, value, range/steps/enum, unit, rights/enabled, gestures: { tap, hold, doubleTap, release, swipe[] } → command templates }`. Settings editors are then generated: a property's type picks the element (bool → toggle, enum → segmented/swipe menu, number with range → fader/encoder + keypad, text → field + virtual keyboard, GUID link → picker list). One schema → renderer, editor, docs (like `gen-docs`), and the sweep can enumerate them.
3. **Gestures are named, not coded per widget.** The kit implements a small fixed gesture set once (press, release, tap, double-tap, hold (≥ 500 ms, with progress ring), drag-relative, drag-absolute, fine (second finger / modifier / Fine latch), swipe-out menu (console style), two-finger scroll, traversing paint across siblings). Descriptors bind gestures to command templates (`Go Sequence {id}`, `Set {path} {value}`, `Edit {id}`), the same way UiKeys bind Press/Release/Hold/ReleaseAfterHold/DoublePress to command lines (§17) — reuse that vocabulary so a key and a touch button are the same thing.
4. **Wire**: the existing WS feeds (`/ws/pools`, `/ws/events`…) carry descriptor trees + value patches keyed by id; the client sends command lines (+ an optional `gesture_id` for coalescing a drag into one undo step, so a fader drag files one Oops step, not 200). Version the descriptor schema under `CLIENT_PROTOCOL`.

### 4.2 Client implementation
- **Plain custom elements** (`customElements.define('luma-fader', …)`), light DOM or shadow DOM with `::part` + CSS custom properties for theming. No framework, no build step; one vendored file `vendor/luma-kit.js` (or a `kit/` folder of ES modules loaded by `<script type="module">`, which works in all three webviews). Lit optional, not needed.
- Elements take one `descriptor` property (object) and a `value` property; attributes only for static layout hints. A tiny `render(container, descriptorTree)` keyed by id updates in place (never re-create an element under an active pointer — else `lostpointercapture` kills the drag).
- **One pointer core** (`kit/pointer.js`, ~200 lines): pointer map by `pointerId`, explicit capture, slop thresholds, hold timer, double-tap window, fine detection (2nd pointer on same element), velocity, coalesced events, cancel on `pointercancel`/`lostpointercapture`, `contextmenu` merge. Every element uses it, so behaviour is uniform and testable in Playwright (which can synthesise multi-touch via CDP `Input.dispatchTouchEvent`).
- **Element set v1**: `luma-button` (modes: momentary/toggle/tap/flash, confirm via doubleTap or hold for dangerous ones), `luma-fader` (relative default, absolute opt-in, `touchZone`, `origin` for bipolar, steps, spring, double-tap → default, value bubble while dragging), `luma-encoder` (vertical drag default + optional rotate style; emits deltas; console-style drag-out → pop-up fader; tap → keypad), `luma-xy`, `luma-list` (virtualized, native scroll, swipe-row actions), `luma-grid` (pool cells, traversing paint), `luma-sheet` (rows×cols, two-finger edit as on consoles), `luma-popup`/`luma-dialog` (`<dialog>` element + focus trap; sheets slide from the bottom on `pointer: coarse`), `luma-keypad`/`luma-calculator` (writes into the command line, `inputmode="none"` fields), `luma-keyboard` (virtual keyboard, layout from server), `luma-swipe-menu` (press-hold, drag out, release on item = run), `luma-settings` (generated from property descriptors), `luma-segmented`, `luma-color` (later).
- **Theming**: CSS custom properties on `:root` (`--luma-bg`, `--luma-surface`, `--luma-accent`, `--luma-text`, `--luma-hit-min`, `--luma-radius`, `--luma-gap`, density scale) overridable per window/container (Open Stage Control pattern); light/dark + a "console dark" theme; DTCG JSON as the optional source format.
- **Global CSS baseline**: `html,body{height:100%;overflow:hidden;overscroll-behavior:none;touch-action:manipulation}`; controls `touch-action:none; user-select:none; -webkit-touch-callout:none; -webkit-tap-highlight-color:transparent`; scroll panes `overflow:auto; overscroll-behavior:contain; touch-action:pan-x pan-y`; `@media (any-pointer: coarse)` bumps `--luma-hit-min` to 44–48 px; `env(safe-area-inset-*)` padding on the outer frame.
- **Native host settings**: WebView2 `IsPinchZoomEnabled=false`, `IsSwipeNavigationEnabled=false`, default context menus off; WKWebView `bounces=false`, `allowsLinkPreview=false`, magnification off; WebKitGTK touch events on.

### 4.3 Plugins
Plugins should not ship DOM code first; they publish descriptors (same schema, via registry objects) and get the kit's rendering, gestures, rights and undo for free. A later escape hatch: a plugin may register a custom element under a `luma-x-<plugin>-*` prefix that must use the pointer core and emit only command lines.

### 4.4 Why not the alternatives
- **Framework SPA (React/Vue/Svelte)**: needs a build step and a large runtime; contradicts "plain JS, vendored only".
- **Canvas-only kit (TouchOSC/Lemur style)**: great perf and pixel control, but loses accessibility, text input, CSS theming, screenshots via DOM, and Playwright selectors. Use canvas inside elements only for meters/DMX/visualizers.
- **Client-to-client sync (Open Stage Control)**: violates "server owns state" and would be a second sync path.

---

## Sources (main)
- Open Stage Control docs: https://openstagecontrol.ammd.net/docs/widgets/properties-reference/ · https://openstagecontrol.ammd.net/docs/widgets/general-mechanics/ · https://openstagecontrol.ammd.net/docs/customization/themes/ · forum https://openstagecontrol.discourse.group/t/traversing-gestures-on-by-default-or-available-only-for-some-widgets/437
- QLC+: https://docs.qlcplus.org/v5/advanced/web-interface · https://forum.qlcplus.org/viewtopic.php?p=33151
- Chataigne: https://github.com/benkuper/chataigne-web-dashboard · https://github.com/benkuper/Chataigne
- Companion: https://companion.free/for-developers/Satellite-API · https://companion.free/user-guide/sidebar/web_buttons
- webaudio-controls: https://github.com/g200kg/webaudio-controls · https://g200kg.github.io/webaudio-controls/docs/tracking.html
- NexusUI: https://nexus-js.github.io/ui/api/
- remote-mixer: https://github.com/kryops/remote-mixer
- Luminosus: https://github.com/ETCLabs/LuminosusEosEdition
- ETC: https://community.etcconnect.com/control_consoles/eos-family-consoles/f/eos-family/58853/how-to-add-encoders-to-etc-eos
- MagicQ: https://docs.chamsys.co.uk/magicq/manual/console_layout.html
- MDN: touch-action https://developer.mozilla.org/en-US/docs/Web/CSS/touch-action · VirtualKeyboard https://developer.mozilla.org/en-US/docs/Web/API/VirtualKeyboard_API · Pointer events https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events
- WCAG 2.5.8: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- WebKit bugs: 213953 (contextmenu), 259770 (interactive-widget), 244232 / 275947 (overscroll)
- WebView2 settings: https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2settings.ispinchzoomenabled
- dockview: https://dockview.dev · golden-layout: https://cdn.jsdelivr.net/npm/golden-layout@2.6.0/README.md · TanStack Virtual: https://tanstack.com/virtual/latest · DTCG: https://www.designtokens.org/tr/2025.10/
