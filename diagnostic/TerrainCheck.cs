using System;
using System.IO;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.Reflection;
using UnityEngine;

namespace ValheimLab {
// Native Unity reference only; this does not read or open a user world/save.
public sealed class TerrainCheck : MonoBehaviour {
 const BindingFlags Flags=BindingFlags.Instance|BindingFlags.Public|BindingFlags.NonPublic;
 const int Width=64, Side=65;
 string output;
 static string F(float v){return v.ToString("R",CultureInfo.InvariantCulture);}
 void Start(){
  output=Path.Combine(Probe.LabRoot,"private-cache","terrain",Probe.HasArg("-lab-terrain-inspect")?"pending-inspection":"pending-patch");
  StartCoroutine(Guard(Run()));
 }
 IEnumerator Guard(IEnumerator e){while(true){bool more=false;object current=null;Exception error=null;try{more=e.MoveNext();if(more)current=e.Current;}catch(Exception x){error=x;}if(error!=null){try{if(!String.IsNullOrEmpty(output)&&Directory.Exists(output))File.WriteAllText(Path.Combine(output,"FAILED.txt"),error.ToString());}catch{}Debug.LogError(error);Application.Quit(2);yield break;}if(!more)yield break;yield return current;}}

 IEnumerator Run(){
  yield return new WaitForSecondsRealtime(8);
  if(FileHelpers.CloudStorageSupportedAndEnabled)throw new Exception("Cloud isolation failed");
  if(Probe.HasArg("-lab-terrain-inspect")){InspectRegion();yield break;}
  string stamp=DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff",CultureInfo.InvariantCulture);
  string root=Path.Combine(Probe.LabRoot,"private-cache","terrain");Directory.CreateDirectory(root);
  string patchRoot=Path.Combine(Probe.LabRoot,"private-cache","terrain-patches");Directory.CreateDirectory(patchRoot);
  output=Path.Combine(patchRoot,"native-"+stamp);Directory.CreateDirectory(output);
  var world=new World("LabTerrainPatch"+stamp,"LabTerrain1");WorldGenerator.Initialize(world);var gen=WorldGenerator.instance;
  string prior=File.ReadAllText(Path.Combine(root,"native-current.txt")).Trim();
  int seed,version;float sharedOrigin,oldX,oldZ;
  using(var r=new BinaryReader(File.OpenRead(Path.Combine(prior,"tile.bin")))){
   if(System.Text.Encoding.ASCII.GetString(r.ReadBytes(4))!="TER1")throw new Exception("Existing terrain reference is not TER1");
   if(r.ReadInt32()!=Width)throw new Exception("Existing reference width changed");
   if(r.ReadSingle()!=1f)throw new Exception("Existing reference spacing changed");
   oldX=r.ReadSingle();oldZ=r.ReadSingle();sharedOrigin=r.ReadSingle();seed=r.ReadInt32();version=r.ReadInt32();
  }
  if(oldX!=-64f||oldZ!=-64f||seed!=world.m_seed||version!=world.m_worldGenVersion)throw new Exception("Existing native center identity does not match requested LabTerrain1 gen2 patch");
  if(gen.GetBiome(new Vector3(-64,0,-64))!=Heightmap.Biome.Meadows)throw new Exception("Existing center is no longer Meadows");

  var tiles=new Dictionary<string,List<float>>();var tileRoots=new Dictionary<string,string>();
  using(var manifest=new StreamWriter(Path.Combine(output,"patch.tsv"),false,new System.Text.UTF8Encoding(false)))
  using(var bounds=new StreamWriter(Path.Combine(output,"tile-bounds.tsv"),false,new System.Text.UTF8Encoding(false)))
  using(var biomeOut=new StreamWriter(Path.Combine(output,"tile-biomes.tsv"),false,new System.Text.UTF8Encoding(false)))
  using(var seams=new StreamWriter(Path.Combine(output,"seams.tsv"),false,new System.Text.UTF8Encoding(false))){
   manifest.WriteLine("file\tcenter_x\tcenter_z");
   bounds.WriteLine("center_x\tcenter_z\tmin_x\tmin_z\tmax_x\tmax_z\tmin_height\tmin_x_at\tmin_z_at\tmax_height\tmax_x_at\tmax_z_at\tcenter_height\tshared_origin");
   biomeOut.WriteLine("center_x\tcenter_z\tbiome\tvertex_count");
   seams.WriteLine("tile_a_x\ttile_a_z\ttile_b_x\ttile_b_z\taxis\tsamples\tmax_abs_height_delta\texact");
   for(int tz=-1;tz<=1;tz++)for(int tx=-1;tx<=1;tx++){
    float cx=-64+tx*64,cz=-64+tz*64;var center=new Vector3(cx,0,cz);
    var data=HeightmapBuilder.instance.RequestTerrainSync(center,Width,1f,false,gen);
    if(data==null)throw new Exception("HeightmapBuilder returned no data at "+F(cx)+","+F(cz));
    var heights=(List<float>)data.GetType().GetField("m_baseHeights",Flags).GetValue(data);
    if(heights==null||heights.Count!=Side*Side)throw new Exception("Unexpected height count at "+F(cx)+","+F(cz));
    string key=Key(cx,cz),relative="t_"+I(cx)+"_"+I(cz);string dir=Path.Combine(output,relative);Directory.CreateDirectory(dir);
    tiles.Add(key,new List<float>(heights));tileRoots.Add(key,dir);
    using(var w=new BinaryWriter(File.Create(Path.Combine(dir,"tile.bin")))){
     w.Write(new byte[]{84,69,82,49});w.Write(Width);w.Write(1f);w.Write(cx);w.Write(cz);w.Write(sharedOrigin);w.Write(world.m_seed);w.Write(world.m_worldGenVersion);foreach(float h in heights)w.Write(h);
    }
    manifest.WriteLine(relative+"/tile.bin\t"+I(cx)+"\t"+I(cz));
    float min=float.PositiveInfinity,max=float.NegativeInfinity,minX=0,minZ=0,maxX=0,maxZ=0;
    var biomeCounts=new SortedDictionary<string,int>(StringComparer.Ordinal);
    for(int iz=0;iz<Side;iz++)for(int ix=0;ix<Side;ix++){
     float h=heights[iz*Side+ix],wx=cx-32+ix,wz=cz-32+iz;
     if(h<min){min=h;minX=wx;minZ=wz;}if(h>max){max=h;maxX=wx;maxZ=wz;}
     string biome=gen.GetBiome(new Vector3(wx,0,wz)).ToString();int n;biomeCounts.TryGetValue(biome,out n);biomeCounts[biome]=n+1;
    }
    bounds.WriteLine(I(cx)+"\t"+I(cz)+"\t"+F(cx-32)+"\t"+F(cz-32)+"\t"+F(cx+32)+"\t"+F(cz+32)+"\t"+F(min)+"\t"+F(minX)+"\t"+F(minZ)+"\t"+F(max)+"\t"+F(maxX)+"\t"+F(maxZ)+"\t"+F(heights[32*Side+32])+"\t"+F(sharedOrigin));
    foreach(var kv in biomeCounts)biomeOut.WriteLine(I(cx)+"\t"+I(cz)+"\t"+kv.Key+"\t"+kv.Value.ToString(CultureInfo.InvariantCulture));
    ExportMesh(dir,center,heights);
    yield return null;
   }
   bool seamsExact=true;
   for(int tz=-1;tz<=1;tz++)for(int tx=-1;tx<=1;tx++){
    float cx=-64+tx*64,cz=-64+tz*64;var a=tiles[Key(cx,cz)];
    if(tx<1){float bx=cx+64;var b=tiles[Key(bx,cz)];float delta=CompareXSeam(a,b);bool exact=delta==0;seamsExact&=exact;seams.WriteLine(I(cx)+"\t"+I(cz)+"\t"+I(bx)+"\t"+I(cz)+"\tx\t65\t"+F(delta)+"\t"+exact);}
    if(tz<1){float bz=cz+64;var b=tiles[Key(cx,bz)];float delta=CompareZSeam(a,b);bool exact=delta==0;seamsExact&=exact;seams.WriteLine(I(cx)+"\t"+I(cz)+"\t"+I(cx)+"\t"+I(bz)+"\tz\t65\t"+F(delta)+"\t"+exact);}
   }
   if(!seamsExact)throw new Exception("At least one native neighboring tile seam is not exact; patch pointer was not updated");
  }
  File.WriteAllText(Path.Combine(output,"identity.txt"),"Native Unity HeightmapBuilder 3x3 patch; no user world/save opened.\nUnity="+Application.unityVersion+"\nassemblyMVID="+typeof(WorldGenerator).Assembly.ManifestModule.ModuleVersionId+"\nseedName=LabTerrain1\nseed="+world.m_seed+"\ngenVersion="+world.m_worldGenVersion+"\ntileWidth=64\nspacing=1\ntiles=9\npatchCenter=-64,-64\nsharedOriginCenter=-64,-64\nsharedOriginHeight="+F(sharedOrigin)+"\nsourceHeights=installed HeightmapBuilder m_baseHeights; absolute native world heights\nmesh=installed Heightmap.RebuildCollisionMesh\nraycasts=100 PhysX MeshCollider raycasts per tile\nseams=all 12 neighboring edges compared at 65 vertices; exact required\n");
  File.WriteAllText(Path.Combine(output,"SUCCESS.txt"),"Native nine-tile terrain patch generated with native meshes, indices, raycasts, bounds, biome coverage, and exact shared-edge checks.");

  // Enter a unique, save-isolated native world before asking ZoneSystem to generate its bounded objects.
  string profileId="LabTerrainPatch"+stamp,worldName="LabTerrainPatch"+stamp;
  var profile=new PlayerProfile(profileId,(FileHelpers.FileSource)2);profile.SetName(profileId);profile.m_firstSpawn=false;
  if(!profile.Save())throw new Exception("Could not save private terrain-reference profile");
  Game.SetProfile(profileId,(FileHelpers.FileSource)2);
  var playWorld=new World(worldName,"LabTerrain1");ZNet.SetServer(true,false,false,"Lab terrain patch reference","",playWorld);
  var menu=UnityEngine.Object.FindFirstObjectByType<FejdStartup>();if(!menu)throw new Exception("Startup menu disappeared before private reference world load");
  typeof(FejdStartup).GetMethod("LoadMainScene",Flags).Invoke(menu,null);
  float deadline=Time.realtimeSinceStartup+180f;
  while((!Player.m_localPlayer||!ZoneSystem.instance)&&Time.realtimeSinceStartup<deadline)yield return null;
  if(!Player.m_localPlayer||!ZoneSystem.instance)throw new Exception("Private reference world did not load player and ZoneSystem");
  if(Game.instance)Game.instance.SkipIntro();Player.m_localPlayer.SetIntro(false);
  yield return new WaitForSecondsRealtime(5f);
  float central=gen.GetHeight(-64,-64);
  string envDir=Path.Combine(output,"environment");
  int generated=-1;
  yield return MeadowsEnvironment.GenerateBounded(new Vector3(-64,0,-64),central,envDir,n=>generated=n);
  File.AppendAllText(Path.Combine(output,"identity.txt"),"\nZoneSystem native zones generated="+generated+"/9; environment extraction is native scene output.\n");
   string capture=Path.Combine(envDir,"environment-capture.txt");int rows=0;if(File.Exists(capture)){foreach(string line in File.ReadAllLines(capture))if(line.StartsWith("Rows: ",StringComparison.Ordinal))Int32.TryParse(line.Substring(6),out rows);}
   if(generated==9&&rows>0){Directory.CreateDirectory(Path.Combine(root,"environment"));File.WriteAllText(Path.Combine(root,"environment","patch-current.txt"),envDir);string envRoot=Path.Combine(Probe.LabRoot,"private-cache","environment");Directory.CreateDirectory(envRoot);File.WriteAllText(Path.Combine(envRoot,"patch-current.txt"),envDir);}
  yield return null;
  yield return CaptureOverview(new Vector3(-64,0,-64),central,output);
  File.WriteAllText(Path.Combine(root,"patch-current.txt"),output);
  Application.Quit(0);
 }

 static string Key(float x,float z){return I(x)+","+I(z);}static string I(float v){return v.ToString("0",CultureInfo.InvariantCulture);}
 static float CompareXSeam(List<float>a,List<float>b){float d=0;for(int i=0;i<Side;i++)d=Math.Max(d,Mathf.Abs(a[i*Side+Side-1]-b[i*Side]));return d;}
 static float CompareZSeam(List<float>a,List<float>b){float d=0;for(int i=0;i<Side;i++)d=Math.Max(d,Mathf.Abs(a[(Side-1)*Side+i]-b[i]));return d;}

 void ExportMesh(string dir,Vector3 center,List<float> heights){
  var obj=new GameObject("Inactive native terrain reference");obj.SetActive(false);var hm=obj.AddComponent<Heightmap>();hm.m_width=Width;hm.m_scale=1f;
  var own=(List<float>)typeof(Heightmap).GetField("m_heights",Flags).GetValue(hm);own.Clear();own.AddRange(heights);
  typeof(Heightmap).GetMethod("RebuildCollisionMesh",Flags).Invoke(hm,null);
  var mesh=(Mesh)typeof(Heightmap).GetField("m_collisionMesh",Flags).GetValue(hm);var vertices=mesh.vertices;var indices=mesh.triangles;
  using(var w=new StreamWriter(Path.Combine(dir,"native-vertices.csv"))){w.WriteLine("index,local_x,height,local_z,world_x,world_z");for(int i=0;i<vertices.Length;i++){var v=vertices[i];w.WriteLine(i+","+F(v.x)+","+F(v.y)+","+F(v.z)+","+F(center.x+v.x)+","+F(center.z+v.z));}}
  File.WriteAllLines(Path.Combine(dir,"native-indices.txt"),Array.ConvertAll(indices,i=>i.ToString(CultureInfo.InvariantCulture)));
  var colliderObject=new GameObject("Native terrain raycast reference");colliderObject.transform.position=center;var collider=colliderObject.AddComponent<MeshCollider>();collider.sharedMesh=mesh;
  using(var w=new StreamWriter(Path.Combine(dir,"native-raycasts.csv"))){w.WriteLine("tile_center_x,tile_center_z,world_x,world_z,height");for(int i=0;i<100;i++){float x=-29.63f+(i%10)*6.17f,z=-29.41f+(i/10)*6.11f;RaycastHit hit;var ray=new Ray(new Vector3(center.x+x,200,center.z+z),Vector3.down);if(!collider.Raycast(ray,out hit,400))throw new Exception("Native terrain ray missed at tile "+F(center.x)+","+F(center.z)+" local "+F(x)+","+F(z));w.WriteLine(F(center.x)+","+F(center.z)+","+F(center.x+x)+","+F(center.z+z)+","+F(hit.point.y));}}
  Destroy(colliderObject);Destroy(obj);
 }

 IEnumerator CaptureOverview(Vector3 center,float centerHeight,string dir){
  var cam=Camera.main;if(!cam){File.WriteAllText(Path.Combine(dir,"native-camera.txt"),"No Camera.main available after private world load.\n");yield break;}
  Vector3 pos=new Vector3(center.x,centerHeight+58f,center.z-105f),target=new Vector3(center.x,centerHeight+4f,center.z);
  cam.transform.position=pos;cam.transform.rotation=Quaternion.LookRotation(target-pos,Vector3.up);cam.fieldOfView=55f;cam.farClipPlane=Mathf.Max(cam.farClipPlane,600f);
  File.WriteAllText(Path.Combine(dir,"native-camera.txt"),"position="+F(pos.x)+","+F(pos.y)+","+F(pos.z)+"\nrotation="+F(cam.transform.rotation.x)+","+F(cam.transform.rotation.y)+","+F(cam.transform.rotation.z)+","+F(cam.transform.rotation.w)+"\nfov="+F(cam.fieldOfView)+"\ncenter="+F(center.x)+","+F(center.z)+"\ncenter_height="+F(centerHeight)+"\nGenerated by native Unity camera on bounded private terrain patch.\n");
  var captureObject=new GameObject("Native patch overview camera");var capture=captureObject.AddComponent<Camera>();capture.CopyFrom(cam);capture.targetTexture=null;capture.transform.position=pos;capture.transform.rotation=Quaternion.LookRotation(target-pos,Vector3.up);capture.fieldOfView=55f;capture.farClipPlane=Mathf.Max(capture.farClipPlane,600f);
  var rt=new RenderTexture(1280,720,24);var previous=RenderTexture.active;Texture2D image=null;
  try{capture.targetTexture=rt;capture.Render();RenderTexture.active=rt;image=new Texture2D(1280,720,TextureFormat.RGB24,false);image.ReadPixels(new Rect(0,0,1280,720),0,0);image.Apply();File.WriteAllBytes(Path.Combine(dir,"native-overview.png"),ImageConversion.EncodeToPNG(image));}
  finally{capture.targetTexture=null;RenderTexture.active=previous;rt.Release();Destroy(rt);if(image)Destroy(image);Destroy(captureObject);}
  yield return null;
 }

 void InspectRegion(){
  var world=new World("LabTerrainRegionInspection","LabTerrain1");WorldGenerator.Initialize(world);var gen=WorldGenerator.instance;
  string root=Path.Combine(Probe.LabRoot,"private-cache","terrain");Directory.CreateDirectory(root);
  string dir=Path.Combine(root,"inspect-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff",CultureInfo.InvariantCulture));Directory.CreateDirectory(dir);output=dir;
  var rows=new List<string>();rows.Add("tile_center_x\ttile_center_z\tsample_x\tsample_z\tbiome\theight");
  for(int tz=-1;tz<=1;tz++)for(int tx=-1;tx<=1;tx++){float cx=-64+tx*64,cz=-64+tz*64;for(int iz=0;iz<=4;iz++)for(int ix=0;ix<=4;ix++){float x=cx-32+ix*16,z=cz-32+iz*16;rows.Add(F(cx)+"\t"+F(cz)+"\t"+F(x)+"\t"+F(z)+"\t"+gen.GetBiome(new Vector3(x,0,z))+"\t"+F(gen.GetHeight(x,z)));}}
  File.WriteAllLines(Path.Combine(dir,"region-inspection.tsv"),rows);
  File.WriteAllText(Path.Combine(dir,"identity.txt"),"Native region suitability inspection; no tiles generated and no world save opened.\nseedName=LabTerrain1\nseed="+world.m_seed+"\ngenVersion="+world.m_worldGenVersion+"\nrequestedCenter=-64,-64\nwaterLevel=30 (compare native height checkpoints; biome enum Ocean denotes water).\n");
  File.WriteAllText(Path.Combine(dir,"SUCCESS.txt"),"Native biome/height grid exported for candidate 3x3 tile patch.");File.WriteAllText(Path.Combine(Path.GetDirectoryName(root),"inspection-current.txt"),dir);Application.Quit(0);
 }
}
}
