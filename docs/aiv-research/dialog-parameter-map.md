# AIV → Dialog: belegte Feld-, Typ- und Validierungskarte

## Zweck und Beweisgrenze

Diese Datei ist die Gegenrichtung zu
[`dialog-resource-catalog.md`](dialog-resource-catalog.md): Ausgangspunkt ist ein Feld in
`aiv.zrd`, Ziel sind Dialog, Control-ID, Datentyp und ein geeignetes
Editor-Widget.

Eine Zuordnung heißt **direkt**, wenn der MFC-`DoDataExchange`-Code eine
Control-ID an dasselbe Editorfeld bindet, in das der AIV-Parser den betreffenden
Wert lädt. **Datenbelegt** bedeutet, dass der Typ beziehungsweise Wertebereich
in allen extrahierten Readern konsistent ist, aber noch keine direkte
Dialogbindung gefunden wurde. **Offen** bezeichnet nur die Semantik oder den
fehlenden Kopierpfad; der technische Datentyp kann trotzdem bekannt sein.

Der Editor hält vor der Parserstruktur einen Header von `0x64` Bytes. Deshalb
entspricht beispielsweise Parseroffset `+0x1c` dem Editoroffset `+0x80`.

## Typnotation und Speicherregeln

| Notation | ZRD-/JSON-Speicherung | Editorregel |
|---|---|---|
| `Array[n]` | Array mit exakt `n` Elementen | Länge beim Laden prüfen; beim Speichern nicht verkürzen oder verlängern |
| `Number` | endliche JSON-Zahl, ganzzahlig oder mit Nachkommastellen | Zahlenfeld; `NaN` und ±`Infinity` ablehnen |
| `Integer` | JSON-Zahl ohne Nachkommastellen | Integerfeld; keine stillschweigende Rundung |
| `BoolInt` | numerisch `0` oder `1`, **nicht** JSON-`true`/`false` | Checkbox; ausschließlich `0` oder `1` speichern |
| `EnumInt{…}` | Integer aus der angegebenen Menge | Dropdown/Radio; keine freien Zahlen zulassen |
| `TriState` | optionaler Tag mit String `"off"`/`"on"` oder Integer `0`/`1`/`2`; fehlender Tag ist Auto | drei Schaltflächen Off/On/Auto; Auto beim bewussten Anklicken durch Entfernen des Tags speichern |
| `String` | String beziehungsweise ZRD-String/Bezeichner | Text- oder Referenzfeld |
| `0 \| String` | numerische Null oder Stringreferenz | Dropdown mit `None`; `None` immer als numerische `0` speichern |

Alle benannten Parametergruppen (`params`, `p2`, `p6` usw.) sind optionale
Tags, deren Wert jeweils ein Array ist. Ein skalarer Index innerhalb des Arrays
ist nicht selbst wieder ein Array.

## AIV-Grundstruktur und Sondertags — Dialog 166 (`p2`)

| AIV-Pfad | ZRD-Typ | Bedeutung / Editorregel | Dialog / Control | Status |
|---|---|---|---|---|
| Root | Folge von `String, Array`-Paaren | Fahrzeugname und Fahrzeugrecord paarweise erhalten | Hauptdialog `Mech AI` | Struktur direkt aus den Daten |
| Fahrzeugname | `String` | Freier, innerhalb der Mission sinnvollerweise eindeutiger Name | Fahrzeugliste / Haupteditor | direkt |
| `record` | Array, mindestens 4 Grundelemente, danach Tag/Wert-Paare | Unbekannte Tags und Reihenfolge unverändert erhalten | Dialog 166 plus Unterdialoge | direkt |
| `record[0]` | `Integer` | Net ID; im Originaldialog schreibgeschützt anzeigen, Auswahl erfolgt über Home Net | `Net ID` 1117; `Home Net` 1238 | Zuordnung semantisch belegt; genauer Home-Net-Kopierpfad offen |
| `record[1]` | `Array[3]<Number>` | Position X/Y/Z | 1200 / 1201 / 1202 | direkt |
| `record[2]` | `Number` | Yaw | 1203 | direkt |
| `record[3]` | `UInt32`; gepackte, nicht leere Liste von Team-IDs in 5-Bit-Segmenten | Team-IDs `0..31` als Mehrfachauswahl bearbeiten; Rohwert zusätzlich verlustfrei erhalten. `Friend/Foe/Other` nur als Legacy-Dialogvorschau anzeigen; bei Mehrfach-Teams können Friend- und Foe-Schnitt gleichzeitig gesetzt sein | 1197–1199 und 1111 | vollständig belegt: Teamnummer ist persistent; Radiowert wird relativ zu den Masken des kontrollierten Mechs abgeleitet und nicht separat gespeichert. Kampfpfade können den Foe-Schnitt ohne Friend-Vorrang prüfen |
| `pilot[0]` | `String` | Pilot Name; maximal 32 Zeichen laut `DDV_MaxChars` | 1243 | direkt |
| `pilot`-Untertag `can_configure[0]` | `String`-Enum `{ "true", "false" }` | Checkbox anzeigen, aber als String speichern; beim Ändern den restlichen `pilot`-Inhalt erhalten | 1279 | Parser und Daten direkt belegt |
| `skin[0]` | `Integer`, `0..100` | Integerfeld mit Min/Max; vorhandene Reader benutzen 0–7 | 1284 | direkt; `DDV_MinMaxInt(0,100)` |
| `lancemate[0]` | `TriState`; beobachtet `"off"`, parserbelegt außerdem `"on"` und numerisch `0..2` | drei Schaltflächen Auto / On / Off direkt unter Friend / Foe / Other; Auto entfernt beim Anklicken den optionalen Tag, unbekannte vorhandene Werte bis zur Änderung erhalten | 1283, Drei-Zustände | direkte Bindung: Editor `+0x131` = AIV `+0xCD`; Laufzeit setzt daraus `is_lancemate` (`+0xBC`, Bit 4) |

`Configuration` (1273), `Show` (1249), `Show InOut` (1282), `Net Edit`
(1250) und `Send home` (1255) sind Editor-/Laufzeitfunktionen ohne eigenes
belegtes AIV-Feld.

## `params` — `Array[9]`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `params[0]` | `Number` | endliches Zahlenfeld | `+0x1c` | `p1`: `Range to` | 1209 | direkt |
| `params[1]` | `BoolInt` | Checkbox `Enable Initial`; nur 0/1 | `+0x3c` | `p1`: `Enable Initial` | 1217 | direkt |
| `params[2]` | `Number` | endliches Zahlenfeld | `+0x20` | `p1`: `Leave Net Range to Target` | 1210 | direkt |
| `params[3]` | `Number` | endliches Zahlenfeld; ein gemeinsamer Wert in beiden Dialogen | `+0x24` | `p1`: `Return to Net Range from Net`; `Retreat`: `Net Distance` | 1211 | direkt, shared/union |
| `params[4]` | `Number` | endliches Zahlenfeld | `+0x28` | `p1` und `Direct`: `Approach Range to Target` | 1212 | direkt, shared |
| `params[5]` | `Number` | endliches Zahlenfeld | `+0x2c` | `p1` und `Direct`: `Minimum Range to Target` | 1214 | direkt, shared |
| `params[6]` | `Number` | endliches Zahlenfeld | `+0x30` | `Fire`: `Maximum Acquisition Range` | 1145 | direkt |
| `params[7]` | `Number` | endliche Zeitangabe | `+0x34` | `Fire`: `Fire Time` | 1191 | direkt |
| `params[8]` | `Number` | endliche Zeitangabe | `+0x38` | `Fire`: `Cooldown Time` | 1216 | direkt |

`params[0]`, `[2]` und `[3]` werden intern quadriert. Der Editor soll die
lesbaren, nicht quadrierten Werte aus der ZRD anzeigen und speichern.

## `p2` — `Array[8]`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `p2[0]` | `EnumInt{0,1,2,3,4}` | Dropdown: Direct, Zig Zag, Circle, Jump, Random Move | `+0x40` | `Attack`: `Strategy` | 1233 | direkt über Auswahl-Handler |
| `p2[1]` | `BoolInt` | Checkbox | `+0xcc`, Bit 0 | `Retreat`: `Net Distance` aktiv | 1221 | direkt |
| `p2[2]` | `BoolInt` | Checkbox | `+0xcc`, Bit 1 | `Retreat`: `Time` aktiv | 1224 | direkt |
| `p2[3]` | `BoolInt` | Checkbox | `+0xcc`, Bit 2 | `Retreat`: `Health` aktiv | 1222 | direkt |
| `p2[4]` | `Number` | Zahlenfeld; Beschriftung strategieabhängig | `+0x44` | `Circle`: `Radius`; `Zig Zag`: `Length` | 1220 / 1219 | direkt, union |
| `p2[5]` | `Number` | Zahlenfeld | `+0x48` | `Zig Zag`: `Target Distance` | 1220 | direkt |
| `p2[6]` | `Number` | endliche Zeitangabe | `+0x4c` | `Retreat`: `Time`-Wert | 1215 | direkt |
| `p2[7]` | `Number` | endlicher Schwellwert | `+0x50` | `Retreat`: `Health`-Wert | 1213 | direkt |

Der Ressourcenname des allgemeinen Fahrzeugdialogs 166 lautet ebenfalls `p2`.
Das ist nicht dasselbe wie dieser serialisierte Parameter-Tag.

## `p6` — `Array[4]`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `p6[0]` | `BoolInt` | Checkbox `Disable` | `+0x54` | `p1`: `Disable` | 1251 | direkt |
| `p6[1]` | `Number` | endliche Zeitangabe | `+0x58` | `Retreat`: Timeout-Wert | 1254 | direkt |
| `p6[2]` | `BoolInt` | Checkbox `For timeout` | `+0x5c` | `Retreat`: `For timeout` | 1253 | direkt |
| `p6[3]` | `BoolInt` | Checkbox `Forever` | `+0x60` | `Retreat`: `Forever` | 1252 | direkt |

## `p8` — `Array[17]`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `p8[0]` | `Number` | Zahlenfeld | `+0x64` | `Jump`: `Delay Min`; `p5`: `Min Delay` | 1223 | direkt, shared |
| `p8[1]` | `Number` | Zahlenfeld | `+0x68` | `Jump`: `Delay Max`; `p5`: `Max Delay` | 1225 | direkt, shared |
| `p8[2]` | `Number` | Zahlenfeld | `+0x6c` | `Jump`: `Duration Min`; `p5`: `Min Hold` | 1226 | direkt, shared |
| `p8[3]` | `Number` | Zahlenfeld | `+0x70` | `Jump`: `Duration Max`; `p5`: `Max Hold` | 1227 | direkt, shared |
| `p8[4]` | `BoolInt` | Checkbox | `+0x80` | `p5`: Collision Avoidance `Enable` | 1246 | direkt |
| `p8[5]` | `Number` | Zahlenfeld | `+0x74` | `p5`: `Alert Dist` | 1228 | direkt |
| `p8[6]` | `Number` | Zahlenfeld | `+0x78` | `p5`: `Min Dist` | 1230 | direkt |
| `p8[7]` | `Number` | Zahlenfeld | `+0x7c` | `p5`: `Jump Ceiling` | 1247 | direkt |
| `p8[8]` | `Number` | Zahlenfeld | `+0x84` | `Attack`: `Approach` | 1248 | direkt |
| `p8[9]` | `BoolInt` | Checkbox | `+0x88` | `p1`: `Always active` | 1231 | direkt |
| `p8[10]` | `EnumInt{0,1}` | Dropdown; keine freie Zahl | `+0x8c` | `p1`: Auswahl neben `Range to`; gespeichert als `selected index == 0` | 1229 | direkt über Combo-Handler |
| `p8[11]` | `Number` | Zahlenfeld | `+0x90` | `Fire`: `Alpha Strike Probability` | 1232 | direkt |
| `p8[12]` | `Number` | Zahlenfeld | `+0x94` | `Attack`: `Attack` | 1234 | direkt |
| `p8[13]` | `BoolInt` | Checkbox | `+0x98` | `p5`, Initial Scan Trigger: `Aim` | 1261 | direkt |
| `p8[14]` | `BoolInt` | Checkbox | `+0x9c` | `p5`, Initial Scan Trigger: `Hit` | 1260 | direkt |
| `p8[15]` | `BoolInt` | Checkbox | `+0xa0` | `p5`, Initial Scan Trigger: `Time` | 1244 | direkt |
| `p8[16]` | `EnumInt{0,1}` | Radio/Dropdown: Scan oder Jump | `+0xa4` | `p5`: `Scan` / `Jump` | 1258 / 1259 | direkt |

Die separaten `Set Value`-Felder des p5-Dialogs liegen bei `+0x974`, `+0x978`
und `+0x97c`. Sie sind keine zusätzlichen `p8`-Indizes.

## `p9` — `Array[5]`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `p9[0]` | `Number` | Zahlenfeld; unbekannte Werte unverändert erhalten | `+0xa8` | keine direkte Dialogbindung | — | Typ datenbelegt, Semantik offen |
| `p9[1]` | `Number` | Zahlenfeld; unbekannte Werte unverändert erhalten | `+0xac` | keine direkte Dialogbindung | — | Typ datenbelegt, Semantik offen |
| `p9[2]` | `Number` | Zahlenfeld; unbekannte Werte unverändert erhalten | `+0xb0` | keine direkte Dialogbindung | — | Typ datenbelegt, Semantik offen |
| `p9[3]` | `BoolInt` | Checkbox | `+0xcc`, Bit 3 | `Retreat`: `Health Change` aktiv | 1271 | direkt |
| `p9[4]` | `Number` | Zahlenfeld | `+0xb4` | `Retreat`: `Health Change`-Wert | 1270 | direkt |

## `p10`, `p11`, `p12`, `p13`

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Parserziel | Dialog / Originalbeschriftung | Control | Status |
|---|---|---|---:|---|---:|---|
| `p10[0]` | `Number` | Zahlenfeld | `+0xb8` | `Attack`: `Accuracy (0 is most accurate)` | 1272 | direkt |
| `p10[1]` | `Integer`, beobachtet `0..4` | Integerfeld; vorerst warnen statt auf 0..4 zu klemmen | `+0xbc` | keine direkte Dialogbindung | — | Typ datenbelegt, Semantik offen |
| `p11[0]` | `Number` | Zahlenfeld | `+0xc0` | `Attack`: `FCS/TC multiplier` | 1274 | direkt |
| `p12[0]` | `String`-Enum | Chassis-Dropdown; unbekannte Tokens verlustfrei anzeigen | `+0xc4` | Fahrzeugdialog 166: `Default Chassis` | 1274 | direkt über Combo-Handler |
| `p12[1]` | `BoolInt` | Checkbox | `+0xcc`, Bit 4 | Fahrzeugdialog 166: `Objective Conditional` | 1275 | direkt |
| `p12[2]` | `BoolInt` | Checkbox | `+0xcc`, Bit 5 | Fahrzeugdialog 166: `Shutdown on start` | 1276 | direkt |
| `p13[0]` | `BoolInt` | Checkbox | `+0xcc`, Bit 6 | Dialog 170 (`p5`): `Quit scan on trigger pull` | 1278 | direkt |

Bekannte `p12[0]`-Tokens aus Editor und Daten sind: `notmech`, `annihilator`,
`avatar`, `blackhawk`, `bushwacker`, `cauldron`, `champion`, `daishi`,
`elemental`, `firefly`, `madcat`, `orion`, `owens`, `puma`, `shadowcat`,
`strider`, `sunder`, `supernova`, `thor`, `vulture`.

## `vlist` — `Array<Array[17]>`

Jeder Eintrag besitzt exakt 17 Positionen. Die Condition-Flags sind trotz ihres
numerischen ZRD-Formats boolesch und dürfen im Editor keine freie Zahleneingabe
erhalten.

| Pfad | ZRD-Typ | Editor-Widget / Validierung | Dialog 172 / Control | Status |
|---|---|---|---|---|
| `vlist[*][0]` | `EnumInt{0,1}` | Dropdown: 0 Close, 1 Open | 1178 / 1179 | direkt |
| `vlist[*][1]` | `BoolInt` | Checkbox Ammo enabled | 1181 | direkt |
| `vlist[*][2]` | `Number` | Ammo threshold; Daten liegen in 0..1, außerhalb zunächst warnen | 1193 | direkt |
| `vlist[*][3]` | `BoolInt` | Checkbox Fired on | 1182 | direkt |
| `vlist[*][4]` | `BoolInt` | Checkbox Health enabled | 1180 | direkt |
| `vlist[*][5]` | `Number` | Health threshold; Daten liegen in 0..1, außerhalb zunächst warnen | 1192 | direkt |
| `vlist[*][6]` | `BoolInt` | Checkbox Heat enabled | 1186 | direkt |
| `vlist[*][7]` | `Number` | Heat threshold; Daten liegen in 0..1, außerhalb zunächst warnen | 1195 | direkt |
| `vlist[*][8]` | `BoolInt` | Checkbox Enters | 1188 | direkt |
| `vlist[*][9]` | `0 \| String` | Regions-Dropdown; `None` als numerische 0 speichern | 1177 | direkt |
| `vlist[*][10]` | `BoolInt` | Checkbox Target LOS | 1190 | direkt |
| `vlist[*][11]` | `Number` | Target-LOS-Reichweite; nicht quadriert speichern | 1196 | direkt |
| `vlist[*][12]` | `BoolInt` | Checkbox Target Distance | 1184 | direkt |
| `vlist[*][13]` | `Number` | Target-Distance-Schwellwert; nicht quadriert speichern | 1194 | direkt |
| `vlist[*][14]` | `BoolInt` | Checkbox Active | 1256 | direkt |
| `vlist[*][15]` | `String` | Valve-Referenzfeld; existierende Namen anbieten, unbekannte erhalten | 1176 | direkt |
| `vlist[*][16]` | `BoolInt` | Checkbox Exits | 1189 | direkt |

`Enters` und `Exits` sind unabhängige Flags und dürfen gleichzeitig `1` sein.
Beide benutzen denselben Regionswert aus Index 9.

## Nicht serialisierte Dialogfelder

| Dialog / Control | Laufzeittyp | Grund für Ausschluss aus dem AIV-Schema |
|---|---|---|
| 167 `Enable Now` (1257) | MFC-Drei-Zustände-Feld bei Editoroffset `+0x984` | kein belegter AIV-Tag oder Arrayindex |
| 170 Scan Trigger `Set Value` | drei Editorfelder bei `+0x974`, `+0x978`, `+0x97c` | kein belegter `p8`-Index |
| 166 `Configuration` (1273) | Comboobjekt `+0x214` | kein belegter AIV-Speicherpfad |
| 166 `Show`, `Show InOut`, `Net Edit`, `Send home` | UI-/Laufzeitstatus beziehungsweise Aktionen | nicht persistente Editorfunktionen |

## Verbleibende offene Semantik

Für die gegenwärtigen Readerdaten sind nur noch folgende skalare Bedeutungen
ohne belastbaren Originalnamen:

- `p9[0]`, `p9[1]`, `p9[2]` — Typ `Number`
- `p10[1]` — Typ `Integer`, beobachtet 0 bis 4

Die frühere offene Frage zu `lancemate[0]` ist geklärt. Das Feld ist
dreistufig: Off erzwingt den Status aus, On erzwingt ihn an und Auto leitet ihn
aus der Friend-Beziehung zum kontrollierten Spielerfahrzeug ab. Ein fehlender
Tag entspricht Auto. Details stehen in
[`legacy-dialog-options/legacy-dialog-166-p2-reference.md`](legacy-dialog-options/legacy-dialog-166-p2-reference.md#lancemate-control-1283).

Die frühere offene Frage zu `record[3]` ist geklärt: Der Wert ist eine gepackte
Team-ID-Liste; `Friend/Foe/Other` wird im Legacy-Editor nur relativ zum aktuell
kontrollierten Mech berechnet. Diese Friend-zuerst-Anzeige ist bei Mehrfach-Teams
keine allgemeine Laufzeitrelation, weil direkte Foe-Filter denselben Record
zugleich als feindfähig einstufen können. Details und Editorregeln stehen in
[`legacy-dialog-options/legacy-dialog-166-p2-reference.md`](legacy-dialog-options/legacy-dialog-166-p2-reference.md#team-ids-und-abgeleitete-zugehörigkeit-controls-11971199-und-1111).

## Konkrete Editor-Validierungsregeln

1. Optionale Tags dürfen fehlen; wenn sie vorhanden sind, muss ihre Arraylänge
   exakt der oben angegebenen Länge entsprechen.
2. `BoolInt` wird als Checkbox dargestellt und ausschließlich als numerische
   `0` oder `1` gespeichert. Ein geladener Wert wie `3` erzeugt eine Diagnose
   und wird nicht stillschweigend geändert.
3. `EnumInt` akzeptiert nur die dokumentierte Menge. Unbekannte geladene Werte
   bleiben bis zu einer bewussten Benutzeränderung erhalten und werden sichtbar
   als unbekannt markiert.
4. `Number` muss endlich sein. Zusätzliche Grenzen werden nur dort hart
   erzwungen, wo MFC-DDV-Code sie belegt; reine Beobachtungsbereiche erzeugen
   lediglich Warnungen.
5. Referenz-Strings bekommen Dropdown/Datalist-Vorschläge, dürfen aber einen
   unbekannten vorhandenen Namen verlustfrei behalten.
6. Der Union-Typ `0 | String` verwendet im UI `None`, schreibt aber numerische
   `0`, niemals `"0"`, `"None"`, `null` oder einen leeren String.
7. `pilot` ist eine verschachtelte Struktur. Das Ändern von `pilot[0]` darf den
   Untertag `can_configure` nicht löschen; die Checkbox speichert dort die
   Strings `"true"` beziehungsweise `"false"`.
8. Unbekannte Tags, zusätzliche Tag/Wert-Paare und deren Reihenfolge bleiben
   beim Roundtrip erhalten.

## Codeanker

- `004df730`: AIV-Parser (`ZRdrAiv.cpp`)
- `004b8bc0`: DDX für Dialog 167 (`p1`)
- `004b8eb0`: DDX für Dialog 166 (allgemeine Fahrzeugdaten)
- `004b92f0`: DDX für Dialog 168 (`Attack`)
- `004b9490`: DDX für Dialog 169 (`Retreat`)
- `004b96a0`: DDX für Dialog 170 (`p5`)
- `004b9890`: DDX für Dialog 171 (`Fire`)
- `004b9930`: DDX für Dialog 172 (`Valve` / `vlist`)
- `004b9c30`: DDX für `Circle`
- `004b9c80`: DDX für `Direct`
- `004b9cf0`: DDX für `Jump`
- `004b9d90`: DDX für `Zig Zag`
- `004bc8f0`: Auswahl-Handler `Default Chassis` (`p12[0]`)
- `004e008e`: Parser des optionalen `lancemate`-Tags nach AIV `+0xcd`
- `004b90a2`: DDX-Bindung des Lancemate-Dreifachzustands (Control 1283)
- `004d5be0`: Auflösung Off / On / Auto in das Laufzeitflag `+0xbc`, Bit 4
- `005ac044`: MFC `DDV_MinMaxInt`; belegt `skin[0]` als Integer 0..100
