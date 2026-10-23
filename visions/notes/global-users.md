# Global users (idea, 2026-10-15)

Not decided — a backbone note from the notification hardening (decisions §33 3c3, N1–N7). Pick it
up by hardening it into a § first.

**The idea.** Today a user lives in one show (`ShowData.Users`, a GUID each) and reaches others
only inside one LumaSession. Later a LumaCore server on the internet could let a user be part of
Teams that span shows, sessions and places, so people elsewhere can send them notifications and
chat.

**What today's design keeps open for it**

- **Identity by GUID.** A user, a Team and a station are already named by GUID, never by name or
  number, so a global identity can map onto them later.
- **Audiences are objects, not places.** A notification addresses a User, a Team, a Station or the
  LumaSession. A wider audience (another session, a server's Team) is one more audience kind;
  messages don't change.
- **Teams hold members by GUID.** A member from elsewhere can be a GUID the show doesn't own.
- **Chat hooks into notifications** (N5): whatever carries chat across sessions raises
  notifications the same way.

**Open questions for that hardening**

- An account outside the show: does a show's User link to it, or is it the same GUID?
- Trust: rigs are isolated, open networks by decision (§19). An internet server changes that and
  needs its own security decision.
- Addresses: how a remote user or Team is written (`User "LD" At Server "x"`?).
- What travels: notifications only, chat, presence?
