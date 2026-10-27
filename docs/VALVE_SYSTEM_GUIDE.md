# How to Use the MechWarrior 3 Valve System

This guide uses **valve** in the useful modding sense: a named flag/value
that other Reader records can change, test, or react to. (The executable's
technical term is a *state channel*.) It is not a single
`if`/`else` table and it is not defined only in `valves.zrd`.

For example, a target can change `comcntr_down` to `1` when its health reaches
zero. That one named change can both play dialogue and complete an objective.

```text
target health reaches 0
  -> comcntr_down = 1
  -> play delayed dialogue
  -> complete the "destroy communications centre" objective
```

## Evidence used in this guide

- **Confirmed** means the Reader parser/data has been joined to an executable
  consumer or producer.
- **Supported** means the layout is present in supplied game data and agrees
  with the runtime structure, but a detail of the final action is not fully
  traced.
- **Unknown** means it should be preserved, not guessed.

The supplied corpus uses `0` and `1` for almost all state values. Internally,
the registry stores an integer, so do not assume every consumer accepts only a
boolean unless its own format says so.

## 1. Overview

A **valve** is a string name plus a current value, usually `0` or `1`. The
game resolves the name in one shared registry; it is not an ID, byte offset,
or per-file index. The same spelling in two loaded Reader files connects them.

`valves.zrd` is best understood as an ordered list of **subscriptions and
derived valves**:

- a valve changes;
- every action registered for that valve matters;
- an action can play a sound, affect an AI vehicle, schedule another valve
  update, or derive a new valve from other valves.

There is no generic comparison-operator menu such as `>`, `<`, or `!=` for
valves. The confirmed general comparison is equality to a requested value.
The specialised producers (`health_below`, VList thresholds, and region tests)
have their comparisons hard-coded by the game.

### Lifetime and scope

**Confirmed:** the namespace is shared across the Reader layers loaded for the
current mission/session. A mission file can therefore write a valve that a
campaign base file consumes. The resolver creates a valve if a name has not
been seen before, so a valve does not need a `valves.zrd` record to exist.

**Unknown:** persistence across saved games, mission changes, and different
game releases has not been established. Treat a valve as mission-runtime
state, not as a campaign-persistent variable. VList *inputs* are also
contextual: they are evaluated for a particular AI owner/subject and, where
used, its current candidate target and position.

## 2. Files involved

All of these files are normally entries in a Reader archive. A mission may
also load matching entries from campaign/training and global base archives.

| File / structure | Purpose | References | Referenced by | Edit directly? |
|---|---|---|---|---|
| `valves.zrd` | Ordered valve/action pairs and derived-valve rules. | Valve names, sounds, AI vehicle names. | Any producer that writes its outer valve. | Yes, with a compatible unpack/repack workflow. |
| `aiv.zrd` `vlist` | 17-field AI/region condition rows that can set a valve. | Region name at index 9; output valve at index 15. | AI vehicle records. | Yes; retain all 17 fields. |
| `reg_*.zrd` | Closed spatial geometry used by VList region rows. | Node links. | `aiv.zrd` VList index 9. | Yes, but it is geometry, not a valve definition. |
| `net_*.zrd` | AI-navigation nodes/edges; can test or assign a valve. | Valves and AI vehicle names. | AI vehicle network use. | Yes; preserve node and connection order. |
| `targets.zrd` | Target health thresholds that write valves. | Valve names in `health_below`. | Mission target records. | Yes, if the target record is understood. |
| `objectives.zrd` / `mpinfo.zrd` | Observe a valve/value for objective progression. | Valve name and requested value. | Objective or multiplayer phase. | Yes; these observe rather than set the state. |

### Vehicle-to-network starting node

An AIV vehicle's `record[0]` selects its Home Net and `record[1]` stores its
initial world position. Neither the AIV record nor the `net_*.zrd` node data
contains an explicit vehicle-to-node `start_node` field. In T1M4, the initial
positions of `bulldog_01` and `owens_01` match `net_04` `node_47` and `node_46`
respectively, to the serialized float precision. This proves authored spatial
alignment, but not a nearest-node algorithm.

The analyzed executable identifies the node-returning step. `FUN_004be690`
calls AI component virtual method `+0x88`; the component vtable installed by
`FUN_004cf330` resolves that slot to `FUN_004c9180`. That function obtains
runtime relation objects through virtual `+0x60`/`+0x74` calls, compares their
pointer identities against an internal vector, and uses the matching vector
index with the relation's virtual `+0x40` method. A missing relation is
appended and receives the next vector index; a null input relation returns no
node. No coordinate read, squared-distance calculation, search radius, or
distance tie-break occurs in this resolver. The returned node is then consumed
by `FUN_004be690`, which reads adjacency and position and creates actions via
`FUN_0047a320`.

The exact implementation of the final relation-class `+0x40` method is still
not identified, so the documentation should describe the proven behavior as
**runtime relation/index association**, not as nearest-node selection. The
editor should show T1M4 coordinate matches as authored evidence and warn when
the runtime relation/index cannot be reconstructed; it must not silently
substitute a nearest-node search.

The ordinary data flow is:

```text
AI / target / network / script
  -> named valve
  -> valves.zrd action(s), network condition(s), objective observer(s)
  -> sound, AI action, route availability, objective result, or another valve
```

Use the complete loaded Reader stack when following a name. A valve with no
same-file definition is not automatically broken.

## 3. Anatomy of a valve definition

In extracted JSON, `valves.zrd` is a pair list. Some exports put that pair list
inside one outer array. The useful sequence is always:

```json
[
  "event_channel", ["action_type", ["action", "arguments"]],
  "event_channel", ["another_action_type", ["arguments"]]
]
```

This is a real c1m1-shaped fragment (shortened only by removing unrelated
pairs):

```json
[
  "bramp", ["shutdown", ["owens_01", 0, 1]],
  "bramp", ["delayupdate", ["c1m1dialogue4_play", 22.0]],
  "bramp", ["delayupdate", ["truck_go", 18.0]],
  "bramp", ["sound_once", ["com_mech", 1]]
]
```

The four `bramp` pairs are **four actions**, not four alternative results.
Duplicate outer names are legal and must be preserved. There is no separate
record-count field: the even-length pair list is the count.

### Fields in an action pair

| Field | Type / known values | Meaning | Typical value | Safe to edit? |
|---|---|---|---|---|
| Outer valve | String | The trigger valve; for aggregate actions, the derived output valve. | `bramp`, `reach_mfb` | Yes, but rename every producer and consumer that should remain connected. |
| `record[0]` | String action type | Chooses the action payload layout. | `sound_once`, `delayupdate` | Use only a documented layout; keep unknown types raw. |
| `record[1...]` | Array or `"namelist"` plus array | Action-specific payload. | See table below. | Change only fields known for that action type. |

### Confirmed and supported action layouts

| Value | Name | Meaning | Practical use | Evidence / notes |
|---|---|---|---|---|
| `delayupdate` | delayed valve update | Schedules an update of another named valve. | Delay a line, route change, or follow-on event. | **Confirmed layout.** `[valve, seconds]`; no explicit output value is serialized. c1m1 shows `bramp` leading to `truck_go` after 18 seconds. |
| `sound` | sound action | Dispatches a named sound action. | Reusable dialogue/feedback. | **Confirmed layout.** `[sound_name, state]`; corpus uses state `1`. |
| `sound_once` | one-shot sound action | Dispatches a one-shot sound action. | One-time dialogue/feedback. | **Confirmed layout.** Same payload shape as `sound`. |
| `destroy` | vehicle action | Targets an AIV vehicle for destruction. | Trigger a destruction sequence from a valve. | **Confirmed layout.** `[vehicle_name, state]`; observed state is `1`. |
| `shutdown` | vehicle action | Targets an AIV vehicle's shutdown-related behaviour. | Disable/enable a named vehicle. | **Supported.** `[vehicle_name, number, state]`; the middle number's exact game meaning is **unknown**. |
| `teleport` | vehicle action | Moves a named AIV vehicle. | Scripted relocation. | **Confirmed layout.** `[vehicle_name, [x,y,z], yaw, state]`. |
| `attack_strategy` | vehicle action | Changes a named AIV vehicle's attack strategy. | Change combat behaviour after an event. | **Confirmed layout.** `[vehicle_name, strategy, state]`; strategies 0–4 are confirmed below. |
| `any_nonzero` | derived valve | True/non-zero when any listed valve is non-zero. | Combine alternate triggers. | **Confirmed layout.** `["any_nonzero", "namelist", [valve, ...]]`. |
| `all_nonzero` | derived state | True/non-zero when all listed channels are non-zero. | Require several known states. | **Confirmed layout.** Same `namelist` layout. |
| `all_zero` | derived state | True/non-zero when all listed channels are zero. | Detect an all-clear state. | **Confirmed layout.** Same `namelist` layout. |

`any_zero`, `update`, `setteam`, `missioncomplete`, `missionabort`,
`missionreset`, `animation_once`, and `animation` are recognized by the
executable's valve parser. Their full Reader payload layouts were not proved by
the supplied data. Do not invent them from the action name alone.

### Attack-strategy values

| Value | Name | Meaning | Practical use | Notes |
|---:|---|---|---|---|
| 0 (`0x0`) | Direct | Direct attack strategy. | Straight attack behaviour. | Confirmed enum. |
| 1 (`0x1`) | Zig Zag | Zig-zag strategy. | Less direct approach. | Confirmed enum. |
| 2 (`0x2`) | Circle | Circling strategy. | Orbiting attack. | Confirmed enum. |
| 3 (`0x3`) | Jump | Jump strategy. | Jump-capable behaviour. | Confirmed enum. |
| 4 (`0x4`) | Random Move | Random-move selection. | Treat with care. | Original text says it behaves like Circle. |

## 4. The 17-field VList producer

`aiv.zrd` is where a vehicle can evaluate an AI/region condition and write a
valve. It does **not** contain a `valves.zrd` action; it produces the state
which other files can consume.

Every shipped VList row in the supplied corpus has exactly 17 positions. A
new row must also have exactly 17 positions. The table uses the current
tooling labels first, followed by the original developer-facing label where it
adds useful context; neither should be treated as an official game term.

```text
index:  0  1    2  3  4     5  6   7  8      9  10   11  12   13  14      15 16
field:  state Ammo thr Fired Health thr Heat thr Enters region LOS range Dist range Active valve Exits
```

| Index | Field | Stored type / known values | Established meaning | Typical data / safe editing |
|---:|---|---|---|---|
| 0 | `Valve state to set` (`close`/`open`) | Integer; corpus `0`, `1` | Desired valve value written when the row fires. | `1`; safe when paired with a valid index 15 valve. |
| 1 | `Condition Ammo Enabled` (`Ammo`) | Boolean integer | Enables the ammo-threshold test at index 2. | `0`; threshold scale is not fully labelled, so copy an existing row. |
| 2 | `Ammo Threshold` | Float | Row fires when the ammo value is **at or below** this threshold. | Corpus `0.0..0.999...`; do not treat it as a raw ammo count without testing. |
| 3 | `Condition Fired Upon` (`Fired on`) | Boolean integer | Row fires when the owner reports being fired upon. | Only `0` occurs in the supplied corpus; changing it is experimental. |
| 4 | `Condition Health Enabled` (`Health`) | Boolean integer | Enables the health-threshold test at index 5. | `0` or `1`; paired with index 5. |
| 5 | `Health Threshold` | Float | Row fires when the vehicle's overall health fraction (1.0 = intact) is **at or below** this threshold; `0.0` means "destroyed". Live-verified in C1M1. | Usually a fraction such as `0.10` or `0.999`; use an existing vehicle row as a template. |
| 6 | `Condition Heat Enabled` (`Heat`) | Boolean integer | Enables the heat-threshold test at index 7. | `0` or `1`; paired with index 7. |
| 7 | `Heat Threshold` | Float | Row fires when heat is **at or above** this threshold. | Often `0.80`; scale is supported by data but has no user-facing unit proof. |
| 8 | `Condition Enters` (`Enters`) | Boolean integer | Selects the **inside** case for the region at index 9. | `0` or `1`; requires a valid region. |
| 9 | `Region Name` | `0` or String | Name of a `reg_*.zrd` geometry record. | `"reg_mfb"`; changing it is safe only when that region is loaded. |
| 10 | `Condition Target LOS Enabled` (`Target LOS`) | Boolean integer | Enables the candidate/line-of-sight distance test at index 11. | No enabled examples in supplied data; experimental. |
| 11 | `Target LOS Range` | Float | Distance-like threshold for index 10. | Stored as ordinary range, squared at runtime; corpus uses `0.0`. |
| 12 | `Condition Target Distance Enabled` (`Target Distance`) | Boolean integer | Enables the candidate-distance test at index 13. | `0` or `1`; paired with index 13. |
| 13 | `Target Distance Threshold` | Float | Distance-like threshold for index 12. | Stored as ordinary distance, squared before comparison; examples include `30`, `60`, `100`, and `1000`. |
| 14 | `Condition Active` (`Active`) | Boolean integer | Parsed and stored, but not read by the confirmed VList evaluator path. | All supplied rows use `1`; leave it `1` until another consumer is proved. |
| 15 | `Valve to set` (`Valve`) | String | Output valve name; resolved through the shared registry. | `"reach_mfb"`; required for a useful row and safest to change when all references are updated. |
| 16 | `Condition Exits` (`Exits`) | Boolean integer | Selects the **outside** case for the region at index 9. | `0` or `1`; requires a valid region. |

### How a row fires

Ammo, Fired on, Health, Heat, Target Distance, and Target LOS are **OR-ed
triggers**: any one that is enabled and satisfied writes the channel. Only
when none of them fired does the region selector decide. A row with no
enabled trigger and no region never fires. Nothing fires while the channel
already holds the row's value. Source: `FUN_004d8e70`, see
`aiv-research/vlist-runtime-semantics.md`.

### Region selector truth table

When no trigger fired and index 9 resolves to a region, the selector bits are
confirmed to work as follows:

| Enters (8) | Exits (16) | Result |
|---:|---:|---|
| 0 | 0 | Never passes the region selector. |
| 1 | 0 | Passes while the subject is inside the region. |
| 0 | 1 | Passes while the subject is outside the region. |
| 1 | 1 | Always passes the region selector. |

The target-distance fields (11 and 13) are transformed to squared distances
internally. Enter ordinary world-distance values in the Reader data, not their
squares.

### How VList conditions combine

A VList row is an `OR`, never an `AND`. The evaluator first skips rows whose
output valve already has the requested state. Then every enabled trigger
(Fired on, Heat, Health, Ammo, Target Distance, Target LOS) is an
alternative: enabling Ammo and Health means the row fires when **either** one
is satisfied. The region selector is the last alternative, consulted only
when no trigger fired; it does **not** restrict the triggers to the region.
For example, Health `0.5` plus Enters `reg_x` fires at half health anywhere
on the map, and also at full health inside `reg_x`.

Practical rule: use one clear condition per VList row, or copy a proven
combination from the same type of mission. If several independent conditions
must mean “any of these,” write separate channels and join them with
`any_nonzero` or a network `valveunion`. If several must mean “all,” use
`all_nonzero`.

## 5. Other condition and assignment forms

### Node and edge tags: what belongs where

The optional tags in `net_*.zrd` and `reg_*.zrd` describe either a spatial
node (a point/area in the network) or one directed edge between two nodes.
They are not all valve conditions. The complete observed tag inventory below
is grouped by location and records both the exact stored layout and the
current confidence level.

At a glance, the tag families are:

| Family | Tags | What they do |
|---|---|---|
| State conditions | `valve`, `valveunion` | Decide whether a node/edge is eligible from named valves. |
| State output | `valve_assign` | Writes a named valve from a node/edge event. |
| Vehicle routing | `gate`, `deploy` | Associate a route or deployment record with an AI vehicle. |
| Movement/timing | `canleave`, `jump`, `speed`, `wait_interval`, `wait_time` | Control travel, leaving, jumping, speed, or waiting. Some detailed semantics remain unproved. |
| Scan behaviour | `scan_aim`, `scan_hit`, `scan_time` | Scan-related controls with known layouts but incomplete behavioural names. |
| Metadata/layout | `arrangenodes`, `auxname`, `name` | Editor/layout or naming data rather than state logic. |

Use the complete inventory as the authority for what can safely be edited. A
tag's presence on an edge does not imply that it is also valid on a spatial
node, and vice versa.

### Network tags

| Value | Name | Meaning | Practical use | Notes |
|---|---|---|---|---|
| `[enabled, valve, target?]` | `valve` | Tests whether `valve == target`. | Gate a route or node by state. | `enabled = 0` disables it. If target is omitted, the navigation code derives it from the outgoing-link index; specify a target explicitly whenever possible. |
| `[[enabled, valve, target?], ...]` | `valveunion` | OR of enabled equality tests. | Allow a route after any alternate trigger. | Missing member target defaults to `1`; evaluation stops after the first true member. |
| `[valve, value]` | `valve_assign` | Writes a valve. | Turn a navigation event into a valve event. | No leading enabled flag; an edge `valve_assign` runs at the `from` node when the unit departs on that link, after the node's own `valve` wait. |
| `[vehicle]` | `gate` | Vehicle-qualified edge rule. | Associate an outgoing edge with a particular AI vehicle. | Not a state condition. |

### Complete observed network-tag inventory

The following is the complete inventory of **16 optional tag names observed in
the supplied `net_*.zrd` corpus**. It is not a guarantee that another release
cannot recognize more names. “Spatial” means an ordinary node with a position;
“edge” means a non-spatial record beginning with `[from, to]` that qualifies
one directed connection.

| Tag | Observed layout | Location in corpus | What is established | Editing guidance |
|---|---|---|---|---|
| `arrangenodes` | `[number]` | Spatial | Layout only; likely an arrangement-related value. | Preserve unless comparing like-for-like nodes. |
| `auxname` | `[string]` | Spatial | Auxiliary text/name. | Safe to rename as text only after checking its consumers. |
| `canleave` | `[enabled, value]` | Spatial and edge | Leading value is an enable flag; the rest is a navigation control. | Do not infer a generic boolean meaning for `value`. |
| `deploy` | `[vehicle, number, string, number, number]` | Spatial | Deployment-related record shape. | Payload semantics are **unknown**; preserve as raw data. |
| `gate` | `[vehicle]` | Edge only | Vehicle-qualified connection rule. | Match the vehicle name exactly; it neither reads nor writes a valve. |
| `jump` | `[enabled, number]` | Spatial | Leading enable flag plus jump-related numeric control. | Numeric unit/precise runtime effect is not fully recovered. |
| `name` | `[string]` | Spatial | Primary node name/text. | Rename cautiously; possible external references are not fully indexed. |
| `scan_aim` | `[enabled, number]` | Spatial and edge | Scan-related control with a leading enable flag. | Exact test/action is **unknown**. |
| `scan_hit` | `[enabled, number]` | Spatial and edge | Scan-related control with a leading enable flag. | Exact test/action is **unknown**. |
| `scan_time` | `[enabled, number]` | Spatial and edge | Scan-related control with a leading enable flag. | Exact test/action is **unknown**. |
| `speed` | `[enabled, number]` | Spatial and edge | Speed-related control with a leading enable flag. | Copy a proven node; numeric unit is not fully recovered. |
| `valve` | `[enabled, valve, target?]` | Spatial and edge | Equality condition on a named valve. | Prefer an explicit target value. |
| `valve_assign` | `[valve, value]` | Spatial and edge | Named-valve assignment. | No enable field; on an edge it fires on departure from the `from` node. |
| `valveunion` | `[[enabled, channel, target?], ...]` | Spatial and edge | OR of enabled valve equality conditions. | Use for alternate state triggers. |
| `wait_interval` | `[enabled, number]` | Spatial and edge | Interval-related navigation control with enable flag. | Copy known values; exact scheduling details are not fully recovered. |
| `wait_time` | `[enabled, number]` | Spatial | Wait-related navigation control with enable flag. | Copy known values; exact scheduling details are not fully recovered. |

The state-condition choices are therefore `valve` (one equality test) and
`valveunion` (OR of equality tests). No network-tag equivalent of logical AND
or NOT was found. Use `all_nonzero` in `valves.zrd` for a derived AND state;
build an explicit valve for more complex logic.

`gate` is not a member of `valveunion` and is not a third kind of valve. It is
an edge-only vehicle qualifier. In c1m1, the `7 -> 8` edge has
`gate ["ttruck_01"]` and the `7 -> 9` edge has `gate ["ttruck_02"]`; later
edges write the separate destroy channels for those two trucks.

### Target and objective forms

| Value | Name | Meaning | Practical use | Notes |
|---|---|---|---|---|
| `[valve, value, health, ...]` | `health_below` | Repeating triples that create health-threshold valve outputs. | Turn target damage into a valve. | Confirmed parser stride is three values. |
| `"valve", [valve]` plus `"value", [value]` | `valve_change` | Objective/multiplayer observer that reacts to a valve equality. | Complete or fail an objective. | It observes the state; it does not set it. |

## 6. Evaluation process

The following is deliberately a data-flow model, not a claim that all files
are evaluated in one single function.

```text
load Reader layers
resolve each channel name in one shared registry

when a producer changes a valve value:
    ignore the write if the value is already current
    notify registered listeners
    run every relevant valves.zrd action record
    allow network / objective consumers to see the new value

when an AI VList row is evaluated:
    resolve its index-15 channel
    return if that channel already equals index 0
    evaluate the row's supported condition path
    if it fires, write index 0 to index 15
```

### What is confirmed

- `FUN_004d3c40` finds or creates a named channel. References are by string
  name, not by a stored numeric valve ID.
- `FUN_004d3c00` writes only when the value changes, then notifies listeners.
- VList rows are evaluated as a list; no first-matching-row break was found.
  Avoid overlapping rows that set conflicting values of the same channel:
  final ordering behaviour for such a conflict is not documented as a feature.
- `valveunion` is logical OR. `any_nonzero`, `all_nonzero`, and `all_zero` are
  explicit group operations; they are the safe way to express simple multiple
  conditions.
- The normal valve comparison is equality to a numeric target. A missing
  network `valve` target is not always equivalent to literal `1`.
- `valves.zrd` duplicate event names represent multiple actions, not a
  first-match branch. Preserve all of them and their serialized order.

### No match and chaining

There is no automatic `else` result. If a VList row does not fire, the channel
stays as it was. If no network state condition matches, that conditional route
does not become available through that condition. If a valve has no action
record, it may still be useful to a network or objective, but otherwise it has
no visible effect.

Actions can chain through a named channel: `delayupdate` can update a second
channel, which can trigger another action record. Aggregate actions also make
derived channels. No general “call valve by index” or arbitrary nested valve
format was found. Do not create circular delayed/aggregate relationships;
their behaviour has not been tested.

### Open questions that affect modding

- The exact value-propagation rule inside `delayupdate` is not fully recovered;
  only its target channel and delay are serialized.
- The middle number in `shutdown`, and the effect of state values other than
  the observed `1` in several action types, are not fully documented.
- The exact outcome of combining multiple threshold/candidate flags in one
  VList row needs targeted runtime tests; use separate rows/aggregates instead.
- `valve_assign` on an edge fires on departure from the `from` node (after
  that node's `valve` wait); the general multi-link route-selection algorithm
  remains unknown.

## 7. What modders normally change

Use this checklist for a small, low-risk change:

1. Find an existing complete chain in the same mission: producer, channel,
   consumer, and outcome.
2. Choose a distinct, descriptive channel name if changing an existing chain
   would affect unrelated readers. Treat spelling and case as significant;
   case-insensitivity is not proved.
3. For a `valves.zrd` action, add or change a whole `channel, record` pair.
   Keep the pair list even and retain duplicate channel pairs.
4. For a VList producer, change only index 0 (state), 9 (region), or 15
   (channel) unless you are deliberately testing a proven threshold template.
   Keep all 17 fields in their original order.
5. When using a region, make sure the same Reader stack contains the matching
   `reg_*.zrd` geometry.
6. When a `destroy`, `shutdown`, `teleport`, `attack_strategy`, or `gate`
   record names a vehicle, make sure it exactly matches an `aiv.zrd` vehicle
   record.
7. Prefer explicit network targets such as `[1,"my_channel",1]`; do not rely
   on the two-item shorthand unless its connection ordering is intentional.
8. Repack with a tool that round-trips the Reader entry, then re-extract or
   reload it and compare the changed arrays. Do not edit a binary `.zbd`
   archive as plain text.

There is no separate VList count to update: the containing list length is the
count. There is also no separate `valves.zrd` pair count. Adding a row or pair
changes the implicit count, but a malformed 16- or 18-field VList row is not a
valid substitute for a 17-field row.

## 8. Worked examples

The first three examples are **synthetic but use confirmed layouts**. The last
one follows supplied c1m1 data exactly.

### Example 1 — a two-state region signal

**Goal:** set `gate_inside = 1` while the player is inside `reg_gate`, and set
it back to `0` while outside. A network can then use the two values as its two
outcomes.

Add these two full rows to the relevant vehicle's `aiv.zrd` `vlist` array:

```json
[
  [
    1, 0, 0.25, 0, 0, 0.10, 0, 0.80,
    1, "reg_gate", 0, 0.0, 0, 60.0, 1, "gate_inside", 0
  ],
  [
    0, 0, 0.25, 0, 0, 0.10, 0, 0.80,
    0, "reg_gate", 0, 0.0, 0, 60.0, 1, "gate_inside", 1
  ]
]
```

Then use explicit network conditions, one for each desired value:

```json
"valve", [1, "gate_inside", 1]
"valve", [1, "gate_inside", 0]
```

Required companion data: `reg_gate.zrd` must be loaded and define the intended
closed region. No `valves.zrd` action is required merely to make the signal
valid; add one only if a sound, AI action, or derived state should react.

### Example 2 — three alternative triggers (OR)

**Goal:** allow one network branch if *any* of three producers has raised an
alert.

```json
"valveunion", [
  [1, "alarm_from_region", 1],
  [1, "alarm_from_target", 1],
  [1, "alarm_from_script", 1]
]
```

The branch is true when any enabled member equals its target. This is better
than trying to combine three unrelated tests in one VList row. A later edge
can turn that route event into another valve:

```json
"valve_assign", ["begin_reinforcement", 1]
```

And a `valves.zrd` action can consume it, for example:

```json
"begin_reinforcement", ["attack_strategy", ["enemy_01", 0, 1]]
```

Required companion data: `enemy_01` must be an AIV vehicle, and each alarm
channel needs a real producer. An edge `valve_assign` fires when the unit
leaves the edge's `from` node, not when it reaches the `to` node.

### Example 3 — why connection order can matter

This compact network form is risky:

```json
"node_00", [12, [100.0, 0.0, 200.0], [1, 2],
            "valve", [1, "route_select"]]
```

With two outgoing links, the missing target is derived from the outgoing-link
index: the last link corresponds to target `1`, and an earlier link to target
`0`. Reordering `[1, 2]` changes which physical route is associated with which
implicit value. The corrected, self-documenting form is to supply the target:

```json
"valve", [1, "route_select", 1]
```

Use the two-item form only when the index-derived behaviour is specifically
desired.

### Example 4 — verified c1m1 chain: target to outcome

This is based on the supplied campaign 1 mission 1 files:

```text
readerm1/targets.zrd:
  satdish health_below ["comcntr_down", 1, 0.0]
      -> comcntr_down = 1

readerm1/valves.zrd:
  "comcntr_down", ["delayupdate", ["c1m1dialogue7_sound", 3.0]]
      -> after 3 seconds, update c1m1dialogue7_sound
  "c1m1dialogue7_sound", ["sound_once", ["snd_c1m1dialogue7", 1]]
      -> play the dialogue sound

c1/reader/objectives.zrd:
  primary valve_change: valve comcntr_down, value 1
      -> primary objective progresses
```

This trace is the important pattern: the target knows only the channel name;
the sound action and objective independently consume the same name. There is
no target-to-objective pointer.

### Example 5 — a common mistake and its correction

**Mistake:** treating the leading `1` in a network condition as “require true”:

```json
"valve", [1, "truck_go"]
```

The leading value enables the test. The target is omitted and therefore comes
from connection position, not from that leading `1`.

**Correction:** state the requested value in the data:

```json
"valve", [1, "truck_go", 1]
```

The supplied c1m1 truck route uses the short form only where each relevant
node has one outgoing connection, which happens to derive target `1`.

## 9. Limits and failure modes

| Situation | What is known | Practical result |
|---|---|---|
| Misspelled or otherwise missing valve name | The resolver creates a valve when absent. | Usually no load failure; the intended connection is silently absent unless another record uses the same misspelling. |
| Missing `valves.zrd` action for a valid channel | Valid by design. | The channel can still drive a network or objective; otherwise it has no visible action. |
| Duplicate outer valve in `valves.zrd` | Legal and meaningful. | Multiple actions, not an override. Do not deduplicate. |
| Unknown action type or payload | Parser string may be known without a proved layout. | Preserve raw record; exact game behaviour is unknown. |
| Wrong VList length/order | Runtime format is exactly 17 fields. | Unsupported/malformed; outcome is not safe to predict. |
| Missing/invalid region reference | VList index 9 cannot supply the intended containment test. | A pure region row may never update; exact failure reporting is version-dependent. |
| Missing AIV vehicle name in an action or `gate` | Action cannot target the intended vehicle. | Exact failure mode is not proved; expect missing behaviour rather than a safe fallback. |
| No VList condition match | No automatic else branch. | Existing channel value remains unchanged. |
| Empty or untested aggregate list | No corpus proof of its edge-case semantics. | Avoid it. |
| Multiple VList rows set conflicting values | Rows are evaluated as a list; no first-match policy was found. | Do not rely on priority or final value without testing. |
| Circular channel chain | No tested loop-prevention rule. | Avoid cycles, especially delayed or aggregate cycles. |
| Excessive entries / name-length limits | No hard Reader limit was recovered for valve pairs, VList count, or channel name length. | Start from a real mission's scale; do not assume tooling limits are game limits. |
| Different release (including Solaris) | The current proof is from the analysed `Mech3.exe`; releases were not fully compared. | Re-test new action types and edge cases on the target executable. |

### Known discrepancy in the supplied data

The c1m1 truck trace lives in
`docs/aiv-research/extracted-reader-json/zbd/c1/readerm1/net_01.zrd.json`,
but that file's embedded `name` record says `net_02`. Preserve both the entry
name and its stored header unless you have a reason to change them together;
the discrepancy is evidence that a filename is not always a reliable semantic
description.

## 10. Quick reference

### Files to inspect

1. `aiv.zrd` — VList producers; channel at `vlist[*][15]`, desired value at
   `[0]`, region at `[9]`.
2. `valves.zrd` — ordered `channel, action` pairs; duplicate channels are
   multiple actions.
3. `net_*.zrd` — `valve`, `valveunion`, and `valve_assign` channel use.
4. `reg_*.zrd` — geometry named by VList `[9]`.
5. `targets.zrd` — `health_below` channel triples.
6. `objectives.zrd` / `mpinfo.zrd` — `valve_change` observers.

### Normal editing sequence

```text
find an existing channel chain
  -> change/add the producer
  -> make every consuming name match exactly
  -> retain 17 VList fields and complete action pairs
  -> use explicit network target values
  -> repack, reload, and compare the changed Reader entry
  -> test in the target game release
```

### Troubleshooting

- No visible result? Search every loaded Reader layer for the exact channel
  spelling, then check whether it has a producer and a consumer.
- Region never changes state? Verify the `reg_*.zrd` name, the inside/outside
  selector bits, and that the VList row has all 17 fields.
- Route opens on the wrong branch? Replace a two-item `valve` with an explicit
  target and inspect connection ordering.
- One expected action vanished? Look for duplicate outer names that were
  collapsed during export.
- Objective never completes? Confirm the producer writes the exact
  `valve_change` value, not merely a similarly named channel.

## Research basis

- `docs/editor-systems/valve-and-state-logic.md` — consolidated cross-file
  state model, c1m1 traces, action shapes, and evidence boundaries.
- `docs/aiv-research/vlist-runtime-semantics.md` and
  `docs/aiv-research/parameter-field-map.md` — 17-field VList layout, region
  truth table, and squared-distance transformation.
- `docs/editor-systems/ai-networks-and-regions.md` — network tag layouts and
  the index-derived target rule.
- `docs/aiv-research/extracted-reader-json/zbd/c1/readerm1/valves.zrd.json`,
  `aiv.zrd.json`, `targets.zrd.json`, and `net_01.zrd.json`; plus
  `docs/aiv-research/extracted-reader-json/zbd/c1/reader/objectives.zrd.json`
  — real c1m1 examples used above.
- `src/types.ts` and `src-tauri/src/lib.rs` — current field names, reference
  extraction, and structural preservation expectations in repository tooling.
- `Mech3.exe` Ghidra routines: `FUN_004d3c40` (find/create channel),
  `FUN_004d3c00` (change-only write and notification), `FUN_004d4600`
  (OR union), `FUN_004d8e70` (VList evaluation), `FUN_004df730` (AIV/VList
  parsing), `FUN_004db1b0` (network parsing), `FUN_004e08c0`
  (`health_below` parsing), and `FUN_004e44b0` (`valves.zrd` construction).
