# The top bar (2026-10-22)

Round 1, asked by the user: take `inspiration/A1 S1.png` (a thin strip of facts across the top),
move the left sidebar's Highlight, Solo, Blind, Freeze, Oops and Meow up into it as minimal
indicators, and give the green LumaCore dot a better design. Design only: nothing is built until
the user decides.

| File | Artifact | What it asks |
|---|---|---|
| `01-top-bar.html` | https://claude.ai/artifact/9VaWNxzi4CEnWRazzK3hCm | Today vs A1 S1 / A1 Whole; three layouts (A the ShowInfo strip, B modes first, C quiet until on), three indicator styles (words, pills, dots), the sidebar after the move, what each indicator does; interactive (modes, unsaved, style, width). Five questions. |
| `02-status-mark.html` | https://claude.ai/artifact/CAHKcRv3J9k6JZ4T3KQcAQ | The mark that replaces the dot: ring, chip or signal, each in seven states (standalone, master, member, busy, output off, reconnecting, lost), the tap pop-up that replaces the bottom ONLINE key. Four questions. |

Found while looking: the top bar's right side reads `Admin · undefined` (`index.html` sets
`${s.user} · ${s.name}` from a status that has no `name`), and the dot and the command line's
ONLINE key both only say the page's socket is open.
