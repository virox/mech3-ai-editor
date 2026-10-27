# Animation callbacks and C2M1 elevator/rocket sequence

## Scope and confidence

This note uses the individual definitions recovered from each campaign's live
`anim.zbd`, under `extracted-anim-json/`. These are the definitions used by the
game; the animation records in the Reader archives are not the source for this
analysis.

The names assigned by the extractor should still be treated as provisional
unless stated otherwise. The callback event's binary shape, however, is
confirmed by the `mech3ax` parser: event type 35 (`CALLBACK`) contains exactly
one four-byte unsigned `VALUE`, and the owning animation definition must set
the `has_callbacks` flag.

## C2M1 startup sequence

The single-player startup animation calls the rocket/elevator controller:

```text
horizon / start_single_player
  -> range / call_rocket
       -> platmid / launch_noruff
       -> shaftkey / platform_up
```

Relevant definitions:

- `c2/anim/horizon-start_single_player.json` calls `call_rocket`.
- `c2/anim/range-call_rocket.json` calls both `launch_noruff` and
  `platform_up`.
- `c2/anim/platmid-launch_noruff.json` moves `platmid` for 6 + 10 seconds.
- `c2/anim/shaftkey-platform_up.json` moves `plattop` from Y=-55 to Y=16 over
  15 seconds.

Both `launch_noruff` and `platform_up` contain an immediate `CALLBACK` event
with value `43`.

## Executable callback path

The executable trace changes the confidence level substantially. The animation
event runner is `FUN_005079b0` in `Mech3.exe`; event type `0x23`/35 dispatches
to `FUN_00507410`. That handler does not directly know about valves:

```text
FUN_005079b0 animation event interpreter
  event type 0x23 / 35
    -> FUN_00507410
       if animation_runtime.callback_fn != NULL:
           callback_fn(callback_context, event.value)
```

The callback function slot is installed by `FUN_00508f00`:

```text
FUN_00508f00(animation, callback_fn, callback_context)
  animation + 0x6c = callback_fn
  animation + 0x70 = callback_context
```

All direct callers of `FUN_00508f00` in `Mech3.exe` were checked. The known
installed callback functions are:

| Installed callback | Install sites | Uses the callback value? | Notes |
|---|---|---|---|
| `0x004d4b10` -> `FUN_004d4a20` | `0x004d4c27`, `0x004d4ef5`, `0x004d509c`, `0x004d50af` | Yes | Reader/state callback path. Explicitly handles `42..49` and `57..60`. |
| `0x0045a0f0` | `0x0045a21e` | Only checks for value `0` | Clears a global/current animation pointer and calls owner cleanup methods when the callback value is zero. No `43` handling. |
| `0x00461c90` | `0x00461c5f`, `0x00461c71`, `0x00466151`, `0x00466163` | No | Unlinks callback/context pointers from animation-related structures. It ignores the numeric callback value. |
| `0x00583120` | `0x00585600` | No | Clears `context + 0x18`. It ignores the numeric callback value. |

The Reader/state subsystem installs `callback_fn = 0x004d4b10`. That function
is only a thunk; it forwards to `FUN_004d4a20`, passing the installed context
and the callback event value.

`FUN_004d4a20` is the confirmed state-channel callback handler. It first gives
the owning object a chance to see every callback through a virtual method, then
handles these state-related callback codes itself:

| Callback values | State action |
|---|---|
| `42`, `44`, `46`, `48` | Write resolved state channel to `0`. |
| `43`, `45`, `47`, `49` | Write resolved state channel to `1`. |

The actual write is through the known shared state writer `FUN_004d3c00`.
Therefore callback `43` is not an arbitrary label and not a direct valve name:
it is the "set selected animation state channel to 1" code for this installed
callback handler.

No other direct installed animation callback function has an explicit `43`
case, and a targeted code scan did not find another immediate compare against
`0x2b`. The remaining caveat is the owner-object virtual notification at the
top of `FUN_004d4a20`: every Reader/state callback value, including `43`, is
forwarded there before the state write. So the executable-backed statement is:
`43` has no other explicit installed callback-handler use found, but it may
still be observed by an owning object as a side notification.

The selected channel is resolved before runtime callback execution. During
callback installation, `FUN_00508f40` walks callback events in candidate
animations and calls collector `0x004d4c80`. For callback values `42..49`, the
collector resolves a channel through `FUN_004d3c40`.

The exact source is narrower than the earlier "linked animation reference"
description. `FUN_0050b5e0` returns the animation runtime record's resolved
class/model pointer at `+0x40`. The c4 `anim.zbd` loader at `FUN_0050a0e0`
builds that pointer from the record's class/model lookup name at `+0x20` by
calling `FUN_004f8bd0`. It is not the record's `anim_refs` array.

The callback pair chooses an ancestor depth in that resolved scene-node
hierarchy:

| Callback pair | Hierarchy depth | Runtime value |
|---|---:|---|
| `42` / `43` | 0 (resolved named node itself) | `0` / `1` |
| `44` / `45` | 1 (parent) | `0` / `1` |
| `46` / `47` | 2 (grandparent) | `0` / `1` |
| `48` / `49` | 3 (great-grandparent) | `0` / `1` |

For each depth step, the collector requires the current node's parent count at
`+0x54` to be exactly `1`, reads the parent-pointer array at `+0x58`, and
follows its first pointer. If the count is zero, greater than one, or the parent
pointer is null before the requested depth is reached, no state channel is
bound for that event. The `gamez.zbd` reader `FUN_004feaf0` loads these two
fields as one node-index array and loads the child count/array separately at
`+0x5c` / `+0x60`; `FUN_004ff7a0` recursively walks the latter as children.
`FUN_004ed8b0` independently follows `+0x54` / `+0x58` to obtain a root node
and logs `Error getting root node. Multiple...` when a step has more than one
parent. Together these paths confirm that the callback walks parents, not
animation refs or child nodes.

After reaching the requested ancestor depth, the node pointer itself is passed to
`FUN_004d3c40` as a C string. The node therefore begins with its state/name
string; that exact, case-sensitive name is looked up and created if absent.
For depth 0 this is the resolved node's name, which matches the animation
record's class/model lookup name: callback `43` on
`shaftkey / platform_up`, `bramp / call_ramp`, and `cl01..cl05` consequently
binds channels with those same names.

At runtime the callback handler does not walk the hierarchy again. The
collector has already stored one resolved channel pointer in the installed
callback context; the even/odd callback value then only selects `0` or `1`.

### `anim.zbd` record fields relevant to state callbacks

`FUN_0050a0e0` confirms that version-39 `anim.zbd` records have a fixed
`0x13c`-byte header followed by variable arrays. The relevant fixed fields are:

| Record offset | Meaning | Extracted JSON field |
|---:|---|---|
| `+0x00` | Animation/sequence definition name | `anim_name` |
| `+0x20` | Class/model lookup name | `name` |
| `+0x40` | Runtime-resolved class/model pointer (the callback walk starts here) | `ptrs.anim_ptr` in the serialized data; replaced/resolved while loading |
| `+0x44` | Root-node lookup name | `anim_root_name` |
| `+0x64` | Runtime-resolved root-node pointer | `ptrs.anim_root_ptr` in the serialized data |

For example, the attached c4 archive contains these two fixed records:

```text
brgprt#0: +0x00 "brgprt#0", +0x20 "brgprt", +0x44 "healthy"
brgprt#1: +0x00 "brgprt#1", +0x20 "brgprt", +0x44 "healthy"
```

Their callback `47` requests ancestor depth 2 from the node resolved from
`"brgprt"`. The animation loader deliberately advances to the next exact-name
match when consecutive records use the same `+0x20` name, so the two records
resolve distinct `brgprt` nodes. Parsing the corresponding c4 `gamez.zbd`
parent arrays gives the exact chains:

```text
brgprt#0 record: brgprt -> m2brg01 -> world1
brgprt#1 record: brgprt -> m2brg02 -> world1
```

Both callback-47 events therefore prebind the state channel `world1` and write
it to `1`. Neither resolves `brgprt#0`, `brgprt#1`, `healthy`, nor an
`anim_refs` entry.

The collector's dispatch table is also useful negative evidence:

- `42..49` are the only state-channel prebind family.
- `50..55` are ignored by the Reader/state collector, so they are not state
  channel writes through this path.
- `56` is routed through `FUN_0040eee0` during collection.
- `57..60` are routed at runtime to `FUN_004c3bd0`, which maps `57 -> 1`,
  `58 -> 0`, `59 -> 3`, and `60 -> 2`, then calls `FUN_004c2a30`.

Only callback values `43` and `47` from this state-write family occur in the
current extracted MechWarrior 3 animation corpus. Values `42`, `44`, `45`,
`46`, `48`, and `49` are supported by the executable path but were not observed
in these files.

This also explains the apparent mismatch between 87 uses of callback `43` and
only 12 Reader valve-name matches: the animation callback path can create or
set state channels even when no Reader file consumes them. The 12 matches are
only the statically visible cases where a Reader valve handler uses the same
channel.

## C2M1 callback-to-valve evidence

`readerm1/valves.zrd` has handlers for both `platmid` and `shaftkey`.

| Animation root | Animation | Callback | Matching C2M1 valve | Observable script effect |
|---|---|---:|---|---|
| `platmid` | `launch_noruff` | 43 | `platmid` | Starts `c2m1dialogue15` after 5 seconds. |
| `shaftkey` | `platform_up` | 43 | `shaftkey` | Holds Thor, then teleports him after 9 seconds and signals `thor_moves` after 14 seconds. |

The `thor_moves` network gate begins at `(379.9, 16.0, -895.5)`, the upper
position of the `plattop` lift. The 14-second script delay therefore tracks the
15-second lift animation and is not a dropship flight delay.

### Current interpretation

The data does **not** encode `43 -> shaftkey`. Instead, the executable installs
a state callback on the animation, resolves the animation's primary name to a
state channel, and then uses callback value `43` as the write value selector:

```text
platform_up in definition shaftkey
  -> CALLBACK(43)
  -> event runner calls installed state callback
  -> callback handler writes resolved channel "shaftkey" to 1
  -> C2M1 valve actions subscribed to "shaftkey" run
```

This is now an executable-backed interpretation. Callback `43` is used broadly
by scenery animations because it is a generic "set the selected animation state
channel to 1" event, not because every use has a visible `valves.zrd` handler.

The multiplayer variant, `shaftkey / multi_platform_up`, has no callback event.
That is consistent with the single-player-only Thor script, but is supporting
evidence rather than proof of the dispatch mechanism.

## All mission-folder Reader/animation/valve comparison

Every extracted live animation definition was compared with every mission,
multiplayer, and instant-action `valves.zrd` file in the corresponding
campaign. The comparison distinguishes the animation definition `name`, the
`anim_name`, and the `anim_root_name`; they are not interchangeable.

The corpus contains 9,395 animation definitions and 46 Reader valve files.
The name comparison applies to the 8,254 definitions with a recovered
definition name. There are no valve-event-to-`anim_name` matches. After
deduplicating repeated handlers for the same valve, there are 12 exact
valve-event-to-definition-name matches; eight contain callback 43:

| Campaign / Reader | Valve event | Definition name | Animation | Root | Callback values |
|---|---|---|---|---|---|
| C1 / `readerm1` | `bramp` | `bramp` | `call_ramp` | `healthy` | 43 |
| C1 / `readerm1` | `bramp` | `bramp` | `hit_ramp` | `bramp` | none |
| C1 / `readerm1` | `bramp` | `bramp` | `multi_ramp` | `bramp` | none |
| C2 / `readerm1` | `platmid` | `platmid` | `launch_noruff` | `platmid` | 43 |
| C2 / `readerm1` | `platmid` | `platmid` | `platmid_blowup` | `healthy` | none |
| C2 / `readerm1` | `shaftkey` | `shaftkey` | `platform_up` | `shaftkey` | 43 |
| C2 / `readerm1` | `shaftkey` | `shaftkey` | `multi_platform_up` | `shaftkey` | none |
| C3 / `readerm5` | `cl01` | `cl01` | `cl01_dest` | `healthy01` | 43 |
| C3 / `readerm5` | `cl02` | `cl02` | `cl02_dest` | `healthy02` | 43 |
| C3 / `readerm5` | `cl03` | `cl03` | `cl03_dest` | `healthy03` | 43 |
| C3 / `readerm5` | `cl04` | `cl04` | `cl04_dest` | `healthy04` | 43 |
| C3 / `readerm5` | `cl05` | `cl05` | `cl05_dest` | `healthy05` | 43 |

The callback-backed rows form the visible subset of a broader executable rule:
`CALLBACK(43)` can set the animation runtime's primary state channel to `1`.
The root is not the dispatch key: C1M1's `call_ramp` has root `healthy`, but
its definition name and consumed valve event are both `bramp`.

Reader valve actions contain no animation-call operation; the observed action
names are `all_nonzero`, `all_zero`, `any_nonzero`, `attack_strategy`,
`delayupdate`, `destroy`, `namelist`, `shutdown`, `sound`, `sound_once`, and
`teleport`.

### C1M1 `bramp`: no alternate Reader producer found

`bramp` is the decisive case. Its callback-bearing animation is
`bramp / call_ramp`, with definition name `bramp`, root `healthy`, and immediate
callback 43. It calls `hit_ramp` 20 seconds later.

All C1M1 Reader references to `bramp` were checked. There is no VList,
objective, target, or network `valve_assign` producer. The only non-handler
network occurrences are `net_04`'s `valve` condition and `net_01`'s
`valveunion` condition; both read `bramp`, they do not write it. This removes
the earlier proposed network-producer explanation.

The `bramp` callback therefore strongly supports the executable trace:
callback 43 writes the resolved animation state channel to `1`, and `bramp` is
the consumed channel in C1M1.

## Callback value corpus and probable uses

The extracted live animation corpus contains these callback values. The
`probable use` column is based on every observed animation context; it is not
a recovered executable enum.

The callback audit parsed all 9,395 definitions. Exactly 167 definitions set
`has_callbacks`, and those same 167 definitions contain all 198 explicit
`CALLBACK` events. There are zero definitions with the flag but no event, and
zero definitions with an event but no flag.

| Campaign archive | Definitions | Callback definitions | Callback events | Values |
|---|---:|---:|---:|---|
| C1 | 2,133 | 36 | 46 | 43, 50, 56, 57, 58 |
| C2 | 1,719 | 20 | 22 | 43, 50, 56, 57, 58 |
| C3 | 1,663 | 68 | 81 | 43, 50, 51, 52, 56, 57, 58 |
| C4 | 1,706 | 30 | 36 | 43, 47, 50, 51, 56, 57, 58 |
| C4B | 1,077 | 13 | 13 | 50, 51, 56 |
| T1 | 1,097 | 0 | 0 | none |

| Value | Uses | Observed animation contexts | Probable use | Confidence |
|---:|---:|---|---|---|
| 43 | 87 | Scenery swaps, bridge/ramp transitions, building destruction, C1/C2/C3 valve-matched animations | State callback value: write the resolved animation state channel to `1` when the Reader/state callback is installed. | High for state-write route; no other explicit installed handler use found. Caveat: owner-object virtual hook still observes the value. |
| 47 | 2 | `brgprt#0` and `brgprt#1` bridge-part animations | State callback value: write a later linked/alternate resolved animation state channel to `1`. | Medium-high for state-write route; exact linked name slot still needs a runtime sample. |
| 50 | 25 | `activate`, `raise_*`, and `turn_on_*` animations for chopper and fixed turrets | Activate or raise a turret/defence object. | Medium-high from consistent corpus context; not a Reader/state channel code, and exact object callback handler is not yet identified. |
| 51 | 6 | `deactivate`, `turnoff*`, `lowr*`, and some turret destruction animations | Deactivate, lower, or mark a turret/defence object destroyed/offline. | Medium-high from consistent corpus context; not a Reader/state channel code, and exact object callback handler is not yet identified. |
| 52 | 1 | `destroy_tur_wallppc02` | A specialized destroyed-turret notification, possibly a turret subtype or alternate turret slot. | Medium-low; corpus context is precise but only one sample, and exact handler is not yet identified. |
| 56 | 43 | Door opening, object destruction, dropship/rail collapse, target and building destruction | General object transition/destruction notification; collector routes it through `FUN_0040eee0`. | Medium-high for executable collection route; medium for exact gameplay meaning. |
| 57 | 17 | Bridge destruction and secondary bridge/ramp collapse variants | Bridge/destruction transition notification: `FUN_004c3bd0` maps `57` to mode `1` and calls `FUN_004c2a30`. | Medium-high for executable route; medium for exact gameplay meaning. |
| 58 | 17 | Same bridge destruction animations as `57`, usually paired in the same definition | Bridge/destruction transition counterpart: `FUN_004c3bd0` maps `58` to mode `0` and calls `FUN_004c2a30`. | Medium-high for executable route; medium for exact gameplay meaning. |

## Remaining verification work

The static executable path is now traced. Runtime verification should focus on
the remaining naming details and side effects:

1. Watch the `shaftkey` and `platmid` valves in Valve Switchboard during C2M1
   startup. They should change to `1` when callback `43` fires.
2. Break on `FUN_004d4a20` or `FUN_004d3c00` and capture the resolved channel
   pointer/name for `shaftkey`, `platmid`, `bramp`, and one `cl01..cl05` case.
3. Capture a callback `47` bridge-part case to identify which linked/alternate
   animation name slot it resolves.
4. Separately trace callback values `50`, `51`, `52`, `56`, `57`, and `58`, because
   their main effects appear to be handled by the owning object's callback
   method and/or `FUN_004c3bd0`, not by the simple state write cases.
