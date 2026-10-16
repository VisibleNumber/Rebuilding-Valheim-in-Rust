# Meadows improvement pass — 16 October 2026

The small Rust clearing now uses the original Beech tree, loose Wood item and wooden floor. It has free mouse-look, directional gathering prompts, clearer health/combat feedback, floor previews and an enemy that uses original walk/attack poses. Launch **Open Playable World.cmd**. This remains a custom gameplay prototype, not a port of Valheim's gameplay or world generator.

## What changed

The baseline capture `evidence/polish-001/before.bmp` showed block trees, checkerboard ground, block resources/floors and a large diagnostic HUD. `after.bmp` uses the same spawn camera, projection and terrain tile. The replacements are original installed assets, not newly drawn lookalikes. The terrain's legacy grass texture replaces checkerboard colouring. Lighting and distance fog are custom approximations. The HUD now concentrates on health, wood, interaction and controls; implementation details are here rather than on the gameplay screen.

Click the world to capture the mouse. Mouse movement looks around continuously; Tab or Esc releases it and pauses simulation. Focus loss also releases capture and pauses. A second Esc closes the window. WASD moves, Shift runs and Q jumps. Arrows remain available for looking. H hides/shows help.

Nearby wood is selected by a proximity/facing cone, highlighted and labelled **E Pick up wood (+3)**. This is custom pickup logic, not a native raycast/inventory system. Logs are aligned to the local terrain surface. B enters building mode; B or a click places a floor. V cancels. The outline is green when valid and red with a reason when blocked. Placement rejects occupied cells, insufficient wood, steep ground, the clearing boundary and nearby trunks. A floor's top is placed above the maximum of nine terrain grid samples; the visible mesh and walking surface use the same elevation. The placement and top-surface collision remain custom, not native support/stability simulation.

The Greyling has a health bar and an attack wind-up warning. Player swings have a centre-screen flash; incoming damage flashes the screen edge. The enemy approaches using the sampled original **Dwarf Walk** clip and attacks using **Zombie Attack**, which is the clip actually referenced by the installed Greyling controller. Contact damage occurs after a custom half-second wind-up, allowing retreat. Animation sampling and custom damage timing are not Valheim combat parity. The enemy starts farther away so the player can learn gathering/building first.

## Independently decoded assets

All three prefabs resolve into bundle `c4210710` and serialized file `CAB-8923bd833c4171316cf3c761b642c1c8`. The existing serialized-v22/type-tree reader is reused. `playable-world/environment.rs` adds static MeshFilter/MeshRenderer traversal, parent transforms, active flags, first-LOD selection, multiple submeshes/materials, inverse-transpose normal transforms, vertex streams, index slices, UV scale/offset and alpha-cutout properties.

| Asset | Prefab / mesh path IDs | Verified visible mesh |
| --- | --- | --- |
| Beech1 | prefab `-3545795938103434052`; mesh `5636204498416708811` | LOD0, 561 unique vertices, 432 triangles, separate leaf/bark materials |
| Wood | prefab `4592554523128777832`; mesh `5560736173076121522` | 90 vertices, 98 triangles |
| wood_floor | prefab `2057548747335490449`; mesh `8980046444124454289` | active New combined mesh, 330 vertices, 220 triangles |

Prefab paths are respectively `Assets/world/Props/Beech/Beech1.prefab`, `Assets/GameElements/Items/materials/Wood.prefab`, and `Assets/GameElements/Pieces/wood_floor.prefab`. Full active-part/material IDs, texture dimensions/formats and counts are in `evidence/polish-001/delivery-assets/reference-map.tsv`. Root spawn positions/rotations are normalized; child transforms are preserved. Inactive worn/broken floor variants and lower tree/floor LODs are excluded. Trees use fixed LOD0 in this small clearing; LOD distance switching and fade are not implemented.

Original leaf/bark textures are BC7, 256/512 pixels; wood/floor use DXT1, 64/128 pixels. The renderer uploads their installed compressed bytes and the GPU decodes them. Leaf cutout uses the observed 0.5 cutoff with two-sided rendering. Native shaders are Custom/Vegetation for the tree, Standard for Wood and Custom/Piece for the floor. Rust approximates their lighting; it does not reproduce vegetation sway, wind, moss, normal mapping, weather, PBR or the complete original shaders.

The static loader intentionally supports the verified same-file, uncompressed Float32 mesh layouts in this slice. External meshes, compressed vertex layouts, arbitrary shader features and general asset compatibility are not claimed. The ignored lower floor LOD includes a built-in Unity cube reference, which is not independently decoded.

## Terrain and animation: native-derived versus independent

The real 64-metre terrain tile still comes from Unity HeightmapBuilder through the private TER1 cache. Rust reconstructs geometry and evaluates its triangle walking surface. That generator has not been ported.

Inspection of installed `Heightmap_basematerial` found `_UVScale=0.5`, a legacy `_MainTex` binding to `Assets/world/terrain/old/grass.png`, and a 16-layer 256×256 diffuse texture array. Installed enum metadata verifies array format 108 is RGBA_BC7_SRGB; the array has one mip and a 1,048,576-byte streamed payload. The current renderer uses the verified legacy grass binding and UV scale as a partial approximation. It does not claim to reproduce the native array layer selection, biome/mask blending, displacement or terrain shader. This distinction matters: finding a material property is not proof that native rendering uses it in the same way.

The original Greyling's mesh, textures and skinning remain independently decoded. Idle, Dwarf Walk (1.1 seconds, 35 sampled frames) and Zombie Attack (about 1.967 seconds, 61 frames) are sampled through Unity AnimationClipPlayable and the original Avatar. Rust maps their matrices to the decoded skeleton and skins/renders them. Independent humanoid clip decoding/retargeting, motion blending and precise foot-speed synchronization remain undone.

Current successful native caches:

- Environment: `private-cache/environment/native-20261016-133557-0455479`.
- Greyling motions: `private-cache/greyling/native-20261016-133706-8350682`.
- Terrain: unchanged from WORLD_PIPELINE.md, selected by its private pointer file.

Native helpers run only in the isolated copy, with save/cloud routing guards. Mesh/texture bytes in the Rust environment importer come directly from the installed bundle, not native mesh exports. Native exports serve as comparison references. Private caches and proprietary evidence are excluded from distributable sources.

## What was actually verified

Native environment references export active LOD0 transformed vertices/normals, materials and actual-shader captures. Rust positions match within 0.641 micrometres for the tree, 0.102 for wood and 0.051 for the floor. Maximum normal difference is below 0.030 degrees. The tree comparison repeats its 561 vertices for each of its two submeshes, so its CSV contains 1,122 entries. This is a transformed-vertex/normal comparison, not a byte-for-byte native index-buffer comparison.

Native and Rust captures for all three assets were visually inspected with matching framing. Wood/floor shape and orientation match; Rust lighting is darker. The tree silhouette and cutout are recognizable and consistent with the source mesh, but the native vegetation shader deforms leaves and changes their lighting; exact pixel parity is not claimed. Rust comparison images are `asset-0.bmp` through `asset-2.bmp`; native images are in the successful private environment cache.

Original walk/attack skinning was compared against native BakeMesh at frame 17/30 on all 970 visible Greyling vertices. Maximum differences were 0.686/0.532 micrometres. This validates Rust skinning of sampled poses, not independent animation evaluation.

`window-walk-final` exercises actual Win32 input and GPU rendering: W movement, E pickup, B preview/placement, F5 save, N new, F9 reload, walk rendering during pursuit, three Space strikes, enemy wind-up/damage/death, another save/new/reload, drag camera input, reset and Tab capture/release state. Test-only enemy repositioning provides a bounded combat fixture. Captures at frames 70/105 retain walk/attack states. Hidden tests deliberately do not warp the desktop cursor; actual relative cursor polling and comfort still need human playtesting.

The simulation scenario also verifies jumping/landing, standing on the placed floor, duplicate placement rejection without resource loss, attack wind-up, resource cost, combat and persistence. A second process verifies the completed save, and a fresh rendered process opens it through `--load-save`; `reopened-world.bmp` and `rendered-restart.log` retain that result. The Windows desktop CMD launcher was tested through built-in Windows PowerShell and produced `desktop-launch.bmp`. The normal launcher requires no pwsh PATH entry.

These are automated behavioral and visual checks, not a claim of human playtesting. There is no tested multiplayer, original-save/mod compatibility, or performance improvement over Unity.

## Retained failures and remaining gaps

The first native Wood reference was empty because disabling a particle object's GameObject also disabled the wood root. The helper now stops particle emission and disables only the particle renderer; the corrected export has all 90 vertices. The earlier directory is retained. Initial static traversal hit a lower-LOD built-in cube before checking LOD selection; traversal now selects supported active LOD0 renderers before resolving mesh payloads. An initial 0.85-metre slope limit blocked the starting floor; the final limit is 1.2 metres with a common elevated surface. One floor test checked height while the character was still airborne; it now waits for landing before asserting the walking surface. Failed scopes, native attempts and captures remain under private evidence/backups.

Trees and resource positions are custom layout. Trunk collision is a simple custom radius, floors have no native side collider or stability model, combat has no original attack hitbox/resistance rules, and there is no player body/weapon animation. The map boundary is still a small bounded tile. Grass clutter, water, weather, native environmental lighting, biome blending, generated locations and independent animation/generator reconstruction remain work.

Save version 3 binds the existing terrain profile but rejects older sandbox formats because floor elevation/interaction rules changed. Older saves are neither migrated nor deleted. F5 keeps a previous save as .bak; all automated tests use their own paths.

Build with Build-Playable.ps1. Use Run-Environment-Reference.ps1, Run-Greyling-Reference.ps1, Compare-Environment.ps1 and the executable's documented diagnostic modes to reproduce the slice. Source/build/game/cache identities are recorded under `evidence/polish-001`. No packages were installed; the diagnostic only added a reference to the existing installed Unity particle module. Next prioritize native terrain texture-array/mask rendering and grass, then independent animation decoding, with the current playable loop retained as the regression gate.
