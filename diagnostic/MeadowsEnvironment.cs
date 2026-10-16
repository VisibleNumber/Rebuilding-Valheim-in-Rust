using System;
using System.Globalization;
using System.IO;
using System.Text;
using System.Collections;
using UnityEngine;

namespace ValheimLab
{
    // Captures only objects already created by Valheim's native zone spawner.
    // This deliberately does not choose a prefab or synthesize a transform.
    public static class MeadowsEnvironment
    {
        static string F(float value) => value.ToString("R", CultureInfo.InvariantCulture);

        // Uses Valheim's installed ZoneSystem.SpawnZone, which in turn calls its
        // own PlaceLocations and PlaceVegetation methods and native Unity RNG.
        // The run is bounded to the 3x3 zones around center and preserves the
        // resulting live scene objects for capture.
        public static IEnumerator GenerateBounded(Vector3 center, float centerHeight, string outputPath, System.Action<int> completed)
        {
            Directory.CreateDirectory(outputPath);
            var status = new StringBuilder();
            var zoneSystem = ZoneSystem.instance;
            if (!zoneSystem)
            {
                File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), "ZoneSystem.instance is absent; native SpawnZone was not invoked.\n");
                completed?.Invoke(-1); yield break;
            }
            var getZone = typeof(ZoneSystem).GetMethod("GetZone", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
            var spawn = typeof(ZoneSystem).GetMethod("SpawnZone", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            var modeType = typeof(ZoneSystem).GetNestedType("SpawnMode", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic);
            if (getZone == null || spawn == null || modeType == null)
            {
                File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), "Installed ZoneSystem native generation entry points could not be resolved.\n");
                completed?.Invoke(-1); yield break;
            }
            var modeNames = Enum.GetNames(modeType);
            status.AppendLine("SpawnMode enum names: " + string.Join(",", modeNames));
            if (Array.IndexOf(modeNames, "Full") < 0)
            {
                File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), status.AppendLine("Native Full mode was not available; no zones were generated.").ToString());
                completed?.Invoke(-1); yield break;
            }
            object fullMode = Enum.Parse(modeType, "Full");
            if (Convert.ToInt32(fullMode, CultureInfo.InvariantCulture) != 0)
            {
                File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), status.AppendLine("Full mode did not resolve to IL-confirmed full-generation value 0; no zones were generated.").ToString());
                completed?.Invoke(-1); yield break;
            }
            var vegetationField = typeof(ZoneSystem).GetField("m_vegetation", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance);
            object vegetation = vegetationField == null ? null : vegetationField.GetValue(zoneSystem);
            status.AppendLine("Entry: installed ZoneSystem.SpawnZone with SpawnMode value 0 (native full generation).");
            status.AppendLine("SpawnZone invokes installed PlaceVegetation for each generated zone.");
            status.AppendLine("Vegetation definitions: " + (vegetation as System.Collections.ICollection)?.Count.ToString(CultureInfo.InvariantCulture));
            WriteVegetationDefinitions(vegetation, outputPath);
            WriteNativeZoneCoverage(zoneSystem, center, outputPath, status);
            var zonePrefab = (GameObject)typeof(ZoneSystem).GetField("m_zonePrefab", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic).GetValue(zoneSystem);
            var zoneHeightmap = zonePrefab ? zonePrefab.GetComponentInChildren<Heightmap>() : null;
            if (!zoneHeightmap) { File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), status.AppendLine("Native zone prefab has no Heightmap; no zones generated.").ToString()); completed?.Invoke(-1); yield break; }
            var isReady = typeof(HeightmapBuilder).GetMethod("IsTerrainReady", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic);
            if (isReady == null) { File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), status.AppendLine("Native HeightmapBuilder.IsTerrainReady could not be resolved.").ToString()); completed?.Invoke(-1); yield break; }
            int generated = 0;
            for (int dz = -1; dz <= 1; dz++) for (int dx = -1; dx <= 1; dx++)
            {
                Vector3 sample = new Vector3(center.x + dx * 64f, center.y, center.z + dz * 64f);
                object zone = getZone.Invoke(null, new object[] { sample });
                Vector3 zonePosition = (Vector3)typeof(ZoneSystem).GetMethod("GetZonePos", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static).Invoke(null, new object[] { zone });
                float deadline = Time.realtimeSinceStartup + 120f;
                int readinessChecks = 0;
                bool ready = false;
                while (Time.realtimeSinceStartup < deadline)
                {
                    ready = (bool)isReady.Invoke(HeightmapBuilder.instance, new object[] { zonePosition, zoneHeightmap.m_width, zoneHeightmap.m_scale, zoneHeightmap.IsDistantLod, WorldGenerator.instance });
                    readinessChecks++;
                    if (ready) break;
                    yield return null;
                }
                status.Append("zone ").Append(zone).Append(" terrainReady=").Append(ready ? "true" : "false").Append(" readinessChecks=").AppendLine(readinessChecks.ToString(CultureInfo.InvariantCulture));
                if (!ready) { status.Append("zone ").Append(zone).AppendLine(" skipped: native terrain did not become ready within 120 seconds"); continue; }
                int spawnAttempts = 0;
                bool ok = false;
                float spawnDeadline = Time.realtimeSinceStartup + 120f;
                do
                {
                    object[] args = { zone, fullMode, null };
                    ok = (bool)spawn.Invoke(zoneSystem, args);
                    spawnAttempts++;
                    if (ok) break;
                    // Native SpawnZone can also return false while its location
                    // spawn precondition is not yet satisfied. Retry next frame;
                    // successful zones are never invoked a second time.
                    yield return null;
                } while (Time.realtimeSinceStartup < spawnDeadline);
                status.Append("zone ").Append(zone).Append(" spawnAttempts=").Append(spawnAttempts.ToString(CultureInfo.InvariantCulture)).Append(" generated=").AppendLine(ok ? "true" : "false");
                if (ok) generated++; else status.Append("zone ").Append(zone).AppendLine(" skipped: native SpawnZone remained false for 120 seconds after terrain readiness");
                yield return null;
            }
            status.AppendLine("Native zones generated: " + generated.ToString(CultureInfo.InvariantCulture) + "/9");
            File.WriteAllText(Path.Combine(outputPath, "generation-status.txt"), status.ToString());
            CaptureExisting(center, centerHeight, 96f, outputPath);
            WriteCapturedZoneCoverage(outputPath);
            completed?.Invoke(generated);
        }

        static void WriteCapturedZoneCoverage(string outputPath)
        {
            string placementPath = Path.Combine(outputPath, "environment.tsv");
            string path = Path.Combine(outputPath, "native-placement-zone-coverage.tsv");
            var groups = new System.Collections.Generic.Dictionary<string, int>(StringComparer.Ordinal);
            var getZone = typeof(ZoneSystem).GetMethod("GetZone", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
            if (getZone != null && File.Exists(placementPath))
            {
                var lines = File.ReadAllLines(placementPath);
                for (int i = 1; i < lines.Length; i++)
                {
                    var f = lines[i].Split('\t');
                    if (f.Length < 4 || !float.TryParse(f[1], NumberStyles.Float, CultureInfo.InvariantCulture, out float x) || !float.TryParse(f[3], NumberStyles.Float, CultureInfo.InvariantCulture, out float z)) continue;
                    object zone = getZone.Invoke(null, new object[] { new Vector3(x, 0f, z) });
                    string key = Convert.ToString(zone, CultureInfo.InvariantCulture) ?? "<unknown>";
                    groups.TryGetValue(key, out int n); groups[key] = n + 1;
                }
            }
            using (var writer = new StreamWriter(path, false, new UTF8Encoding(false)))
            {
                writer.WriteLine("zone\tnative_placement_count");
                foreach (var entry in groups) writer.WriteLine(entry.Key + "\t" + entry.Value.ToString(CultureInfo.InvariantCulture));
                writer.WriteLine("covered_zones\t" + groups.Count.ToString(CultureInfo.InvariantCulture));
            }
        }

        // SpawnZone returns false for zones already present in its native zone
        // cache. Record that cache and live Heightmap coverage separately so a
        // false return is not confused with missing generated terrain.
        static void WriteNativeZoneCoverage(ZoneSystem zoneSystem, Vector3 center, string outputPath, StringBuilder status)
        {
            var coverage = new StringBuilder();
            coverage.AppendLine("source\tzone\tstate");
            foreach (var field in typeof(ZoneSystem).GetFields(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic))
            {
                if (!field.Name.ToLowerInvariant().Contains("zone")) continue;
                object value = null;
                try { value = field.GetValue(zoneSystem); } catch { }
                if (value is System.Collections.IDictionary map)
                {
                    status.AppendLine("Zone cache field " + field.Name + " entries=" + map.Count.ToString(CultureInfo.InvariantCulture));
                    foreach (System.Collections.DictionaryEntry entry in map) coverage.Append("cache\t").Append(entry.Key).Append('\t').AppendLine(entry.Value == null ? "null" : entry.Value.GetType().FullName);
                }
                else if (value is System.Collections.ICollection collection)
                    status.AppendLine("Zone collection field " + field.Name + " entries=" + collection.Count.ToString(CultureInfo.InvariantCulture));
                else if (value != null)
                    status.AppendLine("Zone-related field " + field.Name + " type=" + value.GetType().FullName);
            }
            int heightmaps = 0;
            foreach (var hm in Resources.FindObjectsOfTypeAll<Heightmap>())
            {
                if (!hm || !hm.gameObject.scene.IsValid()) continue;
                var p = hm.transform.position;
                if (Mathf.Abs(p.x - center.x) > 96f || Mathf.Abs(p.z - center.z) > 96f) continue;
                heightmaps++;
                object zone = typeof(ZoneSystem).GetMethod("GetZone", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)?.Invoke(null, new object[] { p });
                coverage.Append("heightmap\t").Append(zone ?? "<unknown>").Append("\t").Append(p.x.ToString("R", CultureInfo.InvariantCulture)).Append(',').Append(p.z.ToString("R", CultureInfo.InvariantCulture)).AppendLine();
            }
            status.AppendLine("Bounded native Heightmap components=" + heightmaps.ToString(CultureInfo.InvariantCulture));
            File.WriteAllText(Path.Combine(outputPath, "native-zone-coverage.tsv"), coverage.ToString());
        }

        static void WriteVegetationDefinitions(object vegetation, string outputPath)
        {
            using (var writer = new StreamWriter(Path.Combine(outputPath, "vegetation-definitions.tsv"), false, new UTF8Encoding(false)))
            {
                writer.WriteLine("index\tprefab\tconfiguration");
                if (vegetation is not System.Collections.IEnumerable items) return;
                int index = 0;
                foreach (object item in items)
                {
                    if (item == null) continue;
                    var entries = new System.Collections.Generic.List<string>();
                    foreach (var field in item.GetType().GetFields(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic))
                    {
                        object value = field.GetValue(item);
                        if (value is GameObject go) value = go ? go.name : "<null>";
                        else if (value is UnityEngine.Object unity) value = unity ? unity.name : "<null>";
                        else value = Convert.ToString(value, CultureInfo.InvariantCulture) ?? "<null>";
                        entries.Add(field.Name + "=" + value.ToString().Replace('\t', ' ').Replace('\r', ' ').Replace('\n', ' '));
                    }
                    var prefabField = item.GetType().GetField("m_prefab", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic);
                    var prefab = prefabField?.GetValue(item) as GameObject;
                    writer.Write(index++.ToString(CultureInfo.InvariantCulture)); writer.Write('\t'); writer.Write(prefab ? prefab.name : "<unknown>"); writer.Write('\t'); writer.WriteLine(string.Join(";", entries));
                }
            }
        }

        public static int CaptureExisting(Vector3 center, float centerHeight, float halfExtent, string outputPath)
        {
            if (string.IsNullOrWhiteSpace(outputPath)) throw new ArgumentException("Output path is required", nameof(outputPath));
            if (halfExtent <= 0 || float.IsNaN(halfExtent) || float.IsInfinity(halfExtent)) throw new ArgumentOutOfRangeException(nameof(halfExtent));
            Directory.CreateDirectory(outputPath);
            string path = Path.Combine(outputPath, "environment.tsv");
            int count = 0;
            var seen = new System.Collections.Generic.HashSet<int>();
            var meshSeen = new System.Collections.Generic.HashSet<int>();
            var assetDumped = new System.Collections.Generic.HashSet<string>(StringComparer.Ordinal);
            string assetPath = Path.Combine(outputPath, "environment-assets.tsv");
            string meshPath = Path.Combine(outputPath, "environment-meshes.tsv");
            string colliderPath = Path.Combine(outputPath, "environment-colliders.tsv");
            using (var assets = new StreamWriter(assetPath, false, new UTF8Encoding(false)))
            using (var meshes = new StreamWriter(meshPath, false, new UTF8Encoding(false)))
            using (var colliders = new StreamWriter(colliderPath, false, new UTF8Encoding(false)))
            using (var writer = new StreamWriter(path, false, new UTF8Encoding(false)))
            {
                writer.WriteLine("prefab\tworld_x\tworld_y\tworld_z\tyaw\tqx\tqy\tqz\tqw\tscale_x\tscale_y\tscale_z\tlocal_x\tlocal_y\tlocal_z\tsource_component");
                assets.WriteLine("prefab\tpart_path\tcomponent\tdetail");
                meshes.WriteLine("prefab\tpart_path\tmesh\tvertex\tx\ty\tz");
                colliders.WriteLine("prefab\tpart_path\ttype\tlayer_id\tlayer_name\tworld_center_x\tworld_center_y\tworld_center_z\tsize_x\tsize_y\tsize_z\tis_trigger\tenabled\tshape_detail");
                foreach (var component in Resources.FindObjectsOfTypeAll<Component>())
                {
                    if (!component || !component.gameObject.scene.IsValid()) continue;
                    string kind = component.GetType().Name;
                    if (kind != "TreeBase" && kind != "Destructible") continue;
                    Transform part = component.transform;
                    Vector3 world = part.position;
                    if (Mathf.Abs(world.x - center.x) > halfExtent || Mathf.Abs(world.z - center.z) > halfExtent) continue;
                    Transform root = part;
                    for (Transform t = part; t != null; t = t.parent)
                    {
                        root = t;
                        if (t.name.EndsWith("(Clone)", StringComparison.Ordinal)) break;
                    }
                    if (!seen.Add(root.GetInstanceID())) continue;
                    Vector3 position = root.position;
                    Vector3 scale = root.lossyScale;
                    float yaw = root.eulerAngles.y;
                    Quaternion rotation = root.rotation;
                    writer.Write(root.name); writer.Write('\t'); writer.Write(F(position.x)); writer.Write('\t'); writer.Write(F(position.y)); writer.Write('\t'); writer.Write(F(position.z)); writer.Write('\t');
                    writer.Write(F(yaw)); writer.Write('\t'); writer.Write(F(rotation.x)); writer.Write('\t'); writer.Write(F(rotation.y)); writer.Write('\t'); writer.Write(F(rotation.z)); writer.Write('\t'); writer.Write(F(rotation.w)); writer.Write('\t');
                    writer.Write(F(scale.x)); writer.Write('\t'); writer.Write(F(scale.y)); writer.Write('\t'); writer.Write(F(scale.z)); writer.Write('\t');
                    writer.Write(F(position.x - center.x)); writer.Write('\t'); writer.Write(F(position.y - centerHeight)); writer.Write('\t'); writer.Write(F(position.z - center.z)); writer.Write('\t'); writer.WriteLine(kind);
                    WriteAssetStructure(root, assets, meshes, colliders, meshSeen);
                    string prefabName = PrefabName(root.name);
                    if (assetDumped.Add(prefabName)) WriteNativeGeometry(root, prefabName, outputPath);
                    count++;
                }
            }
            File.WriteAllText(Path.Combine(outputPath, "environment-capture.txt"),
                "Source: already-instantiated Unity scene objects only.\n" +
                "Bounds: center=" + F(center.x) + "," + F(center.z) + " halfExtent=" + F(halfExtent) + " centerHeight=" + F(centerHeight) + "\n" +
                "Accepted components: TreeBase, Destructible.\n" +
                "Rows: " + count + "\n" +
                (count == 0 ? "No native tree or destructible instances were present in the bounded scene at capture time.\n" : "Positions, yaw and lossy scale are the original Unity instance transforms.\n"));
            return count;
        }

        static string RelativePath(Transform transform, Transform root)
        {
            if (transform == root) return transform.name;
            var names = new System.Collections.Generic.Stack<string>();
            for (var t = transform; t != null && t != root; t = t.parent) names.Push(t.name);
            names.Push(root.name);
            return string.Join("/", names);
        }

        static string PartPath(Transform transform, Transform root)
        {
            if (transform == root) return string.Empty;
            var names = new System.Collections.Generic.Stack<string>();
            for (var t = transform; t != null && t != root; t = t.parent) names.Push(t.name);
            return string.Join("/", names);
        }

        static string PrefabName(string instanceName)
        {
            const string clone = "(Clone)";
            return instanceName.EndsWith(clone, StringComparison.Ordinal) ? instanceName.Substring(0, instanceName.Length - clone.Length).TrimEnd() : instanceName;
        }

        static void WriteNativeGeometry(Transform root, string prefabName, string outputPath)
        {
            string path = Path.Combine(outputPath, prefabName + ".csv");
            var selected = new System.Collections.Generic.HashSet<int>();
            var excluded = new System.Collections.Generic.HashSet<int>();
            foreach (var group in root.GetComponentsInChildren<LODGroup>(true))
            {
                if (!group) continue;
                var lods = group.GetLODs();
                for (int i = 0; lods != null && i < lods.Length; i++) foreach (var r in lods[i].renderers ?? Array.Empty<Renderer>()) if (r) (i == 0 ? selected : excluded).Add(r.GetInstanceID());
            }
            using (var csv = new StreamWriter(path, false, new UTF8Encoding(false)))
            {
                csv.WriteLine("part,index,x,y,z,nx,ny,nz");
                foreach (var renderer in root.GetComponentsInChildren<MeshRenderer>(true))
                {
                    if (!renderer) continue;
                    bool isLod0 = selected.Contains(renderer.GetInstanceID());
                    if (!renderer.gameObject.activeInHierarchy || !renderer.enabled && !isLod0 || excluded.Contains(renderer.GetInstanceID()) && !isLod0) continue;
                    var filter = renderer.GetComponent<MeshFilter>();
                    if (!filter || !filter.sharedMesh) continue;
                    var mesh = filter.sharedMesh;
                    var vertices = mesh.vertices;
                    var normals = mesh.normals;
                    for (int submesh = 0; submesh < mesh.subMeshCount; submesh++) for (int i = 0; i < vertices.Length; i++)
                    {
                        Vector3 p = root.InverseTransformPoint(renderer.transform.TransformPoint(vertices[i]));
                        Vector3 n = normals.Length == vertices.Length ? normals[i] : Vector3.zero;
                        n = root.localToWorldMatrix.transpose.MultiplyVector(renderer.transform.worldToLocalMatrix.transpose.MultiplyVector(n)).normalized;
                        csv.Write(PartPath(renderer.transform, root)); csv.Write('#'); csv.Write(submesh.ToString(CultureInfo.InvariantCulture)); csv.Write(','); csv.Write(i.ToString(CultureInfo.InvariantCulture)); csv.Write(',');
                        csv.Write(F(p.x)); csv.Write(','); csv.Write(F(p.y)); csv.Write(','); csv.Write(F(p.z)); csv.Write(','); csv.Write(F(n.x)); csv.Write(','); csv.Write(F(n.y)); csv.Write(','); csv.WriteLine(F(n.z));
                    }
                }
            }
        }

        static void WriteAssetStructure(Transform root, StreamWriter writer, StreamWriter meshWriter, StreamWriter colliderWriter, System.Collections.Generic.HashSet<int> meshSeen)
        {
            var renderers = root.GetComponentsInChildren<Renderer>(true);
            var lodMembership = new System.Collections.Generic.Dictionary<int, string>();
            foreach (var group in root.GetComponentsInChildren<LODGroup>(true))
            {
                if (!group) continue;
                var lods = group.GetLODs();
                for (int i = 0; lods != null && i < lods.Length; i++)
                    foreach (var renderer in lods[i].renderers ?? Array.Empty<Renderer>())
                    {
                        if (!renderer) continue;
                        string entry = "LODGroup=" + RelativePath(group.transform, root) + ";level=" + i.ToString(CultureInfo.InvariantCulture) + ";screenHeight=" + F(lods[i].screenRelativeTransitionHeight);
                        if (lodMembership.TryGetValue(renderer.GetInstanceID(), out string prior)) entry = prior + "," + entry;
                        lodMembership[renderer.GetInstanceID()] = entry;
                    }
                Row(writer, root.name, RelativePath(group.transform, root), "LODGroup", "levels=" + (lods == null ? 0 : lods.Length).ToString(CultureInfo.InvariantCulture) + ";enabled=" + group.enabled);
            }
            foreach (var renderer in renderers)
            {
                if (!renderer) continue;
                var sharedMaterials = renderer.sharedMaterials ?? Array.Empty<Material>();
                string materials = string.Join("|", Array.ConvertAll(sharedMaterials, m => m ? m.name + " [" + (m.shader ? m.shader.name : "<shader-null>") + "]" : "<null>"));
                string detail = "enabled=" + renderer.enabled + ";materials=" + materials;
                if (lodMembership.TryGetValue(renderer.GetInstanceID(), out string lod)) detail += ";" + lod;
                Mesh mesh = null;
                var filter = renderer.GetComponent<MeshFilter>();
                if (filter) mesh = filter.sharedMesh;
                var skinned = renderer as SkinnedMeshRenderer;
                if (skinned) mesh = skinned.sharedMesh;
                if (mesh)
                {
                    detail += ";mesh=" + mesh.name + ";vertices=" + mesh.vertexCount.ToString(CultureInfo.InvariantCulture) + ";submeshes=" + mesh.subMeshCount.ToString(CultureInfo.InvariantCulture);
                    if (meshSeen.Add(mesh.GetInstanceID()))
                    {
                        var vertices = mesh.vertices;
                        for (int i = 0; i < vertices.Length; i++)
                        {
                            Vector3 v = root.InverseTransformPoint(renderer.transform.TransformPoint(vertices[i]));
                            meshWriter.Write(root.name); meshWriter.Write('\t'); meshWriter.Write(RelativePath(renderer.transform, root)); meshWriter.Write('\t'); meshWriter.Write(mesh.name); meshWriter.Write('\t');
                            meshWriter.Write(i.ToString(CultureInfo.InvariantCulture)); meshWriter.Write('\t'); meshWriter.Write(F(v.x)); meshWriter.Write('\t'); meshWriter.Write(F(v.y)); meshWriter.Write('\t'); meshWriter.WriteLine(F(v.z));
                        }
                    }
                }
                Row(writer, root.name, RelativePath(renderer.transform, root), renderer.GetType().Name, detail);
            }
            foreach (var collider in root.GetComponentsInChildren<Collider>(true))
            {
                if (!collider) continue;
                string layerName = LayerMask.LayerToName(collider.gameObject.layer);
                string detail = "enabled=" + collider.enabled + ";trigger=" + collider.isTrigger + ";layer=" + collider.gameObject.layer.ToString(CultureInfo.InvariantCulture) + ":" + layerName + ";worldBounds=" + F(collider.bounds.size.x) + "," + F(collider.bounds.size.y) + "," + F(collider.bounds.size.z);
                if (collider is MeshCollider meshCollider && meshCollider.sharedMesh)
                    detail += ";mesh=" + meshCollider.sharedMesh.name + ";vertices=" + meshCollider.sharedMesh.vertexCount.ToString(CultureInfo.InvariantCulture) + ";convex=" + meshCollider.convex;
                if (collider is BoxCollider box) detail += ";center=" + F(box.center.x) + "," + F(box.center.y) + "," + F(box.center.z) + ";size=" + F(box.size.x) + "," + F(box.size.y) + "," + F(box.size.z);
                if (collider is SphereCollider sphere) detail += ";center=" + F(sphere.center.x) + "," + F(sphere.center.y) + "," + F(sphere.center.z) + ";radius=" + F(sphere.radius);
                if (collider is CapsuleCollider capsule) detail += ";center=" + F(capsule.center.x) + "," + F(capsule.center.y) + "," + F(capsule.center.z) + ";radius=" + F(capsule.radius) + ";height=" + F(capsule.height) + ";direction=" + capsule.direction;
                Row(writer, root.name, RelativePath(collider.transform, root), collider.GetType().Name, detail);
                Bounds bounds = collider.bounds;
                colliderWriter.Write(root.name); colliderWriter.Write('\t'); colliderWriter.Write(RelativePath(collider.transform, root)); colliderWriter.Write('\t'); colliderWriter.Write(collider.GetType().Name); colliderWriter.Write('\t');
                colliderWriter.Write(collider.gameObject.layer.ToString(CultureInfo.InvariantCulture)); colliderWriter.Write('\t'); colliderWriter.Write(layerName.Replace('\t', ' ')); colliderWriter.Write('\t');
                colliderWriter.Write(F(bounds.center.x)); colliderWriter.Write('\t'); colliderWriter.Write(F(bounds.center.y)); colliderWriter.Write('\t'); colliderWriter.Write(F(bounds.center.z)); colliderWriter.Write('\t');
                colliderWriter.Write(F(bounds.size.x)); colliderWriter.Write('\t'); colliderWriter.Write(F(bounds.size.y)); colliderWriter.Write('\t'); colliderWriter.Write(F(bounds.size.z)); colliderWriter.Write('\t');
                colliderWriter.Write(collider.isTrigger); colliderWriter.Write('\t'); colliderWriter.Write(collider.enabled); colliderWriter.Write('\t');
                if (collider is MeshCollider nativeMesh && nativeMesh.sharedMesh) colliderWriter.Write("mesh=" + nativeMesh.sharedMesh.name + ";convex=" + nativeMesh.convex);
                else if (collider is BoxCollider nativeBox) colliderWriter.Write("localCenter=" + F(nativeBox.center.x) + "," + F(nativeBox.center.y) + "," + F(nativeBox.center.z) + ";localSize=" + F(nativeBox.size.x) + "," + F(nativeBox.size.y) + "," + F(nativeBox.size.z));
                else if (collider is SphereCollider nativeSphere) colliderWriter.Write("localCenter=" + F(nativeSphere.center.x) + "," + F(nativeSphere.center.y) + "," + F(nativeSphere.center.z) + ";radius=" + F(nativeSphere.radius));
                else if (collider is CapsuleCollider nativeCapsule) colliderWriter.Write("localCenter=" + F(nativeCapsule.center.x) + "," + F(nativeCapsule.center.y) + "," + F(nativeCapsule.center.z) + ";radius=" + F(nativeCapsule.radius) + ";height=" + F(nativeCapsule.height) + ";direction=" + nativeCapsule.direction);
                colliderWriter.WriteLine();
            }
        }

        static void Row(StreamWriter writer, string prefab, string path, string component, string detail)
        {
            writer.Write(prefab.Replace('\t', ' ')); writer.Write('\t'); writer.Write(path.Replace('\t', ' ')); writer.Write('\t'); writer.Write(component); writer.Write('\t');
            writer.WriteLine(detail.Replace('\t', ' ').Replace('\r', ' ').Replace('\n', ' '));
        }
    }
}
