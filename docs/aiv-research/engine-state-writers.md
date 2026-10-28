# Who writes state channels (`Mech3.exe 1.2.22`)

Every notifying channel write goes through `FUN_004d3c00(wrapper, value)`:
it is the only caller of the broadcast `FUN_004d3bf0` (see
`valve-action-runtime.md`). Its 13 callers are therefore the complete set of
notifying writers. They are listed below with whether the Behavior Graph shows
them.

| Caller | Writer | Channel and value | In the graph |
|---|---|---|---|
| `FUN_004d8e70` | AIV VList row | index 15 = index 0 | VList producer |
| `FUN_004d2b60`, `FUN_004d2c00`, `FUN_004d2cd0` | `targets.zrd` `health_below` (parser `FUN_004e08c0`) | channel = value once health falls below the threshold | Target health producer |
| `FUN_004cdcf0` | network `valve_assign` | channel = value on departure | Network assignment |
| `FUN_004e3f10` | `valves.zrd` `delayupdate` timer | target = trigger value | Delay Update |
| `FUN_004d4a20` | animation `CALLBACK` 42–49 | scene-node name = callback & 1 | Animation callback (42/43; 44–49 need `gamez.zbd`) |
| `FUN_0045e110` | `objectives.zrd` `set_valve` (parser `FUN_0045c920`, string at `0x45c9ec`) | channel = value when the objective completes | Objective assignment |
| `FUN_0045c920` at `0x45d541` | `objectives.zrd` `valve_change` setup | watched channel = 0 when `FUN_00479480()` is true | not shown; condition not traced |
| `FUN_004d00a0` | network `deploy` | MCB channel = MCB value | MCB deploy site |
| `FUN_0045a780` | player's MCB deploy order | `mcbdeploy` = 1, chosen site = 1, previous site = 0, MCB channels = 0 | MCB deploy order (executable) |
| `FUN_00459e80` | MCB manager start | `mcbdeploy` = 0 | not shown (reset) |
| `FUN_0045a030` | MCB registration | MCB vehicle name (`mobilehq_0N`) = 0 | not shown (reset) |
| `FUN_0045ac70`, `0x45b6d9` | MCB abort/reset paths | MCB channels = 0, `mcbdeploy` = flag | not shown (reset) |

Aggregate channels (`any_nonzero`, `all_nonzero`, `all_zero`) are derived
values computed on read and are shown as aggregate cards.

## MCB deployment (C1M4 `MFB 1`–`MFB 4`)

`deploy [mcb, mcb_value, site, site_value, node]` on a network node
(parser `FUN_004db1b0` at `0x4db41e`) resolves **two** channels, `mcb` and
`site`, through `FUN_004d3c40`. The `MFB n` names exist only there and in
`auxname`; no Reader writes them.

- The static initializer at `0x4cfe5d` binds the engine channel `mcbdeploy` with the
  expected value `1` (`DAT_0060b4a8`/`DAT_0060b4ac`).
- `FUN_0045a780`, the MCB deploy command, takes the selected entry of the
  list at `DAT_0060b544` (`+0x20` index). It reads its name (`vtable+0x24`) and
  writes `mcbdeploy = 1`. It sets the previously ordered site channel to `0`
  and looks up the channel with that name (`FUN_004d3d20`, existing channels
  only); if found, it sets it to `1`. It then resets the MCB channels at
  `+0x20..+0x28` to `0`.
- The navigation action built from `deploy` (`FUN_004bde20` → `FUN_004cff10`,
  vtable `0x5cc898`; runtime in `FUN_004d00a0`) proceeds only when
  `mcbdeploy == 1` and `site != site_value`. It then writes
  `mcb = mcb_value` (`0x4d06b2`).
  Before writing it compares the current `mcb` value with `mcb_value`
  (`0x4d0618`–`0x4d062b`) and skips the write when they already match.
- The only engine **reader** of the MCB channels is the MCB manager's deploy
  state (vtable `0x5c2620`, built at `0x45ab30` with the manager at `+0x8`).
  Its completion check `FUN_0045ab70` (vtable slot `+0x18`, called from the
  update `FUN_0045ac20`) walks the three MCB slots: a slot counts as done when
  its vehicle (`mgr+0x14..+0x1c`) is inactive (`vehicle+0x8 == 0`, set by
  `FUN_0045a030` for surplus MCBs), when it has no channel, or when the
  channel (`mgr+0x20..+0x28`, read through `[[valve+4]]->vtable+4`, the same
  getter `FUN_004d3c00` uses) is nonzero. The state ends once all three are
  done. Slot `+0x0c` (`FUN_0045ac70`) resets the three channels to `0`.
  The update also plays the manager's message `mgr+0xa4` five seconds after
  the state starts (`FUN_0045b7d0` → `FUN_004e6d50`).

The Behavior Graph splits the cycle into two locked cards so it reads left to
right: *MCB deploy order* (writes `mcbdeploy` and the sites) and *MCB deploy
complete* (one read port per distinct `mcb` channel of the network's `deploy`
tags). A `↻` line on each card names the other half of the cycle. No
Reader record reads `mobilehq_0N`.

In C1M4 the four deploy sites are `MFB 1`–`MFB 4` (each with three
`mobilehq_0N` slots). `rivbrg_up = all_nonzero[MFB 1, rivbrg]` and
`mfb234 = any_nonzero[MFB 2, MFB 3, MFB 4]` therefore react to the player's
MCB order.

Not verified: that `vtable+0x24` returns the node's `auxname` rather than the
localized `name` (`MSG_ZRD_MFB_n`); both read `MFB n` in the shipped data.
