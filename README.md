# Valheim Rust Lab

An experimental Rust reconstruction built from investigation of a locally installed Valheim copy. The aim is to make a small, playable world using verified original game data, while documenting which parts are native-derived and which are custom approximations.

This is a work in progress, not a full remake. The current playable prototype has a connected 3×3 Meadows terrain patch, selected original trees and rocks, movement, gathering, one placeable floor, a simple Greyling encounter, and private save/load. Terrain and placements come from native-generated reference data stored in private local caches that are not included here. Full-playable milestone verification is still in progress; check [MEADOWS_MILESTONE.md](MEADOWS_MILESTONE.md) for status before treating it as complete.

## Try the playable prototype

This project currently targets Windows and reads assets from a Valheim installation owned by the person running it. Place this checkout at `steamapps/valheim-lab`, beside `steamapps/common/Valheim`, then double-click `Open Playable World.cmd`. The launcher builds the Rust program when needed and checks the installed game identity before opening it.

Controls: click to capture the mouse; WASD to move; Shift to run; Q to jump; E to gather; B to preview/place a floor; Space to attack; F5 to save; F9 to load; Tab to pause and release the mouse. See [PLAYABLE_WORLD.md](PLAYABLE_WORLD.md) for current details.

You need a compatible installed Valheim version and the Rust compiler (`rustc`) on PATH. The diagnostic and native-reference tools also require the .NET 10 SDK and the installed game's managed assemblies. Do not redistribute game files, generated caches, extracted asset payloads, saves, or compiled binaries.

## What is implemented

- Rust readers for selected Unity asset data and original Greyling, terrain and environment assets.
- A connected terrain patch reconstructed from private native reference output.
- A small Rust gameplay loop for walking, gathering, building, basic combat and save/load.
- Investigation tools and reports that distinguish measured results, native-derived inputs and custom behavior.

The terrain generator is not yet independent. Water, terrain shading, environment placement, combat, AI, collision and animation still contain approximations. Original save, mod and multiplayer compatibility are not established. Rust is not claimed to improve performance or AI automatically.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first. Useful work includes reproducible asset decoding, terrain verification, small playable improvements, and tests or captures that compare results against the installed game. Please keep proprietary game data and personal save data out of commits.

## Project layout

- `playable-world/` — Rust sandbox, rendering, terrain and asset import code.
- `greyling-viewer/` — independent original Greyling viewer.
- `asset-probe/` — Unity bundle inspection and decoding source.
- `diagnostic/` — managed runtime reference tools, built against the installed game.
- `inspector/` and `patcher/` — managed assembly inspection and isolated test-copy support.
- `*.md` reports — current scope, findings, approximations and milestone status.

## License

Project source is licensed under the MIT License; see [LICENSE](LICENSE). Valheim and Unity are trademarks and copyrighted works of their respective owners. This repository does not include their assets or grant rights to them.

