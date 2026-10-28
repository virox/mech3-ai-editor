# Third-party notices

## mech3ax

Mech Reader Studio includes the following crates of
[mech3ax](https://github.com/TerranMechworks/mech3ax), statically linked into
the executable: `mech3ax-reader`, `mech3ax-common`, `mech3ax-archive`,
`mech3ax-api-types` and `mech3ax-anim`.

- Copyright: Toby Fleming and the mech3ax contributors
- Licence: European Union Public Licence v. 1.2 (EUPL-1.2), see `LICENSE`
- Version: tag `v0.7.0-rc3`, commit `cbb838fd8b628417abe61dee6132c350a2375f33`,
  used without modification
- Source: https://github.com/TerranMechworks/mech3ax/tree/v0.7.0-rc3

The Valve Switchboard does not use mech3ax.

Other Rust and npm dependencies keep their own licences; see `Cargo.lock`,
`package-lock.json` and the respective package metadata.

## Game data

Neither application contains MechWarrior 3 game data. Both read a game
installation that the user provides.
