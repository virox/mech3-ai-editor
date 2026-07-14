# Reader document relationships

## Shared Valve/state namespace

MechWarrior 3 uses a global set of named boolean state channels. The name resolver at `FUN_004d3c40` first looks up an existing channel and creates it when absent. Consequently, the absence of a matching `valves.zrd` record is not an error by itself.

| Producer or consumer | Serialized location | Proven code path |
|---|---|---|
| AI condition writes a state | `aiv.zrd` `vlist[*][15]`, state in index 0 | parser `FUN_004df730`; evaluator `FUN_004d8e70` |
| Event actions subscribe to a state | outer key in `valves.zrd` | construction `FUN_004e44b0` |
| Delayed state update | `delayupdate` action | `ZRdrValve.cpp` dispatch |
| Network state conditions and assignments | `valve`, `valveunion`, `valve_assign` in `net_*.zrd` | parser `FUN_004db1b0` |
| Target-health output | `health_below` in `targets.zrd` | parser `FUN_004e08c0` |
| Mission objective observes state | `valve_change` in `objectives.zrd` or `mpinfo.zrd` | corpus plus shared runtime namespace |

`params`, `p2`, `p6`, `p8`, and the other AIV parameter arrays contain no named Valve reference. They meet Valve logic only indirectly through the owning vehicle, through VList, and through actions such as `shutdown`, `destroy`, `teleport`, and `attack_strategy` that target an AIV vehicle name.

## Layered Reader loading

A mission can depend on documents outside its own `readerm*.zbd` archive. The editor therefore resolves names across:

1. the selected mission Reader;
2. the campaign or training base `reader.zbd`;
3. the global `zbd/reader.zbd`;
4. manually attached dependency Readers.

Corpus overlap between mission VList channels and campaign `objectives.zrd` was observed in every campaign: c1 (8), c2 (6), c3 (7), c4 (4), c4b (2), and training (11). Examples include `outside_c1m1`, `reach_mfb`, `op_baker`, and `fired_mgun`.

## Regions

`reg_*.zrd` files have the same version/name/node pair structure handled by `ZRdrAiNet.cpp` as `net_*.zrd`. Their nodes normally contain positions and adjacency only. VList index 9 resolves the region by name; indices 8 and 16 select the inside or outside condition. Runtime containment is evaluated by `FUN_004c57d0` and its helper.

Two references in the supplied corpus currently have no same-name region document: `reg_inside_c4m7` and `reg_outside_c4m7`. They remain diagnostics rather than guessed renames.

## Evidence policy

Semantic labels are marked confirmed only when parser storage can be joined to a runtime consumer or original MFC data-exchange control. Corpus correlation is retained as supporting evidence but does not by itself authorize a semantic rename.
