# Reverse-Engineering-Plan für `aiv.zrd`

## Ziel

Die technische und anschließend spielerische Bedeutung aller Felder der AIV-Tags
`params`, `p2` bis `p13` sowie der 17 Felder eines `vlist`-Eintrags bestimmen.

Die Ergebnisse sollen reproduzierbar dokumentiert werden als:

- Tag-/Index-zu-Strukturoffset-Tabelle
- Datentypen und Bitmasken
- statische und dynamische Belege
- semantische Feldnamen mit Konfidenzstufe
- Ghidra-Struktur, Funktionsnamen und Kommentare, sobald ausreichend abgesichert

## Bisheriger Datenbestand

- 55 `reader*.zbd`-Archive extrahiert
- 48 Dateien `aiv.zrd.json`
- 2.573 `vlist`-Einträge, davon 913 exakt unterschiedliche
- jeder `vlist`-Eintrag besitzt 17 Felder
- 2.768 Parameter-Vorkommen
- 736 unterschiedliche Parameter-Key-/Wert-Kombinationen
- vorkommende Tags: `params`, `p2`, `p6`, `p8`, `p9`, `p10`, `p12`, `p13`
- feste Längen in den Spieldaten:
  - `params`: 9
  - `p2`: 8
  - `p6`: 4
  - `p8`: 17
  - `p9`: 5
  - `p10`: 2
  - `p12`: 3
  - `p13`: 1

Die fehlenden Tags (`p3`, `p4`, `p5`, `p7`, `p11`) werden trotzdem aus dem
Programmcode rekonstruiert, da der Parser sie unterstützt.

## Bereits gefundener Einstieg

- Programm: `Mech3.exe`
- wahrscheinlicher AIV-Record-Parser: `FUN_004df730`
- Referenz auf String `vlist` (`005e9668`) bei `004e00d7`, innerhalb dieses Parsers
- eine Referenz auf `params` (`005e9718`) bei `004df7e3`, ebenfalls im Parser
- eingebetteter Quellpfad: `C:\zipper\Mech3\ZRdrAiv.cpp`
- wiederholt aufgerufener Zugriff auf die nächste Parametergruppe: `FUN_005569e0`
- geschätzte Größe der erzeugten AI-Struktur: mindestens `0xe4` Bytes
- mehrere Bool-Werte werden als Bits um `struct+0xcc` und `struct+0xcd` gespeichert

Hilfreiche Entwicklertexte in der EXE:

- `record "%s": tag p8 is incompatible with tags p3, p4, p5, and p7`
- `%s: p3 scan parameters are ignored in favor of p7 parameters`
- `%s: p3: setting both scan and jump is not allowed, ignoring jump`
- `Player controller has no ai parameters so cannot assume duties of %s`

Vorläufige Schlüsse:

- `p3` enthält Scan-/Jump-Verhalten.
- `p7` enthält höher priorisierte Scan-Parameter.
- `p8` ist ein alternatives umfassendes AI-Profil und ist mit mehreren älteren
  Teilgruppen inkompatibel.
- `params` enthält allgemeine AI-/Vehicle-Grundwerte.

## Arbeitsablauf

### Phase 1: Parser mechanisch rekonstruieren

1. `FUN_004df730` vollständig dekompilieren und abschnittsweise sichern.
2. Jeden Aufruf von `FUN_005569e0` der sequenziellen Tag-Reihenfolge zuordnen.
3. Für jedes Eingabefeld dokumentieren:
   - Tag
   - Feldindex
   - gelesener ZRD-Typ
   - Zieloffset in der AI-Struktur
   - gegebenenfalls Bitposition/Bitmaske
   - Konvertierung oder Sonderbehandlung
4. Kontrollfluss für optionale und inkompatible Tags dokumentieren.
5. `vlist`-Schleife und Struktur eines einzelnen Listeneintrags rekonstruieren.

Geplantes Tabellenformat:

| Tag | Index | Eingabetyp | Zieloffset/Bit | Semantik | Konfidenz | Beleg |
|---|---:|---|---|---|---|---|
| `params` | 0 | float | `+0x1c` | unbekannt | Offset sicher | Parser |

### Phase 2: Verwendung der Zielfelder verfolgen

1. Aufrufer und Leser der rekonstruierten AI-Struktur suchen.
2. Rechenoperationen und Vergleiche auswerten:
   - Distanzen und Quadrate deuten auf Reichweiten hin.
   - Zeitdifferenzen deuten auf Intervalle/Verzögerungen hin.
   - Winkeloperationen deuten auf Sichtfeld/Drehverhalten hin.
   - Geschwindigkeits- oder Beschleunigungslimits deuten auf Bewegung hin.
3. Vorhandene Fehler-, Debug- und Assert-Strings als semantische Belege verwenden.
4. Feldnamen zunächst mit `low`, `medium` oder `high confidence` versehen.

### Phase 3: Dynamische Bestätigung mit x32dbg

1. Breakpoint auf `004df730` setzen.
2. Erwartete Argumente am Eintritt (`__thiscall`):
   - `ECX`: Zielstruktur
   - `[ESP+4]`: geparster ZRD-Record
   - `[ESP+8]`: Name des Records/Fahrzeugs
3. Auf einen ausgewählten Record konditionieren.
4. Zielstruktur vor und nach dem Parser sichern.
5. Je Versuch genau ein Zielfeld im Speicher verändern.
6. Verhalten im Spiel beobachten und protokollieren.
7. Sichere Testwerte verwenden und Änderungen nach jedem Versuch zurücksetzen.

Speicheränderungen werden bevorzugt gegenüber sofortigem Neuverpacken der Archive,
weil sich so ein einzelnes Feld isolieren lässt.

### Phase 4: Dokumentieren und annotieren

1. Ergebnisdateien im Ordner `aiv-research/` fortlaufend aktualisieren;
   generierte Artefakte liegen unter `data/`.
2. Erst ausreichend abgesicherte Funktionen und Felder in Ghidra umbenennen.
3. Strukturtyp für den AIV-Datensatz anlegen und auf Parser/Verbraucher anwenden.
4. Ghidra-Kommentare enthalten Beleg und Konfidenz, nicht nur Vermutungen.
5. Dynamische Experimente in einer separaten Testtabelle protokollieren.

## Priorität

1. `params`
2. `p2`, `p6`, `p8`, `p9`, `p10`, `p12`, `p13`
3. unbenutzte, aber unterstützte Tags `p3`, `p4`, `p5`, `p7`, `p11`
4. `vlist`
5. dynamische Tests für semantisch unklare Felder

## Sicherheits- und Qualitätsregeln

- Pro dynamischem Versuch nur ein Feld verändern.
- Originalarchive nicht überschreiben.
- Beobachtung, statische Ableitung und Vermutung getrennt kennzeichnen.
- Exakte Float-Werte aus dem Binärformat beibehalten.
- Keine Ghidra-Umbenennung ohne belastbaren Beleg.
- Adressen beziehen sich auf die aktuell in Ghidra geladene `Mech3.exe`.

## Fortschritt vom 14. Juli 2026

- Phase 1 ist mechanisch abgeschlossen.
- Die vollständige Zuordnung steht in `parameter-field-map.md`.
- `params`, `p2` bis `p13` sowie alle 17 `vlist`-Positionen sind auf interne
  Offsets beziehungsweise Bitpositionen abgebildet.
- Die interne `vlist`-/`p4`-Eintragsstruktur ist 32 Byte groß.
- Statisch als Distanz-/Reichweitenfelder bestätigt:
  - `params[0]`, `params[2]`, `params[3]`
  - `p3[4]` beziehungsweise `p8[5]`
  - `vlist[11]`, `vlist[13]`
- Beleg: `FUN_004d9b70`/`FUN_004d8e40` quadrieren diese Werte für die Laufzeit;
  `FUN_004d9b20`/`FUN_004d8e20` ziehen für die inverse Darstellung die Wurzel.
- Die AIV-Konfiguration wird beim Controllerbau an dessen Objektanfang kopiert;
  die Offsets bleiben deshalb in den Laufzeit-Verbrauchern unverändert.
- x32dbg ist an der nicht verschobenen `Mech3.exe`-Basis `0x400000` verbunden.
- Ein Software-Breakpoint auf `0x004df730` ist gesetzt und wartet auf den nächsten
  Ladevorgang einer AIV-Datei, beispielsweise beim Start/Neuladen einer Mission.
