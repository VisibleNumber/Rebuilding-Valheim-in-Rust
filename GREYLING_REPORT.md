# Independent Greyling build — 16 October 2026

**Open `Open Greyling Viewer.cmd`.** The original Greyling is now rendered independently in Rust from the installed game assets. Drag to orbit, wheel to zoom, R to reset, K to show its skeleton, U for unlit display and Space to switch between the serialized rest pose and cached Idle playback. This is an interactive asset viewer, not a playable Valheim world. Movement, combat, terrain and networking are not implemented in this executable.

The source installation still matches the original 995-file baseline. The launcher verifies the executable, gameplay assembly, main asset bundle and SoftRef manifest hashes before using this build profile. No packages were downloaded or installed. Rendering uses the existing Windows/OpenGL driver; the tested viewer used Intel Iris Xe, OpenGL 4.6. The native comparison used the isolated Unity test copy. No user game session was stopped and no world was opened for reference sampling.

**What is independently decoded**

The expanded Rust probe reads UnityFS 8 blocks on demand, including LZ4/LZ4HC decompression, serialized-file version 22 metadata, type trees, 64-bit object IDs, object fields, container entries and external-file references. It does not unpack the entire roughly 1 GB uncompressed main bundle. Only selected ranges are decompressed.

The Greyling's geometry, transforms, bind poses, skin weights, materials and BC7 texture bytes are read directly from the user's installation. They are not Unity-exported meshes or replacement assets. Rust computes linear blend skinning. The graphics driver decodes the original BC7 texture through OpenGL BPTC support. The viewer uses its top mip with the observed point-filter setting. Materials use original texture bindings, colors, emission and recorded HSV values, with a custom studio shader. Normal-map data is located but not used by that shader. Original Unity shader lighting, environmental effects, bloom, color management and PBR are not reproduced exactly.

**Asset-reference map**

| Link | Verified identity |
| --- | --- |
| SoftRef asset ID | `55150b54f4c96a662b4eb69f518ae0d4` |
| Prefab | `Assets/Characters/GreyDwarf/Greyling.prefab` |
| Bundle | `c4210710`, SHA256 `2FDCF6840C7A5EA64A3F19B63BB3047D5884C202BA67B7AFC997491F635A1B55` |
| Serialized file | `CAB-8923bd833c4171316cf3c761b642c1c8` |
| Prefab GameObject | path ID `1067265541275873022` |
| Root Transform | path ID `8935594583187484414` |
| Animator | path ID `220421819973725950` |
| Controller / Avatar | `3662712536530021454` (`greydwarf_greyling_animator`) / `-6089095708002280584` (`greydwarfAvatar`) |
| Body mesh | `3637422062829312888`, `Cube`, 962 vertices, 1,176 triangles |
| Eye meshes | `-4433815203273499784` and `6419565910550190968`, four vertices and two triangles each |
| Inactive alternate mesh | `-8487013848631731336`, `Cube.001`, 105 vertices; preserved as inactive |
| Body material / shader | `1025539852073521102`, `greyling` / local `-9132650593587925148`, `Custom/Creature` |
| Eye material / shader | `-3096363250494587055`, `eye_yellow` / external file 3, path ID `-8014961710185377972`, `Standard` in bundle `2c2cce25` |
| Diffuse texture | `-3511417599261641069`, `greydrawrf_diffuse`, 128×128, BC7 format 25, eight mips, sRGB setting |
| Normal texture | `3127667231069444859`, `greydrawrf_diffuse_nrm`, 128×128, BC7 format 25, eight mips, linear setting |
| Texture stream | Same CAB's `.resS`; diffuse offset 87,607,456 and normal offset 171,060,176, each 21,872 bytes |
| Idle clip | `-3652922707874447851`, `Idle`, four seconds, sample rate 30 |

The main serialized file contains 363 type definitions, 372,063 objects and nine external-file entries. Type trees are present. Full renderer and 33-bone maps are in `evidence/greyling-stage-001/renderer-map.csv` and `bone-map.csv`. `external-bundle-map.csv` resolves CAB names to installed bundle names. The graph traversal recorded 137 referenced objects, and retained decoded local JSON for visual objects, meshes, materials, Avatar, controller and ten animation clips under `objects-004`. Components outside this visual traversal are identified but not all semantically decoded.

The main mesh is uncompressed and readable. It uses UInt16 triangle indices, float32 position/normal/tangent data in a 40-byte stream, UV0 in an eight-byte stream, and weights/UInt32 joint indices in a 32-byte stream. Bind poses and 33 bone references are decoded. The eyes instead use one joint index with an implicit unit weight. This distinction was checked against Unity rather than treating every mesh as four-weight skinning. Prefab spawn position/rotation are normalized in both reference and viewer; child transforms, including the effective 40× body scale, are preserved.

**Animation boundary**

Clip fields, bindings and the Avatar/controller are parsed into inspectable data, including the humanoid muscle representation. Independent evaluation of that representation and Avatar retargeting is not implemented. For one working animation demonstration, the native helper sampled the original Idle clip through Unity's `AnimationClipPlayable` and Avatar, retaining 121 world-matrix poses at 30 Hz over four seconds. Rust loads that explicitly private pose cache and performs the mesh skinning and rendering itself. Playback uses sampled frames without interpolation. The viewer labels this origin on screen. It is not evidence of an independent Unity animation decoder or a rebuilt Animator state machine.

**Verification**

Native reference images use the original meshes/materials/shaders in a controlled scene, with the same 1024×768 camera geometry as the Rust captures. Visual inspection confirmed shape, orientation, texture layout and poses. Lighting and emission differ visibly, so shader parity is not claimed.

Native `BakeMesh(false)` output, converted through renderer translation/rotation without applying its scale again, was compared vertex-by-vertex against Rust. All 970 active vertices were matched by part and index:

| Pose | Maximum position difference | Maximum normal-angle difference |
| --- | ---: | ---: |
| Serialized rest | 0.488 micrometres | 0.032° |
| Idle sample 0 | 0.440 micrometres | 0.032° |
| Idle sample 30 | 0.610 micrometres | 0.035° |
| Idle sample 60 | 0.453 micrometres | 0.033° |

The animation comparisons validate Rust skinning against native output using native-sampled bone poses; they do not validate independent clip decoding. The detailed comparison is under `comparison-20261016-135655-0947`. Thresholds were 10 micrometres and 0.1°; none were relaxed.

Actual Win32 message-loop/render tests exercised drag orbit, wheel zoom, skeleton/unlit/Idle toggles and reset, retaining four GPU captures in `controls-001`. Seven decoder tests passed, including malformed/truncated data, signed object IDs and the typeless-data regression. Builds and launcher checks are logged in this evidence directory.

Failures were retained: the first traversal treated a controller's embedded field as a pointer; the first mesh decoder mishandled typeless vertex data and then the eyes' implicit weight; the first native animation-state attempt returned no clip; the first reference vertex conversion applied scale twice; synchronous native animation captures initially showed stale GPU poses. These were corrected through field checks, proper binary-array parsing, observed skin layout, explicit Playable sampling, measured coordinate conversion and per-frame native updates. Earlier sources and outputs remain in backups and dated private caches.

**Reproduce and continue**

`Build-Greyling.ps1` builds the reader, graph tool, viewer and decoder tests with the installed Rust 1.99.0 compiler and no crates. `Open-Greyling.ps1 -CheckOnly` verifies the installed profile and loads the real model. `Compare-Greyling.ps1` repeats the four native/Rust vertex comparisons. `Run-Greyling-Reference.ps1` runs only the isolated game copy and refuses an existing Valheim session; it requires the built/deployed native helper and normal Steam initialization. It opens no world. .NET SDK 10.0.401 compiles that helper against installed Mono/Unity assemblies.

Metadata inspection can be reproduced with `asset-probe --inspect-bundle <installed bundle> <fresh output folder>`. Object inspection is `asset-probe --object <bundle> <metadata folder> <path ID> <new JSON file>`. `inspect-greyling` traverses the verified visual hierarchy. The profile intentionally targets this exact build and rejects changed source hashes through the launcher.

Private metadata, native poses, references, object dumps and any derived asset payload stay under ignored `private-cache` or `evidence`. The distributable source/binary contains no embedded Greyling assets. A source-and-tool manifest records the build identities; deploy/rebuild backups are retained. Keep those private directories out of public packages.

The next reverse-engineering work is independent humanoid clip/Avatar evaluation and a closer reproduction of `Custom/Creature` shading. Generalizing to a second creature would then test whether this profile-specific reader handles another rig and asset layout. No terrain, combat, AI or networking rebuild is implied by this viewer.

Format/API references: [AssetStudio serialized-file layout](https://github.com/Perfare/AssetStudio/blob/master/AssetStudio/SerializedFile.cs), [common type-tree strings](https://github.com/Perfare/AssetStudio/blob/master/AssetStudio/CommonString.cs), [mesh layout](https://github.com/Perfare/AssetStudio/blob/master/AssetStudio/Classes/Mesh.cs), and [Unity 6 BakeMesh API](https://docs.unity3d.com/6000.0/Documentation/ScriptReference/SkinnedMeshRenderer.BakeMesh.html). Installed type trees and native comparisons determine the claims above.

Delivery check: root GreylingViewer.exe passed --check and matches the compiled binary (SHA256 CE143DB89D96654386AB7732275B117B28A64F493C18A158FC786B1E2C5AE484). All 995 original installation files still match baseline. Final save comparison found five changed Steam-cloud paths (link character/current backup, Valhalla world pair, remotecache.vdf); cause is not established. Local save paths match. Do not claim all saves unchanged or restore these files: they may contain newer user progress. Exact paths retained in evidence/greyling-stage-001/preservation-final.json. No original save writes were intentionally performed by this investigation.
