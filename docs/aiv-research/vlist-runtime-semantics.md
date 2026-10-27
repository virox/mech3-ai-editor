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
| 1 | flags bit `0x04` | — | Enables `Ammo` condition using value 2. |
| 2 | float `+0x00` | index 1 | Ammo threshold. |
| 3 | flags bit `0x08` | — | `Fired on` condition. |
| 4 | flags bit `0x10` | — | Enables `Health` condition using value 5. |
| 5 | float `+0x04` | index 4 | Health threshold. |
| 6 | flags bit `0x20` | — | Enables `Heat` condition using value 7. |
| 7 | float `+0x08` | index 6 | Heat threshold. |
| 8 | flags bit `0x100` | index 9 present | Region selector: condition is true while the subject is **inside** region 9. |
| 9 | resolved ref `+0x14` | — | Resolved `reg_*` region; tested against the subject position. |
| 10 | flags bit `0x40` | — | Triggers inside range 11 only when the candidate is visible. |
| 11 | float `+0x0c` | index 10 | Squared range for the visible-candidate trigger. |
| 12 | flags bit `0x80` | — | Triggers inside range 13 without a visibility test. |
| 13 | float `+0x10` | index 12 | Squared range for the proximity trigger. |
| 14 | flags bit `0x01` | — | `Active` in the editor; not directly tested in this evaluator path. |
| 15 | resolved ref `+0x1c` | — | Required named boolean valve/state; read first and set to the value from index 0 when the entry condition fires. |
| 16 | flags bit `0x200` | index 9 present | Region selector: condition is true while the subject is **outside** region 9. |

## Directly reconstructed control flow

Equivalent high-level pseudocode, retaining neutral method names:

```c
if (entry->ref15 == NULL)
    return;

current = entry->ref15->method_04();
if (current == entry->flag_index0)
    return;

reject = false;

if (entry->flag_index3 && owner->method_a0() == 1)
    reject = true;

if (entry->flag_index6 && subject->method_9c() < entry->value7)
    reject = true;

if (entry->flag_index4 && subject->method_98() <= entry->value5)
    reject = true;

if (entry->flag_index1 && owner->method_9c(subject) <= entry->value2)
    reject = true;

if (!reject) {
    if (!*candidate_initialized) {
        *candidate_initialized = true;
        *candidate = FUN_004d6b90(1);
    }

    if (*candidate != NULL) {
        if (entry->flag_index12 &&
            entry->value13 > (*candidate)->distance_squared)
            reject = true;

        if (entry->flag_index10 &&
            entry->value11 > (*candidate)->distance_squared &&
            has_line_of_sight(subject, (*candidate)->object))
            reject = true;
    }
}

if (!reject && entry->region9 != NULL) {
    inside = entry->region9->contains(subject->position());
    selector = (entry->flag_index8 && inside) ||
               (entry->flag_index16 && !inside);
    if (!selector)
        return;
}

set_target_state(entry->flag_index0);
```

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

The exact selector truth table for indices 8/16 is:

| index 8 | index 16 | Result when region 9 exists |
|---:|---:|---|
| 0 | 0 | never |
| 1 | 0 | subject inside region |
| 0 | 1 | subject outside region |
| 1 | 1 | always |

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
