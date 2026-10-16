using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Threading;
using UnityEngine;

namespace ValheimLab {
public sealed class Probe : MonoBehaviour {
 public const string Revision="0.1.2";
 static Probe instance;
 static readonly object gate=new object();
 static readonly Stopwatch clock=Stopwatch.StartNew();
 static StreamWriter writer;
 static string status="Starting";
 static int lines;
 static double began;
 double lastFrame,lastSample,lastFlush,ownMs;
 int frameCount;
 double frameSum,frameMax;
 public static string LabRoot {get {return Path.GetFullPath(Path.Combine(Application.dataPath,"..",".."));}}
 public static bool HasArg(string value) {return Array.IndexOf(Environment.GetCommandLineArgs(),value)>=0;}
 public static void Boot() {
  try {
   if(instance) return;
   // The only supported deployment is an isolated copy under valheim-lab/test-game.
   if(!string.Equals(new DirectoryInfo(Path.Combine(Application.dataPath,"..")).Name,"test-game",StringComparison.OrdinalIgnoreCase)) return;
   Game.isModded=true;
   var go=new GameObject("Valheim investigation recorder"); DontDestroyOnLoad(go); instance=go.AddComponent<Probe>();
   instance.lastFrame=clock.Elapsed.TotalSeconds;
   if(HasArg("-lab-record") || HasArg("-lab-selftest")) instance.Begin();
   if(HasArg("-lab-selftest")) go.AddComponent<SelfTest>();
   if(HasArg("-lab-greyling-assets")) go.AddComponent<GreylingCheck>();
   if(HasArg("-lab-terrain") || HasArg("-lab-terrain-inspect")) go.AddComponent<TerrainCheck>();
   if(HasArg("-lab-worldgen")) go.AddComponent<WorldgenCheck>();
   if(HasArg("-lab-environment")) go.AddComponent<EnvironmentCheck>();
   UnityEngine.Debug.Log("VALHEIM LAB "+Revision+" ready; F8 toggles recording. Separate test saves only.");
  } catch(Exception e) {UnityEngine.Debug.LogError("Valheim Lab boot: "+e);}
 }
 void Begin() {
  try {
   string dir=Path.Combine(LabRoot,"recordings"); Directory.CreateDirectory(dir);
   string path=Path.Combine(dir,"session-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff",CultureInfo.InvariantCulture)+".csv");
   lock(gate) {
    writer=new StreamWriter(new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.Read));
    writer.WriteLine("schema,seconds,kind,thread,id,prefab,x,y,z,vx,vy,vz,health,max_health,stamina,grounded,running,blocking,ai_alerted,target_id,attacker_id,damage_requested,health_before,health_after,frame_count,frame_mean_ms,frame_max_ms,recorder_ms,note");
    began=clock.Elapsed.TotalSeconds; lines=0;
   }
   frameCount=0; frameSum=frameMax=ownMs=0; lastSample=lastFlush=clock.Elapsed.TotalSeconds;
   status="Recording: "+Path.GetFileName(path);
   Marker("session_begin","diagnostic="+Revision+" game="+Version.GetVersionString(false)+" unity="+Application.unityVersion+" managed_mvid="+typeof(Player).Assembly.ManifestModule.ModuleVersionId+" mode="+(HasArg("-lab-selftest")?"automated":"manual"));
  } catch(Exception e) {Fail(e);}
 }
 static string F(double v) {return v.ToString("R",CultureInfo.InvariantCulture);}
 static string C(object v) {string s=v==null?"":Convert.ToString(v,CultureInfo.InvariantCulture);return "\""+s.Replace("\"","\"\"").Replace("\r"," ").Replace("\n"," ")+"\"";}
 static void Row(string kind,Character c,HitData hit,float before,float after,int count,double mean,double max,double overhead,string note) {
  if(writer==null) return;
  try {
   // Character access occurs only on the game thread. Generic markers are safe on the save thread.
   Vector3 p=c?c.transform.position:Vector3.zero, v=c?c.GetVelocity():Vector3.zero;
   Player player=c as Player; BaseAI ai=c?c.GetComponent<BaseAI>():null;
   Character target=ai?ai.GetTargetCreature():null, attacker=hit==null?null:hit.GetAttacker();
   object[] values={1,F(clock.Elapsed.TotalSeconds-began),kind,Thread.CurrentThread.ManagedThreadId,c?c.GetInstanceID():0,c?c.name:"",c?F(p.x):"",c?F(p.y):"",c?F(p.z):"",c?F(v.x):"",c?F(v.y):"",c?F(v.z):"",c?F(c.GetHealth()):"",c?F(c.GetMaxHealth()):"",player?F(player.GetStamina()):"",c?c.IsOnGround().ToString():"",c?c.IsRunning().ToString():"",c?c.IsBlocking().ToString():"",ai?ai.IsAlerted().ToString():"",target?target.GetInstanceID():0,attacker?attacker.GetInstanceID():0,hit!=null?F(hit.GetTotalDamage()):"",float.IsNaN(before)?"":F(before),float.IsNaN(after)?"":F(after),count,mean<0?"":F(mean),max<0?"":F(max),overhead<0?"":F(overhead),note};
   string[] cells=Array.ConvertAll(values,C);
   lock(gate) {if(writer!=null) {writer.WriteLine(string.Join(",",cells)); lines++;}}
  } catch(Exception e) {Fail(e);}
 }
 public static void Marker(string kind,string note) {Row(kind,null,null,float.NaN,float.NaN,0,-1,-1,-1,note);}
 public static void DamageBegin(Character c,HitData hit) {Row("damage_begin",c,hit,c.GetHealth(),float.NaN,0,-1,-1,-1,"ApplyDamage entry; requested damage is before final mitigation");}
 public static void DamageEnd(Character c,HitData hit,float before) {Row("damage_end",c,hit,before,c.GetHealth(),0,-1,-1,-1,"ApplyDamage normal return; compare health, do not infer hit from proximity");}
 public static void AttackAttempt(Character c) {Row("attack_attempt",c,null,float.NaN,float.NaN,0,-1,-1,-1,"Humanoid.StartAttack entry; attempt may be refused");}
 public static Vector3 FilterMovement(Vector3 original) {return HasArg("-lab-selftest") && SelfTest.ControlMovement?SelfTest.Move:original;}
 static void Fail(Exception e) {status="Recorder stopped: "+e.Message; lock(gate) {try{if(writer!=null)writer.Dispose();}catch{} writer=null;} UnityEngine.Debug.LogError(status);}
 void End() {Marker("session_end","rows="+lines);lock(gate) {if(writer!=null)writer.Dispose();writer=null;}status="Recording saved";}
 void LateUpdate() {
  double now=clock.Elapsed.TotalSeconds, dt=(now-lastFrame)*1000; lastFrame=now;
  if(HasArg("-lab-smoke") && now>25) {Application.Quit();return;}
  if(Input.GetKeyDown(KeyCode.F8)) {if(writer==null)Begin();else End();}
  if(writer==null) return;
  double start=clock.Elapsed.TotalSeconds;
  frameCount++;frameSum+=dt;frameMax=Math.Max(frameMax,dt);
  if(now-lastSample>=0.2) {
   Row("frames",null,null,float.NaN,float.NaN,frameCount,frameSum/frameCount,frameMax,ownMs,"Stopwatch wall frames over this same interval; includes recorder overhead; no GPU attribution");
   var p=Player.m_localPlayer;
   if(p) {
    Row("player",p,null,float.NaN,float.NaN,0,-1,-1,-1,"");
    var near=new List<Character>();
    foreach(var c in Character.GetAllCharacters()) if(c && c!=p && (c.transform.position-p.transform.position).sqrMagnitude<=900) near.Add(c);
    near.Sort((a,b)=>(a.transform.position-p.transform.position).sqrMagnitude.CompareTo((b.transform.position-p.transform.position).sqrMagnitude));
    for(int n=0;n<Math.Min(16,near.Count);n++) Row("nearby",near[n],null,float.NaN,float.NaN,0,-1,-1,-1,"Within 30m; nearest 16 cap; instance IDs valid for this process only");
   }
   frameCount=0;frameSum=frameMax=ownMs=0;lastSample=now;
  }
  if(now-lastFlush>2) {try{lock(gate){if(writer!=null)writer.Flush();}}catch(Exception e){Fail(e);}lastFlush=now;}
  if(now-began>1200) End();
  ownMs+=(clock.Elapsed.TotalSeconds-start)*1000;
 }
 void OnGUI() {GUI.Label(new Rect(15,15,900,45),"VALHEIM LAB | F8 Record / Stop | "+status);}
 void OnApplicationQuit() {if(writer!=null) End();}
}
}



