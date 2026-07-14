# Reverse-engineering evidence and editor gaps

This document is the gate for adding semantic field names. A label is accepted
as confirmed only when the ZRD parser destination and an original editor/runtime
consumer can be joined through the same storage offset or flag bit.

## Evidence levels

- **Confirmed / direct**: parser index → internal field → MFC DDX control or
  unambiguous runtime consumer.
- **Confirmed / enum construction**: numeric value and original caption are
  inserted together by executable code.
- **Structural only**: shape, type, order, and destination are known, but the
  developer-facing meaning is not.
- **Observed only**: corpus correlation without a unique code connection. This
  is not sufficient for a semantic UI label.

## Newly confirmed strategy enumeration

`Mech3.exe` function `004b7d80` constructs the Attack strategy collection by
storing each integer and caption in the same item. This is not inferred from
mission data.

| `p2[0]` | Original caption | Code anchor |
|---:|---|---:|
| 0 | `Direct` | `004b7df6` |
| 1 | `Zig Zag` | `004b7e32` |
| 2 | `Circle` | `004b7e6c` |
| 3 | `Jump` | `004b7ea2` |
| 4 | `Random Move` | `004b7ee0` |

Resource dialogs 173–176 and 183 provide the corresponding conditional panels.
`Random Move` explicitly says `Not yet implemented, behaves like Circle.`

## AIV fields still requiring proof

The following labels remain deliberately unresolved:

- `p8[10]`: the MFC connection proves it stores whether the `Range to` combo's
  selected index is zero, but the combo's two original choices still need to be
  recovered.
- `p9[0..2]`: parsed to AI configuration offsets `+0xa8`, `+0xac`, `+0xb0`.
  No direct DDX controls have yet been joined to these offsets. Function
  `004e0540` passes them into the runtime/config copy constructor `004df6a0`,
  but this proves propagation only, not their developer-facing meaning.
- `p10[1]`: parsed to `+0xbc`; no direct DDX binding has yet been found.
- legacy `p3`, `p4`, `p5`, and `p7`: structurally parsed but unioned with newer
  fields. They must not inherit `p8` labels merely because offsets overlap.
- `sound_tag` and older optional tags: structure is known only partially.

## Original internal tools relevant to Reader data

| Resource | Tool/dialog | Likely Reader scope | Current editor status |
|---:|---|---|---|
| 165–176, 183 | Mech AI and conditional panels | `aiv.zrd`, VList | partially rebuilt |
| 139 | `AI Network Editor` | network selection | missing |
| 140 | `Net Edit` | `net_*.zrd` header and radii/dwell values | missing |
| 141 | `Node Edit - Vehicle` | `net_*.zrd` node position, links, team/type | missing |
| 142 | `Map Area Editor` | objective/map-area connection | missing; link unproven |
| 143 | `Vehicle Select` | vehicle reference picker | missing |
| 152 | untitled NetID/Nodes/Position dialog | network/node inspection | missing; code link required |
| 154 | untitled Net/Node/From/To dialog | link or relocation operation | missing; code link required |
| 180 | `Node Parameters` | generic node extension shell | missing; empty resource |
| 181 | `Ai Network Editor` | alternate/open-file editor shell | missing; code link required |
| 182 | `Mech Animation Transitions` | animation transition data | outside initial AI scope |

### Confirmed wiring in `Node Edit - Vehicle`

Resource 141 is constructed by `0049d350`; its class vtable is installed at
`005c8140`. The `DoDataExchange` override at `0049d660` binds the following
controls directly:

| Control | Original field | Dialog storage | Binding |
|---:|---|---:|---|
| 1200 | position X | `+0xa4` | `DDX_Text(float)` |
| 1201 | position Y | `+0xa8` | `DDX_Text(float)` |
| 1202 | position Z | `+0xb0` | `DDX_Text(float)` |
| 1203 | Yaw | `+0xac` | `DDX_Text(float)`, validated `-360..360` |
| 1111 | Vehicle Type | `+0xb4` | text/combo selection value |
| 1041 | Friend / Foe / Other | `+0xb8` | `DDX_Radio` |

This proves that the dialog is internally wired, but not yet that its storage
is the serialized `net_*.zrd` node record. No direct reference to constructor
`0049d350` is currently present in Ghidra's reference graph. The serializer at
`004db8d0` independently proves that AI-network node X/Y/Z are stored at node
offsets `+0x0c/+0x10/+0x14`; a copy/commit handler joining these two layouts is
still required before exposing the dialog fields as `net_*.zrd` fields.

Also note that AI-network-node offsets `+0xa8/+0xac/+0xb0` serialize as `gate`,
`valveunion`, and `valve_assign`. Their numeric coincidence with AIV config
offsets is not a semantic relationship.

## Reader documents that form the next editor layers

1. **Valve definitions (`valves.zrd`)** — ordered event/action pairs; duplicate
   event names are legal and meaningful. This is different from the 17-field
   VList condition editor.
2. **AI networks (`net_*.zrd`)** — version/name header, ordered nodes, positions,
   adjacency, and optional node tags such as `valve`, `deploy`, `name`,
   `auxname`, and `arrangenodes`.
3. **Regions (`reg_*.zrd`)** — network-like polygon/node data referenced by
   VList `Region`.
4. **Targets (`targets.zrd`) and objectives** — mission target/objective
   definitions referenced by AI and scripts.
5. **Way networks (`waynet_*.zrd`)** — additional navigation graphs; their
   relationship to the Net Editor must be proven before sharing one schema.
6. **Mission/world layers** — `location.zrd`, `world.zrd`,
   `vehicle_static.zrd`, environment, sounds, and mission-specific scripts.

## Required next checks

1. Recover every combo/radio item list, especially `p8[10]`, from executable
   construction code.
2. Trace all reads of AI config offsets `+0xa8`, `+0xac`, `+0xb0`, and `+0xbc`.
3. Join Resource 140/141 DDX fields to the `net_*.zrd` parser and writer.
4. Reconstruct each `valves.zrd` action signature from its dispatch table;
   corpus argument counts alone are not enough.
5. Add typed UI only after the corresponding row in this document reaches a
   confirmed evidence level.
