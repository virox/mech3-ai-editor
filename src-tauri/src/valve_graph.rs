use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use crate::{WorkspaceArchive, WorkspaceDocument, WorkspaceSnapshot};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphSnapshot {
    pub scope_key: String,
    pub nodes: Vec<ValveGraphNode>,
    pub edges: Vec<ValveGraphEdge>,
    pub channels: Vec<ValveGraphChannel>,
    pub hosts: Vec<ValveGraphHost>,
    pub diagnostics: Vec<ValveGraphDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphNode {
    pub id: String,
    pub kind: String,
    pub category: String,
    pub label: String,
    pub subtitle: String,
    pub layer: String,
    pub locked: bool,
    pub order: usize,
    pub binding: ValveGraphBinding,
    pub ports: Vec<ValveGraphPort>,
    pub properties: Value,
    pub native_record: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphBinding {
    pub document_id: String,
    pub archive_relative_path: String,
    pub entry_name: String,
    pub record_kind: String,
    pub locator: String,
    pub container_path: Vec<usize>,
    pub start: usize,
    pub width: usize,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphPort {
    pub id: String,
    pub label: String,
    pub direction: String,
    pub channel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    pub binding: ValveGraphPortBinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphPortBinding {
    pub document_id: String,
    pub path: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_container_path: Option<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_width: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub channel: String,
    /// `None` for channel edges. `"route"` links a network gate to an assignment on a
    /// link leaving that node; it is derived from topology and never compiled back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphChannel {
    pub id: String,
    pub name: String,
    pub producer_count: usize,
    pub consumer_count: usize,
    pub layers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphHost {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub document_id: String,
    pub archive_relative_path: String,
    pub entry_name: String,
    pub layer: String,
    pub locked: bool,
    pub path: Vec<usize>,
    pub properties: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValveGraphDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase")]
pub(crate) enum ValveGraphEdit {
    AddBlock {
        block_kind: String,
        host_id: String,
        native_record: Value,
    },
    UpdateBlock {
        node_id: String,
        native_record: Value,
    },
    RemoveBlock {
        node_id: String,
    },
    Connect {
        port_id: String,
        channel_name: String,
    },
    Disconnect {
        port_id: String,
    },
    RenameChannel {
        old_name: String,
        new_name: String,
        #[serde(default)]
        occurrence_ids: Vec<String>,
    },
}

#[derive(Default)]
struct Builder<'a> {
    archives: HashMap<&'a str, (&'a str, bool)>,
    nodes: Vec<ValveGraphNode>,
    hosts: Vec<ValveGraphHost>,
    diagnostics: Vec<ValveGraphDiagnostic>,
    route_edges: Vec<ValveGraphEdge>,
    order: usize,
}

fn pair_list_with_path(root: &Value) -> Option<(&Vec<Value>, Vec<usize>)> {
    let outer = root.as_array()?;
    if outer.len() == 1 && outer[0].is_array() {
        Some((outer[0].as_array()?, vec![0]))
    } else {
        Some((outer, vec![]))
    }
}

fn path_value_mut<'a>(root: &'a mut Value, path: &[usize]) -> Option<&'a mut Value> {
    let mut current = root;
    for index in path {
        current = current.as_array_mut()?.get_mut(*index)?;
    }
    Some(current)
}

fn path_array_mut<'a>(root: &'a mut Value, path: &[usize]) -> Result<&'a mut Vec<Value>, String> {
    path_value_mut(root, path)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("Native record path {path:?} is no longer an array"))
}

fn archive_info<'a>(builder: &Builder<'a>, document: &WorkspaceDocument) -> (&'a str, bool) {
    builder
        .archives
        .get(document.archive_id.as_str())
        .copied()
        .unwrap_or(("unknown", true))
}

fn stable_prefix(document: &WorkspaceDocument, archive_path: &str) -> String {
    format!(
        "{}::{}",
        archive_path.replace('\\', "/"),
        document.entry_name
    )
}

fn binding(
    document: &WorkspaceDocument,
    archive_path: &str,
    record_kind: &str,
    locator: String,
    container_path: Vec<usize>,
    start: usize,
    width: usize,
    mode: &str,
) -> ValveGraphBinding {
    ValveGraphBinding {
        document_id: document.id.clone(),
        archive_relative_path: archive_path.into(),
        entry_name: document.entry_name.clone(),
        record_kind: record_kind.into(),
        locator,
        container_path,
        start,
        width,
        mode: mode.into(),
    }
}

fn port(
    node_id: &str,
    index: usize,
    label: impl Into<String>,
    direction: &str,
    channel: impl Into<String>,
    value: Option<i64>,
    document_id: &str,
    path: Vec<usize>,
) -> ValveGraphPort {
    ValveGraphPort {
        id: format!("{node_id}:port:{index}"),
        label: label.into(),
        direction: direction.into(),
        channel: channel.into(),
        value,
        binding: ValveGraphPortBinding {
            document_id: document_id.into(),
            path,
            remove_container_path: None,
            remove_start: None,
            remove_width: None,
        },
    }
}

fn removable(
    mut port: ValveGraphPort,
    container: Vec<usize>,
    start: usize,
    width: usize,
) -> ValveGraphPort {
    port.binding.remove_container_path = Some(container);
    port.binding.remove_start = Some(start);
    port.binding.remove_width = Some(width);
    port
}

impl<'a> Builder<'a> {
    fn new(archives: &'a [WorkspaceArchive]) -> Self {
        Self {
            archives: archives
                .iter()
                .map(|archive| {
                    (
                        archive.id.as_str(),
                        (archive.relative_path.as_str(), archive.locked),
                    )
                })
                .collect(),
            ..Default::default()
        }
    }

    fn diagnostic(
        &mut self,
        severity: &str,
        code: &str,
        message: impl Into<String>,
        node_id: Option<String>,
        document_id: Option<String>,
    ) {
        self.diagnostics.push(ValveGraphDiagnostic {
            severity: severity.into(),
            code: code.into(),
            message: message.into(),
            node_id,
            document_id,
        });
    }

    fn add_node(&mut self, mut node: ValveGraphNode) {
        node.order = self.order;
        self.order += 1;
        self.nodes.push(node);
    }

    fn add_host(
        &mut self,
        document: &WorkspaceDocument,
        kind: &str,
        label: impl Into<String>,
        path: Vec<usize>,
        locator: &str,
        properties: Value,
    ) {
        let (archive_path, locked) = archive_info(self, document);
        self.hosts.push(ValveGraphHost {
            id: format!(
                "host::{}::{kind}::{locator}",
                stable_prefix(document, archive_path)
            ),
            kind: kind.into(),
            label: label.into(),
            document_id: document.id.clone(),
            archive_relative_path: archive_path.into(),
            entry_name: document.entry_name.clone(),
            layer: document.layer.clone(),
            locked,
            path,
            properties,
        });
    }
}

fn scalar(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => String::new(),
    }
}

/// `FUN_004d8e70`: Fired on (3), Heat (6), Health (4), Ammo (1), Target Distance (12)
/// and Target LOS (10) are alternative triggers; the region selector (8/16 with region 9)
/// is consulted only when none fired. Without either, the row never writes.
fn vlist_row_can_fire(row: &[Value]) -> bool {
    let flag = |index: usize| integer(row.get(index)).is_some_and(|value| value != 0)
        || row.get(index).and_then(Value::as_bool) == Some(true);
    let region = row.get(9).and_then(Value::as_str).is_some_and(|name| !name.is_empty());
    [1, 3, 4, 6, 10, 12].into_iter().any(flag) || (region && (flag(8) || flag(16)))
}

fn integer(value: Option<&Value>) -> Option<i64> {
    value.and_then(|value| {
        value.as_i64().or_else(|| {
            value
                .as_f64()
                .filter(|number| {
                    number.is_finite()
                        && number.fract() == 0.0
                        && *number >= i64::MIN as f64
                        && *number <= i64::MAX as f64
                })
                .map(|number| number as i64)
        })
    })
}

fn find_tag(record: &[Value], tag: &str, start: usize) -> Option<usize> {
    let mut index = start;
    while index + 1 < record.len() {
        if record[index].as_str() == Some(tag) {
            return Some(index);
        }
        index += 2;
    }
    None
}

fn valve_node(
    builder: &mut Builder<'_>,
    document: &WorkspaceDocument,
    archive_path: &str,
    locked: bool,
    root_path: &[usize],
    pair_index: usize,
    event: &str,
    record: &[Value],
) {
    let action = record.first().and_then(Value::as_str).unwrap_or("unknown");
    let node_id = format!(
        "node::{}::valve-pair:{pair_index}",
        stable_prefix(document, archive_path)
    );
    let mut ports = Vec::new();
    let event_path = [root_path, &[pair_index]].concat();
    let mut category = "consumer";
    let mut kind = format!("valve-{action}");
    let mut label = action.replace('_', " ");
    let mut subtitle = event.to_owned();
    let mut properties = json!({"event":event,"actionType":action,"source":"valves.zrd"});

    if matches!(action, "any_nonzero" | "all_nonzero" | "all_zero") {
        category = "logic";
        kind = "derived-state".into();
        label = action.replace('_', " ");
        subtitle = format!("output: {event}");
        if record.get(1).and_then(Value::as_str) != Some("namelist")
            || record.get(2).and_then(Value::as_array).is_none()
        {
            builder.diagnostic(
                "error",
                "malformed-aggregate",
                format!("{event} has a malformed {action} record"),
                Some(node_id.clone()),
                Some(document.id.clone()),
            );
        }
        let names = record
            .get(2)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if names.is_empty() {
            builder.diagnostic(
                "error",
                "empty-aggregate",
                format!("{event} has an empty {action} input list"),
                Some(node_id.clone()),
                Some(document.id.clone()),
            );
        }
        for (index, name) in names.iter().enumerate() {
            let channel = scalar(Some(name));
            let path = [root_path, &[pair_index + 1, 2, index]].concat();
            let container = [root_path, &[pair_index + 1, 2]].concat();
            ports.push(removable(
                port(
                    &node_id,
                    ports.len(),
                    format!("Input {}", index + 1),
                    "read",
                    channel,
                    None,
                    &document.id,
                    path,
                ),
                container,
                index,
                1,
            ));
        }
        ports.push(port(
            &node_id,
            ports.len(),
            "Derived output",
            "write",
            event,
            None,
            &document.id,
            event_path,
        ));
        properties = json!({"event":event,"actionType":action,"members":names});
    } else if action == "delayupdate" {
        category = "logic";
        kind = "delay-update".into();
        label = "Delayed update".into();
        ports.push(port(
            &node_id,
            0,
            "Trigger",
            "read",
            event,
            None,
            &document.id,
            event_path,
        ));
        if let Some(args) = record.get(1).and_then(Value::as_array) {
            let target = scalar(args.first());
            let seconds = args.get(1).cloned().unwrap_or(Value::Null);
            label = format!("Delay to {target}");
            subtitle = format!(
                "{} s · copies trigger value",
                scalar(args.get(1))
            );
            ports.push(port(
                &node_id,
                1,
                "Delayed output",
                "write",
                target.clone(),
                None,
                &document.id,
                [root_path, &[pair_index + 1, 1, 0]].concat(),
            ));
            properties = json!({"event":event,"actionType":action,"target":target,"seconds":seconds,"outputValue":"trigger"});
        } else {
            builder.diagnostic(
                "error",
                "malformed-delayupdate",
                format!("{event} has a malformed delayupdate payload"),
                Some(node_id.clone()),
                Some(document.id.clone()),
            );
        }
    } else {
        ports.push(port(
            &node_id,
            0,
            "Trigger",
            "read",
            event,
            None,
            &document.id,
            event_path,
        ));
        if !matches!(
            action,
            "sound" | "sound_once" | "shutdown" | "destroy" | "teleport" | "attack_strategy"
        ) {
            kind = "unknown-action".into();
            category = "unknown";
            label = if action.is_empty() {
                "unknown action".into()
            } else {
                format!("{action} (raw)")
            };
        }
        if let Some(args) = record.get(1).and_then(Value::as_array) {
            properties = json!({"event":event,"actionType":action,"arguments":args});
        }
    }

    let native_record = Value::Array(vec![
        Value::String(event.into()),
        Value::Array(record.to_vec()),
    ]);
    if kind != "unknown-action" {
        if let Err(message) = validate_native_record(&kind, &native_record) {
            builder.diagnostic(
                "error",
                "malformed-known-valve-action",
                format!("{event}: {message}"),
                Some(node_id.clone()),
                Some(document.id.clone()),
            )
        }
    } else {
        builder.diagnostic(
            "info",
            "unknown-valve-action",
            format!("{event} is preserved as a read-only raw action"),
            Some(node_id.clone()),
            Some(document.id.clone()),
        );
    }
    builder.add_node(ValveGraphNode {
        id: node_id,
        kind,
        category: category.into(),
        label,
        subtitle,
        layer: document.layer.clone(),
        locked,
        order: 0,
        binding: binding(
            document,
            archive_path,
            "valve-action",
            format!("pair:{pair_index}"),
            root_path.to_vec(),
            pair_index,
            2,
            "slice",
        ),
        ports,
        properties,
        native_record,
    });
}

fn collect_valves(builder: &mut Builder<'_>, document: &WorkspaceDocument) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    let Some((list, root_path)) = pair_list_with_path(&document.root) else {
        builder.diagnostic(
            "error",
            "malformed-valves-root",
            format!("{} is not an ordered pair list", document.entry_name),
            None,
            Some(document.id.clone()),
        );
        return;
    };
    builder.add_host(
        document,
        "valves-document",
        format!("{} · {}", document.entry_name, document.layer),
        root_path.clone(),
        "root",
        json!({}),
    );
    for pair_index in (0..list.len().saturating_sub(1)).step_by(2) {
        let Some(event) = list[pair_index].as_str() else {
            continue;
        };
        let Some(record) = list[pair_index + 1].as_array() else {
            builder.diagnostic(
                "error",
                "malformed-valve-record",
                format!("Valve pair at index {pair_index} has no action array"),
                None,
                Some(document.id.clone()),
            );
            continue;
        };
        valve_node(
            builder,
            document,
            &archive_path,
            locked,
            &root_path,
            pair_index,
            event,
            record,
        );
    }
}

fn collect_aiv(builder: &mut Builder<'_>, document: &WorkspaceDocument) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    let Some((list, root_path)) = pair_list_with_path(&document.root) else {
        return;
    };
    for pair_index in (0..list.len().saturating_sub(1)).step_by(2) {
        let Some(vehicle) = list[pair_index].as_str() else {
            continue;
        };
        let Some(record) = list[pair_index + 1].as_array() else {
            continue;
        };
        let vehicle_path = [root_path.as_slice(), &[pair_index + 1]].concat();
        builder.add_host(
            document,
            "vehicle-vlist",
            vehicle,
            vehicle_path.clone(),
            &format!("vehicle:{pair_index}"),
            json!({"vehicle":vehicle}),
        );
        let Some(tag_index) = find_tag(record, "vlist", 4) else {
            continue;
        };
        let Some(rows) = record[tag_index + 1].as_array() else {
            continue;
        };
        let rows_path = [vehicle_path.as_slice(), &[tag_index + 1]].concat();
        for (row_index, row_value) in rows.iter().enumerate() {
            let Some(row) = row_value.as_array() else {
                continue;
            };
            let node_id = format!(
                "node::{}::vehicle:{pair_index}:vlist:{row_index}",
                stable_prefix(document, &archive_path)
            );
            if row.len() != 17 {
                builder.diagnostic(
                    "error",
                    "invalid-vlist-length",
                    format!(
                        "{vehicle} VList row {} has {} fields; expected 17",
                        row_index + 1,
                        row.len()
                    ),
                    Some(node_id.clone()),
                    Some(document.id.clone()),
                );
            }
            if let Err(message) = validate_native_record("vlist-producer", row_value) {
                builder.diagnostic(
                    "error",
                    "malformed-vlist-row",
                    format!("{vehicle} VList row {}: {message}", row_index + 1),
                    Some(node_id.clone()),
                    Some(document.id.clone()),
                )
            }
            if row.len() == 17 && !vlist_row_can_fire(row) {
                builder.diagnostic(
                    "warning",
                    "vlist-never-fires",
                    format!(
                        "{vehicle} VList row {} has no enabled trigger and no usable region selector, so it never writes its channel",
                        row_index + 1
                    ),
                    Some(node_id.clone()),
                    Some(document.id.clone()),
                );
            }
            let channel = scalar(row.get(15));
            let state = integer(row.first());
            let region = scalar(row.get(9));
            let mut ports = Vec::new();
            if row.len() > 15 {
                ports.push(port(
                    &node_id,
                    0,
                    "State output",
                    "write",
                    channel.clone(),
                    state,
                    &document.id,
                    [rows_path.as_slice(), &[row_index, 15]].concat(),
                ));
            }
            builder.add_node(ValveGraphNode {
                id: node_id,
                kind: "vlist-producer".into(),
                category: "producer".into(),
                label: "AI / region condition".into(),
                subtitle: vehicle.into(),
                layer: document.layer.clone(),
                locked,
                order: 0,
                binding: binding(
                    document,
                    &archive_path,
                    "vlist-row",
                    format!("vehicle:{pair_index}:vlist:{row_index}"),
                    rows_path.clone(),
                    row_index,
                    1,
                    "element",
                ),
                ports,
                properties: json!({"vehicle":vehicle,"region":region,"state":state,"rowIndex":row_index}),
                native_record: Value::Array(row.to_vec()),
            });
        }
    }
}

fn collect_targets(builder: &mut Builder<'_>, document: &WorkspaceDocument) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    let Some((list, root_path)) = pair_list_with_path(&document.root) else {
        return;
    };
    for pair_index in (0..list.len().saturating_sub(1)).step_by(2) {
        let Some(target) = list[pair_index].as_str() else {
            continue;
        };
        let Some(record) = list[pair_index + 1].as_array() else {
            continue;
        };
        let record_path = [root_path.as_slice(), &[pair_index + 1]].concat();
        builder.add_host(
            document,
            "target-health",
            target,
            record_path.clone(),
            &format!("target:{pair_index}"),
            json!({"target":target}),
        );
        let Some(tag_index) = find_tag(record, "health_below", 1) else {
            continue;
        };
        let Some(values) = record[tag_index + 1].as_array() else {
            continue;
        };
        let values_path = [record_path.as_slice(), &[tag_index + 1]].concat();
        if values.len() % 3 != 0 {
            builder.diagnostic(
                "error",
                "invalid-health-below-length",
                format!("{target}.health_below is not a sequence of triples"),
                None,
                Some(document.id.clone()),
            );
        }
        for triple_index in 0..values.len() / 3 {
            let start = triple_index * 3;
            let channel = scalar(values.get(start));
            let state = integer(values.get(start + 1));
            let threshold = values.get(start + 2).cloned().unwrap_or(Value::Null);
            let node_id = format!(
                "node::{}::target:{pair_index}:health:{triple_index}",
                stable_prefix(document, &archive_path)
            );
            let native_record = Value::Array(values[start..start + 3].to_vec());
            if let Err(message) = validate_native_record("target-health-producer", &native_record) {
                builder.diagnostic(
                    "error",
                    "malformed-health-below",
                    format!("{target}: {message}"),
                    Some(node_id.clone()),
                    Some(document.id.clone()),
                )
            }
            builder.add_node(ValveGraphNode {
                id: node_id.clone(),
                kind: "target-health-producer".into(),
                category: "producer".into(),
                label: "Target health threshold".into(),
                subtitle: target.into(),
                layer: document.layer.clone(),
                locked,
                order: 0,
                binding: binding(
                    document,
                    &archive_path,
                    "health-below",
                    format!("target:{pair_index}:health:{triple_index}"),
                    values_path.clone(),
                    start,
                    3,
                    "slice",
                ),
                ports: vec![port(
                    &node_id,
                    0,
                    "Threshold output",
                    "write",
                    channel.clone(),
                    state,
                    &document.id,
                    [values_path.as_slice(), &[start]].concat(),
                )],
                properties: json!({"target":target,"state":state,"threshold":threshold}),
                native_record,
            });
        }
    }
}

fn collect_network(builder: &mut Builder<'_>, document: &WorkspaceDocument) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    let Some((list, root_path)) = pair_list_with_path(&document.root) else {
        return;
    };
    // Gate blocks (valve / valveunion) by the network node or link that hosts them,
    // and assignments by the link they sit on, for the derived route edges below.
    let mut gates: Vec<(NetworkSite, String, String)> = Vec::new();
    let mut assignments: Vec<(NetworkSite, String, String)> = Vec::new();
    for pair_index in (0..list.len().saturating_sub(1)).step_by(2) {
        let Some(node_key) = list[pair_index].as_str() else {
            continue;
        };
        if !node_key.starts_with("node_") {
            continue;
        }
        let Some(record) = list[pair_index + 1].as_array() else {
            continue;
        };
        let record_path = [root_path.as_slice(), &[pair_index + 1]].concat();
        let spatial = record.first().is_some_and(Value::is_number)
            && record.get(1).is_some_and(Value::is_array);
        let site = network_site(node_key, record);
        builder.add_host(
            document,
            "network-tags",
            format!("{} · {node_key}", document.entry_name),
            record_path.clone(),
            &format!("node:{pair_index}"),
            json!({"node":node_key,"spatial":spatial}),
        );
        // Outgoing links of a spatial node; a bare `valve` target is derived from them.
        let link_count = if spatial { record.get(2).and_then(Value::as_array).map(Vec::len) } else { None };
        let mut tag_index = if spatial { 3 } else { 1 };
        while tag_index + 1 < record.len() {
            let Some(tag) = record[tag_index].as_str() else {
                tag_index += 2;
                continue;
            };
            if !matches!(tag, "valve" | "valveunion" | "valve_assign") {
                tag_index += 2;
                continue;
            }
            let Some(value) = record[tag_index + 1].as_array() else {
                tag_index += 2;
                continue;
            };
            let node_id = format!(
                "node::{}::net:{pair_index}:tag:{tag_index}",
                stable_prefix(document, &archive_path)
            );
            let mut ports = Vec::new();
            let (kind, category, label) = match tag {
                "valve_assign" => {
                    if let Some(channel) = value.first().and_then(Value::as_str) {
                        ports.push(port(
                            &node_id,
                            0,
                            "Assigned state",
                            "write",
                            channel,
                            integer(value.get(1)),
                            &document.id,
                            [record_path.as_slice(), &[tag_index + 1, 0]].concat(),
                        ));
                    }
                    ("net-assignment", "producer", "Network assignment")
                }
                "valve" => {
                    if let Some(channel) = value.get(1).and_then(Value::as_str) {
                        ports.push(port(
                            &node_id,
                            0,
                            "Route condition",
                            "read",
                            channel,
                            integer(value.get(2)),
                            &document.id,
                            [record_path.as_slice(), &[tag_index + 1, 1]].concat(),
                        ));
                    }
                    ("net-condition", "consumer", "Network valve condition")
                }
                _ => {
                    for (member_index, member) in value.iter().enumerate() {
                        let Some(member) = member.as_array() else {
                            continue;
                        };
                        let Some(channel) = member.get(1).and_then(Value::as_str) else {
                            continue;
                        };
                        let container = [record_path.as_slice(), &[tag_index + 1]].concat();
                        ports.push(removable(
                            port(
                                &node_id,
                                ports.len(),
                                format!("Alternative {}", member_index + 1),
                                "read",
                                channel,
                                integer(member.get(2)).or(Some(1)),
                                &document.id,
                                [record_path.as_slice(), &[tag_index + 1, member_index, 1]]
                                    .concat(),
                            ),
                            container,
                            member_index,
                            1,
                        ));
                    }
                    ("net-union", "consumer", "Network valve OR")
                }
            };
            let native_record = Value::Array(vec![
                Value::String(tag.into()),
                Value::Array(value.to_vec()),
            ]);
            if let Err(message) = validate_native_record(kind, &native_record) {
                builder.diagnostic(
                    "error",
                    "malformed-network-valve-record",
                    format!("{node_key}: {message}"),
                    Some(node_id.clone()),
                    Some(document.id.clone()),
                )
            }
            if let Some(site) = site {
                let entry = (site, node_id.clone(), node_key.to_owned());
                if kind == "net-assignment" { assignments.push(entry) } else { gates.push(entry) }
            }
            builder.add_node(ValveGraphNode {
                id: node_id,
                kind: kind.into(),
                category: category.into(),
                label: label.into(),
                subtitle: format!("{} · {node_key}", document.entry_name),
                layer: document.layer.clone(),
                locked,
                order: 0,
                binding: binding(
                    document,
                    &archive_path,
                    "network-tag",
                    format!("net:{pair_index}:tag:{tag_index}"),
                    record_path.clone(),
                    tag_index,
                    2,
                    "slice",
                ),
                ports,
                properties: json!({"tag":tag,"node":node_key,"spatial":spatial,"linkCount":link_count}),
                native_record,
            });
            tag_index += 2;
        }
    }
    for (site, assignment_id, assignment_key) in &assignments {
        let NetworkSite::Link(from, to) = *site else {
            continue;
        };
        for (gate_site, gate_id, gate_key) in &gates {
            // A unit only takes the link from..to once the gate on `from` (or on the
            // link itself) lets it pass.
            if *gate_site == NetworkSite::Node(from) || *gate_site == NetworkSite::Link(from, to) {
                builder.route_edges.push(ValveGraphEdge {
                    id: format!("route::{gate_id}::{assignment_id}"),
                    from: gate_id.clone(),
                    to: assignment_id.clone(),
                    channel: format!("route {gate_key} → {assignment_key} (link {from}→{to})"),
                    kind: Some("route".into()),
                });
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NetworkSite {
    Node(u64),
    Link(u64, u64),
}

/// `node_NN` records are network nodes; records that start with `[from, to]` are links.
fn network_site(node_key: &str, record: &[Value]) -> Option<NetworkSite> {
    if let Some(pair) = record.first().and_then(Value::as_array) {
        return match (pair.first().and_then(Value::as_u64), pair.get(1).and_then(Value::as_u64), pair.len()) {
            (Some(from), Some(to), 2) => Some(NetworkSite::Link(from, to)),
            _ => None,
        };
    }
    node_key.strip_prefix("node_")?.parse().ok().map(NetworkSite::Node)
}

fn mission_key(selected_mission: &str) -> String {
    let stem = Path::new(selected_mission)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    stem.strip_prefix("reader").unwrap_or(&stem).to_owned()
}

fn objective_scopes<'a>(root: &'a Value, key: &str) -> Vec<(Vec<usize>, &'a Value)> {
    let Some((list, root_path)) = pair_list_with_path(root) else {
        return vec![(vec![], root)];
    };
    for index in (0..list.len().saturating_sub(1)).step_by(2) {
        if list[index]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(key))
        {
            return vec![(
                [root_path.as_slice(), &[index + 1]].concat(),
                &list[index + 1],
            )];
        }
    }
    let staged = (0..list.len().saturating_sub(1))
        .step_by(2)
        .filter_map(|index| {
            let name = list[index].as_str()?;
            let suffix = name.get(key.len()..)?;
            if name.len() > key.len()
                && name[..key.len()].eq_ignore_ascii_case(key)
                && suffix.chars().all(|character| character.is_ascii_digit())
            {
                Some((
                    [root_path.as_slice(), &[index + 1]].concat(),
                    &list[index + 1],
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if staged.is_empty() {
        let scope = if root
            .as_array()
            .is_some_and(|array| array.len() == 1 && array[0].is_array())
        {
            &root[0]
        } else {
            root
        };
        vec![(root_path, scope)]
    } else {
        staged
    }
}

fn objective_text(record: &[Value], tag: &str) -> String {
    let mut index = 0;
    while index + 1 < record.len() {
        if record[index].as_str() == Some(tag) {
            return record[index + 1]
                .as_array()
                .and_then(|values| values.first())
                .map(|value| scalar(Some(value)))
                .unwrap_or_default();
        }
        index += 2;
    }
    String::new()
}

fn collect_objective_array(
    builder: &mut Builder<'_>,
    document: &WorkspaceDocument,
    archive_path: &str,
    locked: bool,
    value: &Value,
    path: Vec<usize>,
    phase: &str,
    host_counter: &mut usize,
) {
    let Some(array) = value.as_array() else {
        return;
    };
    let proven_list = array
        .chunks_exact(2)
        .any(|pair| pair[0].as_str() == Some("valve_change") && pair[1].is_array());
    if !phase.is_empty() && phase != "fail" && proven_list {
        let host_id = *host_counter;
        *host_counter += 1;
        builder.add_host(
            document,
            "objective-list",
            format!("{} · {}", phase, document.entry_name),
            path.clone(),
            &format!("objective:{phase}:{host_id}"),
            json!({"phase":phase}),
        );
    }
    let mut index = 0;
    while index + 1 < array.len() {
        let Some(tag) = array[index].as_str() else {
            index += 1;
            continue;
        };
        let child_path = [path.as_slice(), &[index + 1]].concat();
        if matches!(tag, "primary" | "secondary" | "final" | "fail") {
            collect_objective_array(
                builder,
                document,
                archive_path,
                locked,
                &array[index + 1],
                child_path,
                tag,
                host_counter,
            );
            index += 2;
            continue;
        }
        if tag == "set_valve" {
            if let Some(record) = array[index + 1].as_array() {
                let channel = scalar(record.first());
                let state = integer(record.get(1));
                let node_id = format!(
                    "node::{}::objective-set:{}:{}:{}",
                    stable_prefix(document, archive_path),
                    phase,
                    path.iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join("-"),
                    index
                );
                let native_record = Value::Array(vec![
                    Value::String("set_valve".into()),
                    Value::Array(record.to_vec()),
                ]);
                if let Err(message) =
                    validate_native_record("objective-set-valve", &native_record)
                {
                    builder.diagnostic(
                        "error",
                        "malformed-objective-set-valve",
                        message,
                        Some(node_id.clone()),
                        Some(document.id.clone()),
                    )
                }
                builder.add_node(ValveGraphNode {
                    id: node_id.clone(),
                    kind: "objective-set-valve".into(),
                    category: "producer".into(),
                    label: format!(
                        "Set {channel} = {}",
                        state.map_or_else(|| "?".into(), |value| value.to_string())
                    ),
                    subtitle: {
                        let description = objective_text(array, "desc");
                        if description.is_empty() {
                            format!("{} objective action", if phase.is_empty() { "mission" } else { phase })
                        } else {
                            description
                        }
                    },
                    layer: document.layer.clone(),
                    locked,
                    order: 0,
                    binding: binding(
                        document,
                        archive_path,
                        "objective-set-valve",
                        format!(
                            "objective-set:{phase}:{}:{index}",
                            path.iter()
                                .map(usize::to_string)
                                .collect::<Vec<_>>()
                                .join("-")
                        ),
                        path.clone(),
                        index,
                        2,
                        "slice",
                    ),
                    ports: vec![port(
                        &node_id,
                        0,
                        "Assigned state",
                        "write",
                        channel.clone(),
                        state,
                        &document.id,
                        [path.as_slice(), &[index + 1, 0]].concat(),
                    )],
                    properties: json!({
                        "phase":phase,
                        "description":objective_text(array,"desc"),
                        "nav":objective_text(array,"nav"),
                        "channel":channel,
                        "value":state
                    }),
                    native_record,
                });
            }
            index += 2;
            continue;
        }
        if tag == "valve_change" {
            if let Some(record) = array[index + 1].as_array() {
                let mut valve_index = None;
                let mut value_index = None;
                let mut cursor = 0;
                while cursor + 1 < record.len() {
                    match record[cursor].as_str() {
                        Some("valve") => valve_index = Some(cursor),
                        Some("value") => value_index = Some(cursor),
                        _ => {}
                    }
                    cursor += 2;
                }
                if let Some(valve_index) = valve_index {
                    let channel = record[valve_index + 1]
                        .as_array()
                        .and_then(|values| values.first())
                        .map(|value| scalar(Some(value)))
                        .unwrap_or_default();
                    let requested = value_index
                        .and_then(|at| record.get(at + 1))
                        .and_then(Value::as_array)
                        .and_then(|values| integer(values.first()));
                    let node_id = format!(
                        "node::{}::objective:{}:{}:{}",
                        stable_prefix(document, archive_path),
                        phase,
                        path.iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join("-"),
                        index
                    );
                    let channel_path = [path.as_slice(), &[index + 1, valve_index + 1, 0]].concat();
                    let native_record = Value::Array(vec![
                        Value::String("valve_change".into()),
                        Value::Array(record.to_vec()),
                    ]);
                    if let Err(message) =
                        validate_native_record("objective-observer", &native_record)
                    {
                        builder.diagnostic(
                            "error",
                            "malformed-objective-observer",
                            message,
                            Some(node_id.clone()),
                            Some(document.id.clone()),
                        )
                    }
                    builder.add_node(ValveGraphNode {
                        id: node_id.clone(),
                        kind: "objective-observer".into(),
                        category: "consumer".into(),
                        label: if phase == "fail" {
                            "Mission failure".into()
                        } else {
                            format!(
                                "{} objective",
                                if phase.is_empty() { "mission" } else { phase }
                            )
                        },
                        subtitle: objective_text(record, "desc"),
                        layer: document.layer.clone(),
                        locked,
                        order: 0,
                        binding: binding(
                            document,
                            archive_path,
                            "objective-observer",
                            format!(
                                "objective:{phase}:{}:{index}",
                                path.iter()
                                    .map(usize::to_string)
                                    .collect::<Vec<_>>()
                                    .join("-")
                            ),
                            path.clone(),
                            index,
                            2,
                            "slice",
                        ),
                        ports: vec![port(
                            &node_id,
                            0,
                            "Observed state",
                            "read",
                            channel.clone(),
                            requested,
                            &document.id,
                            channel_path,
                        )],
                        properties: json!({
                            "phase":phase,
                            "description":objective_text(record,"desc"),
                            "nav":objective_text(record,"nav"),
                            "value":requested
                        }),
                        native_record,
                    });
                } else {
                    builder.diagnostic(
                        "error",
                        "malformed-objective-valve",
                        "valve_change is missing its valve field",
                        None,
                        Some(document.id.clone()),
                    );
                }
            }
            index += 2;
            continue;
        }
        if let Some(child) = array.get(index + 1).filter(|value| value.is_array()) {
            collect_objective_array(
                builder,
                document,
                archive_path,
                locked,
                child,
                child_path,
                phase,
                host_counter,
            );
            index += 2;
        } else {
            index += 1;
        }
    }
}

fn collect_objectives(
    builder: &mut Builder<'_>,
    document: &WorkspaceDocument,
    selected_mission: &str,
) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    let key = mission_key(selected_mission);
    let mut counter = 0;
    for (path, scope) in objective_scopes(&document.root, &key) {
        collect_objective_array(
            builder,
            document,
            &archive_path,
            locked,
            scope,
            path,
            "",
            &mut counter,
        );
    }
}

/// Callbacks 42..=49 write a scene node's exact name as a state channel:
/// depth = (callback - 42) >> 1 ancestors above the definition's node, value =
/// callback & 1 (docs/aiv-research/animation-callbacks.md).
const STATE_CALLBACKS: std::ops::RangeInclusive<i64> = 42..=49;

/// Kinds whose native record body cannot be updated or removed from the graph.
pub(crate) fn read_only_kind(kind: &str) -> bool {
    matches!(kind, "unknown-action" | "animation-callback")
}

/// Kinds whose channel ports cannot be rebound. An animation callback's channel
/// is the definition `NAME`, which also identifies sibling definitions and
/// world nodes, so rewriting it from the graph would silently break the anim.
pub(crate) fn read_only_ports(kind: &str) -> bool {
    kind == "animation-callback"
}

pub(crate) fn is_animation_document(root: &Value) -> bool {
    pair_list_with_path(root)
        .and_then(|(list, _)| list.first())
        .and_then(Value::as_str)
        == Some("ANIMATION_DEFINITIONS")
}

fn tag_first<'v>(record: &'v [Value], tag: &str) -> Option<&'v Value> {
    let index = find_tag(record, tag, 0)?;
    record[index + 1].as_array()?.first()
}

fn find_animation_definitions<'v>(
    value: &'v Value,
    path: &mut Vec<usize>,
    out: &mut Vec<(Vec<usize>, &'v Vec<Value>)>,
) {
    let Some(items) = value.as_array() else {
        return;
    };
    let mut index = 0;
    while index < items.len() {
        if items[index].as_str() == Some("ANIMATION_DEFINITION") {
            if let Some(record) = items.get(index + 1).and_then(Value::as_array) {
                path.push(index + 1);
                out.push((path.clone(), record));
                path.pop();
                index += 2;
                continue;
            }
        }
        if items[index].is_array() {
            path.push(index);
            find_animation_definitions(&items[index], path, out);
            path.pop();
        }
        index += 1;
    }
}

fn animation_definitions(root: &Value) -> Vec<(Vec<usize>, &Vec<Value>)> {
    let mut out = Vec::new();
    find_animation_definitions(root, &mut Vec::new(), &mut out);
    out
}

fn tagged_records<'v>(record: &'v [Value], tag: &'v str) -> impl Iterator<Item = (usize, &'v Vec<Value>)> {
    (0..record.len().saturating_sub(1))
        .step_by(2)
        .filter(move |index| record[*index].as_str() == Some(tag))
        .filter_map(|index| record[index + 1].as_array().map(|value| (index + 1, value)))
}

pub(crate) struct AnimationCallback {
    pub callback: i64,
    /// Ancestor steps above the definition's scene node. Only depth 0 can be
    /// named from Reader data; deeper ones need the gamez.zbd hierarchy.
    pub depth: i64,
    pub value: i64,
    pub channel: String,
    pub animation_name: String,
    pub root_name: String,
    pub activation: String,
    pub execution_range: Option<Value>,
    pub sequence: String,
    pub start_time: Option<Value>,
    pub definition_index: usize,
    pub sequence_index: usize,
    pub name_path: Vec<usize>,
    pub sequence_path: Vec<usize>,
    pub event_index: usize,
    pub native_record: Value,
}

/// Every state-writing `CALLBACK` (42..=49) in an animation document. Other
/// callback values notify owner objects and do not write a named state channel.
pub(crate) fn animation_state_callbacks(root: &Value) -> Vec<AnimationCallback> {
    let mut out = Vec::new();
    for (definition_index, (definition_path, record)) in animation_definitions(root).into_iter().enumerate() {
        let Some(name_index) = find_tag(record, "NAME", 0) else {
            continue;
        };
        let channel = scalar(tag_first(record, "NAME"));
        let animation_name = scalar(tag_first(record, "ANIMATION_NAME"));
        for (sequence_index, (sequence_offset, sequence)) in tagged_records(record, "SEQUENCE_DEFINITION").enumerate() {
            for (event_offset, args) in tagged_records(sequence, "CALLBACK") {
                let Some(callback) = integer(tag_first(args, "VALUE")).filter(|value| STATE_CALLBACKS.contains(value)) else {
                    continue;
                };
                out.push(AnimationCallback {
                    callback,
                    depth: (callback - 42) >> 1,
                    value: callback & 1,
                    channel: channel.clone(),
                    animation_name: animation_name.clone(),
                    root_name: scalar(tag_first(record, "ANIMATION_ROOT_NAME")),
                    activation: scalar(tag_first(record, "ACTIVATION")),
                    execution_range: tag_first(record, "EXECUTION_BY_RANGE").cloned(),
                    sequence: scalar(tag_first(sequence, "NAME")),
                    start_time: find_tag(args, "START_TIME", 0).map(|index| args[index + 1].clone()),
                    definition_index,
                    sequence_index,
                    name_path: [definition_path.as_slice(), &[name_index + 1, 0]].concat(),
                    sequence_path: [definition_path.as_slice(), &[sequence_offset]].concat(),
                    event_index: event_offset - 1,
                    native_record: json!(["CALLBACK", args]),
                });
            }
        }
    }
    out
}

/// `(callee animation name, caller label)` for every `CALL_ANIMATION` event.
fn animation_calls(root: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (_, record) in animation_definitions(root) {
        let caller = format!(
            "{} ({})",
            scalar(tag_first(record, "ANIMATION_NAME")),
            scalar(tag_first(record, "NAME"))
        );
        for (_, sequence) in tagged_records(record, "SEQUENCE_DEFINITION") {
            for (_, args) in tagged_records(sequence, "CALL_ANIMATION") {
                let callee = scalar(tag_first(args, "NAME"));
                if !callee.is_empty() {
                    out.push((callee, caller.clone()));
                }
            }
        }
    }
    out
}

fn collect_animation_callbacks(
    builder: &mut Builder<'_>,
    document: &WorkspaceDocument,
    callers: &HashMap<String, Vec<String>>,
) {
    let (archive_path, locked) = archive_info(builder, document);
    let archive_path = archive_path.to_owned();
    for callback in animation_state_callbacks(&document.root) {
        if callback.depth > 0 {
            let relation = ["parent", "grandparent", "great-grandparent"]
                .get(callback.depth as usize - 1)
                .copied()
                .unwrap_or("ancestor");
            builder.diagnostic(
                "info",
                "animation-callback-ancestor-unresolved",
                format!(
                    "{} / {} CALLBACK {} writes the {relation} scene node of '{}' = {}; that name lives in gamez.zbd and is not shown",
                    callback.channel, callback.animation_name, callback.callback, callback.channel, callback.value
                ),
                None,
                Some(document.id.clone()),
            );
            continue;
        }
        let locator = format!(
            "animation:{}:sequence:{}:event:{}",
            callback.definition_index, callback.sequence_index, callback.event_index
        );
        let node_id = format!("node::{}::{locator}", stable_prefix(document, &archive_path));
        let called_by = callers
            .get(&callback.animation_name)
            .cloned()
            .unwrap_or_default();
        builder.add_node(ValveGraphNode {
            id: node_id.clone(),
            kind: "animation-callback".into(),
            category: "producer".into(),
            label: "Animation callback".into(),
            subtitle: format!("{} / {}", callback.channel, callback.animation_name),
            layer: document.layer.clone(),
            locked,
            order: 0,
            binding: binding(
                document,
                &archive_path,
                "animation-callback",
                locator,
                callback.sequence_path.clone(),
                callback.event_index,
                2,
                "slice",
            ),
            ports: vec![port(
                &node_id,
                0,
                format!("Callback {} output", callback.callback),
                "write",
                callback.channel.clone(),
                Some(callback.value),
                &document.id,
                callback.name_path.clone(),
            )],
            properties: json!({
                "callback": callback.callback,
                "definitionName": callback.channel,
                "animationName": callback.animation_name,
                "rootName": callback.root_name,
                "activation": callback.activation,
                "executionRange": callback.execution_range,
                "sequence": callback.sequence,
                "startTime": callback.start_time,
                "calledBy": called_by,
            }),
            native_record: callback.native_record,
        });
    }
}

fn channel_id(name: &str) -> String {
    format!("channel::{name}")
}

fn finish_graph(mut builder: Builder<'_>, scope_key: String) -> ValveGraphSnapshot {
    let mut channel_map: BTreeMap<String, (usize, usize, BTreeSet<String>)> = BTreeMap::new();
    let mut edges = std::mem::take(&mut builder.route_edges);
    for node in &builder.nodes {
        for graph_port in &node.ports {
            if graph_port.channel.is_empty() {
                builder.diagnostics.push(ValveGraphDiagnostic {
                    severity: "error".into(),
                    code: "empty-channel-name".into(),
                    message: format!("{} has an empty channel binding", node.label),
                    node_id: Some(node.id.clone()),
                    document_id: Some(node.binding.document_id.clone()),
                });
                continue;
            }
            let entry = channel_map.entry(graph_port.channel.clone()).or_default();
            entry.2.insert(node.layer.clone());
            let channel = channel_id(&graph_port.channel);
            if graph_port.direction == "write" {
                entry.0 += 1;
                edges.push(ValveGraphEdge {
                    id: format!("edge::{}::{}", graph_port.id, channel),
                    from: graph_port.id.clone(),
                    to: channel,
                    channel: graph_port.channel.clone(),
                    kind: None,
                });
            } else {
                entry.1 += 1;
                edges.push(ValveGraphEdge {
                    id: format!("edge::{}::{}", channel, graph_port.id),
                    from: channel,
                    to: graph_port.id.clone(),
                    channel: graph_port.channel.clone(),
                    kind: None,
                });
            }
        }
    }

    for (name, (producers, consumers, _)) in &channel_map {
        if *producers == 0 {
            builder.diagnostics.push(ValveGraphDiagnostic {
                severity: "info".into(),
                code: "channel-without-known-producer".into(),
                message: format!("'{name}' has no producer in the loaded Reader stack"),
                node_id: None,
                document_id: None,
            });
        }
        if *consumers == 0 {
            builder.diagnostics.push(ValveGraphDiagnostic {
                severity: "info".into(),
                code: "channel-without-known-consumer".into(),
                message: format!("'{name}' has no consumer in the loaded Reader stack"),
                node_id: None,
                document_id: None,
            });
        }
    }

    let mut by_folded: HashMap<String, BTreeSet<String>> = HashMap::new();
    for name in channel_map.keys() {
        by_folded
            .entry(name.to_ascii_lowercase())
            .or_default()
            .insert(name.clone());
    }
    for variants in by_folded.values().filter(|names| names.len() > 1) {
        builder.diagnostics.push(ValveGraphDiagnostic {
            severity: "warning".into(),
            code: "channel-case-mismatch".into(),
            message: format!(
                "Case-differing channel names remain separate: {}",
                variants.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
            node_id: None,
            document_id: None,
        });
    }

    let mut objective_use: HashMap<String, (bool, bool)> = HashMap::new();
    for node in builder
        .nodes
        .iter()
        .filter(|node| node.kind == "objective-observer")
    {
        let fail = node.properties.get("phase").and_then(Value::as_str) == Some("fail");
        for graph_port in &node.ports {
            let entry = objective_use.entry(graph_port.channel.clone()).or_default();
            if fail {
                entry.0 = true
            } else {
                entry.1 = true
            }
        }
    }
    for (name, (fail, success)) in objective_use {
        if fail && success {
            builder.diagnostics.push(ValveGraphDiagnostic {
                severity: "warning".into(),
                code: "objective-success-fail-overlap".into(),
                message: format!(
                    "'{name}' is observed by both success and failure objective records"
                ),
                node_id: None,
                document_id: None,
            });
        }
    }

    let mut channel_links: HashMap<String, Vec<String>> = HashMap::new();
    for node in &builder.nodes {
        let reads: Vec<_> = node
            .ports
            .iter()
            .filter(|port| port.direction == "read")
            .map(|port| port.channel.clone())
            .collect();
        let writes: Vec<_> = node
            .ports
            .iter()
            .filter(|port| port.direction == "write")
            .map(|port| port.channel.clone())
            .collect();
        for read in &reads {
            channel_links
                .entry(read.clone())
                .or_default()
                .extend(writes.iter().cloned());
        }
    }
    if graph_has_cycle(&channel_links) {
        builder.diagnostics.push(ValveGraphDiagnostic {
            severity: "warning".into(),
            code: "derived-channel-cycle".into(),
            message: "The loaded state graph contains a derived or delayed channel cycle".into(),
            node_id: None,
            document_id: None,
        });
    }

    let channels = channel_map
        .into_iter()
        .map(
            |(name, (producer_count, consumer_count, layers))| ValveGraphChannel {
                id: channel_id(&name),
                name,
                producer_count,
                consumer_count,
                layers: layers.into_iter().collect(),
            },
        )
        .collect();

    ValveGraphSnapshot {
        scope_key,
        nodes: builder.nodes,
        edges,
        channels,
        hosts: builder.hosts,
        diagnostics: builder.diagnostics,
    }
}

fn graph_has_cycle(links: &HashMap<String, Vec<String>>) -> bool {
    fn visit(
        node: &str,
        links: &HashMap<String, Vec<String>>,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node.into());
        if links.get(node).is_some_and(|next| {
            next.iter()
                .any(|name| visit(name, links, visiting, visited))
        }) {
            return true;
        }
        visiting.remove(node);
        visited.insert(node.into());
        false
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    links
        .keys()
        .any(|node| visit(node, links, &mut visiting, &mut visited))
}

pub(crate) fn build_valve_graph(
    documents: &[WorkspaceDocument],
    archives: &[WorkspaceArchive],
    selected_mission: &str,
) -> ValveGraphSnapshot {
    let mut builder = Builder::new(archives);
    let mut callers: HashMap<String, Vec<String>> = HashMap::new();
    for document in documents.iter().filter(|document| is_animation_document(&document.root)) {
        for (callee, caller) in animation_calls(&document.root) {
            let list = callers.entry(callee).or_default();
            if !list.contains(&caller) {
                list.push(caller);
            }
        }
    }
    for document in documents {
        match document.kind.as_str() {
            "valves" => collect_valves(&mut builder, document),
            "aiv" => collect_aiv(&mut builder, document),
            "targets" => collect_targets(&mut builder, document),
            "network" => collect_network(&mut builder, document),
            "objectives" => collect_objectives(&mut builder, document, selected_mission),
            _ if is_animation_document(&document.root) => {
                collect_animation_callbacks(&mut builder, document, &callers)
            }
            _ => {}
        }
    }
    let mut graph = finish_graph(builder, mission_key(selected_mission));
    augment_diagnostics(&mut graph, documents);
    graph
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.chars().count()).collect::<Vec<_>>();
    for (a_index, a) in left.chars().enumerate() {
        let mut current = vec![a_index + 1];
        for (b_index, b) in right.chars().enumerate() {
            current.push(
                (previous[b_index + 1] + 1)
                    .min(current[b_index] + 1)
                    .min(previous[b_index] + usize::from(a != b)),
            )
        }
        previous = current
    }
    previous.last().copied().unwrap_or(0)
}

fn augment_diagnostics(graph: &mut ValveGraphSnapshot, documents: &[WorkspaceDocument]) {
    let mut regions = HashSet::new();
    let mut vehicles = HashSet::new();
    for document in documents {
        if document.kind == "region" {
            regions.insert(
                document
                    .entry_name
                    .trim_end_matches(".zrd")
                    .to_ascii_lowercase(),
            );
        }
        if document.kind == "aiv" {
            if let Some((list, _)) = pair_list_with_path(&document.root) {
                for pair in list.chunks_exact(2) {
                    if let Some(name) = pair[0].as_str() {
                        vehicles.insert(name.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    for node in &graph.nodes {
        if node.kind == "vlist-producer" {
            if let Some(region) = node
                .properties
                .get("region")
                .and_then(Value::as_str)
                .filter(|name| !name.is_empty())
            {
                if !regions.contains(&region.to_ascii_lowercase()) {
                    graph.diagnostics.push(ValveGraphDiagnostic {
                        severity: "warning".into(),
                        code: "missing-region".into(),
                        message: format!(
                            "{} references region '{region}', which is not loaded",
                            node.subtitle
                        ),
                        node_id: Some(node.id.clone()),
                        document_id: Some(node.binding.document_id.clone()),
                    })
                }
            }
        }
        if matches!(
            node.kind.as_str(),
            "valve-shutdown" | "valve-destroy" | "valve-teleport" | "valve-attack_strategy"
        ) {
            if let Some(vehicle) = node
                .properties
                .get("arguments")
                .and_then(Value::as_array)
                .and_then(|args| args.first())
                .and_then(Value::as_str)
            {
                if !vehicles.contains(&vehicle.to_ascii_lowercase()) {
                    graph.diagnostics.push(ValveGraphDiagnostic {
                        severity: "warning".into(),
                        code: "missing-vehicle".into(),
                        message: format!(
                            "{} targets vehicle '{vehicle}', which is not loaded",
                            node.label
                        ),
                        node_id: Some(node.id.clone()),
                        document_id: Some(node.binding.document_id.clone()),
                    })
                }
            }
        }
    }
    let mut duplicate_events: HashMap<(&str, &str), usize> = HashMap::new();
    let mut valve_events = HashSet::new();
    for node in graph
        .nodes
        .iter()
        .filter(|node| node.binding.record_kind == "valve-action")
    {
        if let Some(event) = node.properties.get("event").and_then(Value::as_str) {
            *duplicate_events
                .entry((&node.binding.document_id, event))
                .or_default() += 1;
            if node.kind != "derived-state" {
                valve_events.insert(event.to_owned());
            }
        }
    }
    for ((document, event), count) in duplicate_events.into_iter().filter(|(_, count)| *count > 1) {
        graph.diagnostics.push(ValveGraphDiagnostic {
            severity: "info".into(),
            code: "duplicate-outer-event".into(),
            message: format!("'{event}' has {count} ordered valves.zrd actions; all are preserved"),
            node_id: None,
            document_id: Some(document.into()),
        })
    }
    for channel in &graph.channels {
        if !valve_events.contains(&channel.name) {
            graph.diagnostics.push(ValveGraphDiagnostic {
                severity: "info".into(),
                code: "channel-without-valve-action".into(),
                message: format!("'{}' has no loaded valves.zrd action", channel.name),
                node_id: None,
                document_id: None,
            })
        }
    }
    let produced = graph
        .channels
        .iter()
        .filter(|channel| channel.producer_count > 0)
        .map(|channel| channel.name.as_str())
        .collect::<Vec<_>>();
    for channel in graph
        .channels
        .iter()
        .filter(|channel| channel.producer_count == 0 && channel.name.len() >= 4)
    {
        if let Some((candidate, distance)) = produced
            .iter()
            .map(|candidate| {
                (
                    *candidate,
                    edit_distance(
                        &channel.name.to_ascii_lowercase(),
                        &candidate.to_ascii_lowercase(),
                    ),
                )
            })
            .filter(|(_, distance)| *distance > 0 && *distance <= 2)
            .min_by_key(|(_, distance)| *distance)
        {
            graph.diagnostics.push(ValveGraphDiagnostic{severity:"warning".into(),code:"near-spelling-channel".into(),message:format!("'{}' has no known producer; did you mean '{candidate}'? (edit distance {distance})",channel.name),node_id:None,document_id:None})
        }
    }
    let mut locked: HashMap<&str, usize> = HashMap::new();
    for host in graph.hosts.iter().filter(|host| host.locked) {
        *locked.entry(&host.archive_relative_path).or_default() += 1
    }
    for (path, count) in locked {
        graph.diagnostics.push(ValveGraphDiagnostic {
            severity: "info".into(),
            code: "locked-hosts".into(),
            message: format!("{count} native graph host(s) in {path} are locked"),
            node_id: None,
            document_id: None,
        })
    }
}

fn validate_native_record(kind: &str, record: &Value) -> Result<(), String> {
    let values = record.as_array().ok_or("Native record must be an array")?;
    match kind {
        "vlist-producer" => {
            if values.len() != 17 || !values.get(15).is_some_and(Value::is_string) {
                Err("A VList row must contain exactly 17 fields and a channel at index 15".into())
            } else {
                Ok(())
            }
        }
        "target-health-producer" => {
            if values.len() != 3
                || !values.first().is_some_and(Value::is_string)
                || !values.get(1).is_some_and(Value::is_i64)
                || !values.get(2).is_some_and(Value::is_number)
            {
                Err("A health_below block must be [channel, integer state, threshold]".into())
            } else {
                Ok(())
            }
        }
        "objective-observer" => {
            if values.len() != 2 || values.first().and_then(Value::as_str) != Some("valve_change") {
                return Err("An objective observer must be a valve_change pair".into());
            }
            let fields = values
                .get(1)
                .and_then(Value::as_array)
                .ok_or("valve_change fields must be an array")?;
            let valve = find_tag(fields, "valve", 0)
                .and_then(|at| fields.get(at + 1))
                .and_then(Value::as_array)
                .and_then(|v| v.first())
                .and_then(Value::as_str);
            let state = find_tag(fields, "value", 0)
                .and_then(|at| fields.get(at + 1))
                .and_then(Value::as_array)
                .and_then(|v| v.first())
                .and_then(Value::as_i64);
            if valve.is_none() || state.is_none() {
                Err("valve_change requires a valve name and integer value".into())
            } else {
                Ok(())
            }
        }
        "objective-set-valve" => {
            if values.len() != 2 || values.first().and_then(Value::as_str) != Some("set_valve") {
                return Err("An objective assignment must be a set_valve pair".into());
            }
            let args = values
                .get(1)
                .and_then(Value::as_array)
                .ok_or("set_valve arguments must be an array")?;
            if args.len() != 2
                || !args.first().is_some_and(Value::is_string)
                || integer(args.get(1)).is_none()
            {
                Err("set_valve requires [channel, integer value]".into())
            } else {
                Ok(())
            }
        }
        "unknown-action" => Err("Unknown valve actions are read-only".into()),
        kind if matches!(
            kind,
            "derived-state" | "derived-any-nonzero" | "derived-all-nonzero" | "derived-all-zero"
        ) =>
        {
            let action = values
                .get(1)
                .and_then(Value::as_array)
                .ok_or("A derived block must be an event/action pair")?;
            if values.len() != 2
                || !values.first().is_some_and(Value::is_string)
                || !matches!(
                    action.first().and_then(Value::as_str),
                    Some("any_nonzero" | "all_nonzero" | "all_zero")
                )
                || action.get(1).and_then(Value::as_str) != Some("namelist")
                || !action
                    .get(2)
                    .and_then(Value::as_array)
                    .is_some_and(|names| !names.is_empty() && names.iter().all(Value::is_string))
            {
                Err("A derived block requires a non-empty namelist of channels".into())
            } else {
                Ok(())
            }
        }
        kind if matches!(kind, "delay-update" | "valve-delayupdate") => {
            validate_valve_action(values, "delayupdate", |args| {
                args.len() == 2
                    && args.first().is_some_and(Value::is_string)
                    && args.get(1).is_some_and(Value::is_number)
            })
        }
        kind if matches!(kind, "valve-sound" | "valve-sound_once") => {
            validate_valve_action(values, kind.trim_start_matches("valve-"), |args| {
                args.len() == 2
                    && args.first().is_some_and(Value::is_string)
                    && args.get(1).is_some_and(Value::is_i64)
            })
        }
        "valve-shutdown" => validate_valve_action(values, "shutdown", |args| {
            args.len() == 3
                && args.first().is_some_and(Value::is_string)
                && args[1..].iter().all(Value::is_number)
        }),
        "valve-destroy" => validate_valve_action(values, "destroy", |args| {
            args.len() == 2
                && args.first().is_some_and(Value::is_string)
                && args.get(1).is_some_and(Value::is_i64)
        }),
        "valve-teleport" => validate_valve_action(values, "teleport", |args| {
            args.len() == 4
                && args.first().is_some_and(Value::is_string)
                && args
                    .get(1)
                    .and_then(Value::as_array)
                    .is_some_and(|position| {
                        position.len() == 3 && position.iter().all(Value::is_number)
                    })
                && args.get(2).is_some_and(Value::is_number)
                && args.get(3).is_some_and(Value::is_i64)
        }),
        "valve-attack_strategy" => validate_valve_action(values, "attack_strategy", |args| {
            args.len() == 3
                && args.first().is_some_and(Value::is_string)
                && args.get(1).is_some_and(Value::is_i64)
                && args.get(2).is_some_and(Value::is_i64)
        }),
        kind if matches!(kind, "net-assignment" | "net-valve-assign") => {
            validate_network_tag(values, "valve_assign", |args| {
                args.len() == 2
                    && args.first().is_some_and(Value::is_string)
                    && args.get(1).is_some_and(Value::is_i64)
            })
        }
        kind if matches!(kind, "net-condition" | "net-valve") => {
            validate_network_tag(values, "valve", |args| {
                (2..=3).contains(&args.len())
                    && args.first().is_some_and(Value::is_number)
                    && args.get(1).is_some_and(Value::is_string)
                    && args.get(2).is_none_or(Value::is_i64)
            })
        }
        kind if matches!(kind, "net-union" | "net-valveunion") => {
            validate_network_tag(values, "valveunion", |args| {
                !args.is_empty()
                    && args.iter().all(|member| {
                        member.as_array().is_some_and(|row| {
                            (2..=3).contains(&row.len())
                                && row.first().is_some_and(Value::is_number)
                                && row.get(1).is_some_and(Value::is_string)
                                && row.get(2).is_none_or(Value::is_i64)
                        })
                    })
            })
        }
        "valve-action" | "vlist-row" | "health-below" | "network-tag" => Ok(()),
        _ => Ok(()),
    }
}

fn validate_valve_action(
    values: &[Value],
    expected: &str,
    payload: impl FnOnce(&[Value]) -> bool,
) -> Result<(), String> {
    if values.len() != 2
        || !values
            .first()
            .and_then(Value::as_str)
            .is_some_and(|name| !name.is_empty())
    {
        return Err("A valves.zrd block must have a non-empty event name".into());
    }
    let action = values
        .get(1)
        .and_then(Value::as_array)
        .ok_or("A valves.zrd action must be an array")?;
    if action.first().and_then(Value::as_str) != Some(expected) {
        return Err(format!("Expected {expected} action payload"));
    }
    let args = action
        .get(1)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{expected} arguments must be an array"))?;
    if payload(args) {
        Ok(())
    } else {
        Err(format!("Malformed {expected} arguments"))
    }
}

fn validate_network_tag(
    values: &[Value],
    expected: &str,
    payload: impl FnOnce(&[Value]) -> bool,
) -> Result<(), String> {
    if values.len() != 2 || values.first().and_then(Value::as_str) != Some(expected) {
        return Err(format!("Expected {expected} network tag"));
    }
    let args = values
        .get(1)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{expected} value must be an array"))?;
    if payload(args) {
        Ok(())
    } else {
        Err(format!("Malformed {expected} network tag"))
    }
}

fn document_mut<'a>(
    snapshot: &'a mut WorkspaceSnapshot,
    document_id: &str,
) -> Result<&'a mut WorkspaceDocument, String> {
    snapshot
        .documents
        .iter_mut()
        .find(|document| document.id == document_id)
        .ok_or_else(|| format!("Document '{document_id}' is no longer loaded"))
}

fn ensure_unlocked(snapshot: &WorkspaceSnapshot, document_id: &str) -> Result<(), String> {
    let document = snapshot
        .documents
        .iter()
        .find(|document| document.id == document_id)
        .ok_or("Document is no longer loaded")?;
    let locked = snapshot
        .archives
        .iter()
        .find(|archive| archive.id == document.archive_id)
        .map_or(true, |archive| archive.locked);
    if locked {
        Err(format!("{} is locked", document.entry_name))
    } else {
        Ok(())
    }
}

fn replace_binding(
    document: &mut WorkspaceDocument,
    graph_binding: &ValveGraphBinding,
    native_record: Value,
) -> Result<(), String> {
    validate_native_record(&graph_binding.record_kind, &native_record)?;
    let container = path_array_mut(&mut document.root, &graph_binding.container_path)?;
    if graph_binding.start + graph_binding.width > container.len() {
        return Err("The native record moved since the graph was refreshed".into());
    }
    if graph_binding.mode == "element" {
        if graph_binding.width != 1 {
            return Err("Invalid element binding width".into());
        }
        container[graph_binding.start] = native_record;
        return Ok(());
    }
    let replacement = native_record
        .as_array()
        .ok_or("Slice replacement must be an array")?;
    if replacement.len() != graph_binding.width {
        return Err(format!(
            "Expected {} native values, got {}",
            graph_binding.width,
            replacement.len()
        ));
    }
    container.splice(
        graph_binding.start..graph_binding.start + graph_binding.width,
        replacement.iter().cloned(),
    );
    Ok(())
}

fn remove_binding(
    document: &mut WorkspaceDocument,
    graph_binding: &ValveGraphBinding,
) -> Result<(), String> {
    let container = path_array_mut(&mut document.root, &graph_binding.container_path)?;
    if graph_binding.start + graph_binding.width > container.len() {
        return Err("The native record moved since the graph was refreshed".into());
    }
    container.drain(graph_binding.start..graph_binding.start + graph_binding.width);
    Ok(())
}

fn ensure_tag_array<'a>(
    record: &'a mut Vec<Value>,
    tag: &str,
    start: usize,
) -> Result<&'a mut Vec<Value>, String> {
    if let Some(index) = find_tag(record, tag, start) {
        return record[index + 1]
            .as_array_mut()
            .ok_or_else(|| format!("{tag} is not an array"));
    }
    record.push(Value::String(tag.into()));
    record.push(Value::Array(vec![]));
    record
        .last_mut()
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("Could not create {tag}"))
}

fn add_to_host(
    snapshot: &mut WorkspaceSnapshot,
    host: &ValveGraphHost,
    block_kind: &str,
    native_record: Value,
) -> Result<(), String> {
    ensure_unlocked(snapshot, &host.document_id)?;
    let compatible = match host.kind.as_str() {
        "valves-document" => {
            block_kind.starts_with("valve-")
                || block_kind.starts_with("derived-")
                || block_kind == "derived-state"
                || block_kind == "delay-update"
        }
        "vehicle-vlist" => block_kind == "vlist-producer",
        "target-health" => block_kind == "target-health-producer",
        "network-tags" => block_kind.starts_with("net-"),
        "objective-list" => block_kind == "objective-observer",
        _ => false,
    };
    if !compatible {
        return Err(format!(
            "{} blocks cannot be added to a {} host",
            block_kind, host.kind
        ));
    }
    validate_native_record(block_kind, &native_record)?;
    let document = document_mut(snapshot, &host.document_id)?;
    match host.kind.as_str() {
        "valves-document" => {
            let values = native_record
                .as_array()
                .ok_or("Valve pair must be an array")?;
            if values.len() != 2 {
                return Err("Valve pair must have two values".into());
            }
            path_array_mut(&mut document.root, &host.path)?.extend(values.iter().cloned());
        }
        "vehicle-vlist" => {
            let row = native_record
                .as_array()
                .ok_or("VList row must be an array")?;
            if row.len() != 17 {
                return Err("A VList row must contain exactly 17 fields".into());
            }
            let record = path_array_mut(&mut document.root, &host.path)?;
            ensure_tag_array(record, "vlist", 4)?.push(native_record);
        }
        "target-health" => {
            let triple = native_record
                .as_array()
                .ok_or("health_below triple must be an array")?;
            if triple.len() != 3 {
                return Err("health_below needs [channel, state, threshold]".into());
            }
            let record = path_array_mut(&mut document.root, &host.path)?;
            ensure_tag_array(record, "health_below", 1)?.extend(triple.iter().cloned());
        }
        "network-tags" => {
            let pair = native_record
                .as_array()
                .ok_or("Network tag must be an array")?;
            if pair.len() != 2 {
                return Err("Network tag needs [tag, value]".into());
            }
            path_array_mut(&mut document.root, &host.path)?.extend(pair.iter().cloned());
        }
        "objective-list" => {
            let pair = native_record
                .as_array()
                .ok_or("Objective record must be an array")?;
            if pair.len() != 2 || pair[0].as_str() != Some("valve_change") {
                return Err("Only confirmed valve_change objective observers can be added".into());
            }
            if host.properties.get("phase").and_then(Value::as_str) == Some("fail") {
                return Err("Do not add parallel fail observers; connect the existing fail observer to a derived channel".into());
            }
            path_array_mut(&mut document.root, &host.path)?.extend(pair.iter().cloned());
        }
        _ => return Err(format!("Host kind '{}' cannot receive blocks", host.kind)),
    }
    Ok(())
}

fn apply_edits_to_staged(
    snapshot: &mut WorkspaceSnapshot,
    edits: &[ValveGraphEdit],
) -> Result<(), String> {
    for edit in edits {
        let graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        match edit {
            ValveGraphEdit::AddBlock {
                block_kind,
                host_id,
                native_record,
            } => {
                let host = graph
                    .hosts
                    .iter()
                    .find(|host| host.id == *host_id)
                    .ok_or("Graph host is no longer available")?;
                add_to_host(snapshot, host, block_kind, native_record.clone())?;
            }
            ValveGraphEdit::UpdateBlock {
                node_id,
                native_record,
            } => {
                let node = graph
                    .nodes
                    .iter()
                    .find(|node| node.id == *node_id)
                    .ok_or("Graph block is no longer available")?;
                if read_only_kind(&node.kind) {
                    return Err(format!("{} blocks are read-only", node.kind));
                }
                ensure_unlocked(snapshot, &node.binding.document_id)?;
                validate_native_record(&node.kind, native_record)?;
                replace_binding(
                    document_mut(snapshot, &node.binding.document_id)?,
                    &node.binding,
                    native_record.clone(),
                )?;
            }
            ValveGraphEdit::RemoveBlock { node_id } => {
                let node = graph
                    .nodes
                    .iter()
                    .find(|node| node.id == *node_id)
                    .ok_or("Graph block is no longer available")?;
                if read_only_kind(&node.kind) {
                    return Err(format!("{} blocks are read-only", node.kind));
                }
                ensure_unlocked(snapshot, &node.binding.document_id)?;
                remove_binding(
                    document_mut(snapshot, &node.binding.document_id)?,
                    &node.binding,
                )?;
            }
            ValveGraphEdit::Connect {
                port_id,
                channel_name,
            } => {
                if channel_name.is_empty() {
                    return Err("Channel names cannot be empty".into());
                }
                let (node, graph_port) = graph
                    .nodes
                    .iter()
                    .flat_map(|node| node.ports.iter().map(move |port| (node, port)))
                    .find(|(_, port)| port.id == *port_id)
                    .ok_or("Graph port is no longer available")?;
                if read_only_ports(&node.kind) {
                    return Err(format!("{} ports are read-only", node.kind));
                }
                ensure_unlocked(snapshot, &graph_port.binding.document_id)?;
                let document = document_mut(snapshot, &graph_port.binding.document_id)?;
                *path_value_mut(&mut document.root, &graph_port.binding.path)
                    .ok_or("Channel field moved since the graph was refreshed")? =
                    Value::String(channel_name.clone());
            }
            ValveGraphEdit::Disconnect { port_id } => {
                let graph_port = graph
                    .nodes
                    .iter()
                    .flat_map(|node| &node.ports)
                    .find(|port| port.id == *port_id)
                    .ok_or("Graph port is no longer available")?;
                let Some(container_path) = &graph_port.binding.remove_container_path else {
                    return Err("This is a required port; remove the block or connect it to another channel".into());
                };
                ensure_unlocked(snapshot, &graph_port.binding.document_id)?;
                let start = graph_port
                    .binding
                    .remove_start
                    .ok_or("Disconnect binding is incomplete")?;
                let width = graph_port.binding.remove_width.unwrap_or(1);
                let container = path_array_mut(
                    &mut document_mut(snapshot, &graph_port.binding.document_id)?.root,
                    container_path,
                )?;
                if start + width > container.len() {
                    return Err("Input moved since the graph was refreshed".into());
                }
                container.drain(start..start + width);
            }
            ValveGraphEdit::RenameChannel {
                old_name,
                new_name,
                occurrence_ids,
            } => {
                if old_name.is_empty() || new_name.is_empty() {
                    return Err("Channel names cannot be empty".into());
                }
                let selected: HashSet<_> = occurrence_ids.iter().collect();
                let mut bindings = Vec::new();
                for node in &graph.nodes {
                    for graph_port in &node.ports {
                        if graph_port.channel != *old_name {
                            continue;
                        }
                        if read_only_ports(&node.kind) {
                            if selected.contains(&graph_port.id) {
                                return Err(format!(
                                    "{} occurrences of '{old_name}' are read-only",
                                    node.kind
                                ));
                            }
                            continue;
                        }
                        if selected.is_empty() || selected.contains(&graph_port.id) {
                            bindings.push(graph_port.binding.clone());
                        }
                    }
                }
                if bindings.is_empty() {
                    return Err(format!(
                        "No loaded occurrences of '{old_name}' were selected"
                    ));
                }
                for graph_binding in bindings {
                    ensure_unlocked(snapshot, &graph_binding.document_id)?;
                    let document = document_mut(snapshot, &graph_binding.document_id)?;
                    *path_value_mut(&mut document.root, &graph_binding.path)
                        .ok_or("Channel occurrence moved since the graph was refreshed")? =
                        Value::String(new_name.clone());
                }
            }
        }
    }
    let graph = build_valve_graph(
        &snapshot.documents,
        &snapshot.archives,
        &snapshot.selected_mission,
    );
    snapshot.valve_graph = graph;
    Ok(())
}

pub(crate) fn apply_edits(
    snapshot: &mut WorkspaceSnapshot,
    edits: &[ValveGraphEdit],
) -> Result<(), String> {
    let baseline = build_valve_graph(
        &snapshot.documents,
        &snapshot.archives,
        &snapshot.selected_mission,
    )
    .diagnostics
    .into_iter()
    .filter(|diagnostic| diagnostic.severity == "error")
    .map(|diagnostic| {
        format!(
            "{}|{:?}|{:?}|{}",
            diagnostic.code, diagnostic.node_id, diagnostic.document_id, diagnostic.message
        )
    })
    .collect::<HashSet<_>>();
    let mut staged = snapshot.clone();
    apply_edits_to_staged(&mut staged, edits)?;
    if let Some(error) = staged.valve_graph.diagnostics.iter().find(|diagnostic| {
        diagnostic.severity == "error"
            && !baseline.contains(&format!(
                "{}|{:?}|{:?}|{}",
                diagnostic.code, diagnostic.node_id, diagnostic.document_id, diagnostic.message
            ))
    }) {
        return Err(error.message.clone());
    }
    *snapshot = staged;
    Ok(())
}

pub(crate) fn validate_for_export(snapshot: &WorkspaceSnapshot) -> Result<(), String> {
    let graph = build_valve_graph(
        &snapshot.documents,
        &snapshot.archives,
        &snapshot.selected_mission,
    );
    if let Some(error) = graph
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == "error")
    {
        Err(format!(
            "Behavior Graph validation failed: {}",
            error.message
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(id: &str, layer: &str, locked: bool) -> WorkspaceArchive {
        WorkspaceArchive {
            id: id.into(),
            path: format!("C:/{id}.zbd"),
            relative_path: format!("zbd/c3/{id}.zbd"),
            layer: layer.into(),
            locked,
        }
    }

    fn document(
        id: &str,
        archive_id: &str,
        entry: &str,
        kind: &str,
        layer: &str,
        root: Value,
    ) -> WorkspaceDocument {
        WorkspaceDocument {
            id: id.into(),
            archive_id: archive_id.into(),
            entry_name: entry.into(),
            kind: kind.into(),
            layer: layer.into(),
            root,
        }
    }

    fn workspace(
        documents: Vec<WorkspaceDocument>,
        archives: Vec<WorkspaceArchive>,
    ) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            session_id: 1,
            root_path: "C:/MW3".into(),
            selected_mission: "C:/MW3/zbd/c3/readerm4.zbd".into(),
            archives,
            documents,
            references: vec![],
            diagnostics: vec![],
            valve_graph: ValveGraphSnapshot::default(),
        }
    }

    #[test]
    fn vlist_rows_without_trigger_or_region_never_fire() {
        let row = |value: Value| value.as_array().unwrap().clone();
        // c1m1 owens_01: Health <= 0.0 -> firefly_02_go (live-verified to fire on death).
        assert!(vlist_row_can_fire(&row(json!([1,0,0.25,0,1,0.0,0,0.8,0,0,0,0.0,0,60.0,1,"firefly_02_go",0]))));
        // c1m1 player: Enters reg_firefly_retreat.
        assert!(vlist_row_can_fire(&row(json!([1,0,0.25,0,0,0.999,0,0.8,1,"reg_firefly_retreat",0,0.0,0,60.0,1,"firefly_retreat",0]))));
        // c1m1 player: Enters bit without a region (live-verified never to fire).
        assert!(!vlist_row_can_fire(&row(json!([1,0,0.25,0,0,0.999,0,0.8,1,0,0,0.0,0,60.0,1,"firefly_02_go",0]))));
        // Region present but neither selector bit.
        assert!(!vlist_row_can_fire(&row(json!([1,0,0.25,0,0,0.999,0,0.8,0,"reg_x",0,0.0,0,60.0,1,"x",0]))));
    }

    #[test]
    fn projects_c3m4_bridge_chain_and_selected_objectives() {
        let targets = document(
            "targets",
            "mission",
            "targets.zrd",
            "targets",
            "mission",
            json!([["m4factbrg", [2, "health_below", ["m4factbrg", 1, 0.0]]]]),
        );
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[
                "bridge_out",
                ["any_nonzero", "namelist", ["m4factbrg", "m4bpost"]]
            ]]),
        );
        let objectives = document(
            "objectives",
            "base",
            "objectives.zrd",
            "objectives",
            "campaign-base",
            json!([[
                "m3",
                [
                    "fail",
                    ["valve_change", ["valve", ["outside_c3m3"], "value", [1]]]
                ],
                "m4",
                [
                    "primary",
                    [
                        "valve_change",
                        ["desc", ["MSG_BRIDGE"], "valve", ["m4factbrg"], "value", [1]]
                    ],
                    "fail",
                    [
                        "valve_change",
                        [
                            "desc",
                            ["MSG_OUTSIDE"],
                            "valve",
                            ["outside_c3m4"],
                            "value",
                            [1]
                        ]
                    ]
                ]
            ]]),
        );
        let graph = build_valve_graph(
            &[targets, valves, objectives],
            &[
                archive("mission", "mission", false),
                archive("base", "campaign-base", true),
            ],
            "C:/MW3/zbd/c3/readerm4.zbd",
        );
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.kind == "target-health-producer"
                && node.ports[0].channel == "m4factbrg"));
        assert!(graph.nodes.iter().any(|node| node.kind == "derived-state"
            && node
                .ports
                .iter()
                .any(|port| port.channel == "bridge_out" && port.direction == "write")));
        assert!(graph.nodes.iter().any(
            |node| node.kind == "objective-observer" && node.ports[0].channel == "outside_c3m4"
        ));
        assert!(!graph
            .nodes
            .iter()
            .any(|node| node.ports.iter().any(|port| port.channel == "outside_c3m3")));
    }

    #[test]
    fn projects_t1m1_objective_assignment_and_delayed_success_chain() {
        let objectives = document(
            "objectives",
            "base",
            "objectives.zrd",
            "objectives",
            "campaign-base",
            json!([[
                "m113", [
                "primary", [
                    "reach_area", [
                        "desc", ["MSG_OBJ_REACH_THE_FINAL_OP_POINT"],
                        "nav", ["Baker"],
                        "set_valve", ["mis_end", 1]
                    ]
                ]],
                "m114", [
                "primary", [
                    "valve_change", [
                        "desc", ["MSG_OBJ_REACH_THE_FINAL_OP_POINT"],
                        "valve", ["mis_success"],
                        "value", [1]
                    ]
                ]],
                "m21", [
                "primary", [
                    "set_valve", ["target_fire", 1]
                ]]
            ]]),
        );
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([["mis_end", ["delayupdate", ["mis_success", 4.0]]]]),
        );
        let graph = build_valve_graph(
            &[objectives, valves],
            &[
                archive("base", "campaign-base", true),
                archive("mission", "mission", false),
            ],
            "C:/MW3/zbd/t1/readerm1.zbd",
        );

        let assignment = graph
            .nodes
            .iter()
            .find(|node| node.kind == "objective-set-valve")
            .expect("objective set_valve producer");
        assert_eq!(assignment.ports[0].channel, "mis_end");
        assert_eq!(assignment.ports[0].value, Some(1));
        assert_eq!(assignment.properties["nav"], "Baker");

        let delay = graph
            .nodes
            .iter()
            .find(|node| node.kind == "delay-update")
            .expect("delayupdate node");
        assert!(delay
            .ports
            .iter()
            .any(|port| port.direction == "read" && port.channel == "mis_end"));
        assert!(delay
            .ports
            .iter()
            .any(|port| port.direction == "write" && port.channel == "mis_success"));
        assert_eq!(delay.properties["seconds"], 4.0);
        assert_eq!(delay.properties["outputValue"], "trigger");

        assert!(graph.nodes.iter().any(|node| {
            node.kind == "objective-observer"
                && node.ports[0].channel == "mis_success"
                && node.ports[0].value == Some(1)
        }));
        assert_eq!(
            graph
                .channels
                .iter()
                .find(|channel| channel.name == "mis_end")
                .expect("mis_end channel")
                .producer_count,
            1
        );
        assert!(!graph
            .nodes
            .iter()
            .any(|node| node.ports.iter().any(|port| port.channel == "target_fire")));
    }

    #[test]
    fn preserves_duplicate_valve_pairs_and_unknown_records() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[
                "bramp",
                ["sound", ["one", 1]],
                "bramp",
                ["delayupdate", ["truck_go", 18.0]],
                ":odd",
                [":", "shutdown", [1], ":"]
            ]]),
        );
        let graph = build_valve_graph(
            &[valves],
            &[archive("mission", "mission", false)],
            "readerm1.zbd",
        );
        assert_eq!(
            graph
                .nodes
                .iter()
                .filter(|node| node
                    .ports
                    .iter()
                    .any(|port| port.channel == "bramp" && port.direction == "read"))
                .count(),
            2
        );
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.kind == "unknown-action" && node.subtitle == ":odd"));
    }

    #[test]
    fn atomically_adds_aggregate_and_rewires_fail() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[]]),
        );
        let objectives = document(
            "objectives",
            "base",
            "objectives.zrd",
            "objectives",
            "campaign-base",
            json!([[
                "m4",
                [
                    "fail",
                    [
                        "valve_change",
                        [
                            "desc",
                            ["MSG_OUTSIDE"],
                            "valve",
                            ["outside_c3m4"],
                            "value",
                            [1]
                        ]
                    ]
                ]
            ]]),
        );
        let mut snapshot = workspace(
            vec![valves, objectives],
            vec![
                archive("mission", "mission", false),
                archive("base", "campaign-base", false),
            ],
        );
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let host = snapshot
            .valve_graph
            .hosts
            .iter()
            .find(|host| host.kind == "valves-document")
            .unwrap()
            .id
            .clone();
        let fail_port = snapshot
            .valve_graph
            .nodes
            .iter()
            .find(|node| node.properties.get("phase").and_then(Value::as_str) == Some("fail"))
            .unwrap()
            .ports[0]
            .id
            .clone();
        apply_edits(
            &mut snapshot,
            &[
                ValveGraphEdit::AddBlock {
                    block_kind: "derived-state".into(),
                    host_id: host,
                    native_record: json!([
                        "m4_failure",
                        ["any_nonzero", "namelist", ["outside_c3m4", "m4factbrg"]]
                    ]),
                },
                ValveGraphEdit::Connect {
                    port_id: fail_port,
                    channel_name: "m4_failure".into(),
                },
            ],
        )
        .unwrap();
        assert!(snapshot
            .valve_graph
            .channels
            .iter()
            .any(|channel| channel.name == "m4_failure"));
        assert!(snapshot.valve_graph.nodes.iter().any(|node| node
            .properties
            .get("phase")
            .and_then(Value::as_str)
            == Some("fail")
            && node.ports[0].channel == "m4_failure"));
    }

    #[test]
    fn refuses_locked_and_missing_hosts() {
        let valves = document(
            "valves",
            "base",
            "valves.zrd",
            "valves",
            "campaign-base",
            json!([[]]),
        );
        let mut snapshot = workspace(vec![valves], vec![archive("base", "campaign-base", true)]);
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let host = snapshot.valve_graph.hosts[0].id.clone();
        let error = apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::AddBlock {
                block_kind: "derived-state".into(),
                host_id: host,
                native_record: json!(["test", ["any_nonzero", "namelist", ["a"]]]),
            }],
        )
        .unwrap_err();
        assert!(error.contains("locked"));
        let error = apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::AddBlock {
                block_kind: "derived-state".into(),
                host_id: "missing".into(),
                native_record: json!([]),
            }],
        )
        .unwrap_err();
        assert!(error.contains("host"));
    }

    #[test]
    fn projects_c3m5_nested_aggregate_chain() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[
                "m5_all",
                ["all_nonzero", "namelist", ["m5brg_a", "m5brg_b"]],
                "m5_fail",
                ["any_nonzero", "namelist", ["m5_all", "outside_c3m5"]]
            ]]),
        );
        let graph = build_valve_graph(
            &[valves],
            &[archive("mission", "mission", false)],
            "readerm5.zbd",
        );
        let all = graph
            .nodes
            .iter()
            .find(|node| node.properties.get("event").and_then(Value::as_str) == Some("m5_all"))
            .unwrap();
        let any = graph
            .nodes
            .iter()
            .find(|node| node.properties.get("event").and_then(Value::as_str) == Some("m5_fail"))
            .unwrap();
        assert!(all
            .ports
            .iter()
            .any(|port| port.direction == "write" && port.channel == "m5_all"));
        assert!(any
            .ports
            .iter()
            .any(|port| port.direction == "read" && port.channel == "m5_all"));
    }

    #[test]
    fn disconnect_and_selected_rename_preserve_network_order() {
        let network = document(
            "net",
            "mission",
            "net_01.zrd",
            "network",
            "mission",
            json!([[
                "node_00",
                [
                    12,
                    [0, 0, 0],
                    [],
                    "valveunion",
                    [[1, "bridge_a", 1], [1, "bridge_b", 0]],
                    "wait_time",
                    [1, 2.0]
                ]
            ]]),
        );
        let mut snapshot = workspace(vec![network], vec![archive("mission", "mission", false)]);
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let union = snapshot
            .valve_graph
            .nodes
            .iter()
            .find(|node| node.kind == "net-union")
            .unwrap();
        let first = union.ports[0].id.clone();
        let second = union.ports[1].id.clone();
        apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::RenameChannel {
                old_name: "bridge_a".into(),
                new_name: "bridge_main".into(),
                occurrence_ids: vec![first],
            }],
        )
        .unwrap();
        apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::Disconnect { port_id: second }],
        )
        .unwrap();
        assert_eq!(
            snapshot.documents[0].root,
            json!([[
                "node_00",
                [
                    12,
                    [0, 0, 0],
                    [],
                    "valveunion",
                    [[1, "bridge_main", 1]],
                    "wait_time",
                    [1, 2.0]
                ]
            ]])
        );
    }

    #[test]
    fn failed_multi_document_rename_rolls_back_every_occurrence() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([["shared", ["sound", ["snd", 1]]]]),
        );
        let objectives = document(
            "objectives",
            "base",
            "objectives.zrd",
            "objectives",
            "campaign-base",
            json!([[
                "m4",
                [
                    "fail",
                    [
                        "valve_change",
                        ["desc", ["MSG_FAIL"], "valve", ["shared"], "value", [1]]
                    ]
                ]
            ]]),
        );
        let mut snapshot = workspace(
            vec![valves, objectives],
            vec![
                archive("mission", "mission", false),
                archive("base", "campaign-base", true),
            ],
        );
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let before = snapshot
            .documents
            .iter()
            .map(|document| document.root.clone())
            .collect::<Vec<_>>();
        assert!(apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::RenameChannel {
                old_name: "shared".into(),
                new_name: "renamed".into(),
                occurrence_ids: vec![]
            }]
        )
        .is_err());
        assert_eq!(
            snapshot
                .documents
                .iter()
                .map(|document| document.root.clone())
                .collect::<Vec<_>>(),
            before
        );
    }

    #[test]
    fn add_then_remove_last_pair_is_an_inverse_and_preserves_duplicates() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[
                "bramp",
                ["sound", ["one", 1]],
                "bramp",
                ["sound_once", ["two", 1]]
            ]]),
        );
        let mut snapshot = workspace(vec![valves], vec![archive("mission", "mission", false)]);
        let original = snapshot.documents[0].root.clone();
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let host = snapshot.valve_graph.hosts[0].id.clone();
        apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::AddBlock {
                block_kind: "valve-destroy".into(),
                host_id: host,
                native_record: json!(["cleanup", ["destroy", ["truck", 1]]]),
            }],
        )
        .unwrap();
        let added = snapshot
            .valve_graph
            .nodes
            .iter()
            .find(|node| node.properties.get("event").and_then(Value::as_str) == Some("cleanup"))
            .unwrap()
            .id
            .clone();
        apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::RemoveBlock { node_id: added }],
        )
        .unwrap();
        assert_eq!(snapshot.documents[0].root, original);
    }

    #[test]
    fn malformed_records_block_export_but_unknown_actions_stay_read_only() {
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([[
                "broken",
                ["all_zero", "namelist", []],
                "raw",
                ["update", ["x", 1]]
            ]]),
        );
        let mut snapshot = workspace(vec![valves], vec![archive("mission", "mission", false)]);
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        assert!(snapshot
            .valve_graph
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.code == "empty-aggregate" && diagnostic.severity == "error"
            ));
        assert!(validate_for_export(&snapshot).is_err());
        let raw = snapshot
            .valve_graph
            .nodes
            .iter()
            .find(|node| node.kind == "unknown-action")
            .unwrap()
            .id
            .clone();
        assert!(apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::RemoveBlock { node_id: raw }]
        )
        .unwrap_err()
        .contains("read-only"));
    }

    #[test]
    fn animation_callback_43_writes_definition_name_and_stays_read_only() {
        let ramp = document(
            "ramp",
            "mission",
            "m1ramp.zrd",
            "raw",
            "mission",
            json!([[
                "ANIMATION_DEFINITIONS",
                [
                    "ANIMATION_LIST",
                    [
                        "ANIMATION_DEFINITION",
                        [
                            "NAME", ["bramp"],
                            "ANIMATION_NAME", ["call_ramp"],
                            "ANIMATION_ROOT_NAME", ["healthy"],
                            "ACTIVATION", ["ON_CALL"],
                            "EXECUTION_BY_RANGE", [500],
                            "SEQUENCE_DEFINITION",
                            [
                                "NAME", ["effect_sequence"],
                                "CALLBACK", ["VALUE", [43]],
                                "CALLBACK", ["VALUE", [47]],
                                "CALLBACK", ["VALUE", [57]],
                                "CALL_ANIMATION", ["NAME", ["hit_ramp"]]
                            ]
                        ],
                        "ANIMATION_DEFINITION",
                        [
                            "NAME", ["horizon"],
                            "ANIMATION_NAME", ["start_single_player_m1"],
                            "SEQUENCE_DEFINITION",
                            [
                                "NAME", ["call_most"],
                                "CALL_ANIMATION", ["NAME", ["call_ramp"]],
                                "CALLBACK", ["VALUE", [42]]
                            ]
                        ]
                    ]
                ]
            ]]),
        );
        let valves = document(
            "valves",
            "mission",
            "valves.zrd",
            "valves",
            "mission",
            json!([["bramp", ["sound_once", ["com_mech", 1]]]]),
        );
        let mut snapshot = workspace(vec![ramp, valves], vec![archive("mission", "mission", false)]);
        snapshot.valve_graph = build_valve_graph(
            &snapshot.documents,
            &snapshot.archives,
            &snapshot.selected_mission,
        );
        let callbacks: Vec<_> = snapshot
            .valve_graph
            .nodes
            .iter()
            .filter(|node| node.kind == "animation-callback")
            .collect();
        assert_eq!(callbacks.len(), 2, "42 and 43 write named channels; 47 needs gamez; 57 writes none");
        let node = callbacks[0];
        assert_eq!(node.ports[0].channel, "bramp");
        assert_eq!(node.ports[0].value, Some(1));
        assert_eq!((callbacks[1].ports[0].channel.as_str(), callbacks[1].ports[0].value), ("horizon", Some(0)));
        assert!(snapshot.valve_graph.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "animation-callback-ancestor-unresolved"
                && diagnostic.severity == "info"
                && diagnostic.message.contains("CALLBACK 47 writes the grandparent")
        }));
        assert_eq!(node.properties["calledBy"], json!(["start_single_player_m1 (horizon)"]));
        assert_eq!(node.properties["executionRange"], json!(500));
        let bramp = snapshot
            .valve_graph
            .channels
            .iter()
            .find(|channel| channel.name == "bramp")
            .unwrap();
        assert_eq!((bramp.producer_count, bramp.consumer_count), (1, 1));

        let (node_id, port_id) = (node.id.clone(), node.ports[0].id.clone());
        let before = snapshot.documents.clone();
        for edit in [
            ValveGraphEdit::RemoveBlock { node_id: node_id.clone() },
            ValveGraphEdit::UpdateBlock { node_id, native_record: json!(["CALLBACK", ["VALUE", [1]]]) },
            ValveGraphEdit::Connect { port_id: port_id.clone(), channel_name: "other".into() },
            ValveGraphEdit::RenameChannel {
                old_name: "bramp".into(),
                new_name: "other".into(),
                occurrence_ids: vec![port_id],
            },
        ] {
            assert!(apply_edits(&mut snapshot, &[edit]).unwrap_err().contains("read-only"));
        }
        // An unselected rename still rewrites the editable consumer but never the animation NAME.
        apply_edits(
            &mut snapshot,
            &[ValveGraphEdit::RenameChannel {
                old_name: "bramp".into(),
                new_name: "other".into(),
                occurrence_ids: vec![],
            }],
        )
        .unwrap();
        assert_eq!(snapshot.documents[0].root, before[0].root);
        assert_ne!(snapshot.documents[1].root, before[1].root);
    }
}
