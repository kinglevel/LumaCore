# The view bar and the View pool (2026-10-22)

Round 1, asked by the user: a stored view button doesn't show in the right sidebar; a View pool
window to see a profile's views; assigning views — later macros, sequences, any object — to view
buttons. LumaCore's code was read first. Design only: nothing is built until the user decides.

| File | Artifact | What it asks |
|---|---|---|
| `01-view-bar.html` | https://claude.ai/artifact/2sFkEKGPDijk9RkrWGxoEL | The bug (the sidebar compares view numbers with the buttons' GUIDs; `/api/views` finds the profile by the login's name), dedicated pool windows (Views, Groups, Macros, Sequences — one window type each, asked by the user) and the view bar (drag, Assign, hold → Assign…, Store on an empty button, the type colours), eight questions. |

The bug, for the build: `index.html` `viewForSlot` reads `viewData.buttons[n]` (a GUID since
step 6d) and matches it against `v.no`; `ConsoleBridge::user_views` finds the profile by name
instead of `ShowData::profile_for_user`. The bar is a `fetch('/api/views')`, not a feed.
