# `valves.zrd` action runtime: `delayupdate`

Static analysis of `Mech3.exe 1.2.22` (Ghidra 12.1.4, auto-analysis), checked
against a live C1M1 trace. All addresses are virtual addresses in that build.

## Channel objects

`FUN_004d3c40(name)` returns the channel wrapper for a named valve and
creates it on first use. The wrapper has:

| Offset | Content |
|---:|---|
| `+0x04` | Storage object; vtable `+0x04` is the getter, `+0x08` the setter. |
| `+0x08` | Listener (single object, or a composite list). |

`FUN_004d3c00(wrapper, value)` reads the old value. Only if it differs, it
calls the setter and then `FUN_004d3bf0`, which calls `listener->vtable[+0x08](wrapper)`.

`FUN_004d4310(wrapper, listener)` attaches a listener at `wrapper+0x08`
(`FUN_004d4320`). A second listener replaces the slot with a composite
(`FUN_004d3fd0`, vtable `0x5ccd70`). Its `+0x08` method `FUN_004d41a0` forwards
the same wrapper to every listener in order. Several action records on the
same channel (C1M1 `bramp` has two `delayupdate` rows) are all notified.

## Parsing

`FUN_004e2330` parses one action record into a 0xc8-byte block. For
`delayupdate` it stores the target name (`_strdup`) at `+0x80` and the delay
float at `+0x84`. There is no value field.

`FUN_004e44b0` builds the runtime objects for the block. Its first step is
`local_18 = FUN_004d3c40(record channel)`. At `0x4e4c92..0x4e4cb2`, if `+0x80`
is set, it creates the action with `FUN_004e4020(block+0x80)` and attaches it
to `local_18`, the record's **own** channel, with `FUN_004d4310`.

## Action object (vtable `0x5cd528`)

Constructor `FUN_004e3f50`:

| Offset | Content |
|---:|---|
| `+0x04` | `FUN_004d3c40(target name)`, the target channel wrapper. |
| `+0x08` | Delay in seconds (`block+0x84`). |

Change handler `vtable+0x08` = `0x4e3fb0` (`this`, `trigger_wrapper`):

```c
if (this->target != NULL) {
    value = trigger_wrapper->storage->get();          // call [edx+4] at 0x4e3fd9
    new DelayTimer(this, value);                       // FUN_004e3e60
}
```

There is no comparison against the value. Every change notification of the
trigger channel schedules a timer carrying the trigger's **new** value.

## Timer object (vtable `0x5cd518`)

Constructor `FUN_004e3e60(action, value)`:

| Offset | Content |
|---:|---|
| `+0x04` | Target wrapper (copied from `action+0x04`). |
| `+0x08` | Deadline = `action+0x08` + float at `0x60991c`. |
| `+0x0c` | Captured value. |

The timer registers itself in the global list at `0x60b514` (`FUN_004d4480`).

Tick `vtable+0x04` = `FUN_004e3f10(now)`: if `deadline < now`, it calls
`FUN_004d3c00(target, value)`, deletes itself through `vtable+0x00(1)` and
returns `1`. Otherwise it returns `0` and stays queued.

## Result

`delayupdate [target, seconds]` copies the trigger channel's value to
`target`, `seconds` after each change:

- trigger `0 → 1` writes `1` after the delay;
- trigger `1 → 0` writes `0` after the delay;
- each change has its own timer; no pending timer is cancelled or restarted;
- the target write goes through `FUN_004d3c00`, so it only notifies the
  target's own listeners (further actions, conditions) when the value changes.

The float at `0x60991c` is used as the current time base. The live delays
fit that reading; who writes it was not traced.

## Live cross-check (C1M1)

`readerm1/valves.zrd` contains `bramp → delayupdate [truck_go, 18.0]` and
`bramp → delayupdate [c1m1dialogue4_play, 22.0]`. In the memory trace:

| Event | Run 1 | Run 2 |
|---|---:|---:|
| `bramp 0 → 1` | +82.21 s | +185.39 s |
| `truck_go 0 → 1` | +100.21 s (18.00 s) | +203.38 s (17.99 s) |
| `c1m1dialogue4_play 0 → 1` | +104.21 s (22.00 s) | +207.38 s (21.99 s) |

The `1 → 0` forwarding follows from the code above; it was not observed live,
because no C1M1 trigger channel is reset to `0` during the trace.
