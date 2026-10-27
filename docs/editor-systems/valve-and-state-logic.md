# Valve and named-state logic

This is the cross-file reference for the Reader data that the game calls
“valves”. The important distinction is that a valve is not a single file
format:

* a **state channel** is a named, runtime value (normally `0` or `1`);
* `valves.zrd` attaches actions and derived-state rules to those channels;
* `aiv.zrd` VLists, `targets.zrd`, and network edge rules can **write** a
  channel;
* network `valve`/`valveunion` rules and mission `valve_change` records can
  **read** a channel.

The executable resolves all of these names through one shared registry. A
name matching across two Reader files is therefore a real connection; there
is no per-file valve namespace.

The statements below use these evidence labels:

* **Confirmed** — a parser destination is joined to an executable consumer or
  producer.
* **Observed** — the shape occurs in the supplied Reader corpus, but the
  complete runtime action has not yet been followed.
* **Recognized** — the string is present in the Mech3.exe valve parser, but no
  example was found in the supplied corpus.

## 1. The shared state registry

The registry is the hub connecting the Reader documents.

| Operation | Executable evidence | Effect |
|---|---|---|
| Resolve a name | `FUN_004d3c40` → `FUN_004d3d20` / `FUN_004d3b20` | Looks up a channel and creates it when it is absent. |
| Read a value | channel vtable slot `+4` | Returns the current integer value. |
| Write a value | `FUN_004d3c00` | Writes only when the value changes and then calls the registry notification path (`FUN_004d3bf0`). |
| Compare a condition | `FUN_004d4490` | Compares one `(channel,target)` pair. |
| Evaluate a union | `FUN_004d4600` | Loops the pairs and returns true as soon as one matches — logical OR. |

Consequences:

1. A channel can be created by a VList, target, net, script, or another
   action without having an entry in `valves.zrd`. A missing valve action is
   not automatically an error.
2. The word `destroy` in a channel name has no intrinsic meaning. It is only
   a name until a `valves.zrd` action consumes it.
3. The values are represented as integers. Most game logic uses `0`/`1`,
   but the Reader format also serializes explicit numeric targets, so the
   editor must not silently coerce every value to a boolean.

The data flow is therefore:

```text
 aiv.vlist[*][0] (desired state) + [15] (channel) ─┐
 targets.health_below triples              		   ├─> named state channel
 net edge valve_assign                   		   │
 valves update/delayupdate             		      ─┘

 named channel ──> valves trigger/action
               ──> net valve / valveunion condition
               ──> objectives/mpinfo valve_change
               ──> other script/runtime consumers
```

## 2. `valves.zrd`: event/action and derived-state records

`unzbd.exe` emits the logical contents as an ordered pair list. A JSON
wrapper array may surround it, so the useful shape is:

```text
[
  event_or_output_name, action_record,
  event_or_output_name, action_record,
  ...
]
```

An action record starts with its type. **Duplicate outer names are legal and
meaningful.** They register multiple actions; they must not be loaded into a
dictionary that keeps only the last value.

### Action forms seen in real data

The table describes the serialized shape, not invented editor fields. Where
the executable has not yet been traced far enough, the remaining numeric
arguments are deliberately left technical.

| Action | Serialized form in the corpus | What is established |
|---|---|---|
| `delayupdate` | [`"delayupdate"`, [channel, seconds]] | A delayed update is scheduled for the named channel. The action supplies the update value; no extra value is serialized in this form. |
| `sound` | [`"sound"`, [sound_name, value]] | Plays/dispatches the named sound when the outer channel rule fires. |
| `sound_once` | [`"sound_once"`, [sound_name, value]] | One-shot sound action; the same sound/value payload shape is used. |
| `shutdown` | [`"shutdown"`, [vehicle, number, number]] | Targets an AIV vehicle. The first number and final state/enable number are passed to the vehicle action; their historical captions are not yet proven. |
| `destroy` | [`"destroy"`, [vehicle, state]] | Targets an AIV vehicle and is observed with state `1`. |
| `teleport` | [`"teleport"`, [vehicle, [x,y,z], yaw, state]] | Targets an AIV vehicle and carries a position, yaw, and final state/enable value. |
| `attack_strategy` | [`"attack_strategy"`, [vehicle, strategy, state]] | Targets an AIV vehicle; the strategy integer is the same strategy domain exposed by the AI editor (`0..4`), and the final value is the action state/enable. |
| `any_nonzero` | [`"any_nonzero"`, `"namelist"`, [channel, ...]] | Derived-state rule over a list of channels; the name means any member is non-zero. |
| `all_nonzero` | [`"all_nonzero"`, `"namelist"`, [channel, ...]] | Derived-state rule over a list; all members must be non-zero. |
| `all_zero` | [`"all_zero"`, `"namelist"`, [channel, ...]] | Derived-state rule over a list; all members must be zero. |

The binary also contains parser strings for `any_zero`, `update`, `setteam`,
`missioncomplete`, `missionabort`, `missionreset`, `animation_once`, and
`animation`. They are **recognized** by the executable, but the supplied
Reader corpus did not provide a reliable serialized example for each one.
Their action signatures should remain raw until the dispatch path is traced.

The outer name is the channel context for the record. For ordinary action
records (`delayupdate`, `destroy`, `sound_once`, and so on) it is the event
channel that causes the action. The `any_*`/`all_*` records are the special
derived-state form: they consume the `namelist` and expose the outer name as
the derived channel. Both cases still use the same registry above.

### Concrete `c1m1` example

The `valves.zrd` in `zbd/c1/readerm1.zbd` contains (irrelevant records
omitted):

```text
"destroy_truck_01", ["destroy", ["ttruck_01", 1]],
"destroy_truck_02", ["destroy", ["ttruck_02", 1]],

"bramp", ["shutdown", ["owens_01", 0, 1]],
"bramp", ["delayupdate", ["c1m1dialogue4_play", 22.0]],
"bramp", ["delayupdate", ["truck_go", 18.0]],
"bramp", ["sound_once", ["com_mech", 1]],

"comcntr_down", ["delayupdate", ["c1m1dialogue7_sound", 3.0]],
"c1m1dialogue7_sound", ["sound_once", ["snd_c1m1dialogue7", 1]],

"reach_mfb", ["delayupdate", ["reach_mfb_sound", 2.0]],
"reach_mfb_sound", ["sound", ["snd_c1m1dialogue8", 1]]
```

The four `bramp` entries are four actions attached to one trigger name, not
four competing values. The current corpus contains no AIV VList that emits
`bramp`; that channel can therefore be supplied by a mission script, a base
Reader, or another runtime system. This is a normal consequence of the
shared registry and layered Reader loading.

One malformed-looking record is also present in the wider corpus:

```text
":m3n2go_delay", [":", "shutdown", ["avatar_02", 0, 1], ":"]
```

The editor keeps this raw record instead of guessing that `":"` is a valid
action type.

## 3. `aiv.zrd` VLists: state producers and region predicates

`params`, `p2`, `p6`, `p8`, and the other AIV parameter arrays do not contain a
named valve reference. The connection is through the vehicle’s VList.

The VList parser is `FUN_004df730`; the runtime evaluator is
`FUN_004d8e70`. For the 17-value rows used by the current editor, the fields
that matter to the valve graph are:

| VList index | Confirmed role |
|---:|---|
| `0` | Desired state value written to the channel (the corpus uses `0`/`1`). |
| `8` | One of the region inside/outside selector flags. |
| `9` | Region name, resolved as a `reg_*.zrd` geometry reference. |
| `15` | Named state channel to update. |
| `16` | The second region inside/outside selector flag. |

The other indices participate in the condition (thresholds, timing and AI
parameters), but their old editor captions are not needed to establish the
cross-file link. The evaluator first checks whether the requested state is
already current, applies the region/other predicates, and then calls the
registry write path. A VList is therefore a **producer** of a named channel,
not a `valves.zrd` action by itself.

### Actual VList records

Both c1m1 trucks contain this VList row:

```text
ttruck_01.vlist[0] =
[1, 0, 0.25, 0, 1, 0.9990000129, 0, 0.8000000119,
 0, 0, 0, 0.0, 0, 60.0, 1, "truck_explode", 0]

ttruck_02.vlist[0] =
[1, 0, 0.25, 0, 1, 0.9990000129, 0, 0.8000000119,
 0, 0, 0, 0.0, 0, 60.0, 1, "truck_explode", 0]
```

Both vehicles consequently write the same `truck_explode` state channel.
The name is not automatically tied to either vehicle; the vehicle identity is
outside the VList row.

The player VLists in the same Reader show the region form explicitly:

```text
[1, ..., 1, "reg_mfb",          ..., 1, "reach_mfb",     0]
[0, ..., 1, "reg_inside_c1m1",  ..., 1, "inside_c1m1",   0]
[1, ..., 0, "reg_inside_c1m1",  ..., 1, "inside_c1m1",   1]
[1, ..., 0, "reg_outside_c1m1", ..., 1, "outside_c1m1",  1]
```

Here the ellipses are unchanged serialized VList fields; the shown positions
are indices `0`, `8`, `9`, `15`, and `16`. `FUN_004c57d0` performs the region
containment test. The two selector bits are combined by the evaluator as the
inside/outside predicate; their old dialog captions are not inferred from the
channel names.

## 4. `net_*.zrd`: channel conditions and assignments

Network records are directed graphs. A spatial node contains a position and
an adjacency list. A record whose first value is `[from,to]` is an **edge
rule**, not another map location.

The executable parser is `FUN_004db1b0`. The valve-related tags have these
exact layouts:

| Tag | Layout | Runtime meaning |
|---|---|---|
| `valve` | [enabled, channel, target?] | If `enabled` is non-zero, wait for `channel == target`. When the third value is omitted, the navigation code derives the target from the current outgoing-link index: `current_index == link_count - 1`. |
| `valveunion` | [[enabled, channel, target?], ...] | Ignore members whose first value is zero; default a missing member target to `1`; evaluate the remaining `(channel,target)` pairs as OR via `FUN_004d4600`. |
| `valve_assign` | [channel, value] | Directly writes the named channel. There is no leading enable flag. |
| `gate` | [vehicle] | Vehicle-qualified edge rule, not a channel condition. |

This is why the leading `1` in `[1,"truck_go"]` is not itself the desired
state. It enables the condition. On a node with one outgoing link the omitted
target happens to derive to `1`; with a different link count it need not.

### Actual c1m1 `net_01` chain

The relevant records are:

```text
node_00 ... "valveunion", [[1,"bramp",1], [1,"truck_explode",1]]
node_01 ... "wait_interval", [1,1.0],
             "valveunion", [[1,"bramp",1], [1,"truck_explode",1]]

node_08 ... "valve", [1,"truck_go"]
node_09 ... "valve", [1,"truck_go"]

[7,8]  "gate",         ["ttruck_01"]
[7,9]  "gate",         ["ttruck_02"]
[8,15] "valve_assign", ["destroy_truck_01",1]
[9,16] "valve_assign", ["destroy_truck_02",1]
```

The directed route is:

```text
ttruck_01: 0 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 15
ttruck_02: 1 -> 12 -> 13 -> 14 -> 5 -> 6 -> 7 -> 9 -> 16
```

The static data proves the vehicle-qualified branches and the two distinct
assignments. It does **not** prove a generic visit-count/round-robin rule for
choosing `[8,9]`; that selection happens later in the navigation runtime. It
also does not prove whether the assignment fires on edge entry or edge
completion. The editor therefore displays the rule on the directed edge and
does not invent a timing label.

Combining the files gives this state path:

```text
valves.zrd: bramp --delay 18 s--> truck_go
        -> net_01 node_08/node_09 valve condition
        -> edge valve_assign destroy_truck_01/02 = 1
        -> valves.zrd destroy action
        -> AIV vehicle ttruck_01/ttruck_02
```

The `truck_explode` VList output is a separate producer of the same global
namespace and is one of the two members of the `valveunion` at nodes 0 and 1.

## 5. `reg_*.zrd`: geometry used by VLists

`reg_*.zrd` files use the same ordered `version`/`name`/`node_*` pair-list
structure and the same AI-network parser (`ZRdrAiNet`). In the supplied
corpus their nodes contain positions and adjacency, but no optional valve
tags. A region is not a valve definition; it becomes relevant when a VList
names it at index 9.

For c1m1, the Reader contains `reg_inside_c1m1` and
`reg_outside_c1m1`. A shortened `reg_outside_c1m1` record is:

```text
"name", ["reg_outside_c1m1"],
"node_00", [12, [-3810.5,-31.0,-376.5], [1]],
"node_01", [12, [-848.9,9.8,-369.5], [2]],
...
"node_05", [12, [-5144.7,7.6,-1357.8], [0]]
```

The VList region predicate turns containment in this geometry into a channel
transition. In the same mission, the resulting `outside_c1m1` channel is
observed by the objective failure record below.

## 6. `targets.zrd`: health as a channel producer

`targets.zrd` uses `health_below` as a flattened sequence of triples:

```text
health_below = [channel_name, state_value, health_threshold, ...]
```

The parser (`FUN_004e08c0`) resolves each channel with `FUN_004d3c40` and
creates one health transition per triple. In c1m1:

```text
"satdish", [
  2, "destroy_anim", ["destroy_comm_radar"],
  "target", ["satdish"],
  "centroid", ["satdish"],
  "health", [35.0],
  "health_below",
    ["comcntr_hit", 1, 35.0,
     "comcntr_down", 1, 0.0]
]
```

So the target-health system writes `comcntr_hit = 1` when the threshold 35.0
is crossed and `comcntr_down = 1` at 0.0. These names are then consumed by
both `valves.zrd` and `objectives.zrd`; no direct target-to-objective pointer
exists.

## 7. `objectives.zrd` and `mpinfo.zrd`: channel observers

Mission objectives do not perform the state transition. A `valve_change`
record observes a named channel and completes/fails its enclosing objective
phase when the channel equals the requested value.

The c1 campaign `objectives.zrd` contains this m1 structure (abridged only in
the localized description text):

```text
"m1", [
  "primary", ["valve_change", [
    "desc", ["MSG_OBJ_0xA00"],
    "nav",  ["Able"],
    "valve", ["comcntr_down"],
    "value", [1]
  ]],
  ...,
  "final", ["valve_change", [
    "desc", ["MSG_OBJ_0xA03"],
    "nav", ["Baker"],
    "valve", ["reach_mfb"],
    "value", [1]
  ]],
  "fail", ["valve_change", [
    "desc", ["MSG_OBJ_LEFT_MISSION_BOUNDARIES"],
    "valve", ["outside_c1m1"],
    "value", [1]
  ]]
]
```

The same observer shape occurs in multiplayer `mpinfo.zrd`; for example
`zbd/c3/readermp3.zbd` contains `valve_change` on `outside_mp3c3` with value
`1`. `objectives.zrd` and `mpinfo.zrd` therefore sit on the consumer side of
the same channel registry as networks and VLists.

## 8. Three complete data-driven chains

### A. Destroying the c1m1 communication centre

```text
targets.zrd satdish.health_below
  -> comcntr_down = 1
  -> valves.zrd comcntr_down / delayupdate (3.0 s)
  -> c1m1dialogue7_sound / sound_once
  -> snd_c1m1dialogue7
  -> objectives.zrd m1.primary valve_change(comcntr_down, 1)
```

One health event can therefore both play mission feedback and satisfy an
objective. Neither consumer knows that `comcntr_down` originated in
`targets.zrd`; they only see the shared name.

### B. Reaching the MFB and leaving the mission area

```text
player VList: reg_mfb -> reach_mfb = 1
  -> valves.zrd reach_mfb / delayupdate / sound
  -> objectives.zrd m1.final valve_change(reach_mfb, 1)

player VList: reg_outside_c1m1 -> outside_c1m1 = 1
  -> objectives.zrd m1.fail valve_change(outside_c1m1, 1)
```

The region geometry is the predicate; the objective and valve records are
only named-channel consumers.

### C. The two mission trucks

```text
external/script or another Reader: bramp = 1
  -> valves.zrd delayupdate truck_go (18.0 s)
  -> net_01 node_08/node_09 valve(truck_go)
  -> edge 8->15 or 9->16 valve_assign destroy_truck_01/02 = 1
  -> valves.zrd destroy_truck_01/02
  -> destroy ttruck_01/ttruck_02
```

Separately, each truck VList writes `truck_explode`; that channel is an OR
member of the `valveunion` on nodes 0 and 1. The gate records select the
vehicle-specific outgoing edge; they are not additional valve states.

## 9. Layering and reference resolution

The editor resolves references across the selected mission Reader, campaign or
training base `reader.zbd`, global `zbd/reader.zbd`, and explicitly attached
dependency Readers. A channel may therefore be defined in one archive and
consumed in another. Do not report a missing same-file producer as a broken
link until the complete Reader stack and mission scripts have been checked.

The editor’s reference index intentionally reports roles separately:

| Role | Examples |
|---|---|
| `writes-valve` | VList index 15, `health_below`, `valve_assign`, objective `set_valve` |
| `writes-valve-delayed` | `valves.zrd` `delayupdate` |
| `reads-valve` | net `valve`, `valveunion`, `valves.zrd` namelist aggregates |
| `handles-valve` | outer `valves.zrd` event/output name |
| `observes-valve` | objective `valve_change` |
| `uses-region` | VList index 9 |
| `uses-vehicle` | `gate`, `destroy`, `shutdown`, `teleport`, `attack_strategy` |

## 10. What is still deliberately unresolved

The relationships above are sufficient to follow real mission chains, but a
few details should not be guessed in the editor:

* the exact immediate-value semantics of the binary-recognized `update`
  action and the unobserved mission/animation actions;
* whether a net `valve_assign` fires on edge entry or edge completion;
* the generic outgoing-edge selection algorithm when several links are
  eligible (the serialized data alone does not prove round-robin behavior);
* old dialog captions for the two VList region selector fields;
* any action type or argument shape not listed in the observed corpus.

Unknown records remain raw and round-trip unchanged. The primary executable
anchors used for this document are `FUN_004d3c40`, `FUN_004d3c00`,
`FUN_004d4490`, `FUN_004d4600`, `FUN_004df730`, `FUN_004d8e70`,
`FUN_004db1b0`, `FUN_004e08c0`, and the `ZRdrValve.cpp` construction path at
`FUN_004e2330`/`FUN_004e44b0`.
