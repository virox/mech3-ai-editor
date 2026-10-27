# AI networks and spatial regions

## Editor navigation

The graph viewport uses a camera-style SVG view box. Mouse-wheel zoom is
centered at the pointer, `+`/`−` zoom around the viewport center, empty-space
dragging pans the camera, and **Fit** restores the complete graph view. Camera
changes are UI state only and never modify Reader data. Node symbols, labels,
edge widths, and edge badges retain a constant screen size while the distances
between them scale. Node dragging is separate and writes X/Z while retaining Y
and all unrelated record values.

Every serialized connection is directed and must be rendered, including edges
whose source index is greater than the target index. Non-spatial `node_*`
records begin with a `[from,to]` pair and attach rules such as `gate` or
`valve_assign` to that directed edge; they are not additional map locations.

In campaign 1 mission 1 `net_01`, both `node_04` and `node_14` point to
`node_05`. At `node_07`, the outgoing candidates `[8,9]` have separate edge
records: `7→8` has `gate("ttruck_01")`, while `7→9` has
`gate("ttruck_02")`. The later `8→15` and `9→16` edges assign the distinct
`destroy_truck_01` and `destroy_truck_02` state channels. This structure is
evidence for rule-qualified outgoing candidates, not evidence for round-robin
selection. The exact generic selection algorithm remains a runtime research
item.

The parser at `FUN_004daee0` copies non-negative connection indices into its
vector in serialized order. The writer at `FUN_004db8d0` preserves that order
and warns/caps only when a BattleSpace-style network exceeds three outgoing
connections. Neither function implements or proves visit-count round-robin;
selection happens later in the runtime navigation path.

### Campaign 1 mission 1 Valve trace

This mission provides a complete cross-document example:

1. The `bramp` event in `valves.zrd` schedules
   `delayupdate("truck_go", 18.0)`.
2. Spatial `node_08` and `node_09` carry `valve [1,"truck_go"]`. The leading
   `1` enables the condition. Because the optional target value is omitted,
   the engine derives it from the outgoing-connection index; both nodes have
   one outgoing connection, so the derived target is `truck_go == 1` here.
3. Edge-rule records on `8→15` and `9→16` perform
   `valve_assign ["destroy_truck_01",1]` and
   `valve_assign ["destroy_truck_02",1]`.
4. `valves.zrd` consumes those state changes with
   `destroy ["ttruck_01",1]` and `destroy ["ttruck_02",1]`.

Thus `valve_assign` is an output/action, not another condition, and the word
`destroy` in a channel name has no intrinsic engine behavior. Destruction is
caused by the corresponding `valves.zrd` consumer. Nodes 15 and 16 have no
outgoing connections and serve as terminal cleanup endpoints. Static analysis
has not yet proven whether the edge assignment occurs on edge entry or edge
completion; the editor does not claim either timing.

Both `net_*.zrd` and `reg_*.zrd` use a wrapped ordered pair list with `version`, `name`, and `node_*` records. Ordinary spatial nodes contain a numeric type/flags value, `[x,y,z]`, adjacency indices, and optional tag/value pairs.

The shipped `net_*.zrd` corpus contains 16 distinct optional tag names:
`arrangenodes`, `auxname`, `canleave`, `deploy`, `gate`, `jump`, `name`,
`scan_aim`, `scan_hit`, `scan_time`, `speed`, `valve`, `valve_assign`,
`valveunion`, `wait_interval`, and `wait_time`. This is an inventory of the
supplied data, not a claim that the executable cannot recognize additional
names. `reg_*.zrd` uses the same structural parser, but no optional tags were
observed on its nodes in the supplied corpus.

Some names are attached to spatial nodes, some to non-spatial directed-edge
records, and some occur in both forms. `gate` is edge-only in the corpus. An
edge record starts with `[from,to]`, so its tags qualify that particular
directed connection; it is not an additional map node. The runtime tag
dispatch is hard-coded in the executable (`ZRdrAiNet` parsing, including
`FUN_004db1b0`), while the actual tag/value pairs originate in the serialized
Reader data. The editor uses one purple marker for any spatial node carrying
one or more tags; the selected-node color remains a selection indicator.

## Vehicle-to-network node association

An AIV vehicle has a Home Net relationship and an initial world position, but
the serialized AIV record has no explicit `start_node` field. The relevant
fields are `record[0]` (Net ID) and `record[1]` (initial X/Y/Z position). A
network node has its own position and adjacency list. This makes spatial
association necessary, but does not by itself prove the selection algorithm.

The T1M4 data gives a particularly strong correspondence:

| Vehicle | Home Net | AIV initial position | Matching `net_04` node |
|---|---:|---|---|
| `bulldog_01` | 4 | `(1197.2, 3.0, -1789.6)` | `node_47` |
| `owens_01` | 4 | `(1159.3, 3.0, -1841.9)` | `node_46` |

The positions match to the serialized float precision, and the matching nodes
carry the expected vehicle-specific valve gates (`bulldog` and `firefly`).
This proves that the authored spawn positions were deliberately aligned with
those nodes, but it does not prove that the executable chooses a node by
distance.

The executable trace establishes the surrounding runtime structure and now
identifies the node-returning resolver. In `Mech3.exe`:

* `FUN_004be690` builds the navigation runtime for network type `0x6b` and
  iterates the loaded runtime objects.
* For each object it obtains an AI/network component through virtual method
  `+0x60`, then calls that component's virtual method `+0x88` to obtain the
  current node object. The component vtable installed by `FUN_004cf330`
  (`PTR_FUN_005cc698`) resolves slot `+0x88` to `FUN_004c9180`.
* `FUN_004c9180` does not read world coordinates. It obtains a relation/object
  through the input object's `+0x60`/`+0x74` chain, rejects a null relation,
  and checks pointer identity against the component's relation vector.
  It then obtains the owning relation through `this + vtable[+0x10]`, finds
  that relation by pointer identity in the same vector, and calls the
  relation's virtual `+0x40` with the matching vector index. If that relation
  is absent, it appends it with `FUN_0049ced0` and calls `+0x40` with the old
  vector length.
* The returned object is then used by `FUN_004be690`: its adjacency list is
  read, its position is obtained through virtual method `+0x18`, and the
  position and outgoing-node data are passed to `FUN_0047a320`.
* `FUN_0047a320` stores the supplied position and outgoing-node information;
  it does not compare the position against all network-node positions.
* `FUN_004bd670` and `FUN_004bec10` build serialized-node index tables and
  adjacency metadata; they also do not perform a distance comparison.

Therefore the current evidence supports this model:

```text
AIV record[0] → Home Net
AIV record[1] → authored spawn position
                 ↓
 runtime object/relation identity and vector-index lookup
                 ↓
vehicle's current/start network node → directed route actions
```

This disproves the earlier nearest-node assumption for the inspected resolver:
there is no squared-distance calculation, candidate-radius test, or distance
tie-break in `FUN_004c9180`. The observable fallback is structural rather than
spatial: a null input relation returns no node; a relation not yet present in
the vector is appended and assigned the next vector index; the final node
object is produced by the relation's `+0x40` method. The exact implementation
of that relation-class `+0x40` method remains the only unresolved part of the
last hop, but the selection shown in the net setup is identity/index based,
not “closest node to the vehicle.”

For the editor, coordinate matches should therefore be displayed as authored
evidence and a warning should be used when the runtime relation/index cannot
be reconstructed. The editor must not silently implement a nearest-node
search, because the executable path inspected here does not do that.

The editor treats X/Z as the map plane and preserves Y separately. Special link records such as `gate` and `valve_assign` are rendered as edge properties rather than invented spatial nodes. A region is filled only when adjacency yields one unambiguous closed cycle.

## Leading values in tag arrays

Several tags use a compact `[enabled, parameter]` form. In the parser
(`FUN_004db1b0`) the first element is stored as a flag, and the navigation
builder (`FUN_004bd790`) checks that flag before creating the corresponding
runtime action. The second element is the payload. This applies to the
observed forms of `speed`, `wait_interval`, `wait_time`, `canleave`,
`scan_aim`, `scan_hit`, `scan_time`, and `jump`. Thus, for example,
`wait_interval [1, 5.0]` means an enabled five-second interval; the leading
`1` is not another duration.

`valve` uses a related but three-part form: `[enabled, channel, target]`. The
first element enables the condition, the second names the state channel, and
the optional third element is the desired state value. If the third element
is omitted, `FUN_004bd790` derives the target from the current outgoing-link
index (`current_index == link_count - 1`). This is why `valve [1,"truck_go"]`
on a node with exactly one outgoing connection resolves to `truck_go == 1`,
but the two-element form should not be interpreted as an explicit literal
`1` in every context.

`valveunion` contains one or more such state members. The parser ignores a
member whose first element is zero, defaults a missing member target to `1`,
and stores `(channel,target)` pairs. `FUN_004d4600` evaluates the collection
as a union/OR: the compound condition is true when any enabled member matches.
By contrast, `valve_assign [channel,value]` has no leading enable flag; both
elements are payload (the channel and the value to assign). Other tags such
as `gate`, `deploy`, `name`, and `auxname` have their own layouts and should
not be decoded by the generic leading-`1` rule.

The original Resource 141 `Node Edit - Vehicle` dialog proves controls for X, Y, Z, Yaw, Vehicle Type, and Friend/Foe/Other, but those dialog offsets are not assigned to serialized fields until their commit/copy path is joined to the AI-net serializer. Unproven fields retain technical labels.
