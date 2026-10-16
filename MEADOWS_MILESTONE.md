# Connected Meadows milestone - 16 October 2026

Double-click Open Playable World.cmd, then click the window. Rust uses private native reference caches and installed original assets; it runs without Unity or Steam. This remains a bounded reconstruction with custom gameplay.

## Area and provenance

Nine connected native HeightmapBuilder tiles cover 192 x 192 metres. Seed LabTerrain1 / 1121644293, generator version 2. Tile centres X/Z are {-128,-64,0}; original bounds are [-160,32] on both axes. All 38,025 tile biome samples report Meadows. The region was inspected before selection: 225 coarse checkpoints also report Meadows. Native heights span 27.5735378 to 82.5209045m.

Rust local X/Z = native world X/Z minus (-64,-64). Local Y = native height minus 57.71041m. There is no rescaling. Unity Z is reflected at the OpenGL view boundary. Rust stitches a single 193 x 193 grid, using the installed collision triangulation. Shared vertices, height lookup and normal sampling cross tile boundaries together. Movement, jumping, floors and save validation use the expanded bounds.

This is a private native-generated cache, NOT an independent world generator. worldgen_probe.rs remains separate: its verified hash and arithmetic still depend on native base heights/noise/rivers. Proprietary payloads/caches/evidence remain outside distributable source.

## Appearance and interactions

The terrain now reads the installed Heightmap material's _DiffuseArrayTex directly: 16 layers, 256x256 BC7, one mip, format108, and _UVScale=0.5. The layer contact sheet is evidence/meadows-001/terrain-layers.bmp. Layer0 was visually inspected and selected as a grass appearance approximation. Native biome/paintmask blending, displacement, normal mapping, lighting and colour-management parity are unsupported. Geometry is retained in an OpenGL display list; no rendering technology, engine or package was added.

The low western corner requires water at the inspected material level30m. Water is a flat opaque blue surface: waves, shore foam, refraction, swimming and buoyancy are absent.

Native environment root position/quaternion/scale are retained in a separate placement TSV; original prefab meshes/material bindings are independently read from the installation. Unsupported prefabs are omitted and counted, never replaced with arbitrary objects. Fixed LOD0 and approximate lighting omit native wind, particles and LOD fading. Selected rocks use conservative native collider AABBs; trees use scaled custom trunk radii. Exact native MeshCollider/PhysX collision and canopy view-blocking are not reproduced.

Loose Wood supplies near each tile centre are explicit custom gameplay fixtures, separate from native vegetation placement. Gathering, resource cost, floor validation/top collision and Greyling combat remain custom rules. Idle/walk/attack animation poses remain Unity-sampled.

F5 saves to private-cache/sandbox/meadows-patch.save; .bak retains its previous version. Earlier world.save is retained. F9 loads; closing does not autosave. Seed/version/centre/width bind the custom save to the patch; single-tile saves are rejected. Automated tests use separate saves.

## Verification and reproduction

All 38,025 native vertices match exactly; all 221,184 native indices match exactly. 900 PhysX raycasts differ by at most0.0000076293945m. 33,489 fractional stitched/tile heights differ by at most0.000015258789m. All12 neighbouring edges x65 samples match exactly; loader checks776 overlapping edge/corner entries. Twelve simulated walking/jumping/landing boundary crossings pass. A single shared grid prevents independent tile-edge shading normals.

Final input/persistence/traversal/geometry results are recorded below. Tests are AUTOMATED, not human playtesting. Timing records include glFinish; frame intervals include explicit10ms sleep, scheduling and captures. No comparison establishes a performance improvement over Unity.

Run Verify-Meadows.ps1 from built-in Windows PowerShell for a fresh evidence directory. Native regeneration uses Run-Terrain-Reference.ps1 and Run-Environment-Reference.ps1, exclusively through the isolated test copy with no competing Valheim session. The normal CMD launcher retains its absolute built-in Windows PowerShell path; pwsh.exe is not required.

## Retained attempts and limits

The Windows Store Python alias could not execute; bundled Python was used. A Rust parser return-type ambiguity was fixed. Restricted native launches failed Steam initialization. The inspection argument initially did not attach TerrainCheck; its boot hook was fixed and the inspection rerun. The first environment export stopped after85 partial rows due to a null LOD renderer; the directory and provisional tests remain but are excluded by the completed-cache gate. Native prefab normal conversion was corrected to the part-to-root inverse-transpose. One document write failed because the host's legacy text encoding did not support a multiplication sign; this report was rewritten explicitly through PowerShell.

Installed executable/gameplay assembly/bundle/SoftRef hashes matched their recorded baseline before reliance on earlier findings. The full995-file installation audit passed unchanged. No original executable, user save or GTA project was edited. Historical unexplained Steam-cloud differences are not restored or attributed to this milestone.

## Final delivery evidence

The connected milestone passed final integration verification in evidence/meadows-001/delivery-005, invoked through built-in Windows PowerShell. Terrain, gameplay scenario, separate-process restart, all nine tile fixtures, actual Win32 controls, rendered restart, seam traversal, all ten asset comparisons and the CMD desktop launcher completed. desktop-error.log is empty and desktop.bmp exists and was visually inspected. This is automated evidence, not a human playtest.

The checkpoint extraction had already completed successfully: private-cache/terrain-patches/native-20261016-144946-9906337. Its native process exited normally, all nine explicit generated=true records are present, and every placement cell has records. There are 1,018 unique placements and 1,845 collider records; Rust selects 977 supported placements and 197 rock AABB proxies. Omitted instances: Birch2=10, FirTree_oldLog=7, Oak1=1, RaspberryBush=18, stubbe=5. See environment-audit-001.txt. No additional native launch was needed. The incomplete-cache rejection remains active.

The refreshed asset reference private-cache/environment/native-20261016-145218-6077174 includes all ten supported assets. The final comparison includes Birch1 and Beech_small2: maximum position error 0.641 micrometres and normal error 0.030751 degrees. Matched-camera overview, Birch1 and small2 Rust captures are in delivery-004; native captures remain in their reference directories. Visual inspection confirms matching shapes and scene density, with visible differences in lighting, fog, foliage filtering, water and the absent distant world. No shader/image parity is claimed.

The original fixed traversal hit real obstacles in delivery-003 and is retained as failed. The test now selects a 96m square using a bounded normal-movement simulation against the loaded trees/rocks. It selected local start (-52,-56), then sent actual W/D/S/A, Shift and Q messages through the window without disabling collisions. Final 1,284 recorded frames span X=-52.54749..43.7029 and Z=-56.48509..39.41133, crossing both internal seams on both axes; F5/N/F9 also passed. Tree lookup indices are cached to avoid repeatedly scanning every placement during collision checks; collision geometry/rules are unchanged.

Final measured render + glFinish time after the first ten frames: mean 22.119ms, median 23.479ms, p95 31.479ms, maximum 50.383ms on Intel Iris Xe/OpenGL 4.6. Frame intervals also include sleep, scheduling and capture overhead. Native Unity was absent. These are one-run timings, not a performance improvement claim.

The initial delivery-004 launcher test exposed incorrect quoting of a spaced capture path and CMD pause masking failure. Both are fixed: the launcher preserves its child exit code; verification launches CMD with an explicit quoted command and requires the capture file. delivery-005 validates the complete corrected script under built-in Windows PowerShell. Failed logs remain. Final binary SHA256: 6E1668A55A82B51BDE67B8352238CC2FCF56B4526A500FDBC5CE4DEEE833F9FC. Source/build/cache manifests accompany delivery-005. The repeated original installation audit checked all 995 files unchanged (delivery-004/installation-preservation.json).

The milestone is complete within this bounded cache-based reconstruction. Remaining gaps include independent world generation and animation evaluation, full environment prefab support, exact native collision, terrain paintmask/shader/clutter/wind, water simulation and human control feedback. No original save/mod/server/multiplayer compatibility is established. No original game, user save or GTA project was edited.
