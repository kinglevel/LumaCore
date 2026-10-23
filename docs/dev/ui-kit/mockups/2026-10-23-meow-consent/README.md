# The Meow consent (2026-10-23)

Round 1, asked by the user before the repo goes public: every login asks the user to accept the
"Meow License", and says plainly that LumaCore is vibe-coded software with no warranty. Design
only until the user decided.

| File | What it asks |
|---|---|
| `01-consent.html` | A working mock of the login surface → the consent → the desk, in the kit's dark theme: the dialog (title, the good-vibes text, the vibe-coded warning in red above the buttons, the fine print), two button layouts (A one gold "I consent — let the meow in" + a quiet "Not me"; B Decline beside Consent), Guest with `?autologin=now`, the phone sheet and the terminal's text dialog. Five questions. |

Decided in the round's questions (all as recommended):

- the text as drafted;
- buttons A: one gold button and a quiet "Not me" back to the user picker;
- every login on every path asks (the page, the app, `Login` on any terminal, autologin and
  `?autologin=now`, a user switch); unlocking a DeskLock does not;
- each consent writes one line to the station log, nothing goes into the show file;
- the repo's code license stays as it is: the Meow License is the in-app consent only.

Approved 2026-10-23 ("Meow consent A is good"; record 0290). Built without a wire change: the
dialog reuses the station's `Dialog` (an alert detail, the note, topic `consent`), so the mock's
"one dialog kind, protocol bump" note did not apply.
