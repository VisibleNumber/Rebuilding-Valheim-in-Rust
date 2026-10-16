using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Playables;
using UnityEngine.Animations;
namespace ValheimLab {
// Native reference and animation sampling only. The Rust asset reader is independent of this helper.
public sealed class GreylingCheck : MonoBehaviour {
 const string AssetPath="Assets/Characters/GreyDwarf/Greyling.prefab";
 string output;
 static string F(float v){return v.ToString("R",CultureInfo.InvariantCulture);}
 void Start(){output=Path.Combine(Probe.LabRoot,"private-cache","greyling","native-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff"));Directory.CreateDirectory(output);StartCoroutine(Guard(Run()));}
 IEnumerator Guard(IEnumerator e){while(true){bool more=false;object current=null;Exception error=null;try{more=e.MoveNext();if(more)current=e.Current;}catch(Exception x){error=x;}if(error!=null){File.WriteAllText(Path.Combine(output,"FAILED.txt"),error.ToString());Debug.LogError("GREYLING ASSET CHECK FAILED "+error);Application.Quit(2);yield break;}if(!more)yield break;yield return current;}}
 static string Relative(Transform t,Transform root){if(t==root)return "";return t.parent==root?t.name:Relative(t.parent,root)+"/"+t.name;}
 static void Matrix(BinaryWriter w,Matrix4x4 m){for(int col=0;col<4;col++)for(int row=0;row<4;row++)w.Write(m[row,col]);}
 void Capture(GameObject root,string name,float yaw){
  var cameraObject=new GameObject("Greyling reference camera");var cam=cameraObject.AddComponent<Camera>();cam.cullingMask=1<<31;cam.clearFlags=CameraClearFlags.SolidColor;cam.backgroundColor=new Color(.055f,.07f,.09f,1);cam.fieldOfView=35;cam.nearClipPlane=.1f;cam.farClipPlane=100;cam.allowHDR=false;cam.allowMSAA=false;
  float pitch=.12f,d=4.8f;cam.transform.position=new Vector3(Mathf.Sin(yaw)*Mathf.Cos(pitch)*d,.78f+Mathf.Sin(pitch)*d,Mathf.Cos(yaw)*Mathf.Cos(pitch)*d);cam.transform.LookAt(new Vector3(0,.78f,0));
  var rt=new RenderTexture(1024,768,24);var old=RenderTexture.active;Texture2D image=null;
  try{cam.targetTexture=rt;cam.Render();RenderTexture.active=rt;image=new Texture2D(1024,768,TextureFormat.RGB24,false);image.ReadPixels(new Rect(0,0,1024,768),0,0);image.Apply();File.WriteAllBytes(Path.Combine(output,name+".png"),ImageConversion.EncodeToPNG(image));}finally{RenderTexture.active=old;cam.targetTexture=null;rt.Release();Destroy(rt);if(image)Destroy(image);Destroy(cameraObject);}
 }
 void WriteMeshes(GameObject root,string filename){using(var w=new StreamWriter(Path.Combine(output,filename))){w.WriteLine("part,index,x,y,z,nx,ny,nz");foreach(var r in root.GetComponentsInChildren<SkinnedMeshRenderer>(true)){if(!r.gameObject.activeInHierarchy)continue;var mesh=new Mesh();r.BakeMesh(mesh,false);var positions=mesh.vertices;var normals=mesh.normals;var bakeToWorld=Matrix4x4.TRS(r.transform.position,r.transform.rotation,Vector3.one);for(int i=0;i<positions.Length;i++){var p=bakeToWorld.MultiplyPoint3x4(positions[i]);var n=r.transform.TransformDirection(normals[i]).normalized;w.WriteLine(r.name+","+i+","+F(p.x)+","+F(p.y)+","+F(p.z)+","+F(n.x)+","+F(n.y)+","+F(n.z));}Destroy(mesh);}}}
 IEnumerator Run(){
  yield return new WaitForSecondsRealtime(8);
  if(FileHelpers.CloudStorageSupportedAndEnabled)throw new Exception("Private-copy cloud isolation failed");
  GameObject prefab=null;var bundles=new List<string>();foreach(var bundle in AssetBundle.GetAllLoadedAssetBundles()){bundles.Add(bundle.name);if(bundle.name=="c4210710")prefab=bundle.LoadAsset<GameObject>(AssetPath);}
  if(!prefab)foreach(var go in Resources.FindObjectsOfTypeAll<GameObject>()){if(go.name=="Greyling"&&go.GetComponent<Character>()){prefab=go;break;}}
  File.WriteAllLines(Path.Combine(output,"loaded-bundles.txt"),bundles.ToArray());if(!prefab)throw new Exception("Greyling prefab is not available in the startup resource set");
  var holder=new GameObject("Greyling reference stage");holder.SetActive(false);var root=new GameObject("Greyling");root.transform.SetParent(holder.transform,false);
  var visualSource=prefab.GetComponentInChildren<Animator>(true);if(!visualSource)throw new Exception("Native visual Animator missing");
  var visual=Instantiate(visualSource.gameObject,root.transform,false);visual.name=visualSource.gameObject.name;
  foreach(var t in root.GetComponentsInChildren<Transform>(true))t.gameObject.layer=31;
  foreach(var c in root.GetComponentsInChildren<MonoBehaviour>(true))DestroyImmediate(c);
  foreach(var c in root.GetComponentsInChildren<Collider>(true))c.enabled=false;
  foreach(var r in root.GetComponentsInChildren<Rigidbody>(true)){r.isKinematic=true;r.detectCollisions=false;}
  foreach(var l in root.GetComponentsInChildren<LODGroup>(true))l.enabled=false;
  foreach(var r in root.GetComponentsInChildren<SkinnedMeshRenderer>(true))r.updateWhenOffscreen=true;
  foreach(var a in root.GetComponentsInChildren<Animator>(true)){a.enabled=false;a.applyRootMotion=false;a.fireEvents=false;a.cullingMode=AnimatorCullingMode.AlwaysAnimate;}
  holder.SetActive(true);yield return null;
  RenderSettings.fog=false;RenderSettings.ambientMode=AmbientMode.Flat;RenderSettings.ambientLight=new Color(.42f,.42f,.42f,1);
  var lightObject=new GameObject("Reference key light");var light=lightObject.AddComponent<Light>();light.type=LightType.Directional;light.cullingMask=1<<31;light.intensity=1;lightObject.transform.rotation=Quaternion.Euler(35,-30,0);
  using(var stats=new StreamWriter(Path.Combine(output,"native-assets.txt"))){stats.WriteLine("Native reference from "+AssetPath+" | Unity="+Application.unityVersion+" | ColorSpace="+QualitySettings.activeColorSpace);stats.WriteLine("CSV coordinate adapter: BakeMesh(false), then renderer translation/rotation without reapplying lossy scale");foreach(var r in root.GetComponentsInChildren<SkinnedMeshRenderer>(true)){var mesh=r.sharedMesh;stats.WriteLine("part="+Relative(r.transform,root.transform)+" mesh="+mesh.name+" active="+r.gameObject.activeInHierarchy+" vertices="+mesh.vertexCount+" bones="+r.bones.Length+" readable="+mesh.isReadable+" position="+r.transform.position.ToString("R")+" lossyScale="+r.transform.lossyScale.ToString("R"));foreach(var mat in r.sharedMaterials)stats.WriteLine("material="+mat.name+" shader="+mat.shader.name);}}
  WriteMeshes(root,"native-rest-vertices.csv");Capture(root,"native-rest-front",0);Capture(root,"native-rest-angle",.65f);
  var animator=root.GetComponentInChildren<Animator>(true);if(!animator)throw new Exception("Animator missing");
  var transforms=root.GetComponentsInChildren<Transform>(true);var paths=new List<string>();foreach(var t in transforms)paths.Add(Relative(t,root.transform));File.WriteAllLines(Path.Combine(output,"pose-paths.txt"),paths.ToArray());
  AnimationClip idle=null;foreach(var clip in animator.runtimeAnimatorController.animationClips)if(clip.name=="Idle")idle=clip;
  if(!idle||Mathf.Abs(idle.length-4)>0.001f)throw new Exception("Expected original four-second Idle clip");
  File.WriteAllText(Path.Combine(output,"sampled-clips.txt"),idle.name+" length="+F(idle.length)+" method=Unity AnimationClipPlayable, native Avatar retargeting");
  animator.enabled=true;animator.Rebind();animator.Update(0);animator.applyRootMotion=false;
  var graph=PlayableGraph.Create("Greyling native reference sampling");graph.SetTimeUpdateMode(DirectorUpdateMode.Manual);
  var playable=AnimationClipPlayable.Create(graph,idle);playable.SetApplyFootIK(false);var animationOutput=AnimationPlayableOutput.Create(graph,"Greyling original Idle",animator);animationOutput.SetSourcePlayable(playable);graph.Play();
  using(var w=new BinaryWriter(File.Create(Path.Combine(output,"native-idle-poses.bin")))){
   w.Write(new byte[]{(byte)'G',(byte)'P',(byte)'O',(byte)'1'});w.Write(121);w.Write(transforms.Length);w.Write(4f);
   for(int f=0;f<=120;f++){
    playable.SetTime(f/30.0);graph.Evaluate(0);yield return null;
    foreach(var t in transforms)Matrix(w,t.localToWorldMatrix);
    if(f==0||f==30||f==60){WriteMeshes(root,"native-idle-vertices-"+f+".csv");Capture(root,"native-idle-"+f,0);}
   }
  }
  graph.Destroy();
  var clipNames=new List<string>();foreach(var c in animator.runtimeAnimatorController.animationClips)clipNames.Add(c.name+" length="+F(c.length));File.WriteAllLines(Path.Combine(output,"available-clips.txt"),clipNames.ToArray());
  foreach(string role in new[]{"walk","attack"}){AnimationClip clip=null;foreach(var c in animator.runtimeAnimatorController.animationClips)if(c.name.IndexOf(role,StringComparison.OrdinalIgnoreCase)>=0){clip=c;break;}if(!clip){File.WriteAllText(Path.Combine(output,role+"-UNAVAILABLE.txt"),"No matching original clip");continue;}int frames=(int)Math.Ceiling(clip.length*30)+1;var g=PlayableGraph.Create("Native "+role);g.SetTimeUpdateMode(DirectorUpdateMode.Manual);animator.Rebind();var p=AnimationClipPlayable.Create(g,clip);p.SetApplyFootIK(false);var o=AnimationPlayableOutput.Create(g,role,animator);o.SetSourcePlayable(p);g.Play();using(var w=new BinaryWriter(File.Create(Path.Combine(output,"native-"+role+"-poses.bin")))){w.Write(new byte[]{71,80,79,49});w.Write(frames);w.Write(transforms.Length);w.Write(clip.length);for(int f=0;f<frames;f++){p.SetTime(Math.Min(clip.length,f/30.0));g.Evaluate(0);yield return null;foreach(var t in transforms)Matrix(w,t.localToWorldMatrix);if(f==0||f==frames/2){WriteMeshes(root,"native-"+role+"-vertices-"+f+".csv");Capture(root,"native-"+role+"-"+f,0);}}}g.Destroy();File.WriteAllText(Path.Combine(output,role+"-clip.txt"),clip.name+" duration="+F(clip.length)+" frames="+frames+" Native Unity sampling, not independent animation decoding");}
  File.WriteAllText(Path.Combine(output,"SUCCESS.txt"),"Native bind-pose reference and Idle poses exported. Animation poses were sampled through Unity; not an independently decoded clip.");
  File.WriteAllText(Path.Combine(Probe.LabRoot,"private-cache","greyling","native-current.txt"),output);
  Debug.Log("GREYLING ASSET CHECK SUCCESS "+output);Application.Quit(0);
 }
}
}

