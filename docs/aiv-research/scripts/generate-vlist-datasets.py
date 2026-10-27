import csv
import json
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "data"
EXTRACTED = ROOT / "extracted-reader-json"
SOURCE_READER_ZIPS = ROOT / "source-reader-zips"


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


DATA.mkdir(exist_ok=True)

occurrences = []


def walk(node, source, path=(), entity=None):
    if isinstance(node, list):
        current_entity = entity
        if node and isinstance(node[0], str):
            current_entity = node[0]

        index = 0
        while index < len(node):
            if node[index] == "vlist" and index + 1 < len(node):
                vlist = node[index + 1]
                if isinstance(vlist, list):
                    for entry_index, entry in enumerate(vlist):
                        occurrences.append(
                            {
                                "source": source,
                                "entity": current_entity or "",
                                "json_path": "/".join(map(str, path + (index + 1, entry_index))),
                                "entry": entry,
                            }
                        )
                index += 2
                continue
            walk(node[index], source, path + (index,), current_entity)
            index += 1
    elif isinstance(node, dict):
        for key, value in node.items():
            walk(value, source, path + (key,), entity)


sources = sorted(EXTRACTED.rglob("aiv.zrd.json"))
for source_path in sources:
    with source_path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    walk(data, source_path.relative_to(EXTRACTED).as_posix())

by_entry = defaultdict(list)
for occurrence in occurrences:
    by_entry[canonical(occurrence["entry"])].append(occurrence)

unique_rows = []
for set_id, entry_json in enumerate(sorted(by_entry), start=1):
    group = by_entry[entry_json]
    entry = group[0]["entry"]
    unique_rows.append(
        {
            "set_id": set_id,
            "occurrence_count": len(group),
            "source_count": len({row["source"] for row in group}),
            "entity_count": len({row["entity"] for row in group}),
            "entry": entry,
            "entry_json": entry_json,
            "example_source": group[0]["source"],
            "example_entity": group[0]["entity"],
        }
    )

set_id_by_json = {row["entry_json"]: row["set_id"] for row in unique_rows}

with (DATA / "vlist-entry-set.json").open("w", encoding="utf-8") as handle:
    json.dump(
        [
            {
                "set_id": row["set_id"],
                "occurrence_count": row["occurrence_count"],
                "source_count": row["source_count"],
                "entity_count": row["entity_count"],
                "entry": row["entry"],
                "example_source": row["example_source"],
                "example_entity": row["example_entity"],
            }
            for row in unique_rows
        ],
        handle,
        ensure_ascii=False,
        indent=2,
    )
    handle.write("\n")

max_fields = max((len(row["entry"]) for row in unique_rows if isinstance(row["entry"], list)), default=0)
field_names = [f"field_{index:02d}" for index in range(max_fields)]
with (DATA / "vlist-entry-set.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    writer = csv.DictWriter(
        handle,
        fieldnames=[
            "set_id",
            "occurrence_count",
            "source_count",
            "entity_count",
            *field_names,
            "example_source",
            "example_entity",
            "entry_json",
        ],
    )
    writer.writeheader()
    for row in unique_rows:
        output = {key: row[key] for key in ("set_id", "occurrence_count", "source_count", "entity_count", "example_source", "example_entity", "entry_json")}
        if isinstance(row["entry"], list):
            output.update({f"field_{index:02d}": canonical(value) for index, value in enumerate(row["entry"])})
        writer.writerow(output)

with (DATA / "vlist-entry-occurrences.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    writer = csv.DictWriter(handle, fieldnames=["set_id", "source", "entity", "json_path", "entry_json"])
    writer.writeheader()
    for occurrence in occurrences:
        entry_json = canonical(occurrence["entry"])
        writer.writerow(
            {
                "set_id": set_id_by_json[entry_json],
                "source": occurrence["source"],
                "entity": occurrence["entity"],
                "json_path": occurrence["json_path"],
                "entry_json": entry_json,
            }
        )

values_by_position = defaultdict(Counter)
lengths = Counter()
for occurrence in occurrences:
    entry = occurrence["entry"]
    if isinstance(entry, list):
        lengths[len(entry)] += 1
        for position, value in enumerate(entry):
            values_by_position[position][canonical(value)] += 1

summary = {
    "aiv_files_found": len(sources),
    "aiv_files_with_vlist": len({row["source"] for row in occurrences}),
    "vlist_occurrences": len(occurrences),
    "unique_vlist_entries": len(unique_rows),
    "entry_lengths": dict(sorted(lengths.items())),
    "values_by_position": {
        f"field_{position:02d}": [
            {"value": json.loads(value), "count": count}
            for value, count in sorted(counter.items(), key=lambda item: (-item[1], item[0]))
        ]
        for position, counter in sorted(values_by_position.items())
    },
}
with (DATA / "vlist-field-summary.json").open("w", encoding="utf-8") as handle:
    json.dump(summary, handle, ensure_ascii=False, indent=2)
    handle.write("\n")

occurrences_by_source = Counter(row["source"] for row in occurrences)
with (DATA / "archive-inventory.csv").open("w", encoding="utf-8-sig", newline="") as handle:
    writer = csv.DictWriter(
        handle,
        fieldnames=["archive", "extracted_directory", "has_aiv", "vlist_entry_count"],
    )
    writer.writeheader()
    for zip_path in sorted(SOURCE_READER_ZIPS.rglob("reader*.zip")):
        relative_zip = zip_path.relative_to(SOURCE_READER_ZIPS)
        extracted_directory = relative_zip.with_suffix("").as_posix()
        aiv_source = f"{extracted_directory}/aiv.zrd.json"
        writer.writerow(
            {
                "archive": relative_zip.with_suffix(".zbd").as_posix(),
                "extracted_directory": extracted_directory,
                "has_aiv": (EXTRACTED / aiv_source).is_file(),
                "vlist_entry_count": occurrences_by_source[aiv_source],
            }
        )

print(json.dumps({key: summary[key] for key in ("aiv_files_found", "aiv_files_with_vlist", "vlist_occurrences", "unique_vlist_entries", "entry_lengths")}, indent=2))
