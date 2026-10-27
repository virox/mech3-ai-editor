# MechWarrior 3 Valves: The Haunted-House Version

Imagine a MechWarrior 3 mission as a haunted house full of sensors, doors,
sound effects, traps, and rooms. A **valve** is the house's shared signal wire.
It has a name and is usually either off (`0`) or on (`1`).

```text
"player_in_crypt" = 0   the house thinks the player is elsewhere
"player_in_crypt" = 1   the house knows the player entered the crypt
```

The important part: every room in the house can use the same named wire. The
wire is not owned by one file.

## The cast

| Game data | Haunted-house job | Plain meaning |
|---|---|---|
| `aiv.zrd` VList | Motion sensor | A vehicle/player condition can turn a named signal on or off. |
| `reg_*.zrd` | Room boundary | Defines where “inside the crypt” or “outside the crypt” is. |
| `targets.zrd` | Breakable prop sensor | A destroyed object can turn on a signal. |
| `net_*.zrd` | Hallway routing | An AI route can wait for a signal, choose a branch, or set a signal. |
| `valves.zrd` | House control panel | Says what should happen when a signal changes: play a sound, move/destroy an AI vehicle, or trigger another signal later. |
| `objectives.zrd` / `mpinfo.zrd` | Scorekeeper | Watches a signal to complete or fail an objective. |

## A simple scare

Suppose the player walks into a room named `reg_crypt`.

1. A player VList row in `aiv.zrd` is the motion sensor. It sets
   `player_in_crypt = 1` when the player is inside `reg_crypt`.
2. `valves.zrd` sees `player_in_crypt` and plays `ghost_laugh`.
3. A network node can use that same signal to let an enemy take a new route.
4. An objective can watch the signal and mark “Entered the crypt” complete.

```text
region sensor -> player_in_crypt = 1
              -> sound plays
              -> enemy route opens
              -> objective completes
```

The sound, route, and objective do not need to know *why* the signal changed.
They only need its exact name.

## What `valves.zrd` really is

`valves.zrd` is not a list of doors with one result each. It is a list of
"when this wire changes, do this" instructions.

```json
"player_in_crypt", ["sound_once", ["ghost_laugh", 1]]
```

Read it as: “When `player_in_crypt` is triggered in the required state, play
`ghost_laugh` once.”

The same signal may appear more than once:

```json
"player_in_crypt", ["sound_once", ["ghost_laugh", 1]]
"player_in_crypt", ["delayupdate", ["release_skeleton", 3.0]]
```

That means **do both things**. Do not merge duplicate signal names or discard
one of them.

## The three most useful network tags

### `valve`: a locked door

```json
"valve", [1, "player_in_crypt", 1]
```

This route is available when `player_in_crypt` equals `1`. The first `1` means
the test is enabled; the final `1` is the value being tested.

Always write the final value explicitly. Leaving it out makes the game derive
it from route order, which is easy to misread.

### `valveunion`: two keys for one door

```json
"valveunion", [
  [1, "player_in_crypt", 1],
  [1, "crypt_door_broken", 1]
]
```

This is an **OR** condition. The route opens if either signal is on. It is
useful when the house should react to either of two different events.

### `valve_assign`: a tripwire that flips a switch

```json
"valve_assign", ["skeleton_released", 1]
```

When the relevant route event happens, this writes `skeleton_released = 1`.
Another `valves.zrd` record can then react to that signal.

## What is a `gate`?

A `gate` is not a valve. It is a sign on a particular hallway saying which AI
vehicle the hallway belongs to.

```json
"gate", ["skeleton_guard"]
```

Think: “This branch is for the skeleton guard.” It does not turn a signal on,
test a signal, or combine signals. It qualifies an AI-network edge by vehicle
name.

## How to make a safe small change

1. Find one existing signal chain in the same mission.
2. Keep the signal spelling identical everywhere.
3. Change one outcome first: for example, replace a sound name or change a
   delay in `delayupdate`.
4. If you add a new VList row, keep all 17 values. Copy a nearby row and only
   change the region, output signal, and desired state.
5. If a route tests a signal, use `[1, "signal_name", 0]` or
   `[1, "signal_name", 1]` rather than the shortened form.
6. Repack, reload, and test the mission.

## Things that surprise new modders

- A signal with no `valves.zrd` entry is still valid. It may be used only by a
  route or objective.
- A misspelled signal quietly becomes a **different wire**, so nothing useful
  may happen.
- There is no automatic “otherwise” branch. If a condition does not match,
  the signal normally stays as it was.
- `valveunion` is OR. For an “all keys must be present” rule, use the
  `all_nonzero` derived-state action in `valves.zrd` instead.
- A region is only the room outline. It does nothing until a VList row names
  it and writes a signal.

## One-line memory aid

```text
Something happens -> a named wire changes -> anything listening to that wire reacts.
```

For the full field-by-field reference and real c1m1 examples, see
[`VALVE_SYSTEM_GUIDE.md`](VALVE_SYSTEM_GUIDE.md).
