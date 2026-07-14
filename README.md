# Mech Reader Studio

A Windows-first, lossless editor for MechWarrior 3 Reader archives. It opens a
game folder as a layered workspace and combines the global, campaign, and
mission Reader data without flattening archive ownership.

Implemented workspace modules:

- **AI Vehicles** — recovered `aiv.zrd` fields, VList conditions/actions, and
  source-key labels.
- **Valve Logic** — ordered `valves.zrd` event/action pairs, including duplicate
  event names and a raw fallback for unknown action types.
- **Networks & Regions** — `net_*.zrd` and `reg_*.zrd` node graphs, positions,
  links, special edges, optional tags, and region-cycle visualization.
- **References** — cross-layer Valve/state, vehicle, and region references plus
  structural diagnostics. Targets and objectives are available read-only.

Base Readers start locked. Unlocking is explicit, and **Export changes** writes
only modified archives below a separately chosen output directory while
preserving their original relative paths. Source game files are never
overwritten by that workflow. A direct ZRD/JSON compatibility mode remains
available through **Open file**.

## Research documentation

- [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) — implemented architecture
  and remaining phases.
- [`docs/READER_RELATIONSHIPS.md`](docs/READER_RELATIONSHIPS.md) — layer and
  reference relationships.
- [`docs/VALVE_LOGIC.md`](docs/VALVE_LOGIC.md) — Valve/state model and action
  shapes.
- [`docs/AINET_AND_REGIONS.md`](docs/AINET_AND_REGIONS.md) — network/region
  serialization model.
- [`RESEARCH_GAPS.md`](RESEARCH_GAPS.md) — evidence rules and deliberately
  unresolved semantics.

## Development

Requirements: Node 20+, Rust 1.88+, and the local mech3ax checkout at
`C:/Dev/TerranMechworks/mech3ax`.

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
