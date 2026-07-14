# Integrated Reader workspace implementation plan

## Confirmed architecture

- AIV `params`, `p2`, `p8`, and the other parameter groups do not reference valves directly.
- AIV `vlist[15]` resolves a named boolean state through `FUN_004d3c40`; `vlist[0]` is the state written when the condition fires.
- `FUN_004d3c40` is a global find-or-create resolver. A state name does not need a `valves.zrd` action definition to be valid.
- `valves.zrd` registers ordered actions on state channels. Duplicate event names are legal and meaningful.
- `net_*.zrd` resolves `valve`, `valveunion`, and `valve_assign` through the same state resolver (`FUN_004db1b0`).
- `targets.zrd` resolves `health_below` outputs through the same resolver (`FUN_004e08c0`).
- Campaign `objectives.zrd` and multiplayer `mpinfo.zrd` observe the same channels through `valve_change`, including channels produced by mission AIV files.
- `reg_*.zrd` uses the `ZRdrAiNet.cpp` node format as a closed spatial region. VList `Enters`/`Exits` uses the region containment path `FUN_004c57d0`.

## Delivery stages

1. Replace the single-document model with a layered workspace. Scan an MW3 directory, list mission Reader archives, and load the selected mission with its campaign and global base Readers.
2. Build a typed, lossless document index and a cross-document reference graph. Locked dependency layers remain read-only until explicitly unlocked.
3. Keep AI Vehicles as one workspace, add a separate Valve Logic workspace, and add a separate Networks & Regions workspace with a large X/Z graph and numeric property editor.
4. Export only changed archives to a separate directory while preserving their MW3-relative paths. Re-open written archives as verification.
5. Keep Targets and Objectives navigable and reference-aware but read-only until their full schemas and original dialogs are recovered.

## Safety and evidence rules

- Preserve record order, duplicate keys, unknown tags, raw variants, and untouched archive entries.
- Missing Valve action definitions are informational, not errors. Missing region documents, invalid node links, or missing vehicle targets are warnings.
- Use semantic labels only where parser, runtime, or original MFC wiring proves them. Retain ZRD indices for unresolved fields.
- The known unresolved region references `reg_inside_c4m7` and `reg_outside_c4m7` are reported, never silently corrected.
- Original game archives are never overwritten by the normal export workflow.

## Acceptance gates

- Index all supplied Reader archives, including archives without `aiv.zrd`.
- Resolve references across mission, campaign-base, and global-base layers.
- Round-trip AIV, Valves, Net, and Region documents without losing unknown data.
- Preserve duplicate Valve event/action pairs.
- Dragging a spatial node updates X/Z only and leaves Y and unrelated values unchanged.
- Verify exported archives by reading them back.
- Pass Rust tests, Svelte/TypeScript checks, Vite production build, and the Tauri release build.
