# c1m1 example: play `shots_fired` after the player fires

This is the smallest known Reader-side pattern for **playing one sound after the player's first detected shot** in c1m1.

It uses the same player VList pattern that the training mission uses to set `fired_mgun`. The pattern creates a latch: once its condition becomes true, it writes `shots_fired = 1`. The valve action reacts to that change and plays the sound.

> This is a **one-time, first-shot** setup. It does not currently give us a confirmed way to play the sound again on every trigger pull. Repeating the same value (`1`) does not re-run the action.

## What to add

You add two things, in two files under c1m1's Reader data:

| File | Add | Purpose |
| --- | --- | --- |
| `zbd/c1/readerm1/aiv.zrd` | a VList row on `player` | Detect the same ammunition-related event used by the training mission and set `shots_fired` to `1`. |
| `zbd/c1/readerm1/valves.zrd` | a valve/action pair | Play the named sound when `shots_fired` changes to `1`. |

The extracted, editable reference copies in this workspace are:

- `docs/aiv-research/extracted-reader-json/zbd/c1/readerm1/aiv.zrd.json`
- `docs/aiv-research/extracted-reader-json/zbd/c1/readerm1/valves.zrd.json`

## 1. Add the player condition in `aiv.zrd`

Open the `player` record, find its `vlist` array, and append this **new row**. Do not replace the existing rows.

```json
[
  1,
  1,
  0.9990000128746033,
  0,
  0,
  0.10000000149011612,
  0,
  0.800000011920929,
  0,
  0,
  0,
  0.0,
  0,
  60.0,
  1,
  "shots_fired",
  0
]
```

The important parts are:

- The second field is `1`: this enables the ammunition-style condition.
- `0.9990000128746033` is the threshold used by the known `fired_mgun` example. Leave it unchanged for the first test.
- The fifteenth field is `1`: write the on-state.
- The sixteenth field is `"shots_fired"`: the name of the valve this condition writes.

The zero in the tenth field means this example has no region requirement. It can therefore detect the event anywhere in the mission.

This row is deliberately copied from the player VList in `zbd/t1/readerm3/aiv.zrd`, except that its output name is changed from `fired_mgun` to `shots_fired` and its unused region reference is cleared. That training mission uses `fired_mgun = 1` to fail an objective after firing.

## 2. Add the sound result in `valves.zrd`

Add this pair anywhere inside the top-level array. Keep the comma rules correct for the surrounding JSON.

```json
"shots_fired",
[
  "sound_once",
  [
    "shots_fired",
    1
  ]
]
```

In the compact style already used by c1m1, the same entry is:

```json
"shots_fired", ["sound_once", ["shots_fired", 1]]
```

Read that as: **when the valve named `shots_fired` becomes `1`, play the sound resource named `shots_fired` once.**

## 3. Provide the sound itself

The action above does not import an audio file. `"shots_fired"` must already be a valid sound resource/name in the game data available to this mission. The existing c1m1 entries use names such as `snd_c1m1dialogue4`.

If your source asset has a different internal name, use that name in the second `"shots_fired"` position instead:

```json
"shots_fired", ["sound_once", ["ACTUAL_SOUND_RESOURCE_NAME", 1]]
```

The first `"shots_fired"` is the valve name. The sound name may be different.

## Expected result

At mission start, `shots_fired` is effectively off (`0`). When the copied player condition detects the first qualifying weapon use, it changes the valve to `1`. That change runs `sound_once`, which plays the named sound.

It should play once per mission load. The valve setter suppresses a repeated write of the same value, so later shots do not replay it.

## Test procedure

1. Start c1m1 and do not fire for a few seconds. The sound should not play.
2. Fire the weapon once.
3. Confirm that `shots_fired` plays once.
4. Fire again. With this latch recipe, it should not play again.

If it plays at mission start or never plays, revert only this new VList row and report the observed behavior. The training data proves this is the game's existing `fired_mgun` pattern, but the exact weapon/ammunition metric behind its `0.999...` threshold has not yet been named by the executable. It may need a small threshold adjustment for c1m1's player loadout.

## Do not use the `Fired Upon` field for this

The third VList condition is labelled **Fired Upon** in the recovered data. That means the player being shot at/hit, not the player firing a weapon. It is not the condition used in the recipe above.

## If you need a sound on every shot

Do not try to reset `shots_fired` immediately with a second valve action: a repeated `1` write is intentionally ignored, and the currently documented Reader actions do not provide a confirmed safe reset-and-retrigger sequence.

An every-shot effect needs a weapon-fire callback or another proven repeating event source. That requires further executable/script research; it is not established by the existing VList and valve data.
