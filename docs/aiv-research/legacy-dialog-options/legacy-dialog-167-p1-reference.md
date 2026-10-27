# Legacy-Dialog 167 — `p1` (Aktivierung und Verfolgung)

Diese Datei beschreibt Resource 167 des eingebetteten MFC-Editors. Der Dialog
trägt im Original die Caption `p1`. Das ist nur der Ressourcenname: Er darf
nicht mit dem serialisierten ZRD-Tag `p1` verwechselt werden.

Die Tabelle trennt die sichtbare Beschriftung, die DDX-Bindung des alten
Editors und den nachgewiesenen Speicherort im geparsten AIV-Datensatz. Wo ein
Control nur einen Laufzeit- oder Editorzustand hält, wird ausdrücklich kein
ZRD-Feld erfunden.

## Vollständiges Control-Inventar

| Control / Originaltext | ID und Win32-Typ | DDX-/Modellbindung | ZRD-Feld oder Tag | Typ | Bedeutung und Status |
|---|---|---|---|---|---|
| `Always active` | 1231, Checkbox | Editor-Flag | `p8[9]` | `BoolInt` (0/1) | Aktivierungsoption. Sie erscheint optisch im Abschnitt „Activation“, wird aber im separaten `p8`-Array gespeichert. |
| *(unbeschriftetes Feld neben „Range to“)* | 1209, Edit | Editor-Zahlenfeld | `params[0]` (`+0x1c`) | `Number` | Äußerer Kontakt-/Aktivierungsbereich. Im Spiel wird der Wert quadriert; der Editor zeigt den lesbaren, nicht quadrierten Wert. |
| *(unbeschriftete Auswahl neben „Range to“)* | 1229, ComboBox | Combo-Handler; gespeichert als `selected index == 0` | `p8[10]` (`+0x8c`) | `EnumInt`, nur `0` oder `1` | Auswahl-/Moduswert. Die ursprünglichen sichtbaren Einträge sind aus der Resource nicht ersichtlich; deshalb darf das Feld nicht als freies Zahlenfeld behandelt werden. |
| `Enable Initial` | 1217, Checkbox | Dialogflag/Editorfeld | `params[1]` (`+0x3c`) | `BoolInt` (0/1) | Anfangswert des Aktivierungs-Gates. `0` lässt den frühen Laufzeitpfad sofort zurückkehren; `1` erlaubt die weiteren Ziel-/Ankerprüfungen. Es ist keine Reichweite. |
| *(unbeschriftetes Feld unter „Leave Net Range to Follow Target“)* | 1210, Edit | Editor-Zahlenfeld | `params[2]` (`+0x20`) | `Number` | Bereichsschwelle für den nahen Ziel-/Kontaktzweig. Intern quadriert; die genaue historische Funktionsbezeichnung ist im Code nicht vollständig isoliert. |
| *(unbeschriftetes Feld unter „Return to Net Range from Net“)* | 1211, Edit | Editor-Zahlenfeld | `params[3]` (`+0x24`) | `Number` | Gemeinsamer Wert mit `Retreat: Net Distance`. Begrenzt die Abweichung von einer gespeicherten Position/einem Anker; intern quadriert. |
| *(unbeschriftetes Feld unter „Approach Range to Target“)* | 1212, Edit | Editor-Zahlenfeld | `params[4]` (`+0x28`) | `Number` | Annäherungsbereich; dasselbe `params[4]`-Feld wird auch im Dialog `Direct` angezeigt. |
| *(unbeschriftetes Feld unter „Minimum Range to Target“)* | 1214, Edit | Editor-Zahlenfeld | `params[5]` (`+0x2c`) | `Number` | Mindestabstand zum Ziel; dasselbe `params[5]`-Feld wird auch im Dialog `Direct` angezeigt. |
| `Disable` | 1251, Checkbox | Editor-Flag | `p6[0]` (`+0x54`) | `BoolInt` (0/1) | Nachweislich der Disable-Schalter des Rückzugs-/Abschlussverhaltens. Obwohl er im Aktivierungsrahmen liegt, ist er kein `params`-Eintrag. |
| `Enable Now` | 1257, `BS_AUTO3STATE`-Checkbox | Editorzustand bei `+0x984` | kein AIV-Feld | Laufzeit-/Editorzustand | Aktion bzw. Zustand für die aktuelle Editor-/Spielinstanz. Er wird nicht als neuer ZRD-Wert serialisiert; die genaue Nebenwirkung des Befehls ist im aktuellen Codepfad nicht vollständig isoliert. |
| `Range to` | —, Static | keine DDX-Bindung | — | — | Reine Beschriftung für Control 1209 und die danebenliegende Auswahl 1229. |
| `Leave Net Range to Target` | —, Static | keine DDX-Bindung | — | — | Reine Beschriftung für Control 1210. |
| `Return to Net Range from Net` | —, Static | keine DDX-Bindung | — | — | Reine Beschriftung für Control 1211. |
| `Approach Range to Target` | —, Static | keine DDX-Bindung | — | — | Reine Beschriftung für Control 1212. |
| `Minimum Range to Target` | —, Static | keine DDX-Bindung | — | — | Reine Beschriftung für Control 1214. |
| `Leave Net to Follow Target` | —, GroupBox | keine DDX-Bindung | — | — | Visuelle Gruppierung; legt keinen zusätzlichen Tag oder Bereich fest. |
| `Follow Target Parameters` | —, GroupBox | keine DDX-Bindung | — | — | Visuelle Gruppierung; die enthaltenen Werte bleiben die einzelnen `params`-Felder. |
| `Activation` | —, GroupBox | keine DDX-Bindung | — | — | Visuelle Gruppierung für `Always active`, `Range to`, die Auswahl 1229 und `Disable`; die Speicherung liegt in drei verschiedenen Arrays. |

## Was der Dialog tatsächlich in `aiv.zrd` zeigt

Der Dialog 167 verwendet die ersten sechs Einträge des Tags `params`:

| ZRD-Pfad | Parserziel | Originalfeld | Kurzbeschreibung |
|---|---:|---|---|
| `params[0]` | `+0x1c` | `Range to` | äußerer Aktivierungs-/Kontaktbereich; intern `range²` |
| `params[1]` | `+0x3c` | `Enable Initial` | boolesches Start-Gate |
| `params[2]` | `+0x20` | `Leave Net Range to Target` | Nahbereichs-/Zielzweig; intern `range²` |
| `params[3]` | `+0x24` | `Return to Net Range from Net` | Anker-/Positionsabweichung; intern `range²` |
| `params[4]` | `+0x28` | `Approach Range to Target` | Annäherungsbereich |
| `params[5]` | `+0x2c` | `Minimum Range to Target` | Mindestabstand |

`params[6]` bis `params[8]` gehören zwar zum selben neunelementigen ZRD-Array,
werden aber in Resource 167 nicht dargestellt. Sie erscheinen im alten
`Fire`-Dialog (`Maximum Acquisition Range`, `Fire Time`, `Cooldown Time`).

Die drei Entfernungswerte `params[0]`, `[2]` und `[3]` werden vom Parser bzw.
Initialisierungscode quadriert. Ein kompatibler Editor sollte daher die
lesbare Entfernung annehmen und beim Speichern nicht bereits quadrierte Werte
schreiben.

## Die Controls, die absichtlich nicht unter `params` liegen

### `Always active` → `p8[9]`

`Always active` ist kein Alias für `params[1]`. Der Checkboxwert wird als
`p8[9]` gelesen/geschrieben (`+0x88`). Damit enthält der Dialog bereits ein
Beispiel für eine optische Gruppe, deren Felder aus verschiedenen ZRD-Arrays
kommen.

### Auswahl 1229 → `p8[10]`

Der Combo-Handler speichert einen zweistufigen Wert: `0`, wenn der erste
Eintrag ausgewählt ist, sonst `1`. Die Datenstruktur erlaubt deshalb nur diese
beiden Werte. Die ursprünglichen Eintragsnamen sind in der Resource nicht
enthalten; im modernen Editor sollte das als Dropdown mit genau zwei
verständlichen, dokumentierten Optionen erscheinen, nicht als freies Integer.

### `Disable` → `p6[0]`

Der Wert wird aus dem Tag `p6` genommen (`+0x54`). Er steuert den nachgewiesenen
Disable-Pfad nach einem Rückzug/Abschluss. Die Platzierung im Rahmen
„Activation“ ist nur Layout und kein Beweis für eine gemeinsame Speicherung.

### `Enable Now` → Laufzeitstatus

Der DDX-Handler von Resource 167 liegt bei `0x004B8BC0`; für Control 1257 wird
das Modelloffset `+0x984` gebunden. Es gibt keinen isolierten Parserpfad zu
`aiv.zrd`. Wegen `BS_AUTO3STATE` ist der Wert außerdem ein Zustands-/Aktionsfeld
des alten Editors, nicht ein zusätzliches boolesches `params`-Element. Ein
ZRD-Editor sollte es daher nicht speichern.

## Geteilte Werte mit anderen Legacy-Dialogen

Ein Wert darf nur einmal im AIV-Datensatz existieren, auch wenn er in mehreren
Dialogen erscheint:

| Control in `p1` | Gleiche Speicherung | Weitere Darstellung |
|---|---|---|
| 1211 `Return to Net Range from Net` | `params[3]` | `Retreat`: `Net Distance` |
| 1212 `Approach Range to Target` | `params[4]` | `Direct`: gleichnamiger Bereich |
| 1214 `Minimum Range to Target` | `params[5]` | `Direct`: gleichnamiger Bereich |

Ändert ein Editorfeld einen dieser Werte, müssen alle Ansichten denselben
Datensatzwert wiedergeben.

## Relevante Code-Anker

- `0x004B8BC0`: DDX-Handler für Resource 167; bestätigt insbesondere das
  Laufzeitfeld `Enable Now` bei Modelloffset `+0x984`.
- `0x004DF7E3`: Parserblock für `params`.
- `0x004D9B70`: Initialisierung/Normalisierung, die `params[0]`, `[2]` und
  `[3]` für interne Distanzvergleiche quadriert.
- `0x004E8F10`: früher Aktivierungspfad; prüft das Gate `params[1]` bei
  `+0x3c`.
- `0x004D9C80` / `0x004D9CA0`: Laufzeitsetter für dieses Gate.
- `0x004DFD38`: Parserblock für `p8`, einschließlich `[9]` und `[10]`.
- `0x004DFC9A`: Parserblock für `p6`, einschließlich `p6[0]`.

Die Anker belegen die Bindung und die relevanten Verbraucher. Wo die alte
Beschriftung oder ein genauer Verhaltenname im ausgelieferten Code nicht
eindeutig rekonstruiert werden kann, bleibt die Beschreibung bewusst neutral.
