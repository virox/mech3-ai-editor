# Mech AI Editor implementation plan

## Implemented foundation

- Windows-first Tauri 2 application with Svelte 5 and TypeScript.
- Direct loading and Save As for extracted JSON and standalone ZRD files.
- Direct loading and safe reconstruction of Reader ZBD archives via mech3ax.
- Ordered vehicle list with duplicate, create, rename, delete, and reorder actions.
- Recovered field groups: `params`, `p2`, `p6`, `p8`, `p9`, `p10`, `p11`, `p12`, and `p13`.
- Full 17-value VList condition editor.
- Optional groups have an explicit presence switch.
- `valves.zrd` and `net_*.zrd` are loaded as read-only linked symbol sources.
- Unknown tags remain in the raw vehicle record and are visible read-only.
- ZBD output is always selected through Save As; the source archive is not overwritten automatically.

## Verification gates

- Svelte/TypeScript static check.
- Vite production build.
- Rust unit test for duplicate vehicle names and record order.
- Real `readerm1.zbd` load/write/reload roundtrip with identical `aiv.zrd` JSON.
- Windows release executable and NSIS installer build.

## Next research increments

1. Recover the exact enum captions for `p2[0]` and display the original conditional subdialog for each strategy.
2. Resolve `p8[10]`, `p9[0..2]`, and `p10[1]` through dialog cross-references and runtime traces before naming them.
3. Map all `net_*.zrd` forms and add navigation from VList region/event references.
4. Add typed editors for `valves.zrd` actions while retaining duplicate event keys and order.
5. Add field-level validation ranges only where the executable or original dialog resources prove them.

The editor intentionally uses factual numeric labels for unresolved fields. It does not guess original semantics.
