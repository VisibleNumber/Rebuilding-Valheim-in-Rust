# Playable Meadows build

Double-click **Open Playable World.cmd**, then click inside the world to play. This is the independent Rust prototype. Steam is only needed for optional native reference collection, not for playing it.

- Mouse: look. WASD: move. Shift: run. Q: jump.
- E: gather highlighted wood. Each pickup gives three wood.
- B: show the floor preview. Click or B again: place for two wood. V: cancel.
- Click or Space: strike while outside building mode. Space also works during building.
- Tab: pause and release/capture the mouse. Esc: release/pause, then Esc again to quit.
- F5: save. F9: load. N: fresh clearing. H: toggle help.

The connected Meadows patch covers 192 x 192 metres across nine stitched native tiles. All nine native environment zones completed generation: 1,018 placements were captured, of which 977 have supported independently decoded meshes and textures. The 41 omitted instances are Birch2 (10), FirTree_oldLog (7), Oak1 (1), RaspberryBush (18) and stubbe (5). Native tree/bush/rock transforms are retained; loose Wood supplies are custom fixtures. Rust reconstructs the terrain, skins and renders the Greyling, and runs the custom gameplay. Terrain generation and Idle/walk/attack poses still use private Unity-generated caches. Collision proxies, gathering, building and combat rules, lighting and terrain shading remain approximate. Gameplay refuses an incomplete environment cache.

The floor preview explains blocked placement. Placed floors sit above the sampled terrain and share their elevation with the walking surface. Approach the Greyling to engage it; its wind-up warning gives time to back away. This is a small clearing, not the complete Valheim world.

Saving is explicit: closing does not autosave. Your patch sandbox file is private-cache/sandbox/meadows-patch.save; its .bak retains the previous save. Earlier world.save is retained. Position, health, wood, depleted pickups, placed floors and enemy state persist. Starting fresh does not overwrite that file until you press F5. Save identity binds the file to this patch; older/single-tile saves are rejected without migration or deletion. These are custom sandbox files.

See MEADOWS_MILESTONE.md for current evidence and limitations. POLISH_REPORT.md, WORLD_PIPELINE.md and GREYLING_REPORT.md retain earlier investigation details. Automated controls, nine tile fixtures, seam traversal, save/restart, all-ten asset geometry comparisons and the desktop launcher are verified; human control comfort remains untested.

Build with Build-Playable.ps1. Diagnostics include --scenario <new save>, --verify-save <save>, --load-save <save> --capture <new BMP>, --playtest <new directory>, --asset-dump <new directory>, --asset-view <0 tree|1 wood|2 floor> --capture <new BMP>, and --dump-motion <walk|attack> <frame> <new CSV>. Use Compare-Environment.ps1 to compare asset dumps to a successful private native reference. Automated outputs are separate from the manual sandbox save.
