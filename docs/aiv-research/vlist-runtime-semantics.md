# `vlist` runtime semantics

## Evaluator

`FUN_004d8e70` evaluates one 32-byte runtime `vlist` entry. Its accesses match
the parser/runtime layout in `parameter-field-map.md` exactly.

The function arguments are provisionally named:

```c
evaluate_vlist_entry(
    VListEntry *entry,
    Entity *owner,
    Entity *subject,
    bool *candidate_initialized,
    Candidate **candidate);
```

The internal `Valve` editor resource and its live MFC data-exchange handler at
`004b9930` provide the original developer-facing names for every field.

## Entry layout with observed use

| JSON index | Runtime field | Enable flag | Observed use in `FUN_004d8e70` |
|---:|---:|---:|---|
| 0 | flags bit `0x02` | — | Desired boolean state passed to the final valve/state setter. |
| 1 | flags bit `0x04` | — | Enables the `Ammo` trigger using value 2. |
| 2 | float `+0x00` | index 1 | Fires when `owner->method_9c(subject) <= value`. |
| 3 | flags bit `0x08` | — | `Fired on` trigger: fires when `owner->method_a0() == 1`. |
| 4 | flags bit `0x10` | — | Enables the `Health` trigger using value 5. |
| 5 | float `+0x04` | index 4 | Fires when the subject's overall health fraction `<= value` (live-verified). |
| 6 | flags bit `0x20` | — | Enables the `Heat` trigger using value 7. |
| 7 | float `+0x08` | index 6 | Fires when `subject->method_9c() >= value`. |
| 8 | flags bit `0x100` | index 9 present | Region selector: passes while the subject is **inside** region 9. |
| 9 | resolved ref `+0x14` | — | Resolved `reg_*` region; tested against the subject position. |
| 10 | flags bit `0x40` | — | Fires when the candidate is closer than range 11 **and** visible. |
| 11 | float `+0x0c` | index 10 | Squared range for the visible-candidate trigger. |
| 12 | flags bit `0x80` | — | Fires when the candidate is closer than range 13, without a visibility test. |
| 13 | float `+0x10` | index 12 | Squared range for the proximity trigger. |
| 14 | flags bit `0x01` | — | `Active` in the editor; not directly tested in this evaluator path. |
| 15 | resolved ref `+0x1c` | — | Required named boolean valve/state; read first and set to the value from index 0 when the entry condition fires. |
| 16 | flags bit `0x200` | index 9 present | Region selector: condition is true while the subject is **outside** region 9. |

## Directly reconstructed control flow

Equivalent high-level pseudocode, retaining neutral method names. It is
reconstructed from the disassembly (`0x4d8e70..0x4d8fd1`): every condition
branch jumps to `0x4d8f75` (`BL = 1`), and `BL != 0` jumps straight to the
setter at `0x4d8fbc`. The conditions are therefore **OR-ed triggers**, not
rejection tests. An earlier revision of this file read them as rejections;
that reading contradicted the live C1M1 trace below.

```c
if (entry->ref15 == NULL)
    return;

current = entry->ref15->method_04();
if (current == entry->flag_index0)
    return;                                  // already at the desired value

fire = false;

if (entry->flag_index3 && owner->method_a0() == 1)
    fire = true;                             // Fired on
else if (entry->flag_index6 && subject->method_9c() >= entry->value7)
    fire = true;                             // Heat
else if (entry->flag_index4 && subject->method_98() <= entry->value5)
    fire = true;                             // Health
else if (entry->flag_index1 && owner->method_9c(subject) <= entry->value2)
    fire = true;                             // Ammo
else {
    if (!*candidate_initialized) {
        *candidate_initialized = true;
        *candidate = FUN_004d6b90(1);
    }
    if (*candidate != NULL) {
        if (entry->flag_index12 &&
            (*candidate)->distance_squared < entry->value13)
            fire = true;                     // Target Distance
        else if (entry->flag_index10 &&
            (*candidate)->distance_squared < entry->value11 &&
            has_line_of_sight(subject, (*candidate)->object))
            fire = true;                     // Target LOS
    }
}

if (!fire) {
    if (entry->region9 == NULL)
        return;                              // no trigger and no region: never fires
    inside = entry->region9->contains(subject->position());
    fire = (entry->flag_index8 && inside) ||
           (entry->flag_index16 && !inside);  // both set: no containment test
}

if (fire)
    set_target_state(entry->flag_index0);
```

The region selector is only consulted when no trigger fired. A row with
neither a trigger nor a region never writes its channel.

## Health value

For mechs (vtable `0x5bcef8`), `method_98` is `0x41ba40`. It returns a cached
float at `+0x40c` (dirty flag `+0x410`). On recomputation it returns `0.0` if
`+0x578 == 10`, `1.0` if the byte at `+0x630` is set, and otherwise the product
over the mech's components of `1 - weight * (1 - component_health)`, with
weights `1.0` (`+0x3a4`), `0.5` (`+0x38c`, `+0x390`), `0.2` (`+0x394`,
`+0x398`) and `1.0`/`0.5`/`0.5` for the three sub-parts of `+0x39c`. Each
component health comes from its own `vtable+0x48`. The mapping of those
offsets to torso, arms, and legs is not established.

For vehicles (vtables `0x5c8460`, which loads `vehicle.zrd`, and `0x5c88e8`),
`method_98` is `0x4ad820`: `[[this+0xe4]+0x744] / [[this+0xe4]+0x740]`, i.e.
current over maximum health. `FUN_004acbc0` reads the `vehicle.zrd` key
`health` (string `0x5e8600`) into both fields. The damage handler
`FUN_004ad3f0` subtracts the damage from `+0x744` and destroys the vehicle at
`<= 0`; `FUN_004ad2d0` sets `+0x744` to `0`. The value is therefore
continuous, not a destroyed/intact flag.

Example: `ttruck` has `health [20.0]`. The C1M1 `ttruck_01`/`ttruck_02` row
Health `0.999` → `truck_explode = 1` fires as soon as the truck has lost more
than 0.02 points, i.e. on the first hit, not only when it explodes. The
channel name describes the designer's intent, not the condition.

## Live verification (C1M1, `Mech3.exe 1.2.22`)

`owens_01` has two rows: Health `0.8` → `firefly_retreat = 1`, and Health
`0.0` → `firefly_02_go = 1`. A memory trace of the channels and of
`owens_01+0x40c` showed:

- `firefly_02_go` went to `1` 60 ms after Owens' health reached `0.000`,
  never earlier, although he had been damaged for two minutes.
- `firefly_retreat` went to `1` while Owens' health was still `1.000`, so it
  came from the player's row (`Enters reg_firefly_retreat`), not from Owens.
- The player's row `[1,0,…,1,0,…,"firefly_02_go",0]` has the Enters bit but
  no region and no trigger. It never fired, as the reconstruction predicts.

Two further runs separated the two `firefly_retreat` producers by logging the
player position (`[[player+4]+0x38]+0x84`) against the region polygon:

- Player kept outside `reg_firefly_retreat` (x −2302, z −1498) and shot Owens:
  `firefly_retreat` went to `1` in the same 20 ms poll in which Owens' health
  crossed `0.824 → 0.799`, i.e. Owens' own Health `0.8` row.
- Player walked in without firing: `firefly_retreat` went to `1` 20 ms after
  the player entered `reg_firefly_retreat` at x −2051, z −1591, with Owens at
  full health, i.e. the player's Enters row.
- `Net2help1` went to `1` one second after load from the player's Enters
  `reg_p1` row (the start area).

The index-15 reference is resolved by `FUN_004d3c40`. Its wrapper exposes a
boolean getter and setter; `FUN_004d3c00` reads the old value, writes the new
one only if it differs, and broadcasts the change. Names such as `begin`,
`radar`, `topspeed` and scripted dialogue names are therefore all named
boolean valves/state channels rather than direct function calls.

## Confirmed transformations

Values at indices 11 and 13 are squared in `FUN_004d9250` before runtime use
and square-rooted by the inverse/export path. They are therefore
distance/range-like thresholds. The comparisons above operate on the squared
runtime values.

## Candidate and region semantics

`FUN_004d6b90(1)` walks the subject's contact list at `+0xac..+0xb0` and
returns the first contact whose object/team mask intersects the mask at
`subject+0xc8`. A contact record contains the referenced object at `+0x04`
and its squared distance at `+0x0c`. The result is cached for the whole list
evaluation, which explains the two extra arguments passed through
`FUN_004d9290`.

`FUN_004d5980` traces between the subject and candidate positions. Other AI
callers use a true result as the unobstructed/visible case and penalize a false
result, so index 10 is a line-of-sight-qualified proximity trigger. Index 12
is the corresponding proximity trigger without that qualification.

The exact selector truth table for indices 8/16 is (used only when no trigger
fired):

| index 8 | index 16 | Result when region 9 exists | Result without region |
|---:|---:|---|---|
| 0 | 0 | never | never |
| 1 | 0 | subject inside region | never |
| 0 | 1 | subject outside region | never |
| 1 | 1 | always | never |

This follows directly from `FUN_005a85e0(subject)` (position lookup) followed
by `FUN_004c57d0(region, position)` (containment test); no random function is
involved.

## Original editor labels

The exact control-to-field proof is recorded in `dialog-resource-catalog.md`. The
original condition names are `Ammo`, `Fired on`, `Health`, `Heat`, `Enters`,
`Exits`, `Target LOS`, and `Target Distance`. Index 15 is the named `Valve`,
index 14 is `Active`, and index 0 selects `close (zero)` versus `open (one)`.

## Next static targets

1. Determine whether index 14 (`Active`) is consumed by another
   list path.
