# Legacy-Dialog 166 — `p2` (Fahrzeugdaten)

Dies ist die eigenständige Referenz für Resource 166. Die frühere Sammel-README
wurde in diese Datei umbenannt; die Aussagen beziehen sich hier ausschließlich
auf den alten Fahrzeugdialog `p2`.

Diese Referenz erklärt Optionen aus den eingebetteten, im Spiel nicht mehr direkt
erreichbaren MFC-Dialogen. Sie trennt dabei vier verschiedene Dinge, die nicht
miteinander verwechselt werden dürfen:

1. die ursprüngliche Dialogbeschriftung und Control-ID,
2. die Speicherung in `aiv.zrd`,
3. die interne Repräsentation nach dem Parsen und
4. die tatsächlich nachweisbare Wirkung in der ausgelieferten `Mech3.exe`.

Eine Dialogbindung beweist nur, dass der alte Editor ein Feld anzeigen und
speichern konnte. Sie beweist nicht automatisch, dass das Spiel dieses Feld zur
Laufzeit noch auswertet.

## Resource 166 — ursprünglicher Dialog `p2`

Der DDX-Handler bei `0x004B8EB0` bindet die folgenden Controls. Dies ist das
vollständige Control-Inventar der Resource, einschließlich der Felder, die nur
Editor- oder Laufzeitzustand und keine serialisierten AIV-Werte darstellen.

| Control / UI-Feld | DDX-gebundener Wert | AIV-Feld oder Tag | Status / Hinweise |
|---|---|---|---|
| `Configuration`-Combo (1273) | Editor-Comboobjekt `+0x214` | kein belegter AIV-Index | Laufzeit-/Konfigurationsauswahl; verdrahtet, aber kein Speicherpfad im AIV-Parser belegt |
| `Default Chassis`-Combo (1274) | Editor-Comboobjekt `+0x254`; Auswahl im Dialog bei `+0x128` | `p12[0]` | direkt über Auswahlhandler `0x004BC8F0` |
| `Skin`-Edit (1284) | Editor-Integer `+0x12C` | `skin[0]` | numerisches DDX-Feld; aktuelle AIV-Daten bestätigen numerische Werte |
| `Pilot Name`-Edit (1243) | Dialog-`CString` bei `+0x60` | `pilot[0]` | direkte DDX-Stringbindung; der Tag-Writer ist ein separater Pfad |
| `Objective Conditional` (1275) | Dialogflag `+0x64` → Editorflag Bit 4 | `p12[1]` | direkte Bindung; kein Laufzeitverbraucher nachgewiesen |
| `can configure` (1279) | Editorflag Bit 7 bei `+0xCC` | `pilot.can_configure[0]` | direkte Tag-zu-Flag-Abbildung; kein `p12`-Index |
| `Shutdown on start` (1276) | Dialogflag `+0x68` → Editorflag Bit 5 | `p12[2]` | direkte Bindung und aktiver Laufzeitverbraucher |
| `Home Net`-Combo (1238) | Editor-Netzlisten-Combo `+0x1D4` | Netzreferenz / Beziehung zu `record[0]` | Auswahl bestimmt die schreibgeschützte Net ID; exakter serialisierter Kopierpfad noch nicht isoliert |
| `Net Edit` (1250) | Command-Button | kein AIV-Feld | öffnet den Netzwerkeditor für das gewählte Home Net |
| `Net ID`-Edit, read-only (1117) | Editor-Integer `+0x68` | Basisfeld `record[0]` | semantische Übereinstimmung mit dem ersten Parserfeld; wird aus der Home-Net-Auswahl befüllt |
| `Show` (1249) | Editorflag/-zustand `+0x1CC` | kein belegtes AIV-Feld | dreistufiger Anzeige-/Editorbefehl |
| `Show InOut` (1282) | Editorbyte `+0x1D1` | kein belegtes AIV-Feld | Laufzeitoption der Netzanzeige |
| `Initial Vehicle Position` X (1200) | Editor-Float `+0x6C` | `record[1][0]` | direktes Basispositionsfeld |
| `Initial Vehicle Position` Y (1201) | Editor-Float `+0x70` | `record[1][1]` | direktes Basispositionsfeld |
| `Initial Vehicle Position` Z (1202) | Editor-Float `+0x74` | `record[1][2]` | direktes Basispositionsfeld |
| Initial Vehicle Position `yaw` (1203) | Editor-Float `+0x78` | `record[2]` / Parser-Basisoffset `+0x14` | direktes Basisorientierungsfeld |
| Team `Friend` / `Foe` / `Other` (1197–1199) | DDX-Radiowert `+0x4DC` | kein eigenes AIV-Feld; abgeleitete Ansicht von `record[3]` | `0x004BC260` vergleicht die Teammaske mit den Friend-/Foe-Masken des kontrollierten Mechs; Radioänderungen wählen über `0x004BC300` eine passende Team-ID |
| Teamnummer (1111) | vorzeichenloser Editorwert `+0x7C` | `record[3]` | direkt persistent; der Integer packt eine oder mehrere Team-IDs in 5-Bit-Segmente und ist kein Allegiance-Enum |
| `Send home` (1255) | Command-Button | kein AIV-Feld | Aktion, kein persistenter Fahrzeugparameter |
| `Lancemate` (1283) | Editorbyte `+0x131` = eingebettetes AIV-Byte `+0xCD` | optionales Tag `lancemate[0]` | direkte Dreizustandsbindung: Off, On oder Auto; Auto leitet den Status aus der Freundbeziehung zum kontrollierten Spielerfahrzeug ab |

Der Resourcetitel `p2` bezeichnet den allgemeinen Fahrzeugdialog. Er darf nicht
mit dem serialisierten AIV-Tag `p2` verwechselt werden, der Angriffs- und
Rückzugsparameter enthält.

### `Objective Conditional` (Control 1275)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | `p12[1]` |
| ZRD-Typ | numerisches Boolean (`0` oder `1`) |
| Interne Speicherung | AIV-Struktur `+0xCC`, Bit 4 (`0x10`) |
| Parser | `FUN_004DF730`, Schreibpfad um `0x004DFF00` |
| Laufzeitstatus | kein nachweisbarer Verbraucher; Legacy/ungenutzt |

Der AIV-Parser übernimmt `p12[1]` korrekt in Bit 4 des Flagbytes. Auch der
ursprüngliche Dialog bindet seine Checkbox korrekt an dieses Bit. Eine Prüfung
des Bits durch den Spielcode wurde dagegen nicht gefunden. Die nachweisbaren
Verbraucher desselben Flagbytes prüfen andere Masken wie `0x20`, `0x40` und
`0x80`, aber nicht `0x10`.

Auch die Daten stützen diesen Befund: Alle 309 in den extrahierten Readern
gefundenen `p12`-Arrays enthalten an Index 1 den Wert `0`.

Damit ist lediglich belegt, dass das Feld einmal vorgesehen und vollständig
serialisierbar war. Eine Verbindung zu `objectives.zrd`, zur Aktivierung eines
Objectives oder zu dessen Erfolg/Misserfolg ist **nicht** belegt. Aus der
Beschriftung allein darf keine solche Semantik abgeleitet werden.

Empfohlene Darstellung im neuen Editor:

```text
Objective Conditional (legacy/unused) (p12[1])
```

Der Editor soll nur `0` und `1` zulassen, das Feld beim Roundtrip jedoch erhalten.

### `can configure` (Control 1279)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | Untertag `pilot.can_configure[0]` |
| ZRD-Typ | String-Boolean (`"true"` oder `"false"`) |
| Interne Speicherung | AIV-Struktur `+0xCC`, Bit 7 (`0x80`) |
| Parser | `FUN_004DF730`, `pilot`-Block vor `0x004DFC4D` |
| Laufzeitverbraucher | `FUN_0049E810`, Prüfung um `0x0049EB25` |
| Laufzeitstatus | aktiv verwendet |

Das Feld ist kein Bestandteil des `p12`-Arrays. Es liegt verschachtelt im
`pilot`-Wert, beispielsweise:

```json
"pilot",
[
  "MSG_AIV_LANCEMATE",
  "can_configure",
  [
    "true"
  ]
]
```

Beim Aufbau einer Mission durchläuft `FUN_0049E810` die AIV-Records. Ein Record
wird dem Konfigurationssystem übergeben, wenn unter anderem folgende Bedingungen
erfüllt sind:

- der passende Singleplayer-Modus ist aktiv,
- der Record besitzt ein gültiges Default Chassis,
- er gehört zur selben Seite beziehungsweise zum selben Team wie der gewählte
  Spieler-Record und
- `can_configure` ist gesetzt.

Der Record wird dann nicht unmittelbar über den normalen Fahrzeug-Erzeugungspfad
`FUN_0049FCE0` instanziiert. Stattdessen erzeugt das Spiel einen
Konfigurationseintrag (`FUN_00433CC0`), fügt ihn in die Liste konfigurierbarer
Fahrzeuge ein (`FUN_004351C0`) und setzt deren Anzahl (`FUN_00435230`). Dadurch
kann die Spieler-/Missionskonfiguration die tatsächliche Fahrzeugkonfiguration
bestimmen. Ohne das Flag wird der Record direkt mit seinen AIV-Vorgaben erzeugt.

Das Feld verändert kein KI-Verhalten während des Gefechts und erlaubt der KI
auch keine selbstständige Umrüstung.

Empfohlene Darstellung im neuen Editor:

```text
Allow Player Configuration (pilot.can_configure)
```

Die Checkbox muss beim Speichern wieder die Strings `"true"` beziehungsweise
`"false"` erzeugen; numerische Werte wären nicht formatgetreu.

#### Tatsächlich gesetzte Vorkommen

In den extrahierten Readern ist `can_configure` bei **81 Records in 32
verschiedenen `aiv.zrd`-Dateien** auf `"true"` gesetzt. Die folgende Liste zählt
nur echte `"true"`-Werte, nicht bloß das Vorhandensein des Tags.

| Reader/AIV-Datei | Records mit `can_configure = "true"` |
|---|---|
| `c1/readeria1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattilla` |
| `c1/readeria2/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c1/readeria3/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c1/readerm3/aiv.zrd` | `bushwacker_01` |
| `c2/readeria1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c2/readeria2/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c2/readeria3/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c2/readerm1/aiv.zrd` | `Dominic_Paine` |
| `c2/readerm2/aiv.zrd` | `Dominic_Paine` |
| `c2/readerm3/aiv.zrd` | `Dominic_Paine` |
| `c2/readerm4/aiv.zrd` | `Dominic_Paine` |
| `c3/readeria1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readeria2/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readeria3/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readerm1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi` |
| `c3/readerm2/aiv.zrd` | `mobilehq_01`, `mobilehq_02`, `mobilehq_03`, `Dominic_Paine`, `Epona_Rhi` |
| `c3/readerm3/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readerm4/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readerm5/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c3/readerm6/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Matilla` |
| `c4/readeria1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c4/readeria2/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c4/readeria3/aiv.zrd` | `Lancemate_01`, `Lancemate_02`, `Lancemate_03` |
| `c4/readerm1/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila`, `cauldron_01` |
| `c4/readerm2/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c4/readerm3/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c4b/readerm4/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `c4b/readerm6/aiv.zrd` | `Dominic_Paine`, `Epona_Rhi`, `Alan_Mattila` |
| `t1/readerm1/aiv.zrd` | `bushwacker_01` |
| `t1/readerm2/aiv.zrd` | `bushwacker_01` |
| `t1/readerm3/aiv.zrd` | `bushwacker_01` |
| `t1/readerm4/aiv.zrd` | `bushwacker_01`, `bushwacker_02` |

Die Pfade sind relativ zu `aiv-research/extracted-reader-json/zbd/`; dort tragen die
extrahierten Dateien zusätzlich die Endung `.json`.

### `Shutdown on start` (Control 1276)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | `p12[2]` |
| ZRD-Typ | numerisches Boolean (`0` oder `1`) |
| Interne Speicherung | AIV-Struktur `+0xCC`, Bit 5 (`0x20`) |
| Parser | `FUN_004DF730`, Schreibpfad um `0x004DFF1C` |
| Fahrzeug-Erzeugung | `FUN_0049FA20`, Prüfung um `0x0049FC7E` |
| Shutdown-Zustand | `FUN_0041CC30(mech, 1)` |
| Laufzeitstatus | aktiv verwendet |

Das Fahrzeug und sein Controller werden zunächst normal erzeugt. Erst danach
prüft der Erzeugungspfad Bit 5 und versetzt den erzeugten Mech bei gesetztem Flag
in den Shutdown-Zustand. Das Flag verzögert daher nicht den Spawn und macht das
Fahrzeug nicht unsichtbar. Es startet als vorhandener, aber heruntergefahrener
Mech und bleibt so, bis ein anderer Spielpfad es wieder hochfährt.

Auch für das Spielerfahrzeug existiert ein eigener Verbraucher. `FUN_00491150`
übernimmt das Bit beim Missionsstart in den globalen Startzustand. Der normale
automatische Power-up-Pfad in `FUN_00481170` wird dann übersprungen, sodass auch
ein Spielermech tatsächlich heruntergefahren beginnen kann.

In den untersuchten Readern ist das Feld bei 118 von 309 `p12`-Vorkommen gesetzt.
Viele vorpositionierte Gegner verwenden den Wert `1`, während Spieler und direkt
aktive Lancemates meistens `0` verwenden.

Empfohlene Darstellung im neuen Editor:

```text
Start with Mech Shut Down (p12[2])
```

Der Editor soll ausschließlich die numerischen Werte `0` und `1` schreiben.

### `Configuration` (Control 1273)

| Eigenschaft | Ergebnis |
|---|---|
| DDX-Ziel | Comboobjekt des Editors bei `+0x214` |
| AIV-Pfad | keiner belegt |
| Persistenz | nicht als eigenes AIV-Feld belegt |
| Status | kontextabhängige Editor-/Laufzeitauswahl |

Die Combo ist vollständig an den Legacy-Editor angebunden, aber ihre Auswahl
wird nicht durch den AIV-Parser aus einem bestimmten Arrayindex geladen. Sie
gehört zu dem übergeordneten Konfigurationssystem, an das unter anderem Records
mit gesetztem `can_configure` übergeben werden.

Insbesondere darf dieses Control nicht ohne weiteren Beleg mit `p10[1]` oder
einem anderen bislang unbekannten Zahlenfeld gleichgesetzt werden. Im neuen
Editor darf es nur angeboten werden, wenn ein entsprechender
Konfigurationskatalog geladen und dessen Auswahlpfad ausdrücklich implementiert
ist. Andernfalls bleibt es eine reine Kontextanzeige und erzeugt beim Speichern
kein zusätzliches AIV-Feld.

### `Default Chassis` (Control 1274)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | `p12[0]` |
| ZRD-Typ | String / Chassistoken |
| Dialogzustand | Comboobjekt `+0x254`, Auswahlwert `+0x128` |
| Auswahlhandler | `0x004BC8F0` |
| Parserziel | interne Chassis-ID bei AIV `+0xC4` |
| Persistenz | direkt |

`p12[0]` enthält Namen wie `sunder`, `bushwacker` oder `notmech`. Der Parser
wandelt bekannte Namen über `FUN_004845F0` in eine interne Chassis-ID um. Der
Legacy-Dialog zeigte dafür eine Combo, deren Einträge aus dem verfügbaren
Chassiskatalog und nicht aus dem einzelnen AIV-Record stammten.

Der neue Editor muss bekannte Chassis als Dropdown anbieten, unbekannte Tokens
aber verlustfrei anzeigen und erhalten. Die Comboauswahl wird wieder als String
in `p12[0]` gespeichert, nicht als interne numerische ID.

### `Skin` (Control 1284)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | `skin[0]` |
| ZRD-Typ | Integer |
| Dialogzustand | Integer bei `+0x12C` |
| belegter Bereich | `0..100` durch `DDV_MinMaxInt` |
| beobachtete Readerwerte | `0..7` |
| Persistenz | direkt |

Das Control ist ein numerisches Editfeld, kein freier String. Der MFC-Validator
erlaubt Werte von 0 bis 100; dass die vorhandenen Reader nur 0 bis 7 verwenden,
rechtfertigt keine engere Dateiformatgrenze. Der neue Editor soll deshalb nur
ganze Zahlen im belegten MFC-Bereich akzeptieren.

### `Pilot Name` (Control 1243)

| Eigenschaft | Ergebnis |
|---|---|
| AIV-Pfad | `pilot[0]` |
| ZRD-Typ | String |
| Dialogzustand | `CString` bei `+0x60` |
| Persistenz | direkt, innerhalb des zusammengesetzten `pilot`-Werts |

Das Feld zeigt den ersten Wert des `pilot`-Arrays, häufig einen Message-Key wie
`MSG_AIV_LANCEMATE`, teilweise aber auch einen direkten Namen. Beim Bearbeiten
darf der übrige Inhalt des `pilot`-Arrays nicht neu aufgebaut oder verworfen
werden. Insbesondere muss ein vorhandenes `can_configure`-Untertag erhalten
bleiben.

### `Home Net` und `Net ID` (Controls 1238 und 1117)

| Eigenschaft | `Home Net` | `Net ID` |
|---|---|---|
| Controltyp | Combo | schreibgeschütztes Editfeld |
| Editorzustand | Netzlistenobjekt `+0x1D4` | Integer `+0x68` |
| AIV-Beziehung | Auswahl einer geladenen Netzdefinition | Basisfeld `record[0]` |
| Persistenz | indirekte Referenzauswahl | numerischer Basiswert |

Der Legacy-Editor konnte seine Combo aus den bereits geladenen `net_*.zrd`
beziehungsweise sonstigen Netzdefinitionen befüllen. Die lesbare Netzauswahl
und die numerische Net ID sind daher zwei Ansichten derselben Beziehung, nicht
zwei unabhängige AIV-Parameter.

Die semantische Zuordnung von `record[0]` zur angezeigten Net ID ist belegt. Der
vollständige Legacy-Kopierpfad von der Comboauswahl zurück in den Basisrecord ist
noch nicht isoliert. Der neue Editor soll deshalb vorhandene IDs verlustfrei
erhalten, bekannte Netze über den geladenen Readerkontext auflösen und nur bei
einer bewussten Auswahl die entsprechende ID ändern. Eine unbekannte ID darf
nicht automatisch auf das erste verfügbare Netz oder auf 0 gesetzt werden.

### `Net Edit` (Control 1250)

`Net Edit` ist ein Command-Button und kein Datenfeld. Er öffnet den separaten
Netzwerkeditor für das in `Home Net` ausgewählte Netz. Das Control benötigt den
Reader-/Missionskontext und darf beim Speichern der AIV-Datei keinen Tag oder
Arrayindex erzeugen.

Für den neuen Editor entspricht dies einer Navigation zum betreffenden Net,
nicht einem Boolean und nicht einer Eigenschaft des Fahrzeugs.

### `Show` und `Show InOut` (Controls 1249 und 1282)

| Control | Editorzustand | Wirkung | Persistenz |
|---|---:|---|---|
| `Show` | `+0x1CC`, dreistufig | steuert die Anzeige des ausgewählten Netzes/Objekts im Legacy-Editor | kein AIV-Feld belegt |
| `Show InOut` | `+0x1D1`, Byte | ergänzt die Anzeige um Ein-/Ausgangsinformationen | kein AIV-Feld belegt |

Beide Controls verändern die Arbeitsansicht des Editors. Ihre Zustände dürfen
nicht in freie AIV-Felder hineingedeutet werden. Ein moderner Editor kann
äquivalente Ansichtsoptionen besitzen, soll sie aber höchstens in seinen eigenen
UI-Einstellungen und niemals in `aiv.zrd` speichern.

### `Initial Vehicle Position` (Controls 1200–1203)

| Control | AIV-Pfad | ZRD-Typ | Interner Wert |
|---|---|---|---:|
| X (1200) | `record[1][0]` | Number | `+0x6C` im Editormodell |
| Y (1201) | `record[1][1]` | Number | `+0x70` im Editormodell |
| Z (1202) | `record[1][2]` | Number | `+0x74` im Editormodell |
| Yaw (1203) | `record[2]` | Number | `+0x78` im Editormodell; Parserbasis `+0x14` |

Dies sind direkte, persistente Basisfelder des Fahrzeugrecords. Der neue Editor
soll endliche Zahlen verlangen und die ursprüngliche Floatgenauigkeit beim
Roundtrip nicht unnötig reduzieren. Positionsänderungen benötigen keine neuen
Tags; sie ändern die bereits vor den Tag-/Wert-Paaren stehenden Recordwerte.

### Team-IDs und abgeleitete Zugehörigkeit (Controls 1197–1199 und 1111)

| Eigenschaft | Ergebnis |
|---|---|
| Radiogruppe | `Friend`, `Foe`, `Other`, DDX-Wert bei `+0x4DC` |
| Persistenter Wert | `record[3]`, vorzeichenloser Editorwert bei `+0x7C` |
| Kodierung | nicht leere Liste von Team-IDs `0..31`, jeweils in einem 5-Bit-Segment |
| Radio → Teamwert | Handler `0x004B9290` ruft `FUN_004BC300` auf |
| Teamwert → Radio | `FUN_004BC260` |
| Laufzeitinitialisierung | `FUN_0049FA20` und `FUN_00491150` |

#### Was `record[3]` tatsächlich speichert

Der Integer ist eine gepackte Liste von Team-IDs, kein `Friend/Foe/Other`-Enum:

```text
team_id[0] = value        & 0x1f
team_id[1] = value >> 5   & 0x1f
team_id[2] = value >> 10  & 0x1f
...
```

Das erste Segment existiert immer. `record[3] = 0` bedeutet deshalb Mitglied
von Team 0 und nicht „kein Team“. Beispiele:

| Team-IDs | `record[3]` |
|---|---:|
| `[0]` | `0` |
| `[1]` | `1` |
| `[2]` | `2` |
| `[1, 2]` | `65` |
| `[1, 3]` | `97` |

Beim Fahrzeug-Spawn wird daraus eine Bitmaske aufgebaut:

```text
membership_mask = OR(1 << team_id)
friend_mask     = membership_mask & ~1
foe_mask        = ~friend_mask & ~1
```

Team 0 ist besonders: Bit 0 wird aus jeder erzeugten Friend- und Foe-Maske
entfernt. Andere Einheiten erkennen ein reines Team-0-Ziel deshalb weder als
Friend noch als Foe. Das macht die Beziehung jedoch nicht gegenseitig neutral:
Für eine reine Team-0-Einheit ist `friend_mask = 0` und
`foe_mask = 0xFFFFFFFE`; sie selbst betrachtet damit alle Nichtnull-Teams als
Foe. `unclassified` ist deshalb die genauere Bezeichnung als `neutral`.

Bei genau einer Nichtnull-Team-ID ist das Verhalten übersichtlich: dieselbe ID
schneidet die Friend-Maske, alle anderen Nichtnull-IDs schneiden die Foe-Maske.
Mehrere Team-IDs erzeugen dagegen keine globale Allianz und keine symmetrische
Beziehung.

`FUN_004D71A0` kann eine Mitgliedschaftsmaske wieder in das Dateiformat
zurückwandeln. Dabei werden die gesetzten Team-IDs eindeutig und aufsteigend in
5-Bit-Segmente gepackt.

#### Was die drei Radiobuttons tun

Die Radiowahl ist eine Komfortansicht relativ zum **aktuell kontrollierten
Mech**, nicht eine eigene Eigenschaft des AIV-Records.

Beim Anzeigen oder nach einer direkten Änderung der Teamnummer führt
`FUN_004BC260` folgende Prüfung aus:

1. `record[3]` wird in eine Mitgliedschaftsmaske dekodiert.
2. Überschneidet sie sich mit der Friend-Maske des kontrollierten Mechs, wird
   `Friend` angezeigt.
3. Andernfalls wird bei einer Überschneidung mit seiner Foe-Maske `Foe`
   angezeigt.
4. Ohne Überschneidung mit beiden Masken wird `Other` angezeigt.

Bei gemischter Mehrfachmitgliedschaft besitzt `Friend` **nur in dieser
Legacy-Dialogfunktion** Vorrang vor `Foe`. Das ist eine Anzeigeregel und keine
allgemeine Kampfregel.

Die drei Controls 1197–1199 teilen sich den Handler bei `0x004B9290`. Nach einer
Änderung ruft er `FUN_004BC300` auf. Diese Funktion ersetzt die Team-ID-Liste
durch genau eine Team-ID:

- `Friend`: niedrigste ID aus der Friend-Maske des kontrollierten Mechs,
- `Foe`: niedrigste ID aus seiner Foe-Maske,
- `Other`: niedrigste ID, die in keiner der beiden Masken enthalten ist.

Gibt es keine passende ID zwischen 0 und 31, setzt der Legacy-Code den
Sentinelwert `0xFFFFFFFF`. Eine Radiowahl ist für einen Record mit mehreren
Team-IDs daher verlustbehaftet.

#### Mehrfach-Teams im Laufzeitcode

Die drei Masken gehören jeweils dem betrachtenden Fahrzeug. Laufzeitprüfungen
haben grundsätzlich die Richtung

```text
actor.friend_mask & target.membership_mask
actor.foe_mask    & target.membership_mask
```

Bei einem Ziel mit mehreren Team-IDs können **beide** Ergebnisse gleichzeitig
ungleich null sein. Der Code löst diesen Widerspruch nicht an einer zentralen
Stelle auf:

- `FUN_004BC260` (Legacy-Dialog) prüft Friend zuerst und zeigt deshalb
  `Friend`.
- `FUN_00440B00` besitzt ebenfalls einen Friend-zuerst-Zweig für seine lokale
  Relationseinstufung.
- Kampfnahe Auswahlpfade wie `FUN_0045C4A0` prüfen dagegen direkt
  `actor.foe_mask & target.membership_mask != 0`, ohne zuvor einen
  Friend-Treffer auszuschließen.
- Auch `FUN_00419200` und `FUN_004A9530` enthalten solche direkten Foe-Tests.

Eine Mehrfachmitgliedschaft kann daher abhängig vom Subsystem zugleich als
Friend und als angreifbares Foe-Ziel gelten. Sie darf nicht als Möglichkeit zum
Erzeugen einer sauberen neutralen oder verbündeten Zwischenpartei verstanden
werden.

Beispiel mit Player auf Team 1, Owens auf Team 2 und `ttruck_01` auf
`[1, 2]` (`record[3] = 65`):

| Fahrzeug | Membership | Friend | Foe |
|---|---:|---:|---:|
| Player, Team 1 | `0x00000002` | `0x00000002` | `0xFFFFFFFC` |
| Owens, Team 2 | `0x00000004` | `0x00000004` | `0xFFFFFFFA` |
| Truck, Teams 1+2 | `0x00000006` | `0x00000006` | `0xFFFFFFF8` |

| Richtung | Friend-Schnitt | Foe-Schnitt | praktische Folge |
|---|---:|---:|---|
| Truck → Player | Team-1-Bit | keiner | Truck behandelt Player nicht als Foe |
| Truck → Owens | Team-2-Bit | keiner | Truck behandelt Owens nicht als Foe |
| Player → Truck | Team-1-Bit | Team-2-Bit | Truck ist zugleich Friend und Foe; direkte Foe-Filter lassen ihn als Ziel zu |
| Owens → Truck | Team-2-Bit | Team-1-Bit | Truck ist zugleich Friend und Foe; direkte Foe-Filter lassen ihn als Ziel zu |

Player und Owens bleiben untereinander normale Gegner. Der Truck greift keinen
von beiden aufgrund seiner Foe-Maske an, kann aber von beiden als feindfähiges
Ziel ausgewählt werden.

#### Belegte Readerkonvention

Alle 768 untersuchten Records verwenden nur Einzelteamwerte:

| Wert | Anzahl | beobachtete Verwendung |
|---:|---:|---|
| `0` | 32 | ausschließlich Records namens `stealth` |
| `1` | 398 | Spieler, Lancemates und andere Verbündete |
| `2` | 338 | Gegner |

Das erklärt die praktische Wirkung in den Originalmissionen, macht die Zahlen
aber nicht zu einem allgemeinen Allegiance-Enum. In einer Mission mit anderen
Masken kann dieselbe Team-ID relativ zum kontrollierten Mech anders angezeigt
werden.

#### Anforderungen an den neuen Editor

- `record[3]` als `UInt32` behandeln und beim Bearbeiten in eindeutige Team-IDs
  von 0 bis 31 zerlegen.
- Team 0 ausdrücklich als echte, unklassifizierte Team-ID anzeigen; nicht als
  fehlenden Wert interpretieren.
- Eine Mehrfachauswahl für Team-IDs anbieten oder zumindest den gepackten
  Rohwert verständlich dekodieren.
- `Friend/Foe/Other` nur dann anzeigen, wenn ein Referenzmech und dessen
  Beziehungsmasken bekannt sind; die Anzeige als abgeleitete
  **Legacy-Dialogvorschau**, nicht als verbindliche Laufzeitrelation,
  kennzeichnen.
- Bei mehreren Team-IDs darauf hinweisen, dass Friend- und Foe-Schnitt zugleich
  gesetzt sein können und kampfnahe Pfade direkte Foe-Tests ausführen.
- Eine Radiowahl nicht stillschweigend auf einen Mehrfachteamwert anwenden, weil
  der Originalhandler ihn auf eine einzige Team-ID reduziert.
- Nichtkanonische oder unbekannte Rohwerte beim unveränderten Roundtrip erhalten;
  erst nach einer bewussten Teamänderung eindeutig und aufsteigend neu packen.

#### Umsetzung im Mech Reader Studio

Der Fahrzeug-Tab setzt diese Trennung inzwischen ausdrücklich um:

- `record[3]` erscheint als Liste der tatsächlich gespeicherten Team-IDs. Der
  gepackte `UInt32`-Rohwert bleibt daneben sichtbar.
- Team 0 trägt die Kennzeichnung `unclassified`; es wird weder als fehlender
  Wert noch als festes `Other`-Enum behandelt.
- Weitere eindeutige Team-IDs von 0 bis 31 können über eine Auswahlliste
  ergänzt und einzeln entfernt werden. Eine kanonische `UInt32`-Kodierung kann
  höchstens sechs unterschiedliche, aufsteigend sortierte Team-IDs aufnehmen.
- Nichtkanonische Segmentfolgen bleiben ohne Bearbeitung bytewerttreu erhalten
  und werden sichtbar gewarnt. Erst eine bewusste Teamänderung kanonisiert sie.
- Darunter zeigt eine getrennte **Legacy Friend / Foe / Other preview** die
  relative Einordnung zum geladenen `player`-Record und begründet das Ergebnis.
  Diese Vorschau bildet ausdrücklich `FUN_004BC260` ab; sie ist bei
  Mehrfach-Teams keine Garantie für das Verhalten aller Kampfsysteme.
- Jeder nachgebildete Legacy-Button nennt vorab die Team-ID, durch die er die
  gesamte Liste ersetzen würde. Bei einer verlustbehafteten Ersetzung verlangt
  der Editor eine Bestätigung. Eine nicht mögliche `Friend`-Zuordnung wird
  deaktiviert, statt den Legacy-Sentinel `0xFFFFFFFF` zu erzeugen.

Die Implementierung liegt in `src/components/TeamEditor.svelte` und die
verlustfrei getestete Kodierungslogik in `src/team.ts` des Editor-Projekts.

### `Send home` (Control 1255)

`Send home` ist eine Aktion auf dem im Legacy-Editor ausgewählten, laufenden
Fahrzeug. Es ist kein persistenter Parameter und besitzt keine DDX-gebundene
ZRD-Nutzlast. Der Button darf daher kein Boolean, keinen Valve-Eintrag und kein
sonstiges AIV-Feld erzeugen.

Ohne Verbindung zu einem laufenden Spiel kann ein dateibasierter Editor diese
Aktion nicht originalgetreu ausführen. Das Control sollte dann ausgeblendet oder
als nicht verfügbare Laufzeitaktion gekennzeichnet werden.

### `Lancemate` (Control 1283)

| Eigenschaft | Ergebnis |
|---|---|
| Controltyp | `BS_AUTO3STATE`, also Off / On / Auto |
| Dialog-/Editorzustand | Editorbyte `+0x131`, Bits 0–1 |
| AIV-Ziel | Parserstruktur `+0xCD`, Bits 0–1 |
| ZRD-Pfad | optionales Tag `lancemate[0]` |
| Laufzeitziel | Fahrzeugbyte `+0xBC`, Bit 4 (`is_lancemate`) |
| Parser | `FUN_004DF730`, Block `0x004E008E–0x004E00D1` |
| Laufzeitauflösung | `FUN_004D5BE0` |

Die Zuordnung ist direkt belegt. Der Editor hält die AIV-Parserstruktur ab
Offset `+0x64`; daher ist Editoroffset `+0x131` exakt das interne AIV-Feld
`+0xCD`. Der DDX-Code bei `0x004B90A2–0x004B912F` bildet dessen zwei Bits auf
die drei Checkboxzustände ab:

| Zustand | interne Bits | ZRD-Eingabe | Wirkung |
|---|---:|---|---|
| Off | `00` | `lancemate`, `["off"]` beziehungsweise numerisch `0` | Fahrzeug nie als Lancemate markieren |
| On | `01` | `lancemate`, `["on"]` beziehungsweise numerisch `1` | Fahrzeug unabhängig von seiner Teamzugehörigkeit als Lancemate markieren |
| Auto | `10` | Tag weglassen; der Parser akzeptiert außerdem numerisch `2` | nur freundliche Nicht-Spieler-Fahrzeuge als Lancemate markieren |

`FUN_00556E90` akzeptiert zusätzlich die Stringpaare `true`/`false`. In den
ausgelieferten Readerdaten ist nur `off` tatsächlich belegt; `on` ist durch den
Parsercode akzeptiert, kommt dort aber nicht vor. Wichtig:
Der von derselben Hilfsfunktion akzeptierte String `default` ergibt den Wert 0
und bedeutet hier daher **Off**, nicht Auto. Fehlt das Tag, initialisiert die
AIV-Defaulttabelle das Feld mit `2`, also Auto.

Im Auto-Zweig liest `FUN_004D5BE0` die Friend-Maske des aktuell kontrollierten
Spielerfahrzeugs und die Membership-Maske des Kandidaten. Bit 4 wird nur gesetzt,
wenn der Kandidat

- ein Fahrzeug beziehungsweise einen Mech besitzt,
- nicht das kontrollierte Spielerfahrzeug selbst ist und
- mindestens eine Team-ID mit der Friend-Maske des Spielers teilt.

Damit ist die Position im Gruppenrahmen `Team` sachlich passend: `record[3]`
liefert die Teamzugehörigkeit, während `lancemate` festlegt, ob daraus der
zusätzliche spielergesteuerte Lancemate-Status entstehen darf. Das Tag verändert
weder `record[3]` noch die Membership-, Friend- oder Foe-Masken.

Der resultierende Laufzeitstatus ist aktiv verwendet. Unter anderem zählt
`FUN_00468250` nur aktive Fahrzeuge mit `+0xBC` Bit 4 für die Lancemate-/Command-
Anzeige und erzeugt deren Einträge über `FUN_00467FA0`; weitere Auswahl-,
Kommandierungs- und Sortierpfade testen dasselbe Bit. Einzelne Verbraucher
prüfen zusätzlich, ob der markierte Mech wirklich freundlich zum kontrollierten
Fahrzeug ist. Ein erzwungenes On macht einen feindlichen Mech daher nicht zum
Freund und ändert auch nicht das Kampfverhalten.

Die vier expliziten Vorkommen in den Readerdaten demonstrieren den Zweck des
Off-Overrides:

| Reader | Record | Team | gespeicherter Wert |
|---|---|---:|---|
| `t1/readerm1` | `bushwacker_01` | 1 | `"off"` |
| `t1/readerm2` | `bushwacker_01` | 1 | `"off"` |
| `t1/readerm3` | `bushwacker_01` | 1 | `"off"` |
| `t1/readerm4` | `bushwacker_02` | 1 | `"off"` |

Diese Instruktor-Mechs teilen Team 1 mit dem Spieler, werden aber ausdrücklich
aus dem Lancemate-/Command-System ausgeschlossen. Alle Records ohne Tag benutzen
den Auto-Default.

Empfohlene Editorabbildung: drei Schaltflächen `Auto / On / Off` unmittelbar
unter den ebenfalls dreiteiligen Team-Schaltflächen `Friend / Foe / Other`.
Damit bleibt sichtbar, dass Lancemate zwar zur Teamgruppe gehört, aber einen
eigenen Status und keine vierte Teamrelation darstellt.

Beim Anklicken schreibt On `["on"]` und Off `["off"]`. Auto entfernt das
optionale Tag und verwendet damit den nativen Default der ausgelieferten Daten.
Der Parser akzeptiert zwar auch den expliziten Auto-Wert `[2]`; lädt der Editor
einen solchen Wert, soll er ihn als Auto anzeigen und bis zu einem bewussten
Klick unverändert erhalten. Ein Klick auf Auto normalisiert ihn anschließend
auf den fehlenden Tag. Unbekannte Werte bleiben ebenfalls bis zu einer bewussten
Auswahl verlustfrei erhalten.

## Verwandte Referenzen

- [`../dialog-resource-catalog.md`](../dialog-resource-catalog.md) — Dialog- und Control-ID nach
  AIV-Feld
- [`../dialog-parameter-map.md`](../dialog-parameter-map.md) — AIV-Feld nach Dialog,
  Datentyp und Validierungsregel
- [`../parameter-field-map.md`](../parameter-field-map.md) — Parserziele und interne
  Strukturfelder
