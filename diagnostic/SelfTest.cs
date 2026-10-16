using System;
using System.Collections;
using System.Globalization;
using System.IO;
using System.Reflection;
using UnityEngine;

namespace ValheimLab {
// Explicitly custom test driver. Runs original controllers and persistence in a new private world.
public sealed class SelfTest : MonoBehaviour {
 public static bool ControlMovement;
 public static Vector3 Move;
 const BindingFlags Flags=BindingFlags.Instance|BindingFlags.Public|BindingFlags.NonPublic;
 float started;
 void Start() {started=Time.realtimeSinceStartup;StartCoroutine(Guard(Run()));}
 IEnumerator Guard(IEnumerator run) {
  while(true) {
   object current=null;bool more=false;Exception error=null;
   try {more=run.MoveNext();if(more)current=run.Current;} catch(Exception e){error=e;}
   if(error!=null) {Result("FAIL",error.ToString());yield break;}
   if(!more)yield break;
   yield return current;
  }
 }
 void Update() {if(Time.realtimeSinceStartup-started>300)Result("FAIL","300s timeout");}
 static string PathFor(string name) {return Path.Combine(Probe.LabRoot,"evidence",name);}
 static void Capture(string phase) {
  if(!Probe.HasArg("-lab-capture"))return;
  var cam=Utils.GetMainCamera();if(!cam)throw new Exception("No native gameplay camera available for capture");
  var old=cam.targetTexture;var active=RenderTexture.active;var rt=new RenderTexture(960,540,24);Texture2D image=null;
  try {
   cam.targetTexture=rt;cam.Render();RenderTexture.active=rt;
   image=new Texture2D(960,540,TextureFormat.RGB24,false);image.ReadPixels(new Rect(0,0,960,540),0,0);image.Apply();
   File.WriteAllBytes(PathFor("native-offscreen-"+phase+"-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff")+".png"),ImageConversion.EncodeToPNG(image));
   Probe.Marker("native_camera_capture","Explicit offscreen camera render; excludes screen-space overlay UI; phase="+phase);
  } finally {cam.targetTexture=old;RenderTexture.active=active;if(image)Destroy(image);rt.Release();Destroy(rt);}
 }
 void Result(string result,string detail) {
  ControlMovement=false;
  Probe.Marker("selftest_"+result,detail);UnityEngine.Debug.Log("VALHEIM LAB SELFTEST "+result+" "+detail);
  File.WriteAllText(PathFor("selftest-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff")+".txt"),result+"\n"+detail);
  Application.Quit(result=="PASS"?0:2);
 }
 static void Require(bool condition,string detail) {if(!condition)throw new Exception(detail);Probe.Marker("assert_pass",detail);}
 IEnumerator Run() {
  yield return new WaitForSecondsRealtime(8);
  Require(!FileHelpers.CloudStorageSupportedAndEnabled,"Cloud storage disabled in isolated copy");
  Require(System.IO.Path.GetFullPath(Utils.GetSaveDataPath((FileHelpers.FileSource)2))==System.IO.Path.Combine(Probe.LabRoot,"test-saves"),"Save root is private test-saves");
  var menu=UnityEngine.Object.FindFirstObjectByType<FejdStartup>();Require(menu!=null,"Original startup scene is running");
  bool reload=Probe.HasArg("-lab-reload");
  var profile=new PlayerProfile("lab-recorder",(FileHelpers.FileSource)2);
  if(reload)Require(profile.Load(),"Native profile reload succeeds");
  else {profile.SetName("Lab Recorder");profile.m_firstSpawn=false;Require(profile.Save(),"New native test profile saved");}
  Game.SetProfile("lab-recorder",(FileHelpers.FileSource)2);
  string worldName=reload?File.ReadAllText(PathFor("native-test-world.txt")):"LabEvidence"+DateTime.UtcNow.ToString("yyyyMMddHHmmss");
  if(!reload) {if(File.Exists(PathFor("native-test-world.txt")))File.Copy(PathFor("native-test-world.txt"),PathFor("native-test-world-previous-"+DateTime.UtcNow.Ticks+".txt"));File.WriteAllText(PathFor("native-test-world.txt"),worldName);}
  var world=World.GetCreateWorld(worldName,(FileHelpers.FileSource)2);
  ZNet.SetServer(true,false,false,"Local investigation","",world);
  Probe.Marker("test_world_start","Native world="+world.m_name+" seed="+world.m_seedName+" reload="+reload);
  typeof(FejdStartup).GetMethod("LoadMainScene",Flags).Invoke(menu,null);
  float timeout=Time.realtimeSinceStartup+180;
  while(!Player.m_localPlayer && Time.realtimeSinceStartup<timeout)yield return null;
  Require(Player.m_localPlayer,"Native player spawned");
  var p=Player.m_localPlayer;if(!reload)Game.instance.SkipIntro();p.SetIntro(false);
  yield return new WaitForSecondsRealtime(5);
  while((!Player.m_localPlayer || Game.instance.WaitingForRespawn()) && Time.realtimeSinceStartup<timeout)yield return null;
  p=Player.m_localPlayer;Require(p,"Native player available after intro/respawn transition");
  if(reload) {
   var expected=File.ReadAllLines(PathFor("native-save-expected.txt"));
   var pos=new Vector3(float.Parse(expected[0],CultureInfo.InvariantCulture),float.Parse(expected[1],CultureInfo.InvariantCulture),float.Parse(expected[2],CultureInfo.InvariantCulture));
   Require(Vector3.Distance(p.transform.position,pos)<5,"Native world/profile restart restores player near saved position");
   bool found=false;
   foreach(var piece in UnityEngine.Object.FindObjectsByType<Piece>(FindObjectsSortMode.None)) {
    var nv=piece.GetComponent<ZNetView>();
    if(nv && nv.IsValid() && nv.GetZDO().GetString("valheim_lab_marker","")=="persistent-piece-1")found=true;
   }
   Require(found,"Placed building piece restored from native world save with marker");
   Capture("reload");
   Result("PASS","Native process restart restored player position and placed piece. See CSV and save files.");yield break;
  }
  Vector3 origin=p.transform.position;
  Move=Vector3.forward;ControlMovement=true;p.SetLookDir(Move,0);
  Probe.Marker("movement_start","Custom scripted movement input through native Player.SetControls");
  yield return new WaitForSecondsRealtime(4);
  Move=Vector3.zero;
  yield return new WaitForSecondsRealtime(1);
  float distance=Vector3.Distance(origin,p.transform.position);
  Require(distance>1,"Native controller moved player "+distance.ToString("F3",CultureInfo.InvariantCulture)+"m");
  Capture("movement");
  var prefab=ZNetScene.instance.GetPrefab("Greyling");Require(prefab,"Native Greyling prefab resolves");
  var enemy=UnityEngine.Object.Instantiate(prefab,p.transform.position+p.transform.forward*2,Quaternion.identity).GetComponent<Character>();
  Require(enemy && enemy.GetComponent<BaseAI>(),"Native enemy with AI instantiated");
  p.SetLookDir((enemy.transform.position-p.transform.position).normalized,0);
  yield return new WaitForSecondsRealtime(2);
  bool attack=p.StartAttack(enemy,false);Probe.Marker("test_attack_result","native StartAttack accepted="+attack);
  yield return new WaitForSecondsRealtime(3);
  float before=enemy.GetHealth();
  var hit=new HitData();hit.m_damage.m_blunt=2;hit.m_point=enemy.GetCenterPoint();hit.m_dir=(enemy.transform.position-p.transform.position).normalized;hit.SetAttacker(p);
  enemy.Damage(hit);
  yield return new WaitForSecondsRealtime(1);
  Require(enemy.GetHealth()<before,"Native Character.Damage/RPC pipeline reduced enemy health");
  Probe.Marker("test_damage_scope","One explicit damage injection verifies damage instrumentation; melee and natural AI outcomes are evaluated separately from CSV");
  yield return new WaitForSecondsRealtime(8);
  Capture("enemy");
  var piecePrefab=ZNetScene.instance.GetPrefab("wood_floor");Require(piecePrefab,"Native wood_floor prefab resolves");
  var pieceObj=UnityEngine.Object.Instantiate(piecePrefab,p.transform.position+Vector3.right*3,Quaternion.identity);
  var view=pieceObj.GetComponent<ZNetView>();Require(view && view.IsValid(),"Building piece has persistent network state");
  view.GetZDO().Set("valheim_lab_marker","persistent-piece-1");
  // Avoid support collapse during a persistence test; this is a test fixture, not a stability claim.
  var wear=pieceObj.GetComponent<WearNTear>();if(wear)wear.enabled=false;
  Capture("piece");
  var loc=p.transform.position;
  if(File.Exists(PathFor("native-save-expected.txt")))File.Copy(PathFor("native-save-expected.txt"),PathFor("native-save-expected-previous-"+DateTime.UtcNow.Ticks+".txt"));
  File.WriteAllLines(PathFor("native-save-expected.txt"),new[]{loc.x.ToString("R",CultureInfo.InvariantCulture),loc.y.ToString("R",CultureInfo.InvariantCulture),loc.z.ToString("R",CultureInfo.InvariantCulture)});
  Game.instance.SavePlayerProfile(false,false);ZNet.instance.Save(true,false,false);
  yield return new WaitForSecondsRealtime(5);
  Require(Directory.GetFiles(System.IO.Path.Combine(Probe.LabRoot,"test-saves"),"*.db2",SearchOption.AllDirectories).Length>0,"Native chunked world database written in test-saves");
  Result("PASS","Movement, enemy prefab/AI, attack attempt, native damage, building persistence save completed. Separate -lab-reload run required to establish disk reload.");
 }
}
}
