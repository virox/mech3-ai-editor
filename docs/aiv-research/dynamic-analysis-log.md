# Dynamische AIV-Analyse mit x32dbg

## Sitzung vom 14. Juli 2026

### Parser-Eintritt bestätigt

Breakpoint: `Mech3.exe+0x0df730` beziehungsweise absolute Adresse `0x004df730`.
Die EXE war bei Basis `0x00400000` geladen.

Aufrufkonvention am Eintritt von `FUN_004df730`:

| Ort | Bedeutung | erster beobachteter Wert |
|---|---|---:|
| `ECX` | Zielstruktur | `0x001afc00` |
| `[ESP+4]` | geparster ZRD-Record | `0x1d42c138` |
| `[ESP+8]` | Recordname (`char *`) | `0x005e1718` = `player` |

Der erste Treffer war der `player`-Record. Seine Position nach dem Parsen war
`(263.5, 2.2, 1884.9)`, seine Rotation intern ungefähr `-0.610865` Radiant.
Damit wurde als geladene Quelle eindeutig bestimmt:

`extracted-reader-json/zbd/c4/readeria1/aiv.zrd.json`

### `vlist` zur Laufzeit

Der Player-Record erzeugte 18 Listeneinträge bei AI-Objekt `+0xd0`.

Erster Eintrag aus der JSON-Datei:

```json
[1,0,0.25,0,0,0.1,0,0.8,1,"reg_p1",0,0.0,0,60.0,1,"begin",0]
```

Erster Laufzeiteintrag, Nutzlast nach den beiden Listenzeigern:

```text
+0x00  0.25
+0x04  0.10
+0x08  0.80
+0x0c  0.0
+0x10  3600.0 (= 60²)
+0x14  0x1d5b1060  (aufgelöstes `reg_p1`)
+0x18  0x00000103  (Flagbyte 0 = 0x03, Flagbyte 1 = 0x01)
+0x1c  0x1d576600  (aufgelöstes `begin`)
```

Damit ist die statische Feldkarte aus `parameter-field-map.md` auch dynamisch
bytegenau bestätigt.

### Nicht-Player-Record

Der nächste Parser-Treffer war `Dominic_Paine`. Seine Rotation nach dem Parsen
war ebenfalls `-35°`, womit `readeria1` gegenüber `readeria2` eindeutig bestätigt
wurde. Alle beobachteten `params`-/`p8`-Werte entsprachen der JSON-Datei.

### Übergang in den Laufzeit-Controller

Breakpoint `0x004d9940` (`FUN_004d9940`) bestätigte den Config-Copy:

- `ECX`: Zieladresse des Laufzeit-Controllers
- `[ESP+4]`: Quelle der geparsten AIV-Konfiguration
- die 0x33 DWORDs ab Config-Offset `+0x04` werden kopiert
- die Liste bei `+0xd0` und die Referenzen bei `+0xd4`/`+0xd8` werden separat kopiert

Der beobachtete Controller gehörte anhand seiner Position
`(-23.1, -3.1, 2503.5)` eindeutig zu `Enemy_07`:

```text
Controllerbasis: 0x1d553d3c
params[0]:       0x1d553d58  (Controller + 0x1c)
```

Auf `0x1d553d58` wurde über den x32dbg-Befehl
`bphws 1D553D58, r, 4` ein Hardware-Lese-Watchpoint gesetzt. Parser- und
temporäre Rücksprung-Breakpoints wurden anschließend entfernt. Der Prozess wurde
fortgesetzt; die MCP-Brücke lief während des wartenden Prozesses in ein Timeout.
Der Hardware-Watchpoint sollte im Debugger weiterhin aktiv sein.

### Falscher Treffer beim Missionsende

Während des Kampfes und bis zur Anzeige `Mission Complete` trat kein Halt auf.
Erst beim Verlassen der Mission stoppte x32dbg mit `EIP = 0x00577c11`.
Die unmittelbar zuvor ausgeführte Instruktion war:

```asm
00577c0e  MOV word ptr [EDI],DX
00577c11  MOVSX EDX,word ptr [ESI + 0x4]
```

Dabei war `EDI = 0x1d553d58`, also exakt die überwachte Adresse von
`Enemy_07.params[0]`. Das ist ein **Schreibzugriff** in einer generischen
16-Bit-Kopierschleife und kein AI-Lesezugriff. Der AI-Controller war beim
Missionsabbau offenbar bereits freigegeben und seine Heap-Adresse wurde
wiederverwendet. Der Treffer darf daher nicht zur Semantik von `params[0]`
gezählt werden. Außerdem hat `bphws 1D553D58, r, 4` in dieser Sitzung einen
Schreibzugriff gemeldet; die gewünschte Zugriffsbeschränkung muss vor dem
nächsten Versuch in x32dbg überprüft werden.

## Nächster dynamischer Schritt

Im nächsten Missionslauf:

1. Einen echten Hardware-Breakpoint vom Typ **read/write** auf ein Feld setzen,
   das im Kampf häufig benutzt wird, oder die statisch ermittelten Consumer-
   Instruktionen direkt mit Ausführungs-Breakpoints versehen.
2. Zugriffsart in der Breakpoint-Ansicht kontrollieren.
3. Beim Halt `EIP`, vorherige Instruktion und vollständige Register sichern.
4. umgebende Funktion in Ghidra dekompilieren.
5. den Watchpoint vor Missionsende entfernen, damit Heap-Reuse keine
   irreführenden Treffer erzeugt.

### Nächster Code-Breakpoint

Wegen der instabilen x32dbg-MCP-Brücke wurde zunächst ein normaler
Ausführungs-Breakpoint verwendet. `0x004da0d0` wurde anfangs irrtümlich als
virtuelle Methode des AIV-Laufzeitcontrollers eingeordnet:

```c
FUN_004da900(..., FUN_004da060());
controller->field_1c = controller->field_110;
```

Der Breakpoint traf beim Missionsstart ein Objekt mit VTable `0x005cd3a0`.
Das ist eine benachbarte, andere Klasse und **nicht** der finale
Mech-AI-Controller (dessen Konstruktor setzt VTable `0x005cc698`). Der
Breakpoint wurde deshalb wieder entfernt.

### Zweiter Lauf: `readerm1`, `owens_01`

Ein normaler Code-Breakpoint auf `0x004d9940` traf zuverlässig jeden
AIV-Controller beim Struktur-Copy. Drei Treffer für `mobilehq_01` bis `_03`
wurden durchlaufen; der vierte Treffer wurde anhand der Position
`(-1767.7, 3.0, -1229.0)` als `owens_01` identifiziert.

```text
Quellstruktur:       0x1621fdc4
Runtime-Controller: 0x1953ebfc
params[0]:           0x1953ec18 (Controller + 0x1c)
Rohwert:             0x49742400 = 1,000,000.0 = 1000²
```

Die echte Assembly von `FUN_004d9940` zeigt:

```asm
LEA ESI,[EBP+0x4]   ; source + 4
LEA EDI,[EBX+0x4]   ; target + 4
MOV ECX,0x33
REP MOVSD
```

Damit bleiben die Feldoffsets beim Copy entgegen einer kurzzeitigen
Decompiler-Fehlinterpretation unverändert. `params[0]` liegt auch im
Runtime-Controller bei `+0x1c`; die ursprüngliche Offsetkarte war korrekt.
Nach Rückkehr aus dem Copy wurde auf `0x1953ec18` ein Hardware-Read/Write-
Watchpoint gesetzt und die Mission fortgesetzt.
