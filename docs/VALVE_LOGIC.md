# Valve logic format

`valves.zrd` is an ordered list of event-name/action-record pairs. Event names may occur more than once; each occurrence registers another action and must retain its position.

Observed action forms:

| Action | Serialized form | Primary reference |
|---|---|---|
| `sound`, `sound_once` | `[soundName, state]` | sound symbol |
| `delayupdate` | `[valveName, delay]` | Valve/state channel |
| `shutdown` | `[vehicleName, value, state]` | AIV vehicle |
| `destroy` | `[vehicleName, state]` | AIV vehicle |
| `teleport` | `[vehicleName, [x,y,z], yaw, state]` | AIV vehicle |
| `attack_strategy` | `[vehicleName, strategy, state]` | AIV vehicle |
| `any_nonzero`, `all_nonzero`, `all_zero` | `["namelist", [names...]]` | named state/object list |

Unknown action types and malformed records must remain available through a raw representation and must round-trip unchanged.
