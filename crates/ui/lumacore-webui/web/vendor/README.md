# Vendored frontend libraries

The UI loads nothing from the internet — rigs run on isolated networks
(decisions §29 "11d"). Each file is copied unchanged from its npm package
(`npm pack`, no install scripts).

| File | Package | License |
|---|---|---|
| `xterm.js`, `xterm.css` | `xterm@5.3.0` (`lib/xterm.js`, `css/xterm.css`) | MIT (`xterm.LICENSE`) |
| `xterm-addon-fit.js` | `xterm-addon-fit@0.8.0` (`lib/xterm-addon-fit.js`) | MIT (`xterm-addon-fit.LICENSE`) |
| `modern-screenshot.js` | `modern-screenshot@4.7.0` (`dist/index.js`, UMD → `window.modernScreenshot`) | MIT (`modern-screenshot.LICENSE`) |
| `fonts/source-sans-3-latin-{400,500,600}-normal.woff2` | `@fontsource/source-sans-3@5.3.1` (`files/`) — the UI font (decisions §30) | OFL 1.1 (`fonts/source-sans-3.LICENSE`) |
| `fonts/jetbrains-mono-latin-{400,600}-normal.woff2` | `@fontsource/jetbrains-mono@5.3.0` (`files/`) — command line, terminals | OFL 1.1 (`fonts/jetbrains-mono.LICENSE`) |
| `lucide/sprite.svg` | `lucide-static@1.55.0` (`icons/<name>.svg`), only the names in `lucide/icons.txt`, plus LumaCore's own glyphs (`kit/glyphs.svg`); built by `node scripts/unix/lucide-sprite.mjs <package dir>` | ISC (`lucide/lucide.LICENSE`) |

To update one: `npm pack <package>@<version>` into an empty folder, unpack,
copy the same file over, and change the version here. A new icon: add its
Lucide name to `lucide/icons.txt` and rebuild the sprite (the tidy test
`frontend_icons_come_from_the_kit` refuses emoji / symbol glyphs as icons).
