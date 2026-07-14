# AI networks and spatial regions

Both `net_*.zrd` and `reg_*.zrd` use a wrapped ordered pair list with `version`, `name`, and `node_*` records. Ordinary spatial nodes contain a numeric type/flags value, `[x,y,z]`, adjacency indices, and optional tag/value pairs.

Confirmed/observed optional network tags include `valve`, `valveunion`, `valve_assign`, `gate`, `deploy`, `arrangenodes`, `name`, `auxname`, `wait_interval`, `wait_time`, `scan_time`, `scan_aim`, `scan_hit`, `canleave`, `speed`, and `jump`.

The editor treats X/Z as the map plane and preserves Y separately. Special link records such as `gate` and `valve_assign` are rendered as edge properties rather than invented spatial nodes. A region is filled only when adjacency yields one unambiguous closed cycle.

The original Resource 141 `Node Edit - Vehicle` dialog proves controls for X, Y, Z, Yaw, Vehicle Type, and Friend/Foe/Other, but those dialog offsets are not assigned to serialized fields until their commit/copy path is joined to the AI-net serializer. Unproven fields retain technical labels.
