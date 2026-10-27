# Keyboard input loss after debugger breaks

## Symptom

After x32dbg pauses and resumes `Mech3.exe`, the game can continue rendering and
simulating while all keyboard input remains dead. Refocusing the window,
Alt-Tab-style focus changes, `WM_ACTIVATEAPP`, and `WM_SETFOCUS` do not recover
the device. The same behavior has previously been observed with Recoil.

## DirectInput initialization

`FUN_0051b0a0` calls `DirectInputCreateA` and stores the DirectInput interface at
`0x00638578`. `FUN_0051a170` creates the keyboard device and stores it at:

```text
0x00638594  IDirectInputDevice* keyboard
```

Relevant keyboard-device VTable calls in `FUN_0051a170`:

```text
+0x18  SetCooperativeLevel
+0x1c  Acquire
+0x2c  SetDataFormat
+0x34  SetProperty
```

The keyboard is initially acquired once during initialization.

## Polling and suspected bug

`FUN_0051a300`, `FUN_0051a5d0`, and `FUN_0051aa00` poll buffered keyboard
events through the device VTable at `+0x28` (`GetDeviceData`). The main frame
poll in `FUN_0051a5d0` contains the following recovery decision:

```asm
0051a601  CALL [ECX+0x28]       ; GetDeviceData
0051a604  TEST EAX,EAX
0051a610  JZ   0x0051a628       ; DI_OK
0051a612  CMP  EAX,0x8007001e   ; DIERR_INPUTLOST only
0051a617  JNZ  0x0051a8c5       ; all other failures: hard error path
0051a61d  ...
0051a625  CALL [EDX+0x1c]       ; Acquire
```

`DIERR_NOTACQUIRED` (`0x8007000c`) is not treated as recoverable. Once a
single `Acquire()` attempt fails or the next poll returns `DIERR_NOTACQUIRED`,
the keyboard can remain disabled indefinitely.

The recovery path has two weaknesses:

1. Only `DIERR_INPUTLOST` is recoverable; `DIERR_NOTACQUIRED` is not.
2. `Acquire()` is attempted only once per recognized failure and its HRESULT
   is ignored.
3. The failed `GetDeviceData()` request is not robustly retried after a
   successful reacquire.

During debugger focus transitions `Acquire()` can fail transiently. The game
then appears able to remain permanently in an input-lost state.

## Proposed durable fix

Implement in Recoil/mwfix or an equivalent hook around the keyboard poll:

1. Call `GetDeviceData()` normally.
2. On `DIERR_INPUTLOST` **or** `DIERR_NOTACQUIRED`, call `Acquire()`.
3. If `Acquire()` returns `DIERR_OTHERAPPHASPRIO`, leave the frame cleanly and
   retry on the next frame.
4. On successful `Acquire()`, retry `GetDeviceData()` once with a reset event
   count/buffer.
5. Never process the stale event count or event buffer from the failed call.

For debugger use, an optional exported/manual recovery action could invoke
`keyboard->Acquire()` on demand.

## Manual recovery experiment

In the affected live session the keyboard object was still valid:

```text
IDirectInputDevice*: 0x13022a8c
VTable:              0x6289e050
Acquire():           0x62885840
```

`Acquire(device)` was invoked inside the paused debuggee with a temporary
return breakpoint. It returned `HRESULT 0` (`DI_OK`). The original stack,
volatile registers, and CPU flags were verified/restored before resuming.
This proves that an explicit on-demand reacquire is technically viable; the
live test showed that one successful manual reacquire alone does not restore
keyboard input. The most likely explanation is that a later frame returns
`DIERR_NOTACQUIRED` and immediately follows the unrecoverable branch above.

## Minimal live diagnostic patch

Temporarily neutralizing the `JNZ` at `0x0051a617` makes every failed keyboard
poll enter the existing `Acquire()` path. This is broader than the final fix,
but is useful to confirm the diagnosis because it retries on subsequent
frames. The production patch should explicitly recognize only
`DIERR_INPUTLOST` and `DIERR_NOTACQUIRED`, check `Acquire()`'s HRESULT, clear
the stale event count, and retry `GetDeviceData()` safely.

The diagnostic patch was applied live and verified:

```text
Address:  0x0051a617
Original: 0F 85 A8 02 00 00
Patched:  90 90 90 90 90 90
Restore:  write the original six bytes above
```

## Additional root cause observed live

After the broad reacquire patch and a successful manual `Acquire()`, keyboard
input was still dead. Reading the live input globals while the game continued
to run showed:

```text
0x00638584 keyboard status = 3
0x00638588 keyboard poll gate = 1
0x0063859c event count = 0
```

The frame dispatcher at `FUN_0051b360` uses an **exact** comparison:

```asm
0051b395  CMP byte ptr [0x00638584],0x1
0051b39c  JNZ 0x0051b3af
0051b3aa  CALL FUN_0051a5d0
```

Because the status byte was `3`, the entire keyboard poll was skipped even
though bit 0 (the initialized/available bit) remained set. This explains why
manual `Acquire()` and the error-path patch alone had no effect.

As a live diagnostic, `0x00638584` was changed atomically from `3` to `1`.
The keyboard still did not recover. An initial follow-up report incorrectly
claimed that the event count had advanced to `3`; that read used offset
`0x0063858c` instead of the real event-count address `0x0063859c`. A corrected
read showed that the status had already returned to `3` and the real event
count remained `0`. Thus the exact-equality dispatcher continued skipping the
keyboard poll.

The durable patch should likely treat the status as a bit field (`status & 1`)
rather than require `status == 1`, in addition to handling both input-lost
HRESULTs robustly. The origin and meaning of status bits `0x02` and higher
still need to be traced before finalizing the production patch.

The bit-field diagnostic patch was applied live:

```text
Address:  0x0051b395
Original: 80 3D 84 85 63 00 01 75 11  ; CMP status,1 / JNZ skip
Patched:  F6 05 84 85 63 00 01 74 11  ; TEST status,1 / JZ skip
```

After resuming, the live state was `status=3`, `gate=1`, and the real event
count at `0x0063859c` was still zero. The dispatcher can no longer skip merely
because the status equals 3; if input remains dead, the next diagnostic point
is the `GetDeviceData()` HRESULT in EAX immediately after `0x0051a601`.

A software breakpoint at `0x0051a604` (immediately after `GetDeviceData`) was
inserted and verified as an active `INT3`. It did not trigger immediately, but
eventually did while the mission continued to simulate normally. At the hit:

```text
EAX / HRESULT:       0x00000000 (DI_OK)
[ESP+0x14] / events: 0
```

Thus the keyboard poll chain can execute, the device is acquired, and
`GetDeviceData()` can succeed while returning no buffered keyboard events. The
breakpoint was then removed so that new physical key events could occur after
resume. The user's observation that weapons can still destroy buildings and
enemies confirms that the simulation itself remains active; the problem is
specific to keyboard event delivery/processing.

Because the same symptom occurs with the Recoil debugger workflow, all Mech3
thread suspend counts were audited. Each thread was briefly suspended and
immediately resumed once; no thread had a pre-existing positive suspend count.
The issue is therefore not explained by x32dbg simply leaving a thread
suspended. A higher-level game input/pause/activation state remains a likely
second cause in addition to the confirmed DirectInput recovery weaknesses.

## Useful addresses

```text
DirectInput initialization: 0x0051b0a0
Keyboard initialization:    0x0051a170
Keyboard shutdown:          0x0051a290
Keyboard poll/reset:        0x0051a300
Buffered keyboard poll:     0x0051aa00
Keyboard device global:     0x00638594
```

These are addresses for the currently analyzed `Mech3.exe` build.
