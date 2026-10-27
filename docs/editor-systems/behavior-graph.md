# Reader Studio Behavior Graph

The Behavior Graph is a typed, cross-document authoring view over MechWarrior
3's named integer state registry. It does not replace the Valve Logic or raw
record editors. Its normalized flow is:

```text
producer -> exact named state channel -> logic/delay -> exact named state channel -> consumer
```

Channel names are case-sensitive and values are integers. Regions, vehicles,
sounds, targets, networks, objective phases, Reader layers, and source paths
are metadata on blocks; none of them is silently converted into a channel.

## Visual vocabulary

| Shape/color | Meaning |
|---|---|
| Green-striped rectangle | Producer that writes a named state. |
| Blue pill | First-class named state channel. |
| Purple-striped rectangle | Aggregate or delayed transform with both input and output ports. |
| Orange-striped rectangle | Consumer that reads a channel. |
| Gray-striped rectangle | Unknown action. It is navigable and shown raw, but read-only. |
| Left orange port | Channel read. |
| Right green port | Channel write. |
| Yellow `→ channel = value` row | What a block writes. Every block with a write port shows it, next to that port. |
| Dashed green line | Network route: a `valve`/`valveunion` gate on node *N* (or on link *N→M*) leads to a `valve_assign` on link *N→M*. Derived from network topology, not a channel; read-only. |

Route edges make `Isolate chain` follow a gate into the assignments it
unlocks, e.g. C1M1 `truck_go` gate on `net_01` node 08 → `destroy_truck_01`
assigned on link 8→15. The game builds the gate and the edge assignment into the same per-link
departure list on node *N* (gate first), so the assignment fires when the
unit leaves *N* on that link; see `valve-and-state-logic.md`.

Blocks use deterministic serialized order within role columns. Existing
derived-state cycles are placed in a separate lower lane and produce a warning.
Moving a block changes only the locally persisted canvas layout. The storage
key is scoped by game root and selected mission; no position is written to a
Reader archive.

The default mission view includes every mission-layer occurrence and every
connected occurrence from loaded campaign, global, and attached layers.
Campaign objectives are projected only from the selected mission subtree.
`All loaded channels` broadens this view. Search, layer filters, and selected-
chain isolation affect presentation only.

## Confirmed native compilation mapping

| Graph block | Existing native host | Native mutation |
|---|---|---|
| AI VList producer | Existing vehicle in `aiv.zrd` | Append/remove one exact 17-field VList row. |
| Target Health Below | Existing target in `targets.zrd` | Append/remove one flattened `[channel,state,threshold]` triple. |
| Network Assign | Existing `net_*.zrd` node/edge | Append/remove `"valve_assign", [channel,value]`. |
| Objective Set Valve | Existing objective action | Project/edit existing `"set_valve", [channel,value]` producer records. |
| Network Valve | Existing `net_*.zrd` node/edge | Append/remove `"valve", [enabled,channel,target]`. |
| Network Valve Union | Existing `net_*.zrd` node/edge | Append/remove `"valveunion", [[enabled,channel,target],...]`. |
| Any Nonzero | Existing `valves.zrd` | Append/remove `[output,["any_nonzero","namelist",inputs]]`. |
| All Nonzero | Existing `valves.zrd` | Append/remove `[output,["all_nonzero","namelist",inputs]]`. |
| All Zero | Existing `valves.zrd` | Append/remove `[output,["all_zero","namelist",inputs]]`. |
| Delay Update | Existing `valves.zrd` | Append/remove `[trigger,["delayupdate",[output,seconds]]]`. |
| Sound / Sound Once | Existing `valves.zrd` | Append/remove the confirmed sound/value action pair. |
| Shutdown / Destroy | Existing `valves.zrd` | Append/remove the confirmed vehicle action pair. |
| Teleport | Existing `valves.zrd` | Append/remove `[trigger,["teleport",[vehicle,[x,y,z],yaw,state]]]`. |
| Attack Strategy | Existing `valves.zrd` | Append/remove `[trigger,["attack_strategy",[vehicle,strategy,state]]]`. |
| Objective Observer | Existing proven objective list | Append/remove a `valve_change` record with description key, nav marker, channel, and value. |

The palette creates canvas drafts. A draft is not native data until all
mandatory properties and an existing writable host have been selected and
**Apply to native record** succeeds. Missing ZRD entries, targets, vehicles,
network nodes, or objective-list containers are never synthesized.

Objective `fail` is deliberately special: the editor does not add parallel
records directly below `fail`. OR/AND behavior must be expressed as an
explicitly named derived channel in `valves.zrd`, after which the existing
failure observer is connected to that channel.

## Mutation and safety model

Every node and port has a semantic binding containing the archive-relative
path, entry name, record kind, and native array locator. `addBlock`,
`updateBlock`, `removeBlock`, `connect`, `disconnect`, and `renameChannel`
run against a cloned workspace. Locks and native shapes are checked before the
clone replaces the current workspace; an invalid operation therefore leaves
all documents unchanged.

Repeated outer Valve names, pair order, wrappers, unknown tags, unknown values,
and unrelated fields remain intact. Rename lists all recognized occurrences
and changes only the selected bindings. It warns because scripts, unknown raw
payloads, and unloaded Readers cannot be rewritten.

The frontend retains 50 document-root undo snapshots. Selection, camera,
filters, and node positions do not enter document history. Export rebuilds the
graph and blocks on structural errors or locked-document changes, then uses the
existing separate-directory write and archive reopen verification.

## Diagnostics

Errors cover malformed known records, non-17-field VLists, malformed target
triples, empty aggregates, and malformed objective/network state records.
Warnings cover missing regions or vehicles, near-spelling channel names,
derived cycles, case-differing names, and channels used by both success and
failure objectives. Locked hosts and unknown actions are reported without
changing them.

These are informational, not errors: a channel with no loaded producer, a
channel with no loaded `valves.zrd` action, and duplicate outer event names.

## Evidence boundary

The block inventory is limited to the serialized forms confirmed in
`valve-and-state-logic.md`. Recognized executable strings without a proven
corpus shape—such as `update`, `missioncomplete`, mission reset/abort, and
animation actions—remain gray raw blocks. Adding typed support for those forms
requires further executable dispatch research. Live values and memory writes
remain the responsibility of the separate Valve Switchboard.
