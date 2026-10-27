# Mechanische Feldkarte des AIV-Parsers

## Status

Diese Datei dokumentiert die statisch belegte Zuordnung aus `FUN_004df730` in
`Mech3.exe`. Die Semantik der meisten Felder ist noch offen; Tag, Index,
Zieloffset und Bitposition sind dagegen direkt durch den Parser belegt.

- Parser: `FUN_004df730` (`004df730` bis ungefähr `004e0470`)
- Quelldatei laut Debugstring: `C:\zipper\Mech3\ZRdrAiv.cpp`
- Zielobjekt während des Parsens: `EBP` (beim Eintritt aus `ECX` übernommen)
- Tag-Lookup: `FUN_005569e0`
- Ein Arrayelement liegt im geparsten ZRD-Knoten jeweils acht Bytes auseinander.
- Der Wert von Index `n` liegt typischerweise bei `payload + 0x0c + 8*n`.

## Grundfelder des Records

Diese Werte stehen vor den benannten Parameter-Tags:

| Eingabe | Zieloffset | Bemerkung |
|---:|---:|---|
| `record[0]` | `+0x04` | direkt kopiert; Net ID |
| `record[1][0]` | `+0x08` | Position X |
| `record[1][1]` | `+0x0c` | Position Y |
| `record[1][2]` | `+0x10` | Position Z |
| `record[2]` | `+0x14` | Yaw / Rotation |
| `record[3]` | `+0x18` | gepackte Team-ID-Liste; bei alten, zu kurzen Records Default über `FUN_004d5b10` |

### `record[3]`: gepackte Team-IDs

`record[3]` wird vom Legacy-Dialog als vorzeichenloser Integer bearbeitet. Der
Wert besteht aus aufeinanderfolgenden 5-Bit-Segmenten. Jedes Segment enthält
eine Team-ID von 0 bis 31:

```text
team_id[0] = value        & 0x1f
team_id[1] = value >> 5   & 0x1f
team_id[2] = value >> 10  & 0x1f
...
```

Das erste Segment wird immer ausgewertet. Der Rohwert `0` bedeutet daher Team 0
und nicht „keine Teamzugehörigkeit“. Beispielkodierungen:

| Team-IDs | Rohwert |
|---|---:|
| `[0]` | `0` |
| `[1]` | `1` |
| `[2]` | `2` |
| `[1, 2]` | `1 | (2 << 5)` = `65` |
| `[1, 3]` | `1 | (3 << 5)` = `97` |

`FUN_004D71A0` führt die Gegenrichtung aus: Eine Team-Bitmaske wird als
aufsteigend sortierte, gepackte Liste eindeutiger Team-IDs kanonisiert.

Beim Erzeugen eines Fahrzeugs wandeln `FUN_0049FA20` und der entsprechende
Spielerpfad in `FUN_00491150` den Wert in eine Mitgliedschaftsmaske um:

```text
membership_mask = OR(1 << team_id)
friend_mask     = membership_mask & ~1
foe_mask        = ~friend_mask & ~1
```

Bit 0 wird aus jeder Friend- und Foe-Maske ausgeschlossen. Ein Team-0-Ziel wird
von anderen Einheiten deshalb weder als Friend noch als Foe erkannt. Die
Team-0-Einheit selbst erhält jedoch `friend_mask = 0` und
`foe_mask = 0xFFFFFFFE` und betrachtet damit alle Nichtnull-Teams als Foe; Team
0 ist unklassifiziert, nicht gegenseitig neutral.

Die Laufzeitbeziehung ist gerichtet. Prüfungen schneiden die Friend- oder
Foe-Maske des Actors mit der Membership-Maske des Ziels. Bei einem Ziel mit
mehreren Team-IDs können beide Schnitte zugleich gesetzt sein. `FUN_004BC260`
zeigt in diesem Fall wegen seiner Prüfungsreihenfolge `Friend`; direkte
Kampffilter wie `FUN_0045C4A0`, `FUN_00419200` und `FUN_004A9530` testen den
Foe-Schnitt dagegen ohne vorgelagerten Friend-Ausschluss. Mehrfach-Teams sind
daher keine symmetrische Allianz: Ein `[1,2]`-Fahrzeug greift Team 1 und Team 2
selbst nicht als Foe an, kann aber für beide Einzelteams weiterhin ein
angreifbares Foe-Ziel sein.

Der Resource-166-Dialog kopiert den persistenten Wert direkt zwischen
`record[3]`, dem AIV-Feld `+0x18` und dem Editorwert `+0x7c`. Die Radiogruppe
`Friend/Foe/Other` ist keine zweite Speicherung. `FUN_004BC260` vergleicht die
dekodierte Mitgliedschaftsmaske mit den aktuellen Freund-/Feindmasken des
kontrollierten Mechs. `FUN_004BC300` setzt umgekehrt die Teamnummer auf die
niedrigste Team-ID, die zur gewählten Relation gehört. Die Radiobox ist damit
nur eine Legacy-Dialogansicht und keine allgemeine Laufzeitklassifikation.

In den extrahierten Readern kommen nur Einzelteamwerte vor:

| `record[3]` | Vorkommen | typische Records |
|---:|---:|---|
| `0` | 32 | ausschließlich `stealth` |
| `1` | 398 | Spieler, Lancemates und andere verbündete Einheiten |
| `2` | 338 | Gegner |

Diese beobachtete Konvention macht `0/1/2` nicht zu einem fest eingebauten
`Other/Friend/Foe`-Enum. Die Radiobedeutung bleibt relativ zum kontrollierten
Mech und kann sich mit dessen Laufzeitmasken ändern.

## `params` — 9 Felder

Tagstring `005e9718`, Parserblock ab `004df7e3`.

| Index | Ziel |
|---:|---:|
| 0 | `+0x1c` |
| 1 | `+0x3c` |
| 2 | `+0x20` |
| 3 | `+0x24` |
| 4 | `+0x28` |
| 5 | `+0x2c` |
| 6 | `+0x30` |
| 7 | `+0x34` |
| 8 | `+0x38` |

Der zweite Wert wird absichtlich außerhalb der ansonsten fortlaufenden Folge bei
`+0x3c` abgelegt.

Defaultwerte vor der Rückumwandlung für Ausgabe/Parsing:

| Index | Default im internen Objekt |
|---:|---:|
| 0 | `250000.0` (`500²`) |
| 1 | `0.0` |
| 2 | `10000.0` (`100²`) |
| 3 | `40000.0` (`200²`) |
| 4 | `50.0` |
| 5 | `42.0` |
| 6 | `200.0` |
| 7 | `2.0` |
| 8 | `1.0` |

`FUN_004d9b70` quadriert die Felder 0, 2 und 3 nach dem Parsen. Es handelt sich
daher mit hoher Sicherheit um drei Distanz-/Reichweitenwerte, die intern als
quadratische Distanzen verglichen werden.

### Erste Laufzeitverbraucher (`FUN_004e9710` / `FUN_004e8f10`)

- `params[0]` (`+0x1c`) wird mit `contact+0x0c`, der quadrierten Entfernung
  zum ersten nach Teammaske passenden Kontakt, verglichen. Nur Kontakte
  **innerhalb** dieser Reichweite lassen den normalen AI-Verarbeitungszweig
  weiterlaufen. Das Feld ist damit die äußere Aktivierungs-/Kontakt-Reichweite
  des AI-Controllers (genauer Spielname noch offen).
- `params[2]` (`+0x20`) wird ebenfalls gegen dieselbe quadrierte
  Kontaktentfernung verglichen und schaltet einen Nahbereichszweig um. Der
  Zweig baut anschließend ein Verhaltensobjekt auf; ob der Originalname
  Feuer-, Ausweich- oder Nahkampf-Reichweite lautet, ist noch nicht sicher.
- `params[3]` (`+0x24`) begrenzt in `FUN_004e8f10` die quadrierte Entfernung
  zwischen aktueller Position und einer gespeicherten Position bei `+0xe4`.
  Zusammen mit `p2`-Flags kann zu große Abweichung die betreffende Aktion
  sperren. Es ist somit eine Radius-/Abweichungsgrenze um einen Anker- oder
  Wegpunkt, nicht einfach eine zweite Kopie von `params[0]`.

- `params[1]` (`+0x3c`) ist kein Reichweitenwert. `FUN_004e8f10` beginnt bei
  `004e8f1b` mit `TEST dword ptr [ESI+0x3c],...`; bei null springt die Funktion
  sofort zum Rückgabepfad `0` (`004e8f20`). Nur bei einem gesetzten Wert werden
  die anschließenden Ziel-/Anker- und `p2`-Prüfungen ausgeführt. Das Feld ist
  damit ein Enable-Gate für diesen AI-Verhaltenspfad, nicht ein Multiplikator
  oder Schalter für `params[0]`.
- Der Wert ist zur Laufzeit veränderbar: `004d9c80` setzt `+0x3c` auf `1`,
  `004d9ca0` auf `0`; diese Setter werden beim Aufbau der Fahrzeug-
  `shutdown`-Steuerung als die beiden Zustandsaktionen registriert. Außerdem
  setzt `FUN_004e9710` bei `004e97d7` den Wert nach dem internen Timerpfad
  wieder auf `1` (unter der `p6[2]`/`For timeout`-Bedingung). `Enable Initial`
  beschreibt daher den Startwert dieses Gates, nicht die Aktivierungsdistanz.

Diese Aussagen beruhen auf den tatsächlichen Laufzeitvergleichen; die
neutralen Bezeichnungen bleiben absichtlich bestehen, bis die umgebenden
Verhaltensklassen vollständig identifiziert sind.

## `p2` — 8 Felder

Tagstring `005e9714`, Parserblock ab `004df844`.

| Index | Ziel |
|---:|---:|
| 0 | `+0x40` |
| 1 | `+0xcc`, Bit 0 |
| 2 | `+0xcc`, Bit 1 |
| 3 | `+0xcc`, Bit 2 |
| 4 | `+0x44` |
| 5 | `+0x48` |
| 6 | `+0x4c` |
| 7 | `+0x50` |

Die Felder 1 bis 3 werden als `value != 0` normalisiert.

## `p3` — 6 Felder

Tagstring `005e9710`, Parserblock ab `004df8e0`.

| Index | Ziel |
|---:|---:|
| 0 | `+0x64` |
| 1 | `+0x68` |
| 2 | `+0x6c` |
| 3 | `+0x70` |
| 4 | `+0x74` |
| 5 | `+0x78` |

Debugstrings bezeichnen diese Gruppe als Scan-/Jump-Parameter. Teile davon können
durch `p7` überschrieben werden.

## `p4` — komplexe Unterstruktur

Tagstring `005e970c`, Parserblock ab `004df931`.

`p4` erzeugt bei vorhandenem Feld 22 eine zusätzliche Listeneintragung im Objekt
bei `+0xd0`. Die ersten vier Werte werden außerdem direkt gespeichert.

| Index | Ziel/Verwendung |
|---:|---|
| 0 | `+0x88` |
| 1 | `+0x8c` |
| 2 | `+0x90` |
| 3 | `+0x94` |
| 4 | temporäres Unterstrukturfeld 0 |
| 5 | temporäres Unterstrukturfeld 1 |
| 6 | temporäres Unterstrukturfeld 2 |
| 7 | temporäres Unterstrukturfeld 3 |
| 8 | Flagbit 1; Flagbit 0 wird fest gesetzt |
| 9 | Flagbit 2 |
| 10 | Unterstrukturwert |
| 11 | Flagbit 3 |
| 12 | Flagbit 4 |
| 13 | Unterstrukturwert |
| 14 | Flagbit 5 |
| 15 | Unterstrukturwert |
| 16 | zweites Flagbyte, Bit 0 |
| 17 | String/Referenz, wenn zugehöriger ZRD-Typcode `3` ist |
| 18 | Flagbit 6 |
| 19 | Unterstrukturwert |
| 20 | Flagbit 7 |
| 21 | Unterstrukturwert |
| 22 | String/Referenz; steuert Erzeugung des Listeneintrags |

Die erzeugte Listenelement-Nutzlast ist ungefähr `0x20` Bytes groß. Ihre genaue
Typdefinition und Semantik müssen noch über die Verbraucher bestimmt werden.

## `pilot` und `can_configure`

- `pilot` (`005e9704`) wird nach Umwandlung über `FUN_00575fb0`/`FUN_0049d250`
  in das Objektfeld bei `+0xd4` übernommen.
- Untertag `can_configure` (`005e96f4`) wird als `value != 0` in `+0xcc`, Bit 7
  gespeichert.

## `p5` — 5 Felder

Tagstring `005e96f0`, Parserblock ab `004dfc4d`.

| Index | Ziel/Verwendung |
|---:|---|
| 0 | `+0x7c` |
| 1 | `+0x84` |
| 2 | `+0x80` |
| 3 | temporär für spätere Scan-/Jump-Auflösung |
| 4 | temporär für spätere Scan-/Jump-Auflösung |

## `p6` — 4 Felder

Tagstring `005e96ec`, Parserblock ab `004dfc9a`.

| Index | Ziel |
|---:|---:|
| 0 | `+0x54` |
| 1 | `+0x58` |
| 2 | `+0x5c` |
| 3 | `+0x60` |

## `p7` — 8 Felder

Tagstring `005e96e8`, Parserblock ab `004dfcce`.

| Index | Ziel/Verwendung |
|---:|---|
| 0 | `+0x88` |
| 1 | `+0x8c` |
| 2 | `+0x90` |
| 3 | `+0x94` |
| 4 | temporärer Override für `+0x68` |
| 5 | temporärer Override für `+0x64` |
| 6 | temporärer Override für `+0x6c` |
| 7 | temporärer Override für `+0x70` |

Der Parser warnt ausdrücklich, dass `p3`-Scanwerte zugunsten von `p7` ignoriert
werden. Die ungewöhnliche Reihenfolge der Overrides ist aus dem späteren
Zuweisungscode übernommen.

## `p8` — 17 Felder

Tagstring `005e96e4`, Parserblock ab `004dfd38`.

| Index | Ziel |
|---:|---:|
| 0 | `+0x64` |
| 1 | `+0x68` |
| 2 | `+0x6c` |
| 3 | `+0x70` |
| 4 | `+0x80` |
| 5 | `+0x74` |
| 6 | `+0x78` |
| 7 | `+0x7c` |
| 8 | `+0x84` |
| 9 | `+0x88` |
| 10 | `+0x8c` |
| 11 | `+0x90` |
| 12 | `+0x94` |
| 13 | `+0x98` |
| 14 | `+0x9c` |
| 15 | `+0xa0` |
| 16 | `+0xa4` |

Der Parser warnt, wenn `p8` gemeinsam mit `p3`, `p4`, `p5` oder `p7` vorkommt.
Das Zielfeld `+0x74` (Index 5 bei `p8`, Index 4 bei `p3`) wird am Parserende
quadriert und ist daher ebenfalls mit hoher Sicherheit eine Distanz/Reichweite.

## `p9` — 5 Felder

Tagstring `005e96e0`, Parserblock ab `004dfe14`.

| Index | Ziel |
|---:|---:|
| 0 | `+0xa8` |
| 1 | `+0xac` |
| 2 | `+0xb0` |
| 3 | `+0xcc`, Bit 3 |
| 4 | `+0xb4` |

## `p10` — 2 Felder

Tagstring `005e96dc`, Parserblock ab `004dfe7f`.

| Index | Ziel |
|---:|---:|
| 0 | `+0xb8` |
| 1 | `+0xbc` |

## `p11` — 1 Feld

Tagstring `005e96d8`, Parserblock ab `004dfeae`.

| Index | Ziel |
|---:|---:|
| 0 | `+0xc0` |

## `p12` — 3 Felder

Tagstring `005e96d4`, Parserblock ab `004dfed1`.

| Index | Ziel/Verwendung |
|---:|---|
| 0 | über `FUN_004845f0` umgewandelt, Ergebnis nach `+0xc4` |
| 1 | `+0xcc`, Bit 4 |
| 2 | `+0xcc`, Bit 5 |

In den vorhandenen Daten ist Index 0 ein Fahrzeug-/Mechname wie `sunder` oder
`bushwacker`. Der Editor bindet dieses Feld über den Auswahl-Handler bei
`004bc8f0` direkt an `Default Chassis` (Dialog 166, Control 1274).

Der interne Editor liefert die Originalbeschriftungen der beiden Flags:

- Index 1 / Bit 4: `Objective Conditional`
- Index 2 / Bit 5: `Shutdown on start`

## `lancemate` — Dreizustands-Tag

Tagstring `005e9670`, Parserblock `004e008e` bis `004e00d1`.

| Zustand | Parserstruktur `+0xcd` | akzeptierte ZRD-Eingabe | Laufzeitwirkung |
|---|---:|---|---|
| Off | `0` | `"off"`, `"false"` oder numerisch `0` | Lancemate-Flag löschen |
| On | `1` | `"on"`, `"true"` oder numerisch `1` | Lancemate-Flag setzen |
| Auto | `2` | Tag fehlt oder numerisch `2` | Flag aus der Friend-Beziehung zum Spieler ableiten |

Die Defaulttabelle `DAT_005e8dc0` initialisiert das Byte `+0xcd` mit `2`.
Fehlt das optionale Tag, ist der Zustand daher Auto. Die generische
Dreizustands-Hilfsfunktion `FUN_00556e90` akzeptiert auch den String `default`,
ordnet ihn jedoch dem Wert 0 und damit Off zu; er ist keine Schreibweise für
Auto.

Der MFC-Dialog 166 bindet Control 1283 direkt an denselben Wert. Die
Editorstruktur besitzt einen Header von `0x64` Bytes, also entspricht
Editoroffset `+0x131` exakt Parseroffset `+0xcd`. Der DDX-Block
`004b90a2` bis `004b912f` zeigt den Wert 2 als unbestimmten Zustand der
`BS_AUTO3STATE`-Checkbox an und schreibt die drei Zustände wieder in die beiden
niedrigen Bits.

`FUN_004d5be0` löst den Zustand in Fahrzeugbyte `+0xbc`, Bit 4 auf. Off und On
kopieren Bit 0 direkt. Bei Auto wird das Bit nur gesetzt, wenn der Kandidat
nicht das kontrollierte Spielerfahrzeug ist und seine Membership-Maske bei
`+0xc0` die Friend-Maske des Spielers bei `+0xc4` schneidet. Das Tag verändert
die Team-ID-Liste und die daraus erzeugten Team-Masken selbst nicht.

Das Laufzeitbit kennzeichnet aktive Lancemates für das Command-/Auswahlsystem.
Beispielsweise zählt `FUN_00468250` nur aktive Objekte mit diesem Bit und
`FUN_00467fa0` erzeugt daraus freundliche Lancemate-Einträge. Weitere
Kommandierungs- und Sortierpfade prüfen dasselbe Bit. Manche Verbraucher prüfen
die Freundbeziehung zusätzlich; ein erzwungenes On macht daher keinen Gegner
zum Freund.

In den extrahierten Daten setzen nur vier Records das Tag explizit, jeweils auf
`"off"`: `bushwacker_01` in `t1/readerm1`, `t1/readerm2` und `t1/readerm3`
sowie `bushwacker_02` in `t1/readerm4`. Diese Instruktor-Mechs und der jeweilige
Player verwenden Team 1. Das Off-Override verhindert somit, dass die ansonsten
automatisch freundlichen Instruktoren im Lancemate-/Command-System erscheinen.

## `p13` — 1 Feld

Tagstring `005e96d0`, Parserblock ab `004dff3b`.

| Index | Ziel |
|---:|---:|
| 0 | `+0xcc`, Bit 6 |

Der interne Editor liefert hierfür eine direkte Originalbeschriftung:

- Dialog 170 (`p5`), Control 1278: `Quit scan on trigger pull`
- DDX-Code: `004b9812` bis `004b984a`

### Laufzeitverbraucher

Das Bit besitzt im Programm genau einen direkten Leser:

```asm
004ca90f  TEST byte ptr [EAX+0xcc],40h
004ca916  JZ   004ca929
004ca918  MOV  CL,byte ptr [EAX+0xf8]
004ca91e  TEST CL,CL
004ca920  JNZ  004ca929
004ca922  MOV  dword ptr [ESI+0x18],0
```

Der Code liegt in `FUN_004ca600`, einem zeitlich begrenzten Ziel-/Lenkverhalten.
Dieses Verhalten sucht einen passenden Kontakt, speichert dessen Objektzeiger,
berechnet die Richtung zu ihm und gibt Lenk-/Geschwindigkeitskommandos aus.
Während es ein Ziel übernommen hat, setzt es den Controllerstatus `+0xf8` auf
1. Andere Controllerpfade löschen diesen Status wieder über `FUN_004e95a0`.

Wenn `p13[0] != 0` und Status `+0xf8 == 0`, setzt der obige Zweig die
Ablaufzeit des Verhaltens (`behavior+0x18`) auf 0. Beim nächsten Update wird
das Verhalten beendet. Der Laufzeitcode bestätigt damit die Wirkung der
Editoroption:

> Das betreffende Ziel-/Lenkverhalten automatisch beenden, sobald der
> Controller seinen übernommenen Zielstatus verliert/freigibt.

## `vlist` — 17 Felder pro Eintrag

Tagstring `005e9668`, Parserblock ab `004e00d7`.

Jeder Eintrag wird in eine 32 Byte große interne Struktur umgewandelt. Der Parser
initialisiert zunächst sieben DWORDs über `FUN_004d8d30`, setzt anschließend die
Felder und ergänzt als achtes DWORD die aufgelöste Referenz aus Index 15.

Defaultwerte der ersten sieben DWORDs bei `005e8da0`:

```text
+0x00 = 0.25f
+0x04 = 0.10f
+0x08 = 0.80f
+0x0c = 0.0f
+0x10 = 3600.0f
+0x14 = 0
+0x18 = 0 (zwei Flagbytes beginnen hier)
```

Feldzuordnung:

| JSON-Index | internes Ziel | Behandlung |
|---:|---:|---|
| 0 | `entry+0x18`, Flagbyte 0, Bit 1 | `value != 0` |
| 1 | `entry+0x18`, Flagbyte 0, Bit 2 | `value != 0` |
| 2 | `entry+0x00` | direkt; Default `0.25f` |
| 3 | `entry+0x18`, Flagbyte 0, Bit 3 | `value != 0` |
| 4 | `entry+0x18`, Flagbyte 0, Bit 4 | `value != 0` |
| 5 | `entry+0x04` | direkt; Default `0.10f` |
| 6 | `entry+0x18`, Flagbyte 0, Bit 5 | `value != 0` |
| 7 | `entry+0x08` | direkt; Default `0.80f` |
| 8 | `entry+0x19`, Flagbyte 1, Bit 0 | `value != 0` |
| 9 | `entry+0x14` | String/Referenz über `FUN_004c5c00` aufgelöst |
| 10 | `entry+0x18`, Flagbyte 0, Bit 6 | `value != 0` |
| 11 | `entry+0x0c` | direkt; Default `0.0f` |
| 12 | `entry+0x18`, Flagbyte 0, Bit 7 | `value != 0` |
| 13 | `entry+0x10` | direkt; Default `3600.0f` |
| 14 | `entry+0x18`, Flagbyte 0, Bit 0 | `value != 0` |
| 15 | `entry+0x1c` | String/Referenz über `FUN_004d3c40` aufgelöst |
| 16 | `entry+0x19`, Flagbyte 1, Bit 1 | `value != 0` |

In den Daten verweist Index 9 typischerweise auf einen `reg_*`-Namen. Index 15
enthält Namen wie `begin`, `radar`, `topspeed`, `inside_t1m1` oder geskriptete
AI-/Dialogereignisse. Diese Beobachtung ist sicher; ob Index 15 eine Aktion, einen
Zustand oder einen Eventkanal bezeichnet, muss noch über die Verbraucher geklärt
werden.

Die fertigen Einträge werden über `FUN_004d9480` in die Liste bei AI-Objekt
`+0xd0` eingefügt. Dieselbe 32-Byte-Struktur wird auch für `p4`-Einträge benutzt.

`FUN_004d9b70` ruft für alle Listeneinträge `FUN_004d8e40` auf. Dabei werden
`entry+0x0c` und `entry+0x10`, also die JSON-Indizes 11 und 13, quadriert. Beide
sind folglich Distanz-/Reichweitenwerte. Die inverse Routine `FUN_004d8e20`
zieht für eine Ausgabe-/Kopierdarstellung wieder die Quadratwurzel.

## Noch offen

Die belegten Originalbeschriftungen der vorhandenen Parametergruppen stehen in
`dialog-parameter-map.md`.

- Semantische Namen der meisten Offsets
- Datentypen aller unbenutzten Tags aus ZRD-Typcodes und Verbrauchern
- genaue `p4`-Unterstruktur
- Funktion des `sound_tag`-Records
- semantische Bedeutung der einzelnen `vlist`-Flags und Zahlenwerte
- Konstruktor-/Defaultwerte aus `FUN_004d9ac0` und `FUN_004d9b20`
