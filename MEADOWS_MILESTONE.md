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

Pending final integration verification. Final results will replace this paragraph before delivery.

Checkpoint: terrain verification and eight clean native asset geometry comparisons pass. The final native environment readiness-retry run is still in progress (native-20261016-144946-9906337). Final expanded-scene traversal, frame timings, successful launcher and all-ten-asset comparison remain pending. See the latest END of HANDOFF.md for exact continuation instructions and provisional/failed attempts. This milestone is not yet accepted as complete.
