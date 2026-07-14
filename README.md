# MechWarrior 3 AI Editor

A Windows-first, file-based editor for MechWarrior 3 `aiv.zrd` data. It opens
standalone ZRD/JSON files and Reader ZBD archives, exposes the recovered AI and
VList fields, keeps unknown data intact, and always writes archives with Save As.

## Development

Requirements: Node 20+, Rust 1.88+, and the local mech3ax checkout at
`C:/Dev/TerranMechworks/mech3ax`.

```powershell
npm install
npm run tauri dev
```

Checks:

```powershell
npm run build
cd src-tauri
cargo test
```

The editor deliberately does not attach to the running game. Unknown fields are
shown in a read-only raw-data view and survive round trips through the editor.

