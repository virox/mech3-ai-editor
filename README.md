# Mech Reader Studio

A Windows-first, lossless editor for MechWarrior 3 Reader archives. It opens a
game folder as a layered workspace and combines the global, campaign, and
mission Reader data without flattening archive ownership.

Implemented workspace modules:

- **AI Vehicles** — recovered `aiv.zrd` fields, VList conditions/actions, and
  source-key labels. Packed `record[3]` team memberships are edited as Team-ID
  lists; the legacy Friend/Foe/Other controls are shown separately as a
  player-relative preview so they cannot be mistaken for stored enum values.
  The Team panel also exposes the optional `lancemate` override as Auto, On, or
  Off; Auto uses the game's native missing-tag default.
- **Valve Logic** — ordered `valves.zrd` event/action pairs, including duplicate
  event names and a raw fallback for unknown action types.
- **Behavior Graph** — a typed drag-and-drop view of producers, exact named
  integer channels, aggregates/delays, and consumers across AIV, targets,
  valves, networks, objectives, and multiplayer metadata. Graph edits compile
  back to existing native records atomically; canvas positions stay local.
- **Animations** — read-only node view of the campaign's compiled `anim.zbd`
  (read with mech3ax). Starting from a definition, callers expand left and
  callees right through `CALL_ANIMATION`, `STOP_ANIMATION`,
  `INVALIDATE_ANIMATION`, `RESET_ANIMATION`, and activation prerequisites.
  State-writing callbacks (42/43) are marked with the channel they set. The
  mission's `start_single_player[_mN]` entry is focused on open.
- **Networks & Regions** — `net_*.zrd` and `reg_*.zrd` node graphs, positions,
  links, special edges, optional tags, and region-cycle visualization.
- **References** — cross-layer Valve/state, vehicle, and region references plus
  structural diagnostics. Confirmed target and objective state records are
  editable through Behavior Graph; their remaining schemas stay read-only.

Base Readers start locked. Unlocking is explicit, and **Export changes** writes
only modified archives below a separately chosen output directory while
preserving their original relative paths. Source game files are never
overwritten by that workflow. A direct ZRD/JSON compatibility mode remains
available through **Open file**.

The mission overview can be collapsed with the arrow in its header; this choice
is remembered locally. In the Networks & Regions map, use the mouse wheel or
the `+`/`−` buttons to zoom, drag empty map space to pan, and use **Fit** to
restore the complete graph view. Dragging a node still changes only its X/Z
position.

## Research documentation

This `README.md` is the top-level entry point for the project and the docs tree.
Use [`docs/README.md`](docs/README.md) as the detailed file-by-file catalog once
you know which area you need.

- [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) — implemented architecture
  and remaining phases.
- [`docs/README.md`](docs/README.md) — detailed catalog for the reorganized
  `docs/` tree.
- [`docs/editor-systems/reader-document-relationships.md`](docs/editor-systems/reader-document-relationships.md)
  — layer and reference relationships.
- [`docs/editor-systems/valve-and-state-logic.md`](docs/editor-systems/valve-and-state-logic.md)
  — Valve/state model and action shapes.
- [`docs/editor-systems/behavior-graph.md`](docs/editor-systems/behavior-graph.md)
  — visual vocabulary, native compilation mapping, safety model, and evidence
  limits for the Behavior Graph.
- [`docs/editor-systems/ai-networks-and-regions.md`](docs/editor-systems/ai-networks-and-regions.md)
  — network/region serialization model.
- [`docs/aiv-research/README.md`](docs/aiv-research/README.md) — AIV/vlist
  corpus, generated datasets, and reverse-engineering notes.
- [`RESEARCH_GAPS.md`](RESEARCH_GAPS.md) — evidence rules and deliberately
  unresolved semantics.

## Development

Requirements: Node 20+, Rust 1.88+, and the local mech3ax checkout at
`C:/Dev/mech3ax`.

```powershell
npm install
npm run tauri dev
```

Checks against the installed game:

```powershell
npm run check
npm run build
$env:MW3_TEST_ROOT='C:\Games\MW Clean\MechWarrior_3_(USA)'
$env:MW3_TEST_READER="$env:MW3_TEST_ROOT\zbd\c1\readerm1.zbd"
cd src-tauri
cargo test
```

The editor does not need the running game or debugger. Unknown values remain in
their original JSON-shaped records and survive archive round trips.

## Valve Switchboard companion

### Industrial Control Panel

The separate industrial panel uses the same live monitor with red glass lamps,
tape channel labels, a large mission label, and a physical panel power switch.
Start it from `switchboard` with `npm run panel:tauri:dev`, or build it with
`npm run panel:tauri:build`. For a browser preview, use `npm run panel:dev`.
`./build-industrial-portable.ps1` builds and copies a standalone executable to
`release/industrial-panel/MechWarrior 3 Industrial Panel.exe`.
Before the first live connection it displays a clearly marked preview circuit;
after a connection is lost it shows no stale lit channels. One compatible game
process is attached automatically. Only state channels are shown; nonzero
valve values light a lamp. The power
switch only switches the display. It never writes to the game.

The monitor currently supplies no mission title, so the large tape defaults to
MECHWARRIOR 3. A browser session can supply `?mission=Operation%20Damocles`.

The separate **MechWarrior 3 Valve Switchboard** attaches to a running mission
and displays every valve in the live registry. It currently supports only the
validated `Mech3.exe 1.2.22` executable profile. The game must run at the same
Windows privilege level as the switchboard.

```powershell
cd switchboard
npm run tauri:dev
npm run tauri:build
```

It polls at 10 Hz, retries unstable registry reads, and highlights changed
values. Direct writes start disabled. Enabling them exposes a per-valve
32-bit integer editor; each request revalidates the game build, re-resolves a
stable live valve, rejects stale values, and reads back the result.

These are raw registry value writes. They do **not** inject code or call the
game's internal valve-notification routine, so a listener that relies only on
that notification may not react even though the valve value has changed.

The **Animations** tab reads the live animation-definition table (5 Hz,
read-only). Each definition shows its `anim_state` (ready, running, executed,
invalidated), running/total runtime instances, health, and for running
definitions the raw per-sequence state and loop count. A transition log records
state changes while the mission runs; `.flt` effect templates are hidden by
default. Animations cannot be started or stopped from the switchboard.

## Valve Switchboard developer build

`switchboard-dev` is a separate research application and binary. It first
loads a selected mission's global, campaign, and mission Reader layers, then
joins their known producers and consumers to the live registry. Expanded cards
show the Reader records and deduplicated native caller addresses observed by
the optional Windows hardware-breakpoint trace. Native tracing is experimental,
off by default, and limited to the validated `Mech3.exe 1.2.22` profile.

```powershell
cd switchboard-dev
npm install
npm run portable
```

Portable outputs are isolated under `release/switchboard-dev`; the stable
viewer build script writes only to `release/switchboard`.

## AI Vehicles layout developer

The normal Reader Studio only renders the versioned AI layout in
`src/ai-layout.json`; it does not include layout-editing UI. To launch the
separate developer build:

```powershell
npm run layout:tauri:dev
```

The layout developer can create, rename, reorder, and delete tabs, ordinary
groups, nested repeater groups, and fields. Fields can be dragged between
groups or dropped on another tab, and the property inspector edits their
descriptions, `aiv.zrd` bindings, control types, format types, widths, option
sources, and reference sources. VList is represented as a configurable
repeater; Team and Raw JSON are configurable field controls. **Live Reader
preview** opens the normal Studio with the current valid draft.

Use **Save repository JSON** to validate and overwrite `src/ai-layout.json`.
The normal `npm run build` then bundles that layout. Invalid or incompatible
bindings (for example, a Team control not linked to `record[3]`) are shown in
the developer toolbar and cannot be saved or previewed.

## Licence

Mech Reader Studio and the Valve Switchboard are licensed under the European
Union Public Licence v. 1.2 (EUPL-1.2), see `LICENSE`. The editor statically
links parts of [mech3ax](https://github.com/TerranMechworks/mech3ax), which is
also EUPL-1.2; see `THIRD-PARTY-NOTICES.md`.
