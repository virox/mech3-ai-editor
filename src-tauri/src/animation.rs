//! Read-only view of a campaign's compiled `anim.zbd`.
//!
//! The full definitions are large (about 10 MB of JSON per campaign), so the
//! frontend receives a compact index with only the fields that form graph
//! edges, and fetches one complete definition at a time for the inspector.

use mech3ax_common::io_ext::CountingReader;
use serde::Serialize;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Event kinds that reference another animation or sequence by name, or that
/// write state (`Callback`). Everything else stays in the full definition.
const LINK_EVENTS: &[&str] = &[
    "CallAnimation",
    "StopAnimation",
    "InvalidateAnimation",
    "ResetAnimation",
    "CallSequence",
    "StopSequence",
    "Callback",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimationIndex {
    pub path: String,
    pub relative_path: String,
    pub mission: Value,
    pub definitions: Vec<AnimationDefinitionSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimationDefinitionSummary {
    pub index: usize,
    pub file_name: String,
    pub name: String,
    pub anim_name: String,
    pub root_name: String,
    pub activation: Value,
    pub execution: Value,
    pub health: Value,
    pub reset_time: Value,
    pub has_callbacks: bool,
    pub prerequisites: Vec<AnimationPrerequisite>,
    pub sequences: Vec<AnimationSequenceSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimationPrerequisite {
    pub kind: String,
    pub name: String,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimationSequenceSummary {
    pub name: String,
    pub seq_state: Value,
    pub event_count: usize,
    pub links: Vec<AnimationLink>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnimationLink {
    pub event_index: usize,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    pub start: Value,
}

pub(crate) struct LoadedAnimations {
    pub mission: Value,
    /// `(file name, definition)` in archive order.
    pub definitions: Vec<(String, Value)>,
}

pub(crate) fn read_anim_file(path: &Path) -> Result<LoadedAnimations, String> {
    let file = File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut reader = CountingReader::new(BufReader::new(file));
    let mut definitions = Vec::new();
    let metadata = mech3ax_anim::mw::read_anim(&mut reader, |item| -> Result<(), mech3ax_common::Error> {
        if let mech3ax_anim::SaveItem::AnimDef { name, anim_def } = item {
            let value = serde_json::to_value(anim_def).map_err(std::io::Error::other)?;
            definitions.push((name.to_owned(), value));
        }
        Ok(())
    })
    .map_err(|error| format!("Could not read {}: {error:?}", path.display()))?;
    let mission = serde_json::to_value(&metadata.mission).unwrap_or(Value::Null);
    Ok(LoadedAnimations { mission, definitions })
}

fn text(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
}

/// A serde enum serializes as `{"Variant": {...}}` or as a bare string.
fn variant(value: &Value) -> Option<(&str, &Value)> {
    match value {
        Value::Object(map) if map.len() == 1 => map.iter().next().map(|(key, inner)| (key.as_str(), inner)),
        Value::String(name) => Some((name.as_str(), &Value::Null)),
        _ => None,
    }
}

fn summarize_sequence(sequence: &Value) -> AnimationSequenceSummary {
    let events = sequence.get("events").and_then(Value::as_array);
    let links = events
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(event_index, event)| {
            let (kind, data) = variant(event.get("data")?)?;
            if !LINK_EVENTS.contains(&kind) {
                return None;
            }
            Some(AnimationLink {
                event_index,
                kind: kind.to_owned(),
                target: data.get("name").and_then(Value::as_str).map(str::to_owned),
                value: data.get("value").and_then(Value::as_i64),
                start: event.get("start").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();
    AnimationSequenceSummary {
        name: text(sequence.get("name")),
        seq_state: sequence.get("seq_state").cloned().unwrap_or(Value::Null),
        event_count: events.map_or(0, Vec::len),
        links,
    }
}

fn summarize_prerequisite(value: &Value) -> Option<AnimationPrerequisite> {
    let (kind, data) = variant(value)?;
    Some(AnimationPrerequisite {
        kind: kind.to_owned(),
        name: text(data.get("name")),
        required: data.get("required").and_then(Value::as_bool),
    })
}

pub(crate) fn summarize(index: usize, file_name: &str, definition: &Value) -> AnimationDefinitionSummary {
    let field = |key: &str| definition.get(key).cloned().unwrap_or(Value::Null);
    AnimationDefinitionSummary {
        index,
        file_name: file_name.to_owned(),
        name: text(definition.get("name")),
        anim_name: text(definition.get("anim_name")),
        root_name: text(definition.get("anim_root_name")),
        activation: field("activation"),
        execution: field("execution"),
        health: field("health"),
        reset_time: field("reset_time"),
        has_callbacks: definition.get("has_callbacks").and_then(Value::as_bool).unwrap_or(false),
        prerequisites: definition
            .get("activ_prereqs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(summarize_prerequisite)
            .collect(),
        sequences: definition
            .get("sequences")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(summarize_sequence)
            .collect(),
    }
}

pub(crate) fn build_index(path: &Path, relative_path: String, loaded: &LoadedAnimations) -> AnimationIndex {
    AnimationIndex {
        path: path.to_string_lossy().into_owned(),
        relative_path,
        mission: loaded.mission.clone(),
        definitions: loaded
            .definitions
            .iter()
            .enumerate()
            .map(|(index, (file_name, definition))| summarize(index, file_name, definition))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn summary_keeps_only_link_events_with_their_positions() {
        let definition = json!({
            "name": "bramp",
            "anim_name": "call_ramp",
            "anim_root_name": "healthy",
            "has_callbacks": true,
            "activation": "OnCall",
            "execution": {"ByRange": {"min": 0.0, "max": 250000.0}},
            "activ_prereqs": [{"Animation": {"name": "hit_citydock1", "required": true}}],
            "sequences": [{
                "name": "effect_sequence",
                "seq_state": "Initial",
                "events": [
                    {"start": null, "data": {"Callback": {"value": 43}}},
                    {"start": null, "data": {"ObjectActiveState": {"node": "healthy", "state": false}}},
                    {"start": {"offset": "Animation", "time": 20.0}, "data": {"CallAnimation": {"name": "hit_ramp"}}},
                    {"start": null, "data": {"Endif": {}}}
                ]
            }]
        });
        let summary = summarize(3, "bramp-call_ramp-healthy", &definition);
        assert_eq!((summary.name.as_str(), summary.anim_name.as_str()), ("bramp", "call_ramp"));
        assert_eq!(summary.prerequisites[0].name, "hit_citydock1");
        let sequence = &summary.sequences[0];
        assert_eq!(sequence.event_count, 4);
        assert_eq!(sequence.links.len(), 2);
        assert_eq!((sequence.links[0].kind.as_str(), sequence.links[0].value), ("Callback", Some(43)));
        assert_eq!(sequence.links[1].event_index, 2);
        assert_eq!(sequence.links[1].target.as_deref(), Some("hit_ramp"));
        assert_eq!(sequence.links[1].start["time"], json!(20.0));
    }

    #[test]
    fn installed_c1_anim_zbd_contains_the_bramp_callback() {
        let Ok(root) = std::env::var("MW3_TEST_ROOT") else { return };
        let path = Path::new(&root).join("zbd").join("c1").join("anim.zbd");
        if !path.exists() {
            return;
        }
        let loaded = read_anim_file(&path).expect("read anim.zbd");
        let index = build_index(&path, "zbd/c1/anim.zbd".into(), &loaded);
        assert!(index.definitions.len() > 1000);
        let ramp = index
            .definitions
            .iter()
            .find(|definition| definition.anim_name == "call_ramp")
            .expect("call_ramp");
        assert_eq!(ramp.name, "bramp");
        assert!(ramp.sequences.iter().flat_map(|sequence| &sequence.links).any(|link| {
            link.kind == "Callback" && link.value == Some(43)
        }));
        assert!(index.definitions.iter().any(|definition| {
            definition.sequences.iter().flat_map(|sequence| &sequence.links).any(|link| {
                link.kind == "CallAnimation" && link.target.as_deref() == Some("call_ramp")
            })
        }));
    }
}
