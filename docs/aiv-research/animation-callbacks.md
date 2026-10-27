# Animation callbacks

## Scope

This document describes animation callback events in the live animation data
from each campaign's `anim.zbd`. The recovered definitions are stored under
`extracted-anim-json/`. Animation records embedded in Reader archives are not
used by the game for this path.

Recovered JSON field names remain provisional. Binary layouts and executable
behavior described here are based on the `Mech3.exe` implementation and are
identified separately from corpus-based interpretations.

## Callback event format

Animation event type `35` (`0x23`, `CALLBACK`) contains one four-byte unsigned
value. A definition containing such an event sets `has_callbacks`.

The event interpreter dispatches callbacks as follows:

```text
FUN_005079b0  animation event interpreter
  event type 35
    -> FUN_00507410
       if animation_runtime.callback_fn != NULL:
           callback_fn(callback_context, event.value)
```

`FUN_00508f00` installs the callback:

```text
animation_runtime + 0x6c = callback_fn
animation_runtime + 0x70 = callback_context
```

The meaning of a callback value therefore depends on the installed callback
handler. It is not a global enum interpreted identically by every animation
owner.

## Installed callback handlers

All direct callers of `FUN_00508f00` in `Mech3.exe` were checked.

| Installed callback | Install sites | Callback-value handling |
|---|---|---|
| `0x004d4b10` -> `FUN_004d4a20` | `0x004d4c27`, `0x004d4ef5`, `0x004d509c`, `0x004d50af` | Reader/state path; explicitly handles `42..49` and `57..60`. |
| `0x0045a0f0` | `0x0045a21e` | Tests only value `0`; clears animation state and invokes owner cleanup. |
| `0x00461c90` | `0x00461c5f`, `0x00461c71`, `0x00466151`, `0x00466163` | Ignores the numeric value; unlinks callback/context pointers. |
| `0x00583120` | `0x00585600` | Ignores the numeric value; clears `context + 0x18`. |

The Reader/state handler first forwards every callback to an owner-object
virtual method. A value may therefore have an owner-specific side effect in
addition to the explicit behavior documented below.

## State-channel callbacks 42-49

The Reader/state subsystem installs `0x004d4b10`, a thunk to
`FUN_004d4a20`. Callback values `42..49` select a scene node and write its
exact name as a state channel through `FUN_004d3c00`.

The callback value encodes both ancestor depth and value:

```text
ancestor_depth = (callback - 42) >> 1
state_value    = callback & 1
```

| Callback pair | Selected node | Value |
|---|---|---|
| `42` / `43` | Resolved node itself | `0` / `1` |
| `44` / `45` | Parent | `0` / `1` |
| `46` / `47` | Grandparent | `0` / `1` |
| `48` / `49` | Great-grandparent | `0` / `1` |

Only `43` and `47` occur in the currently extracted animation corpus. The
other values are supported by the executable but were not observed.

### Channel resolution

Channels are resolved while callbacks are installed, not when an animation
event fires:

1. `FUN_0050b5e0` returns the animation runtime record's resolved scene-node
   pointer at `+0x40`.
2. For each requested ancestor step, collector `0x004d4c80` requires exactly
   one parent at node `+0x54` and follows the first pointer in the array at
   `+0x58`.
3. If a step has zero or multiple parents, or a null parent pointer, the event
   does not bind a state channel.
4. The selected node pointer is passed to `FUN_004d3c40` as a C string. A node
   begins with its exact, case-sensitive name.
5. `FUN_004d3c40` returns the existing named state channel or creates it when
   absent. The resulting channel pointer is stored in the callback context.
6. When the event fires, `FUN_004d4a20` writes the callback's binary value to
   that prebound channel.

The `gamez.zbd` reader confirms the node relationship fields:

| Node offset | Meaning |
|---:|---|
| `+0x54` | Parent count |
| `+0x58` | Parent-pointer array |
| `+0x5c` | Child count |
| `+0x60` | Child-pointer array |

`FUN_004feaf0` loads both arrays. `FUN_004ff7a0` recursively walks
`+0x5c/+0x60` as children, while `FUN_004ed8b0` follows `+0x54/+0x58` toward
the root. The callback collector follows the parent path.

### Animation record fields

`FUN_0050a0e0` loads version-39 `anim.zbd` records. Each definition has a
fixed `0x13c`-byte record followed by variable arrays.

| Record offset | Meaning | Extracted JSON field |
|---:|---|---|
| `+0x00` | Animation definition name | `anim_name` |
| `+0x20` | Scene-node lookup name | `name` |
| `+0x40` | Resolved scene-node pointer used by callback collection | `ptrs.anim_ptr` in serialized data; resolved again while loading |
| `+0x44` | Animation root-node lookup name | `anim_root_name` |
| `+0x64` | Resolved animation root-node pointer | `ptrs.anim_root_ptr` in serialized data |

The loader resolves `+0x20` through `FUN_004f8bd0`. Consecutive animation
records with the same lookup name advance through successive exact-name node
matches. This pairs repeated definitions with distinct same-named scene nodes.

The callback path does not use `anim_refs`, `anim_name`, or `anim_root_name`
to derive the state-channel name.

### Resolution examples

Depth-zero callback:

```text
animation: platform_up
record name: shaftkey
callback: 43

resolve node "shaftkey"
select depth 0: "shaftkey"
write state channel shaftkey = 1
```

Ancestor callback from c4:

```text
brgprt#0: brgprt -> m2brg01 -> world1
brgprt#1: brgprt -> m2brg02 -> world1
```

Both definitions use callback `47`, which selects the grandparent. Both bind
and write `world1 = 1`.

## Other callbacks handled by the Reader/state subsystem

The same installed subsystem also receives the following callback values, but
they do not write named state channels through `FUN_004d3c00`:

| Values | Route |
|---|---|
| `50..55` | Ignored by the Reader/state collector. |
| `56` | Collector binds the game object that owns the animation's scene node; at runtime that object's virtual method `+0x74` receives the callback (see below). |
| `57..60` | Runtime handler calls `FUN_004c3bd0`, which maps `57 -> 1`, `58 -> 0`, `59 -> 3`, and `60 -> 2`, then calls `FUN_004c2a30`. This is a separate transition-mode path, not a named state-channel write. |

Callbacks `50..56` may still be handled by the animation owner's virtual
callback method.

### Callback 56: owner-object binding

Source: installed `Mech3.exe`, SHA-256
`95bc2c114c9b2e5c5ada8e40ca78cc79470f862e45313ffad0d1e8e6d4d916bf`, Ghidra 12.1.4.

The collector at `0x004d4c80` is not recognized as a function by Ghidra's
auto-analysis; it was decoded from bytes. It dispatches through a byte class
table at `0x004d4e04` (values `0..0x3c`) and a jump table at `0x004d4df0`:

| Values | Target | Behavior |
|---|---|---|
| `0`, `56` | `0x004d4db1` | Owner-object binding (below). |
| `42..49` | `0x004d4d5c` | Ancestor walk and state-channel binding. |
| `57..60` | `0x004d4de3` | Marks the event bound; no lookup. |
| everything else | `0x004d4de7` | Ignored. |

The `0`/`56` path:

```text
node  = FUN_0050b5e0(anim_context)      ; scene node at anim record +0x40
owner = thunk 0x0040eee0 -> FUN_0040e920(&node)
        ; node+0x28 bit 16 set -> node+0x40, else FUN_0040e960 / e990 / e8f0
if owner == 0                       -> not bound
if slot already holds another owner -> not bound
else mark bound, slot = owner
```

At runtime `FUN_004d4a20` first calls `(*owner->vtable[0x74])(param_1, value)`
for every callback before its own `switch`. `56` has no `case` there, so its
only effect is that virtual call on the bound game object. Which classes
implement `+0x74`, and what they do with `56`, is not yet traced.

## Corpus audit

The audit parsed all 9,395 extracted definitions. Exactly 167 definitions set
`has_callbacks`, and those definitions contain all 198 explicit callback
events. No definition has the flag without an event or an event without the
flag.

| Archive | Definitions | Callback definitions | Callback events | Values |
|---|---:|---:|---:|---|
| C1 | 2,133 | 36 | 46 | 43, 50, 56, 57, 58 |
| C2 | 1,719 | 20 | 22 | 43, 50, 56, 57, 58 |
| C3 | 1,663 | 68 | 81 | 43, 50, 51, 52, 56, 57, 58 |
| C4 | 1,706 | 30 | 36 | 43, 47, 50, 51, 56, 57, 58 |
| C4B | 1,077 | 13 | 13 | 50, 51, 56 |
| T1 | 1,097 | 0 | 0 | none |

### Observed values and probable uses

Probable gameplay uses are inferred from all observed animation contexts. They
are not recovered enum names.

| Value | Uses | Observed contexts | Probable use | Confidence |
|---:|---:|---|---|---|
| 43 | 87 | Scenery swaps, ramps, bridges, building destruction, and Reader-matched definitions | Select depth-zero node channel and write `1`. | High for the state route; owner callback may also observe it. |
| 47 | 2 | c4 `brgprt#0` and `brgprt#1` | Select grandparent channel `world1` and write `1`. | High. |
| 50 | 25 | `activate`, `raise_*`, and `turn_on_*` turret animations | Activate or raise a turret/defence object. | Medium-high from corpus context; exact owner handler remains unidentified. |
| 51 | 6 | `deactivate`, `turnoff*`, `lowr*`, and turret destruction | Deactivate, lower, or mark a turret/defence object offline. | Medium-high from corpus context; exact owner handler remains unidentified. |
| 52 | 1 | `destroy_tur_wallppc02` | Specialized destroyed-turret notification. | Medium-low; one sample. |
| 56 | 43 | Doors, object destruction, dropships, rail collapse, targets, and buildings | General object transition/destruction notification. | Medium-high for the collection route; medium for precise gameplay meaning. |
| 57 | 17 | Bridge destruction and secondary bridge/ramp collapse | Transition mode `1` through `FUN_004c3bd0`. | Medium-high for the executable route. |
| 58 | 17 | Usually paired with `57` in the same definitions | Transition mode `0` through `FUN_004c3bd0`. | Medium-high for the executable route. |

## Reader/state-channel relationships

All extracted animation definitions were compared with all campaign,
multiplayer, and instant-action `valves.zrd` files. The audit covered 46 Reader
valve files and the 8,254 definitions with a recovered `name`.

There are 12 exact matches between Reader valve events and animation record
`name` values after duplicate handlers are collapsed. Eight matching
definitions contain callback `43`:

| Campaign / Reader | Channel | Animation | Root | Callback |
|---|---|---|---|---:|
| C1 / `readerm1` | `bramp` | `call_ramp` | `healthy` | 43 |
| C2 / `readerm1` | `platmid` | `launch_noruff` | `platmid` | 43 |
| C2 / `readerm1` | `shaftkey` | `platform_up` | `shaftkey` | 43 |
| C3 / `readerm5` | `cl01` | `cl01_dest` | `healthy01` | 43 |
| C3 / `readerm5` | `cl02` | `cl02_dest` | `healthy02` | 43 |
| C3 / `readerm5` | `cl03` | `cl03_dest` | `healthy03` | 43 |
| C3 / `readerm5` | `cl04` | `cl04_dest` | `healthy04` | 43 |
| C3 / `readerm5` | `cl05` | `cl05_dest` | `healthy05` | 43 |

The four additional name matches are callback-free variants of `bramp`,
`platmid`, and `shaftkey`.

No Reader valve event matches an `anim_name`. The state callback can also
create and write channels that no Reader file consumes, so Reader-name matches
represent only the statically visible subset of animation-produced channels.

Reader valve actions do not contain an animation-call operation. Animation
callbacks produce named state changes; Reader valve actions consume those
channels independently.

## Remaining work

- Identify the owner-object handlers responsible for the gameplay effects of
  callbacks `50`, `51`, `52`, and `56`.
- Refine the gameplay interpretation of transition modes used by callbacks
  `57..60`.
- Capture runtime samples where useful to confirm owner-specific side effects
  in addition to the explicit state-handler behavior.
