# Internal AI Editor Dialogs

`Mech3.exe` contains a largely wired MFC-based AI/mission editor. The dialog
resources are not only leftover text: their constructors create the dialogs and
their `DoDataExchange` handlers read and write the AI editor structures.

Dialog resources can be listed reproducibly with:

```powershell
python .\docs\aiv-research\scripts\extract-dialog-resources.py .\Mech3.exe
```

## Evidence and status

The mapping below is **dialog-first**: start with a resource ID, then find the
control and the corresponding `aiv.zrd` field. A mapping is marked **direct**
when the MFC data-exchange code binds the control to the same editor field that
the AIV parser serializes. **Shared/union** means that the same storage is used
by more than one behavior dialog. **Runtime-only** means that the field exists
in the in-memory editor/dialog object but has no proven serialized AIV index.
**Open** means that no reliable control-to-AIV mapping has been recovered yet.

The parser has a `0x64`-byte header before the parameter block. Thus an editor
offset of `+0x80` corresponds to parser/AIV offset `+0x1c`; the same delta is
used throughout the direct mappings.

## How the map is recovered

This does not require manually guessing from one dialog screenshot at a time.
The executable provides a repeatable three-way join:

1. the resource table supplies every control ID and label;
2. each dialog's MFC `DoDataExchange` handler supplies the control ID and the
   bound editor field/type; and
3. the AIV parser supplies the serialized tag/index and target offset.

The result is a complete dialog inventory with explicit `Open` entries where
the third link (the save/copy path) is still missing. Dialog 166 below is the
first full control inventory produced from that process; the same method can
be applied to the remaining resource handlers in one batch.

## Dialog index

| Resource ID | Title | AIV data covered |
|---:|---|---|
| 165 | `Mech AI` | Main AI editor/container; no field-level mapping recovered |
| 166 | `p2` | Vehicle identity/base data, chassis, pilot/team/net fields and flags |
| 167 | `p1` | Activation and target/net ranges; `params`, `p6`, `p8` and one runtime switch |
| 168 | `Attack` | Strategy, approach/attack values, accuracy and FCS/TC; `p2`, `p8`, `p10`, `p11` |
| 169 | `Retreat` | Net-distance, health, time, health-change and timeout triggers; `params`, `p2`, `p6`, `p9` |
| 170 | `p5` | Scan, jump and collision avoidance; `p8`, `p13` and runtime scan values |
| 171 | `Fire` | Acquisition range, fire time, cooldown and alpha-strike probability |
| 172 | `Valve` | All 17 positions of one `vlist` entry |
| 173 | `Zig Zag` | `p2[4]` length and `p2[5]` target distance |
| 174 | `Circle` | `p2[4]` radius |
| 175 | `Jump` | `p8[0..3]` delay/hold ranges and shared scan parameters |
| 176 | `Direct` | `params[4]` approach range and `params[5]` minimum range |
| 183 | `Random Move` | Radius control; serialization mapping not recovered (dialog says not implemented) |

Other internal tools include `AI Network Editor`, `Net Edit`, `Node Edit -
Vehicle`, `Teleport`, `Mech Editor` and `Mech Animation Transitions`. Their
resource IDs and field mappings are not part of the AIV parameter map below.

## Dialog-first AIV field map

### Resource 166 — `p2` (vehicle data)

The DDX handler at `004b8eb0` binds the following controls. This is the
complete control inventory for the resource, including fields that are editor
state rather than serialized AIV values:

| Control / UI field | DDX-bound value | AIV field or tag | Status / notes |
|---|---|---|---|
| `Configuration` combo (1273) | editor combo object `+0x214` | no proven AIV index | Runtime/configuration selector; the combo is wired, but its save path is not the AIV record parser |
| `Default Chassis` combo (1274) | editor combo object `+0x254`; selection stored at dialog `+0x128` | `p12[0]` | Direct through the selection handler at `004bc8f0` |
| `Skin` edit (1284) | editor integer `+0x12c` | `skin[0]` (numeric tag value) | Control is an integer DDX field; current AIV data confirms numeric skin values |
| `Pilot Name` edit (1243) | dialog `CString` at `+0x60` | `pilot[0]` | String tag; DDX is direct to the dialog string and the tag writer is separate |
| `Objective Conditional` (1275) | dialog flag `+0x64` → editor flag bit 4 | `p12[1]` | Direct |
| `can configure` (1279) | editor flag bit 7 at `+0xcc` | `can_configure` | Direct tag-to-flag mapping; not a `p12` array index |
| `Shutdown on start` (1276) | dialog flag `+0x68` → editor flag bit 5 | `p12[2]` | Direct |
| `Home Net` combo (1238) | editor net-list combo `+0x1d4` | net reference / `record[0]` relationship | Selection resolves the read-only Net ID; exact serialized copy path is not yet isolated |
| `Net Edit` (1250) | command button | no AIV field | Opens the network editor for the selected Home Net |
| `Net ID` read-only edit (1117) | editor integer `+0x68` | base record field 0 (`record[0]`) | Semantic match to the parser's first record field; value is populated from Home Net selection |
| `Show` (1249) | editor flag/state `+0x1cc` | no proven AIV field | Tri-state display/editor command |
| `Show InOut` (1282) | editor byte `+0x1d1` | no proven AIV field | Runtime network-display option |
| `Initial Vehicle Position` X (1200) | editor float `+0x6c` | `record[1][0]` | Direct base-record position field |
| Initial Vehicle Position Y (1201) | editor float `+0x70` | `record[1][1]` | Direct base-record position field |
| Initial Vehicle Position Z (1202) | editor float `+0x74` | `record[1][2]` | Direct base-record position field |
| Initial Vehicle Position `yaw` (1203) | editor float `+0x78` | `record[2]` / parser base offset `+0x14` | Direct base-record orientation field |
| Team `Friend` / `Foe` / `Other` (1197–1199) | DDX radio value `+0x4dc` | no separate AIV field; derived from `record[3]` | `004bc260` decodes `record[3]` to a membership mask and compares it with the controlled mech's friend/foe masks. Radio changes call `004bc300`, which selects the lowest matching Team ID and writes it to the numeric team field |
| Team number (1111) | editor unsigned value `+0x7c` | base `record[3]` | Direct and persistent. The unsigned integer packs one or more Team IDs into 5-bit lanes; it is not an allegiance enum |
| `Send home` (1255) | command button | no AIV field | Action; not a persisted vehicle parameter |
| `Lancemate` (1283) | editor byte `+0x131` = embedded AIV byte `+0xcd` | optional `lancemate[0]` tag | Direct tri-state: 0/Off forces the runtime flag off, 1/On forces it on, and 2/Auto derives it from the controlled player's Friend mask; an absent tag defaults to Auto |

The general vehicle dialog also has a resource title `p2`; that title must not
be confused with the serialized tag named `p2` (attack/retreat parameters).
The `Lancemate` DDX mapping is described in detail in
[`legacy-dialog-options/legacy-dialog-166-p2-reference.md`](legacy-dialog-options/legacy-dialog-166-p2-reference.md#lancemate-control-1283).

### Resource 167 — `p1` (activation)

Die vollständige Control- und Laufzeitreferenz steht in
[`legacy-dialog-options/legacy-dialog-167-p1-reference.md`](legacy-dialog-options/legacy-dialog-167-p1-reference.md).

| Control / original label | AIV field | Status / notes |
|---|---|---|
| `Range to` (1209) | `params[0]` | Direct; parser offset `+0x1c` |
| `Enable Initial` (1217) | `params[1]` | Direct; parser offset `+0x3c`; runtime enable gate, not a range |
| `Leave Net Range to Target` (1210) | `params[2]` | Direct; parser offset `+0x20` |
| `Return to Net Range from Net` (1211) | `params[3]` | Direct/shared; also the Retreat `Net Distance` value |
| `Approach Range to Target` (1212) | `params[4]` | Direct/shared with `Direct` |
| `Minimum Range to Target` (1214) | `params[5]` | Direct/shared with `Direct` |
| `Always active` (1231) | `p8[9]` | Direct; activation subsection of Combat Behavior |
| Range-selection combo (1229) | `p8[10]` | Direct through combo handler; stored as `selected index == 0` |
| `Disable` (1251) | `p6[0]` | Direct; post-retreat disable flag |
| `Enable Now` (1257) | no proven AIV field | Runtime-only dialog/editor state at offset `+0x984`; do not serialize as a new `params` index |

The old p1 dialog visually groups `Always active` and `Disable` with the
activation controls, even though their serialized storage belongs to `p8` and
`p6`. `Enable Now` is a command/state control rather than an AIV array field.

`Return to Net Range from Net` (control 1211) is shared storage: the same
`params[3]` value is shown as the Retreat dialog's `Net Distance` threshold in
resource 169. It is not a second value; editing either dialog edits the same
serialized field.

### Resource 168 — `Attack`

| Control / original label | AIV field | Status / notes |
|---|---|---|
| Strategy selector (1233) | `p2[0]` | Direct through selection handler; values 0–4 |
| Circle `Radius` / Zig Zag `Length` (1220 / 1219) | `p2[4]` | Direct/shared; strategy-dependent union |
| Zig Zag `Target Distance` (1220) | `p2[5]` | Direct; strategy-dependent |
| `Approach` (1248) | `p8[8]` | Direct |
| `Attack` value (1234) | `p8[12]` | Direct |
| `Accuracy (0 is most accurate)` (1272) | `p10[0]` | Direct |
| `FCS/TC multiplier` (1274) | `p11[0]` | Direct; control ID is reused by the vehicle chassis combo in resource 166 |

The `FCS/TC multiplier` control ID (1274) is reused by a different control in
resource 166 (`Default Chassis`). The numeric storage is not shared with
`p12[0]`; only the Win32 control ID is reused in different dialog classes.

### Resource 169 — `Retreat`

| Control / original label | AIV field | Status / notes |
|---|---|---|
| `Net Distance` enabled (1221) | `p2[1]` / editor flag bit 0 | Direct |
| `Time` enabled (1224) | `p2[2]` / editor flag bit 1 | Direct |
| `Health` enabled (1222) | `p2[3]` / editor flag bit 2 | Direct |
| `Time` threshold (1215) | `p2[6]` | Direct |
| `Health` threshold (1213) | `p2[7]` | Direct |
| `Net Distance` value (p1 control 1211) | `params[3]` | Direct/shared with resource 167 (`Return to Net Range from Net`) |
| Timeout value (1254) | `p6[1]` | Direct |
| `For timeout` (1253) | `p6[2]` | Direct |
| `Forever` (1252) | `p6[3]` | Direct |
| `Health Change` enabled (1271) | `p9[3]` / editor flag bit 3 | Direct |
| `Health Change` value (1270) | `p9[4]` | Direct |

`Net Distance` in this dialog is `params[3]` (control 1211 in resource 167),
not a separate Retreat-only parameter. Resource 167 labels the same storage
`Return to Net Range from Net`; the two labels describe the same union/shared
field. The remaining timeout fields (`p6[1..3]`) are Retreat-specific.

### Resource 170 — `p5` (scan/jump/collision)

| Control / original label | AIV field | Status / notes |
|---|---|---|
| `Min Delay` (1223) | `p8[0]` | Direct/shared with Jump |
| `Max Delay` (1225) | `p8[1]` | Direct/shared with Jump |
| `Min Hold` (1226) | `p8[2]` | Direct/shared with Jump |
| `Max Hold` (1227) | `p8[3]` | Direct/shared with Jump |
| Collision Avoidance `Enable` (1246) | `p8[4]` | Direct |
| `Alert Dist` (1228) | `p8[5]` | Direct |
| `Min Dist` (1230) | `p8[6]` | Direct |
| `Jump Ceiling` (1247) | `p8[7]` | Direct |
| Scan trigger `Aim` (1261) | `p8[13]` | Direct |
| Scan trigger `Hit` (1260) | `p8[14]` | Direct |
| Scan trigger `Time` (1244) | `p8[15]` | Direct |
| `Scan` / `Jump` mode (1258 / 1259) | `p8[16]` | Direct radio group |
| `Quit scan on trigger pull` (1278) | `p13[0]` / editor flag bit 6 | Direct |
| Scan-trigger `Set Value` columns | no proven AIV index | Runtime/editor fields at `+0x974`, `+0x978`, `+0x97c`; separate from `p8[13..15]` |

The delay/hold controls 1223, 1225, 1226 and 1227 bind to `p8[0..3]`, which
are also edited by resource 175 (`Jump`). The scan-trigger values `p8[13..16]`
are likewise shared with the Jump dialog; the p5 `Set Value` columns remain
runtime/editor-only and must not be mistaken for additional AIV indices.

### Resource 171 — `Fire`

| Control / original label | AIV field | Status |
|---|---|---|
| `Maximum Acquisition Range` (1145) | `params[6]` | Direct |
| `Fire Time` (1191) | `params[7]` | Direct |
| `Cooldown Time` (1216) | `params[8]` | Direct |
| `Alpha Strike Probability` (1232) | `p8[11]` | Direct |

### Resource 172 — `Valve`

Dialog 172 is constructed at `004b9918`; its DDX handler starts at
`004b9930`, obtains the selected 32-byte runtime entry and binds all 17
`vlist` positions. The serialized positions are:

| `vlist` index | Original label | Control ID | Runtime/serialized meaning |
|---:|---|---:|---|
| 0 | `close (zero)` / `open (one)` | 1178 / 1179 | Valve result flag/value |
| 1 | `Ammo` | 1181 | Ammo-condition flag |
| 2 | Ammo value | 1193 | Entry `+0x00` |
| 3 | `Fired on` | 1182 | Fired-upon flag |
| 4 | `Health` | 1180 | Health-condition flag |
| 5 | Health value | 1192 | Entry `+0x04` |
| 6 | `Heat` | 1186 | Heat-condition flag |
| 7 | Heat value | 1195 | Entry `+0x08` |
| 8 | `Enters` | 1188 | Region flag byte, bit 0 |
| 9 | `Region` | 1177 | Entry `+0x14` |
| 10 | `Target LOS` | 1190 | LOS-condition flag |
| 11 | Target-LOS range | 1196 | Entry `+0x0c`, internally squared |
| 12 | `Target Distance` | 1184 | Distance-condition flag |
| 13 | Target-distance value | 1194 | Entry `+0x10`, internally squared |
| 14 | `Active` | 1256 | Active-condition flag |
| 15 | `Valve` | 1176 | Entry `+0x1c`, named boolean target state |
| 16 | `Exits` | 1189 | Region flag byte, bit 1 |

### Resources 173, 174, 175 and 176 — movement subdialogs

| Resource | Dialog | Control / field | AIV field | Status |
|---:|---|---|---|---|
| 173 | `Zig Zag` | `Length` (1219) | `p2[4]` | Direct/shared union with Circle radius |
| 173 | `Zig Zag` | `Target Distance` (1220) | `p2[5]` | Direct |
| 174 | `Circle` | `Radius` (1220) | `p2[4]` | Direct/shared union with Zig Zag length |
| 175 | `Jump` | `Delay Min/Max`, `Duration Min/Max` (1223/1225/1226/1227) | `p8[0..3]` | Direct/shared with p5 |
| 175 | `Jump` | Uses scan parameters | `p8[13..16]` | Shared with p5; exact duplicate-control layout is strategy-dependent |
| 176 | `Direct` | `Approach Range to Target` (1212) | `params[4]` | Direct/shared with p1 |
| 176 | `Direct` | `Minimum Range to Target` (1214) | `params[5]` | Direct/shared with p1 |
| 183 | `Random Move` | Radius | unresolved | Resource exists, but the original dialog says the behavior is not implemented |

The Direct dialog's controls 1212 and 1214 edit the same `params[4]` and
`params[5]` fields shown in resource 167 as `Approach Range to Target` and
`Minimum Range to Target`. They are shared storage, not copies.

## Shared controls and union fields

The following relationships are intentionally repeated in each affected dialog
section above. This matrix is a compact cross-check so that a reverse lookup
never loses the other dialog using the same storage:

| Control/storage | Dialog resources | AIV field | Explanation |
|---|---|---|---|
| Control 1211 / parser `+0x24` | 167 `p1`; 169 `Retreat` | `params[3]` | `Return to Net Range from Net` and `Net Distance` are two labels for one field |
| Control 1212 / parser `+0x28` | 167 `p1`; 176 `Direct` | `params[4]` | Same approach-range storage |
| Control 1214 / parser `+0x2c` | 167 `p1`; 176 `Direct` | `params[5]` | Same minimum-range storage |
| Controls 1223/1225/1226/1227 | 170 `p5`; 175 `Jump` | `p8[0..3]` | Same delay/hold values; Jump is the strategy-specific view |
| Scan mode/trigger values | 170 `p5`; 175 `Jump` | `p8[13..16]` | Shared scan parameters; p5 `Set Value` fields are separate runtime storage |
| Controls 1219/1220 / parser `+0x44` | 173 `Zig Zag`; 174 `Circle` | `p2[4]` | Union field: Zig Zag length or Circle radius depending on strategy; the Zig Zag dialog uses its strategy-specific control layout |
| Control 1220 in Zig Zag | 173 `Zig Zag` | `p2[5]` | Target-distance field in the Zig Zag context; the numeric control ID is reused by the dialog class |
| Control 1274 | 166 `p2`; 168 `Attack` | `p12[0]` / `p11[0]` | Control ID is reused, but the underlying storage is different; it is not a shared AIV value |

## Unresolved or non-serialized values

The following values occur in the current reader data or in the editor model but
do not yet have a proven direct dialog-to-AIV mapping:

- `p9[0]`, `p9[1]`, `p9[2]`
- `p10[1]`
- Resource 166 `Configuration` combo (`+0x214`) and the exact save/copy path
  connecting Home Net/team UI state to the base record
- `Enable Now` (resource 167, control 1257; runtime/editor offset `+0x984`)
- p5 scan-trigger `Set Value` fields (editor offsets `+0x974`, `+0x978`, `+0x97c`)
- legacy groups `p3`, `p4`, `p5`, `p7` and other union-only fields not present in
  the current reader data

The same storage can legitimately appear under several dialogs. Therefore a
dialog label alone does not establish a new AIV index; the control binding and
the parser offset must agree.

## Code anchors

- `004b8bc0`: DDX for resource 167 (`p1`)
- `004b8eb0`: DDX for resource 166 (general vehicle data)
- `004b92f0`: DDX for resource 168 (`Attack`)
- `004b9490`: DDX for resource 169 (`Retreat`)
- `004b96a0`: DDX for resource 170 (`p5`)
- `004b9890`: DDX for resource 171 (`Fire`)
- `004b9918`: construction of resource 172 (`Valve`)
- `004b9930..004b9bf9`: direct exchange with the selected `vlist` entry
- `004b9c30`: DDX for `Circle`
- `004b9c80`: DDX for `Direct`
- `004b9cf0`: DDX for `Jump`
- `004b9d90`: DDX for `Zig Zag`
- `004bc8f0`: `Default Chassis` selection handler (`p12[0]`)

For the inverse organization (AIV field → dialog/control), see
[`dialog-parameter-map.md`](dialog-parameter-map.md).
