# Dokumentationskatalog

Dieser Katalog ergaenzt die Projekt-`README.md` und dient als ausfuehrliches
Nachschlageverzeichnis fuer den reorganisierten `docs/`-Baum. Kuratierte
Dateien verwenden durchgaengig sprechende `kebab-case`-Namen. Rohdaten aus den
Reader-Archiven behalten ihre Originalnamen absichtlich, damit Herkunft,
Vergleichbarkeit und Skript-Reproduzierbarkeit nicht verloren gehen.

## Schnellstart

- `editor-systems/valve-and-state-logic.md`: zentrale Referenz fuer Ventil-,
  Zustands- und Dateiuebergaenge
- `editor-systems/behavior-graph.md`: Bedien- und Implementierungsreferenz fuer
  den dateiuebergreifenden Behavior Graph
- `editor-systems/ai-networks-and-regions.md`: Struktur und Editorverhalten fuer
  `net_*.zrd` und `reg_*.zrd`
- `editor-systems/reader-document-relationships.md`: kompakte Uebersicht ueber
  Schichten, Referenzen und Evidenzregeln
- `aiv-research/README.md`: Einstieg in den AIV-/VList-Forschungskorpus

## Verzeichnisse

- `editor-systems/`: kuratierte Kerndokumente zu Netzwerken, Regionen,
  Valve-Logik und Reader-Beziehungen
- `aiv-research/`: Reverse-Engineering-Unterlagen, Datensaetze, Skripte und
  quellengetreue Reader-Artefakte

## Kerndokumente

- `editor-systems/ai-networks-and-regions.md`: erklaert Graph-Navigation,
  Tag-Layouts, Knoten-/Kantenregeln und die aktuellen Beweise fuer `net_*.zrd`
  und `reg_*.zrd`
- `editor-systems/reader-document-relationships.md`: zeigt, wie Reader-Layer,
  Regionen, VLists und Valve-Kanaele ueber Archive hinweg zusammenspielen
- `editor-systems/valve-and-state-logic.md`: detaillierte Referenz fuer
  benannte Zustandskanaele, `valves.zrd`, VLists, Netze, Targets und Objectives
- `editor-systems/behavior-graph.md`: visuelle Sprache, Blockinventar, native
  Schreibabbildung, Transaktionsregeln und Evidenzgrenzen des Graph-Editors

## AIV-Forschung

- `aiv-research/README.md`: Struktur, Kennzahlen und Inhaltsverzeichnis fuer den
  gesamten AIV-/VList-Bestand
- `aiv-research/reverse-engineering-plan.md`: Arbeitsplan, Prioritaeten und
  Qualitaetsregeln fuer weitere Parser- und Laufzeitanalyse
- `aiv-research/parameter-field-map.md`: mechanische Feldkarte fuer `params`,
  `p2` bis `p13`, `lancemate` und `vlist`
- `aiv-research/dialog-parameter-map.md`: Zuordnung von AIV-Feldern zu
  Legacy-Dialogen, Typen und Validierungsregeln
- `aiv-research/dialog-resource-catalog.md`: Inventar der internen AI-Editor-
  Dialoge samt Control-IDs und Rekonstruktionsstatus
- `aiv-research/dynamic-analysis-log.md`: x32dbg-Sitzungsnotizen und
  Laufzeitbeobachtungen
- `aiv-research/input-loss-notes.md`: Debugger-bezogene Analyse zum Verlust von
  Tastatureingaben
- `aiv-research/vlist-runtime-semantics.md`: konsolidierte Laufzeitsemantik der
  17 `vlist`-Felder
- `aiv-research/legacy-dialog-options/legacy-dialog-166-p2-reference.md`:
  Tiefenanalyse des Fahrzeugdialogs 166 (`p2`)
- `aiv-research/legacy-dialog-options/legacy-dialog-167-p1-reference.md`:
  Tiefenanalyse des Aktivierungsdialogs 167 (`p1`)

## Datensaetze und Werkzeuge

- `aiv-research/data/archive-inventory.csv`: Reader-Archiv-Inventar mit
  Extraktionsziel und `vlist`-Mengen
- `aiv-research/data/vlist-entry-set.json`: deduplizierte `vlist`-Eintraege mit
  Zaehlern und Beispielen
- `aiv-research/data/vlist-entry-set.csv`: dieselben `vlist`-Eintraege in
  Tabellenform
- `aiv-research/data/vlist-entry-occurrences.csv`: jede einzelne `vlist`-
  Fundstelle mit Set-ID, Entity und JSON-Pfad
- `aiv-research/data/vlist-field-summary.json`: Haeufigkeiten und Werte pro
  `vlist`-Feldposition
- `aiv-research/data/parameter-value-set.json`: einzigartige Werte pro
  Parameter-Key
- `aiv-research/data/parameter-value-set.csv`: tabellarische Form derselben
  Parameterwerte
- `aiv-research/data/parameter-occurrences.csv`: alle Parameter-Fundstellen mit
  Quelle, Entity und JSON-Pfad
- `aiv-research/data/parameter-bundles.json`: vollstaendige Parameterbuendel pro
  Record
- `aiv-research/data/parameter-bundles.csv`: tabellarische Form derselben
  Parameterbuendel
- `aiv-research/data/parameter-summary.json`: Schluesselinventar, Haeufigkeiten
  und Kombinationen
- `aiv-research/scripts/generate-vlist-datasets.py`: regeneriert alle
  `vlist`-Datensaetze aus `extracted-reader-json/`
- `aiv-research/scripts/generate-parameter-datasets.py`: regeneriert alle
  Parameter-Datensaetze aus `extracted-reader-json/`
- `aiv-research/scripts/extract-dialog-resources.py`: extrahiert Dialogressourcen
  direkt aus der EXE

## Rohdaten

- `aiv-research/extracted-reader-json/`: extrahierte Reader-Inhalte als JSON;
  Pfade folgen dem Muster `zbd/<kampagne>/<reader>/...`
- `aiv-research/source-reader-zips/`: archivierte Reader-ZIPs mit derselben
  Reader-Struktur wie im Spielbestand

Bei den Rohdaten gelten die Originalnamen als Teil des Belegs. Wenn ein Agent
einen konkreten Missions- oder Basisbestand braucht, sollte er zuerst in
`extracted-reader-json/zbd/...` nachsehen und nur danach auf die erzeugten
Aggregatdateien unter `data/` zurueckgreifen.
