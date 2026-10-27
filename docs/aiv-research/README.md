# AIV-/VList-Forschungskorpus

Dieser Ordner bündelt die AIV-/VList-Forschung für Mech Reader Studio. Kuratierte
Dokumente, Skripte und Datensätze folgen jetzt einer einheitlichen
`kebab-case`-Benennung. Die quellengetreuen Reader-Artefakte behalten bewusst
ihre Originalnamen, damit Herkunft und Vergleichbarkeit erhalten bleiben.

## Struktur

- `data/`: generierte CSV-/JSON-Datensätze aus den Reader-Auswertungen
- `scripts/`: reproduzierbare Generatoren und Hilfsskripte
- `extracted-reader-json/`: extrahierte Reader-Inhalte als JSON, mit originaler
  `zbd/...`-Ordnerstruktur
- `source-reader-zips/`: archivierte Reader-ZIPs, ebenfalls mit originaler
  `zbd/...`-Ordnerstruktur
- `legacy-dialog-options/`: tiefere Einzelanalysen zu den alten AI-Editor-Dialogen

## Ergebnis

- 55 Reader-Archive extrahiert
- 48 enthalten eine `aiv.zrd.json`
- 46 enthalten mindestens eine `vlist`
- 2.573 `vlist`-Vektoren insgesamt
- 913 exakt unterschiedliche Vektoren
- jeder gefundene Vektor besitzt 17 Felder (`field_00` bis `field_16`)

`readeria1` und `readermp1` aus `zbd/t1/` enthalten zwar eine `aiv.zrd.json`,
aber keine `vlist`.

## Datensätze

- `data/archive-inventory.csv`: jedes Reader-Archiv, sein Extraktionsziel und die
  Zahl seiner `vlist`-Einträge
- `data/vlist-entry-set.json`: deduplizierte `vlist`-Einträge mit Zähler und
  Beispiel-Fundort
- `data/vlist-entry-set.csv`: dieselben `vlist`-Einträge tabellarisch in 17
  Spalten
- `data/vlist-entry-occurrences.csv`: alle `vlist`-Fundstellen mit Set-ID,
  Entity und JSON-Pfad
- `data/vlist-field-summary.json`: Werte und Häufigkeiten pro `vlist`-Feldposition
- `data/parameter-value-set.json`: unterschiedliche Werte je Parameter-Key
- `data/parameter-value-set.csv`: dieselben Parameterwerte tabellarisch
- `data/parameter-occurrences.csv`: sämtliche Parameter-Fundstellen
- `data/parameter-bundles.json`: komplette Parameterkombinationen innerhalb eines
  Records
- `data/parameter-bundles.csv`: dieselben Bündel als Tabelle
- `data/parameter-summary.json`: Key-Inventar, Häufigkeiten und Kombinationen

## Skripte

- `scripts/generate-vlist-datasets.py`: baut alle `vlist`-bezogenen Datensätze
  aus `extracted-reader-json/`
- `scripts/generate-parameter-datasets.py`: baut alle Parameter-Datensätze aus
  `extracted-reader-json/`
- `scripts/extract-dialog-resources.py`: liest Win32-Dialogressourcen direkt aus
  der EXE

## Reverse Engineering

- `reverse-engineering-plan.md`: dauerhafter Arbeitsplan und bekannte Anker
- `parameter-field-map.md`: statisch belegte Zuordnung der Tags und
  `vlist`-Indizes zu internen Offsets und Bitfeldern
- `dialog-parameter-map.md`: Zuordnung von AIV-Feldern zu Legacy-Dialogen,
  Typen und Validierungsregeln
- `dialog-resource-catalog.md`: Inventar der internen AI-Editor-Dialoge und
  Controls
- `dynamic-analysis-log.md`: x32dbg-Treffer, Laufzeitadressen und Watchpoint-Protokoll
- `input-loss-notes.md`: Analyse des Tastaturverlusts nach Debugger-Breaks
- `vlist-runtime-semantics.md`: bestätigte Laufzeitsemantik der 17 `vlist`-Felder
- `legacy-dialog-options/legacy-dialog-166-p2-reference.md`: Referenz zum alten
  Dialog 166 (`p2`) samt ZRD-Speicherung und Laufzeitwirkung
- `legacy-dialog-options/legacy-dialog-167-p1-reference.md`: Referenz zum alten
  Dialog 167 (`p1`) samt `params`-/Shared-Control-Bindings

Die Set-Bildung verwendet die exakte JSON-Darstellung. Damit bleiben
insbesondere Ganzzahlen und Gleitkommazahlen sowie die aus dem Binärformat
stammenden exakten Float-Werte unterscheidbar.
