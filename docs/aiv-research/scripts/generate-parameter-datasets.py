import csv
import json
import re
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "data"
EXTRACTED = ROOT / "extracted-reader-json"
PARAM_KEY = re.compile(r"^(?:params|p\d+)$")


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def key_order(key):
    return (-1 if key == "params" else int(key[1:]), key)


DATA.mkdir(exist_ok=True)

occurrences = []
bundles = []


def walk(node, source, path=(), entity=None):
    if isinstance(node, list):
        current_entity = entity
        if node and isinstance(node[0], str) and not PARAM_KEY.fullmatch(node[0]):
            current_entity = node[0]

        local_params = []
        index = 0
        while index < len(node):
            item = node[index]
            if isinstance(item, str) and PARAM_KEY.fullmatch(item) and index + 1 < len(node):
                value = node[index + 1]
                occurrence = {
                    "key": item,
                    "value": value,
                    "source": source,
                    "entity": current_entity or "",
                    "json_path": "/".join(map(str, path + (index,))),
                }
                occurrences.append(occurrence)
                local_params.append({"key": item, "value": value})
                walk(value, source, path + (index + 1,), current_entity)
                index += 2
                continue
            if (
                isinstance(item, str)
                and index + 1 < len(node)
                and isinstance(node[index + 1], list)
            ):
                walk(node[index + 1], source, path + (index + 1,), item)
                index += 2
                continue
            walk(item, source, path + (index,), current_entity)
            index += 1

        if local_params:
            bundles.append(
                {
                    "source": source,
                    "entity": current_entity or "",
                    "json_path": "/".join(map(str, path)),
                    "params": local_params,
                }
            )
    elif isinstance(node, dict):
        for key, value in node.items():
            walk(value, source, path + (key,), entity)


sources = sorted(EXTRACTED.rglob("aiv.zrd.json"))
for source_path in sources:
    with source_path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    walk(data, source_path.relative_to(EXTRACTED).as_posix())

by_key_value = defaultdict(list)
for occurrence in occurrences:
    by_key_value[(occurrence["key"], canonical(occurrence["value"]))].append(occurrence)

set_rows = []
next_set_id = 1
for key, value_json in sorted(by_key_value, key=lambda item: (key_order(item[0]), item[1])):
    group = by_key_value[(key, value_json)]
    set_rows.append(
        {
            "set_id": next_set_id,
            "key": key,
            "value": group[0]["value"],
            "value_json": value_json,
            "occurrence_count": len(group),
            "source_count": len({row["source"] for row in group}),
            "entity_count": len({row["entity"] for row in group}),
            "example_source": group[0]["source"],
            "example_entity": group[0]["entity"],
        }
    )
    next_set_id += 1

set_id_by_key_value = {(row["key"], row["value_json"]): row["set_id"] for row in set_rows}

with (DATA / "parameter-value-set.json").open("w", encoding="utf-8") as handle:
    json.dump(
        [{key: value for key, value in row.items() if key != "value_json"} for row in set_rows],
        handle,
        ensure_ascii=False,
        indent=2,
    )
    handle.write("\n")

max_value_fields = max(
    (len(row["value"]) for row in set_rows if isinstance(row["value"], list)),
    default=0,
)
value_fields = [f"field_{index:02d}" for index in range(max_value_fields)]
with (DATA / "parameter-value-set.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    fields = [
        "set_id", "key", "occurrence_count", "source_count", "entity_count",
        "value_type", "value_length", *value_fields, "value_json",
        "example_source", "example_entity",
    ]
    writer = csv.DictWriter(handle, fieldnames=fields)
    writer.writeheader()
    for row in set_rows:
        output = {
            **{field: row[field] for field in fields if field in row},
            "value_type": type(row["value"]).__name__,
            "value_length": len(row["value"]) if isinstance(row["value"], list) else "",
        }
        if isinstance(row["value"], list):
            output.update(
                {f"field_{index:02d}": canonical(value) for index, value in enumerate(row["value"])}
            )
        writer.writerow(output)

with (DATA / "parameter-occurrences.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    fields = ["set_id", "key", "source", "entity", "json_path", "value_json"]
    writer = csv.DictWriter(handle, fieldnames=fields)
    writer.writeheader()
    for row in occurrences:
        value_json = canonical(row["value"])
        writer.writerow(
            {
                "set_id": set_id_by_key_value[(row["key"], value_json)],
                "key": row["key"],
                "source": row["source"],
                "entity": row["entity"],
                "json_path": row["json_path"],
                "value_json": value_json,
            }
        )

by_bundle = defaultdict(list)
for bundle in bundles:
    signature = canonical(bundle["params"])
    by_bundle[signature].append(bundle)

bundle_rows = []
for bundle_id, signature in enumerate(sorted(by_bundle), start=1):
    group = by_bundle[signature]
    bundle_rows.append(
        {
            "bundle_id": bundle_id,
            "occurrence_count": len(group),
            "source_count": len({row["source"] for row in group}),
            "entity_count": len({row["entity"] for row in group}),
            "keys": [item["key"] for item in group[0]["params"]],
            "params": group[0]["params"],
            "example_source": group[0]["source"],
            "example_entity": group[0]["entity"],
        }
    )

with (DATA / "parameter-bundles.json").open("w", encoding="utf-8") as handle:
    json.dump(bundle_rows, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

with (DATA / "parameter-bundles.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    fields = [
        "bundle_id", "occurrence_count", "source_count", "entity_count",
        "keys", "params_json", "example_source", "example_entity",
    ]
    writer = csv.DictWriter(handle, fieldnames=fields)
    writer.writeheader()
    for row in bundle_rows:
        writer.writerow(
            {
                **{field: row[field] for field in fields if field in row},
                "keys": ",".join(row["keys"]),
                "params_json": canonical(row["params"]),
            }
        )

key_counts = Counter(row["key"] for row in occurrences)
unique_counts = Counter(row["key"] for row in set_rows)
source_counts = {key: len({row["source"] for row in occurrences if row["key"] == key}) for key in key_counts}
entity_counts = {key: len({row["entity"] for row in occurrences if row["key"] == key}) for key in key_counts}
key_combinations = Counter(canonical([item["key"] for item in bundle["params"]]) for bundle in bundles)
key_lengths = defaultdict(Counter)
key_position_values = defaultdict(lambda: defaultdict(Counter))
for row in occurrences:
    if isinstance(row["value"], list):
        key_lengths[row["key"]][len(row["value"])] += 1
        for position, value in enumerate(row["value"]):
            key_position_values[row["key"]][position][canonical(value)] += 1

summary = {
    "aiv_files_found": len(sources),
    "parameter_occurrences": len(occurrences),
    "unique_key_value_pairs": len(set_rows),
    "parameter_bundles": len(bundles),
    "unique_parameter_bundles": len(bundle_rows),
    "keys": [
        {
            "key": key,
            "occurrence_count": key_counts[key],
            "unique_value_count": unique_counts[key],
            "source_count": source_counts[key],
            "entity_count": entity_counts[key],
            "value_lengths": dict(sorted(key_lengths[key].items())),
            "positions": {
                f"field_{position:02d}": {
                    "distinct_value_count": len(counter),
                    "values": [
                        {"value": json.loads(value), "count": count}
                        for value, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
                    ],
                }
                for position, counter in sorted(key_position_values[key].items())
            },
        }
        for key in sorted(key_counts, key=key_order)
    ],
    "key_combinations": [
        {"keys": json.loads(keys), "count": count}
        for keys, count in sorted(key_combinations.items(), key=lambda item: (-item[1], item[0]))
    ],
}
with (DATA / "parameter-summary.json").open("w", encoding="utf-8") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

print(json.dumps(summary, ensure_ascii=False, indent=2))
