# 12d research: editing another station's own (identity) settings — direct, via the master, or both?

Research note, 2026-10-13. Question: `Set Station "Tower" Priority High` run on FOH — should the line
go **straight to Tower** (what 12d part 4 built), **through the session master** (what §30 "12d
hardening" said), or a hybrid? No code changed.

## Where we are (facts from the code)

- **Wiring** (network → NIC, services per NIC) is a row in `ShowData.Stations` (`core::stations`):
  replicated, master-written, Oops-able, pre-configurable; each station applies only its own row
  (`apply_own_row`) and keeps a last-applied copy.
- **Identity** (Name, LumaSession, Location, Priority, Online, Invite, HoldOutput, OutputWhenAlone,
  ThemeOverride, MulticastBase, StartupShow; CID, tokens) is station-local (`station_settings.json`).
- Remote identity edit = `settings_props::remote_station_set`: finds the target among
  `luma_sessions.members`, else `discovery` (a standalone neighbour too), and queues
  `SessionRequest::Send { RemoteRequest::Command { user, cmd } }` to its **UDP command port**. The
  reply is "sent, it answers here". Not found → E-CMD-010 ("isn't reachable … its wiring row
  changes here").
- On the target, `net::handler::execute_remote_command` logs to its sysmon ("command request from
  'x'"), resolves the user against **its own** show (`rights_of(user)`; unknown → refused) and
  runs the line through its own dispatch, which checks rights. No password (IP trust, as decided).
- Priority feeds election (`lumasession::election::elect`: priority → results applied → online
  time → station id) and rides in the discovery beacon, so every station sees the actual value
  within a beacon period.

## How others do it

| Project | Where the node setting lives | Edit is sent to | Node offline | Audit / undo | Conflicts |
|---|---|---|---|---|---|
| **A reference console** | Identity (host name, session, location, master priority, interface): own station only. Connector/protocol wiring: a session table, saved in the show | Identity: **only on that station** (Network menu edits nothing on another station). Wiring: any station, through the session | Wiring rows kept and editable ("pre-configured"), applied when the device shows up; identity can't be touched | Show Oops for the table; identity none | Session table wins for a station it already describes, else the joining station's own row |
| **Kubernetes** | kubelet config = file on the node; Node labels/taints = API object | kubelet config: on the node (dynamic kubelet config was removed in 1.24). Labels: **the API server** (`kubectl label node`); kubelet `--node-labels` only at registration; NodeRestriction stops a kubelet from setting protected labels | Labels stored on the API server; the node object keeps them while NotReady | API audit log; no undo | Last write wins with `resourceVersion` optimistic concurrency; spec (desired) vs status (reported) |
| **Elasticsearch** | Static node settings in `elasticsearch.yml` (paths, `network.host`, `cluster.name`); dynamic cluster settings in cluster state | Static: only on the stopped node. Dynamic: any node → master, persisted in cluster state | Static: n/a. Cluster settings reach a node when it rejoins | Cluster-state history; no undo | Precedence transient > persistent > yml > default |
| **Ceph** | Central config DB on the monitors (`ceph config set osd.3 …`) plus local `ceph.conf` | **The monitors**, not the daemon; `ceph tell` = direct, ephemeral runtime override ("revert … when the daemon restarts") | Stored centrally; a daemon reads it at start | `ceph config log` / `reset` (mon keeps a change history; not on the page fetched) | Fixed precedence: default < mon DB < local file < env < CLI < runtime override — local wins |
| **CockroachDB** | Cluster settings (system table) vs `cockroach start` flags | Cluster settings: SQL to any node, stored in a replicated table. Node flags: only by restarting that node | Cluster settings apply when the node joins | Event log for setting changes | One replicated table, last write wins |
| **Proxmox VE** | `/etc/pve/nodes/<name>/` in pmxcfs (replicated cluster FS) | Any node writes any node's directory; the cluster FS replicates | A node that is down reads the change when back; **no quorum → read-only everywhere** | Task log; no undo | Quorum serialises writes |
| **Consul / Nomad** | Consul agent config files (incl. `node_meta`); Nomad dynamic node meta | Consul: **on the agent** (file + `consul reload`). Nomad `node meta apply`: the **client agent's** API (`/v1/client/metadata`), which then tells the servers ("may take up to 10 seconds") | Consul/Nomad: agent unreachable = no change | Server-side events only | Agent is the source; catalog mirrors it (anti-entropy) |
| **Syncthing** | Each device's own `config.xml`; nothing is sent to change a peer | Nobody can change another device; introducers only suggest devices/folders | n/a | n/a | Each device decides |
| **Home Assistant ZHA / Z-Wave JS** | Settings on the device itself | **Through the coordinator** (HA never talks to the device directly) | Sleeping/battery node: the command is **queued until wake-up** (volatile; lost on restart, community reports disagree) | HA logbook; no undo | Device reports its value back; a stale report can overwrite the set point |
| **UniFi controller** | Desired config on the controller | The controller provisions the device | Offline devices get the **full config re-sent when they reconnect** ("Provisioning") | Controller event log | Controller is authority; local edits are overwritten |
| **Ansible / Salt** | Ansible push over SSH; Salt states / `ansible-pull` | Push: **directly** to each host. Pull: the node fetches desired state | Push: host "unreachable", run fails for it. Pull/state: applied on next check-in | Run output; no undo (re-run with old values) | Last run wins |
| **ETC Eos / Net3** | Console settings local (ECU/shell); gateway config in the gateway, edited in GCE/Concert | GCE/Concert from a console **directly to the device**: "Send Device Configuration" | Device must be online to send | None | Last sent wins |

Patterns:

1. **"Who am I on the network" is edited only on, or directly at, the node** (the reference console,
   Consul agent, Nomad client meta, ES static settings, Syncthing, Eos/GCE, Ansible push). Nobody
   routes identity through a leader whose job is the *shared* data.
2. **Shared "desired" config lives in the replicated store** and the node reconciles it
   (the console's session table, K8s labels, ES cluster settings, Ceph mon DB, Proxmox pmxcfs, UniFi,
   Salt). That's how they handle offline nodes: the desired value waits in the store.
3. **Where both exist, a fixed precedence rule settles it** (Ceph: local file beats the mon DB;
   ES: cluster settings beat yml; the console: the show's row beats the joining station's own row).
4. **Routing through a leader only helps when the leader stores the value.** A relay that doesn't
   store anything (ZHA/Z-Wave) adds a queue that is lost on restart and reports that may be stale.
5. Nobody offers undo for node-local settings. Audit comes from the node's own log or the store's
   history.

## Options for LumaCore

Rules to check against: (R1) one change path per piece of state; (R2) only the master writes the
show; (R3) station-local is never replicated; (R4) light security (open UDP, IP trust, but rights
gate + masking); plus election safety (priority) and split brain.

### A. Direct to the station (built today), hardened

The target's own dispatch is the one change path for its own state, so a remote line is just
another transport into it, like the shell or HTTP.

- **+** R1/R3 hold exactly: the owner of the state writes it, checks rights against its users,
  logs it, saves it. No relay, no copy.
- **+** Works without a master: standalone neighbours (discovery), an IdleMaster, the minutes
  after a master loss. **This matters most for Priority**: fixing election priority while the
  session is leaderless or split is exactly when you need it.
- **+** Matches the reference console (identity = own station) plus a remote door, and Consul/Nomad/Eos.
- **−** Offline station: refused (no pre-configuring identity).
- **−** Audit is split: sysmon on the target and the reply on the sender; nothing in the session
  journal. Oops on the sender doesn't undo it.
- **−** UDP request/reply: a lost packet needs a timeout and a clear "no answer".
- Split brain: harmless for the show (identity isn't show data, no epoch needed); a priority
  change made during a split simply takes part in the next election, which is what was asked.

### B. Through the master (what §30 said)

The sender forwards to the master over the stream; the master forwards to the target.

- **+** One place in the session sees every edit (could be journalled).
- **−** The master **doesn't own or store** the value: it can't validate it (the target's rights
  and ranges decide), can't undo it, can't hold it for an offline target without inventing a
  queue (pattern 4: a volatile queue that's lost on restart).
- **−** No master → no edit. Standalone neighbours and IdleMasters can't be reached at all.
  Priority can't be fixed during the election it decides.
- **−** A second path for station-local writes next to the target's own: breaks R1 in spirit and
  adds a stream message (`SESSION_PROTOCOL` bump) for no data gain.
- Verdict: worst of both; the only gain (a session journal entry) is cheaper as A + a note.

### C. Desired identity in the show's station row (desired-vs-reported, UniFi / K8s style)

Add `desired_priority`, `desired_location`, … (`Option<…>`) to `StationWiring`; each station
applies its own row like the wiring and reports the actual value in its beacon/`StationView`;
the grid shows desired vs reported.

- **+** One change path (R1/R2): Oops, journal, replication, the master serialises conflicts.
- **+** Pre-configure offline stations; a station that comes back takes the value.
- **−** **Breaks R3 and the decision** that identity never travels in the show, and repeats a
  known console bug class: loading a show (or another venue's show, or an Import of the Station
  table) would rename stations, move them to another LumaSession or **change election priority as
  a side effect of a show load**. Name and LumaSession can't be desired at all (Name is how the
  row is matched; LumaSession decides which master's show you get: circular).
- **−** Split brain: two sides' shows carry different desired priorities and the 3-way merge
  decides who leads next. Election data would depend on show data.
- **−** A standalone neighbour in another show can't be reached through our table.

### D. A + seed values on rows made in advance (narrow hybrid)

A, plus: a row made with `Store Station "Tower"` (not yet claimed, `station_id` empty) may carry
**seed** values for a few safe identity fields (Location, Priority, OutputWhenAlone). They apply
**once**, when the station claims that row (`ensure_own_row` takes the station id), and then are
cleared; from then on the station's own setting is the truth and edits are direct (A).

- **+** Covers "prepare the rig before the truck arrives" without making the show the owner of
  identity: a seed is applied once, a show load never overwrites a claimed station.
- **+** Stays inside the existing row-claim flow; Oops/journal cover the seed itself (it's show
  data until claimed).
- **−** One more concept (seed vs desired) to explain; still none for Name / LumaSession / Online.
- **−** Small: a row claimed by name by the wrong box gets its seed (same risk as its wiring today).

## Recommendation: A now (ratify the deviation), D later only if asked

Change §30 "12d hardening": identity of another station is edited **directly on that station**
(its command port, its dispatch, its rights gate), never through the master and never in the show.
The master owns shared data; a station owns itself (the console split, pattern 1). Harden A:

1. **Ack and reported value.** The sender waits for the target's reply (timeout ~1 s, 2 retries on
   the same request id; idempotent because `Set` is absolute). Reply line:
   `Station "Tower": Priority Normal → High` or `Station "Tower" didn't answer [E-CMD-010]`.
   The grid cell shows a pending mark until the beacon / member list reports the new value
   (reported state, no stored desired state); refused → the target's coded error, as it came
   (`errors::reply`).
2. **Rights:** checked **only on the target** against its own users (as built; in a LumaSession
   that's the replicated user table, so it is the same answer everywhere; a standalone neighbour
   in another show checks its own users, unknown user → refused). The sender does no check of its
   own (one rights gate). Declare levels per identity prop: Location / ThemeOverride = Setup;
   Priority / OutputWhenAlone / HoldOutput / Invite = Setup; Name / LumaSession / Online /
   MulticastBase = Admin. Station asks (Online Off, HoldOutput On, priority `Never` on the
   master) are asked on the **sender** before sending, then sent with `/NoConfirmation`.
3. **Audit:** the target logs it (sysmon, masked line, user, sender station, old → new); the
   sender logs the reply. Optional: a member that changes its own Priority/Location reports it in
   its session runtime so the master's `List Health`/sysmon shows "Tower: Priority High (by
   Anna@FOH)". No session-journal entry (the journal is for show subtrees).
4. **Oops:** not undone by the sender's Oops (like running playback and modes, §29 "11c": station
   identity is outside the change path). The reply carries the old value so the inverse line is
   one tap (the reply's "Undo" button = `Set Station "Tower" Priority Normal`, a normal command).
5. **Conflicts:** the target's state thread serialises; last write wins; every grid shows the
   reported value from the beacon, so a lost race is visible within one beacon.
6. **Offline:** stays refused with the hint to the wiring row (built). If pre-configuring
   identity is wanted later → D (seed on unclaimed rows), never C.
7. **Election safety:** a Priority change on the **current master** or that would make a member
   outrank it doesn't trigger a takeover by itself (election runs only on master loss, §21 6c);
   the reply says so ("takes effect at the next election; Claim LumaSession hands over now").
8. Wire: `RemoteRequest::Command` already carries `user`; add the sender station name for the log
   (or take it from discovery by IP) — no `SESSION_PROTOCOL` change needed if the name is taken
   from discovery.

### How it looks

```
Set Station "Tower" Priority High
  → FOH: ask? (only for asks) → UDP Command {user:"Anna", cmd:"Set Station Priority High"} → Tower
  → Tower: rights gate (Setup) → set → save station_settings.json → sysmon "Anna@FOH: Priority Normal → High"
  → reply → FOH: "Station "Tower": Priority Normal → High"   (grid cell: pending → reported via beacon)

Set Station "FOH" + "Tower" + "Dimmer" Location "Stage left"
  → FOH changes here; Tower sent; Dimmer offline → one reply, one line per station:
    FOH: done · Tower: done · Dimmer: didn't answer / isn't reachable [E-CMD-010]

Set Station "Tower" Network "Main" Interface en1      (wiring: unchanged — show row, via master, Oops-able)
Store Station "Tower" /Priority High /Location "Truss"  (D, later: seeds applied once when Tower claims the row)
```

## Sources

- Kubernetes kubelet config file: https://kubernetes.io/docs/tasks/administer-cluster/kubelet-config-file/ · kubelet flags (`--node-labels`): https://kubernetes.io/docs/reference/command-line-tools-reference/kubelet/ · NodeRestriction: https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/#noderestriction
- Elasticsearch settings and precedence: https://www.elastic.co/guide/en/elasticsearch/reference/current/settings.html
- Ceph configuration sources and precedence: https://docs.ceph.com/en/latest/rados/configuration/ceph-conf/
- CockroachDB cluster settings vs start flags: https://docs.cockroachlabs.com/docs/stable/cluster-settings
- Proxmox pmxcfs (`nodes/<name>/`, read-only without quorum): https://pve.proxmox.com/wiki/Proxmox_Cluster_File_System_(pmxcfs)
- Consul agent configuration (local files, reload): https://developer.hashicorp.com/consul/docs/agent/config · Nomad `node meta apply`: https://developer.hashicorp.com/nomad/docs/commands/node/meta/apply
- Syncthing configuration (per device, introducer): https://docs.syncthing.net/users/config.html
- Z-Wave sleeping nodes (queued until wake-up): https://docs.silabs.com/z-wave/1.0.0/z-wave-training-docs/gateway-session4-sleeping-nodes · https://pyzwave.readthedocs.io/en/stable/sleeping.html · ZHA: https://www.home-assistant.io/integrations/zha/
- UniFi re-provisioning on reconnect (community, not vendor docs): https://forums.lawrencesystems.com/t/lets-clarify-few-things-related-to-unifi-products/6669
- Ansible pull: https://docs.ansible.com/ansible/latest/cli/ansible-pull.html · Salt states: https://docs.saltproject.io/en/latest/topics/tutorials/states_pt1.html
- ETC GCE / Net3 Concert ("Send Device Configuration"): https://support.etcconnect.com/ETC/Networking/Net3_Concert/Setting_Manual_Gateway_IP_Address_in_Net3_Concert · https://www.etcconnect.com/Products/Legacy/Networking/Net3-Gateway-Configuration-Editor/Features.aspx
- Not verified (from memory, pages not fetched in full): Ceph `config log`/`reset`, CockroachDB event log for settings, K8s dynamic kubelet config removal in 1.24, Chamsys (no source found; left out).
