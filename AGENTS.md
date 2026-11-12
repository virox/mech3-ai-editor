# AGENTS.md

## Project

Mech Reader Studio: a lossless Windows editor for MechWarrior 3 Reader archives
(`.zbd` containing `.zrd`), built with Tauri 2, Svelte 5 and TypeScript.

- `src/`: Svelte/TS frontend, with Vitest tests next to the code (`*.test.ts`).
- `src-tauri/`: the Rust backend. Archive I/O goes through the mech3ax crates,
  which are path dependencies on `C:/Dev/mech3ax`; that checkout must exist.
- `switchboard/`, `switchboard-dev/`: separate companion apps, each with its
  own `package.json` and Tauri crate. They attach to a running `Mech3.exe 1.2.22`.
- Layout developer build: `layout.html`, `vite.layout.config.ts` and
  `src-tauri/tauri.layout.conf.json`. It edits `src/ai-layout.json`.
- `docs/`, `RESEARCH_GAPS.md`, `IMPLEMENTATION_PLAN.md`: reverse-engineering
  evidence and design. `README.md` is the entry point.

Before editing, read the relevant code, nearby tests and applicable docs. Follow
existing patterns; avoid unrelated refactors. Treat archive-format assertions in
code and documentation as hypotheses until backed by the evidence rules below.

## Commands

The commands run in PowerShell from the repo root unless noted.

```powershell
npm install
npm run tauri dev            # run the editor
npm test                     # Vitest
npm run check                # svelte-check / TypeScript
npm run build                # Vite production build
$env:MW3_TEST_ROOT='C:\Games\MechWarrior_3_(USA)'
$env:MW3_TEST_READER="$env:MW3_TEST_ROOT\zbd\c1\readerm1.zbd"
cd src-tauri; cargo test     # Rust tests
# Additional Rust hygiene checks from src-tauri:
cargo fmt --all -- --check
cargo clippy --all-targets
```

Companion apps: `cd switchboard; npm run build` (or `tauri:dev`, `panel:tauri:dev`)
and `cd switchboard-dev; npm run build` (or `npm run portable`).

## Definition of done

- Run `npm test`, `npm run check` and `cargo test` (with both `MW3_TEST_*`
  variables set) after every change, and all three must pass.
- Run `npm run build` when changing the editor frontend/build configuration.
  For Rust changes, also run `cargo fmt --all -- --check` and
  `cargo clippy --all-targets`; report existing warnings separately.
- If you change `switchboard/` or `switchboard-dev/`, run `npm run build` in
  that folder as well.
- Without `MW3_TEST_ROOT`/`MW3_TEST_READER`, the game-data Rust tests return
  early and pass silently. Don't treat that as a pass.
- Report commands run, pass/fail results, and skipped tests with reasons. If the
  Windows game checkout or mech3ax dependency is unavailable, do not claim
  verification succeeded or silently weaken a test.

## Workflow

- Commit only when asked. Commit directly to `master` and never push.
- Commit subject: an imperative summary, prefixed with the area when one
  applies (`Behavior graph: …`). For a non-trivial change, put the details in a
  short body or a bullet list.
- Releases (`Release vX.Y.Z: …`) happen only on request. The editor,
  switchboard and switchboard-dev have separate version numbers; bump only
  the app(s) being released.
  - Editor: `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`,
    `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and
    `src-tauri/tauri.layout.conf.json` (the layout build follows the editor).
  - Switchboard: `switchboard/src-tauri/Cargo.toml`, `Cargo.lock`,
    `tauri.conf.json`, `switchboard/package-lock.json` (its package.json has
    no version).
  - Tag the release commit `vX.Y.Z` (editor version) locally; don't push it.
- Update `README.md` and the matching `docs/` file in the same commit when
  you change behavior or commands, or confirm a reverse-engineering finding.
- Write docs in English. When editing a German doc (e.g. `docs/README.md`),
  translate the part you touch.
- For a format or behavior discovery, record the executable/version, evidence
  location, observed facts, and remaining uncertainty; update `RESEARCH_GAPS.md`
  instead of presenting an inference as confirmed.

## Boundaries

- Never write to the game install (`C:\Games\...`). The editor's export goes to
  a separate output directory by design.
- Don't commit extracted game data. The gitignored paths under
  `docs/aiv-research/` are not redistributable. Use small synthetic fixtures
  for committed regression tests instead.
- Don't edit `dist/`, `release/`, `src-tauri/target/` or `src-tauri/gen/`.
  They are build output.
- Ask first before adding or upgrading dependencies, changing the
  `C:/Dev/mech3ax` path deps, or deleting docs or research data.
- Do not broaden Tauri capabilities, filesystem scopes, shell/process access,
  or remote-content privileges without a concrete requirement and review.

## Archive correctness and safe I/O

- Round-trips must be lossless. Preserve record order, duplicate keys
  (duplicate valve event/action pairs are meaningful), unknown tags and raw
  variants. Unknown values stay in their original JSON-shaped records.
- Give a field a semantic name or label only when `RESEARCH_GAPS.md`'s evidence
  rules are met: the parser destination is joined to an original editor or
  runtime consumer. Otherwise keep the ZRD index (`p8[10]`, `record[3]`) and
  write down the gap.
- Do not normalize, sort, deduplicate, or reconstruct opaque data merely to
  make it easier to display or edit. Distinguish semantic round-trip equality
  from byte-for-byte identity (e.g. when compression or padding can change).
  Prove preservation of untouched records and document any expected byte changes.
- Graph edits apply to existing native records atomically. Never invent or
  recreate missing records. Validate a complete edit before applying it; if an
  edit fails, leave the in-memory document unchanged.
- Treat `.zbd`/`.zrd` inputs as untrusted. Check headers, offsets, record sizes,
  index bounds and arithmetic overflow before allocation or slicing. Bound
  decompression, entry counts and total memory/disk usage. Reject malformed
  inputs with actionable errors instead of panics or partial edits.
- Validate archive entry names before any extraction or export: reject absolute
  paths, `..` traversal, unsafe links and collisions that escape or overwrite
  unintended destinations. Do not infer filesystem paths from archive names.
- Keep archive parsing, validation and serialization in the Rust/mech3ax layer;
  the Svelte UI should issue typed operations rather than rewrite archive bytes.
- Export only to an explicitly selected destination outside the game install.
  Write a temporary file in that destination, check completion and re-read or
  validate it before publishing the final file. Preserve the previous output on
  failure or cancellation; never report success on a partial write. Do not
  overwrite an existing output without explicit user intent.
- A known-unresolved reference (e.g. `reg_inside_c4m7`) is reported to the
  user. Never silently "fix" it.

## Tauri, UI and switchboard

- Keep a narrow Tauri IPC surface. Validate arguments and allowed paths on the
  Rust side; frontend checks are for usability, not a security boundary. Grant
  each window only necessary Tauri 2 capabilities and plugin scopes.
- Keep long-running parse/export operations off the UI thread, provide progress
  and errors, and prevent overlapping writes to the same document. Clean up
  event listeners and temporary resources on cancellation/unmount.
- In Svelte 5, follow the existing component style. Prefer `$state` for local
  state and pure `$derived` values for computed state; use `$effect` for external
  side effects, not to synchronize derived state. Keep the binary document as
  the source of truth rather than inventing frontend-only archive records.
- The switchboard must never inject code into the game. It writes only when
  the user enables writes, and every write is revalidated and read back.
  Verify the attached executable's expected build/layout before accessing
  addresses; on mismatch, detach or remain read-only. Check target address,
  expected value, size and current process state immediately before writes.

## Regression tests

- For every parser/writer change, add or update tests for no-edit round-trips,
  edited round-trips and preservation of untouched/unknown records. Include
  duplicate keys, ordering, unresolved references and boundary values when
  relevant. Use byte comparisons only where byte identity is a valid contract.
- Test malformed/truncated input, invalid offsets/lengths, oversized payloads,
  and failed/cancelled export paths. Use bounded synthetic hostile fixtures,
  not resource-exhaustion tests on a developer machine.
- For Svelte/IPC changes, cover successful and failed commands, UI state and
  cleanup. Use Vitest and Tauri's `mockIPC` where useful; mocks do not replace
  the real Rust tests or an actual Windows smoke test when behavior depends on
  the OS, WebView, or a running game process.
- Test switchboard write protection (disabled/mismatched build/stale value),
  revalidation and readback whenever its process interaction changes.

## Gotchas

- Addresses like `FUN_004d3c40` in docs refer to `Mech3.exe` 1.2.22 from the
  game install at `C:\Games\MechWarrior_3_(USA)`, as named by Ghidra.
- The editor's Vite watcher ignores `src-tauri`, `release`, `switchboard*` and
  `docs`. It uses port 1420 and the layout build uses 1421, both with `strictPort`.

## Upstream references

Use these when changing the corresponding subsystem; repository evidence and
project-specific constraints above remain authoritative for game formats.

- Tauri security/capabilities: https://v2.tauri.app/security/
- Tauri commands and channels: https://v2.tauri.app/develop/calling-rust/
- Tauri frontend testing/mocks: https://v2.tauri.app/develop/tests/mocking/
- Svelte 5 reactivity: https://svelte.dev/docs/svelte/$derived and
  https://svelte.dev/docs/svelte/$effect
- Vitest mocking: https://vitest.dev/guide/mocking
- Rust tests: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- Archive threat model: https://owasp.org/www-project-web-security-testing-guide/
