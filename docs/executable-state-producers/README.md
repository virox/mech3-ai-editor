# Executable state-channel producers

`producers.json` contains only executable-owned state producers that have a static channel name and a traceable write through the shared state writer. These are immutable graph inputs: Reader Studio can display them but cannot edit them through an archive.

## 2026-07-22 audit

The initial forward audit followed every direct reference to the shared writer `FUN_004D3C00` and the resolver `FUN_004D3C40`. That was supplemented with the inverse pass proposed during review: 710 distinct typed channel names were extracted from 437 `aiv.zrd`, `targets.zrd`, `valves.zrd`, `net_*.zrd`, `objectives.zrd`, and `mpinfo.zrd` documents, then compared case-sensitively with strings in the analysed `Mech3.exe`.

Five exact static-name matches were found: `mcbdeploy`, `start`, `base`, `radar`, and `firefly`.

- `mcbdeploy` is confirmed: it has static references to the resolver and direct writes through `FUN_004D3C00`.
- `start` and `firefly` have no static code references in this executable.
- `base` is referenced by mission configuration parsing, not the state resolver/writer.
- `radar` is a parser/configuration literal in the target-loading code, not the state resolver/writer.

The other regex hits in the bulk scan were substrings of longer executable strings and were excluded after exact C-string validation.

## Scope and limits

This proves only statically named, direct-writer paths in the analysed executable. It does not exclude names assembled dynamically, channels supplied by Reader data, indirect calls not represented as direct Ghidra references, or behavior in another executable build. `sha256` is `null` because the Ghidra bridge provides the analysed program but not its source file for hashing.

For a new confirmed producer, add one object to `producers.json` using the existing field names. Do not add candidate names merely because they exist both in ZRD data and in the executable; require a resolver and writer trace.
