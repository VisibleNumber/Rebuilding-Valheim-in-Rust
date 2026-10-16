using Mono.Cecil;
using Mono.Cecil.Cil;
using System.Security.Cryptography;
using System.Text.Json;

if(args.Length!=1) throw new Exception("Usage: Patcher <valheim-lab root>");
var lab=Path.GetFullPath(args[0]);
if(!new DirectoryInfo(lab).Name.Equals("valheim-lab",StringComparison.OrdinalIgnoreCase)) throw new Exception("Expected separate valheim-lab workspace");
var original=Path.GetFullPath(Path.Combine(lab,"..","common","Valheim"));
var clone=Path.Combine(lab,"test-game");
var managed=Path.Combine(clone,"valheim_Data","Managed");
if(!File.Exists(Path.Combine(clone,".valheim-lab-copy"))) throw new Exception("Isolated-copy marker missing; run Prepare-Lab.ps1");
string Hash(string p)=>Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(p)));
var srcManaged=Path.Combine(original,"valheim_Data","Managed");
if(Hash(Path.Combine(srcManaged,"assembly_valheim.dll"))!="94998885B103D6E5F5480123EAEE0A093E82AF2DDEEEE37536C15BF555F24F3A") throw new Exception("Game changed; reinspect and review hooks before patching");
if(Hash(Path.Combine(srcManaged,"assembly_utils.dll"))!="771D59CBF3EF704007267E8EAF89AAC231C172B052BD93F9C273A18381BD6D59") throw new Exception("Utils changed; reinspect save isolation");
if(Hash(Path.Combine(original,"valheim.exe"))!=Hash(Path.Combine(clone,"valheim.exe"))) throw new Exception("Executable mismatch; no executable patch is supported");
var backup=Path.Combine(lab,"backups","deployment-"+DateTime.UtcNow.ToString("yyyyMMdd-HHmmss-fffffff"));
Directory.CreateDirectory(backup);
foreach(var n in new[]{"assembly_valheim.dll","assembly_utils.dll","ValheimLab.Diagnostic.dll"}) if(File.Exists(Path.Combine(managed,n))) File.Copy(Path.Combine(managed,n),Path.Combine(backup,n));
var resolver=new DefaultAssemblyResolver();resolver.AddSearchDirectory(srcManaged);
var parameters=new ReaderParameters {AssemblyResolver=resolver};
using var game=AssemblyDefinition.ReadAssembly(Path.Combine(srcManaged,"assembly_valheim.dll"),parameters);
using var utils=AssemblyDefinition.ReadAssembly(Path.Combine(srcManaged,"assembly_utils.dll"),parameters);
var probePath=Path.Combine(lab,"diagnostic","bin","Release","net10.0","ValheimLab.Diagnostic.dll");
using var probe=AssemblyDefinition.ReadAssembly(probePath,parameters);
var probeType=probe.MainModule.Types.Single(t=>t.FullName=="ValheimLab.Probe");
MethodDefinition Method(AssemblyDefinition a,string type,string name)=>a.MainModule.Types.Single(t=>t.FullName==type).Methods.Single(m=>m.Name==name);
MethodReference Hook(string name)=>game.MainModule.ImportReference(probeType.Methods.Single(m=>m.Name==name));
void Prefix(MethodDefinition m,params Instruction[] ins) {var il=m.Body.GetILProcessor();var first=m.Body.Instructions[0];foreach(var i in ins)il.InsertBefore(first,i);}
void Marker(MethodDefinition m,string kind,bool end) {
 var il=m.Body.GetILProcessor();
 if(!end) Prefix(m,il.Create(OpCodes.Ldstr,kind),il.Create(OpCodes.Ldstr,m.FullName),il.Create(OpCodes.Call,Hook("Marker")));
 else foreach(var ret in m.Body.Instructions.Where(i=>i.OpCode==OpCodes.Ret).ToArray()) {
  // Replace original ret so branches to it also execute the hook, preserving any return value beneath the two strings.
  ret.OpCode=OpCodes.Ldstr;ret.Operand=kind;
  var note=il.Create(OpCodes.Ldstr,m.FullName);il.InsertAfter(ret,note);
  var call=il.Create(OpCodes.Call,Hook("Marker"));il.InsertAfter(note,call);il.InsertAfter(call,il.Create(OpCodes.Ret));
 }
}
foreach(var type in new[]{"FejdStartup","Game"}) {var m=Method(game,type,"Awake");Prefix(m,Instruction.Create(OpCodes.Call,Hook("Boot")));}
var attack=Method(game,"Humanoid","StartAttack");Prefix(attack,Instruction.Create(OpCodes.Ldarg_0),Instruction.Create(OpCodes.Call,Hook("AttackAttempt")));
var controls=Method(game,"Player","SetControls");Prefix(controls,Instruction.Create(OpCodes.Ldarg_1),Instruction.Create(OpCodes.Call,Hook("FilterMovement")),Instruction.Create(OpCodes.Starg,controls.Parameters[0]));
var damage=Method(game,"Character","ApplyDamage");
var health=game.MainModule.ImportReference(Method(game,"Character","GetHealth"));
var before=new VariableDefinition(game.MainModule.TypeSystem.Single);damage.Body.Variables.Add(before);damage.Body.InitLocals=true;
Prefix(damage,Instruction.Create(OpCodes.Ldarg_0),Instruction.Create(OpCodes.Call,health),Instruction.Create(OpCodes.Stloc,before),Instruction.Create(OpCodes.Ldarg_0),Instruction.Create(OpCodes.Ldarg_1),Instruction.Create(OpCodes.Call,Hook("DamageBegin")));
var dil=damage.Body.GetILProcessor();
foreach(var ret in damage.Body.Instructions.Where(i=>i.OpCode==OpCodes.Ret).ToArray()) {
 ret.OpCode=OpCodes.Ldarg_0;ret.Operand=null;
 var h=dil.Create(OpCodes.Ldarg_1);dil.InsertAfter(ret,h);
 var b=dil.Create(OpCodes.Ldloc,before);dil.InsertAfter(h,b);
 var call=dil.Create(OpCodes.Call,Hook("DamageEnd"));dil.InsertAfter(b,call);dil.InsertAfter(call,dil.Create(OpCodes.Ret));
}
foreach(var pair in new[]{("ZNet","SaveWorldThread"),("ZNet","LoadWorld"),("Game","SavePlayerProfile"),("PlayerProfile","SavePlayerToDisk"),("PlayerProfile","LoadPlayerFromDisk")}) {
 var m=Method(game,pair.Item1,pair.Item2);Marker(m,"enter_"+pair.Item1+"_"+pair.Item2,false);Marker(m,"return_"+pair.Item1+"_"+pair.Item2,true);
}
// Save isolation is unconditional in the test copy, including FileSource.Auto/Cloud callers.
var cloud=Method(utils,"FileHelpers","get_CloudStorageSupportedAndEnabled");cloud.Body.Instructions.Clear();cloud.Body.ExceptionHandlers.Clear();cloud.Body.Variables.Clear();cloud.Body.Instructions.Add(Instruction.Create(OpCodes.Ldc_I4_0));cloud.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
var getPath=Method(utils,"Utils","GetSaveDataPath");getPath.Body.Instructions.Clear();getPath.Body.ExceptionHandlers.Clear();getPath.Body.Variables.Clear();getPath.Body.Instructions.Add(Instruction.Create(OpCodes.Ldstr,Path.Combine(lab,"test-saves")));getPath.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
// Added hooks can lengthen branch distances. Expand short branches before writing.
var opcodes=typeof(OpCodes).GetFields(System.Reflection.BindingFlags.Public|System.Reflection.BindingFlags.Static).Where(f=>f.FieldType==typeof(OpCode)).Select(f=>(OpCode)f.GetValue(null)!).ToDictionary(o=>o.Name);
foreach(var a in new[]{game,utils}) foreach(var t in a.MainModule.Types) foreach(var m in t.Methods.Where(m=>m.HasBody)) foreach(var ins in m.Body.Instructions) if(ins.OpCode.OperandType==OperandType.ShortInlineBrTarget) ins.OpCode=opcodes[ins.OpCode.Name[..^2]];
game.Write(Path.Combine(managed,"assembly_valheim.dll"));utils.Write(Path.Combine(managed,"assembly_utils.dll"));File.Copy(probePath,Path.Combine(managed,"ValheimLab.Diagnostic.dll"),true);
var names=new[]{"assembly_valheim.dll","assembly_utils.dll","ValheimLab.Diagnostic.dll"};
File.WriteAllText(Path.Combine(backup,"deployment.json"),JsonSerializer.Serialize(new{schema=1,utc=DateTime.UtcNow,original,clone,backup,exe_sha256=Hash(Path.Combine(original,"valheim.exe")),files=names.Select(n=>new {name=n,sha256=Hash(Path.Combine(managed,n))})},new JsonSerializerOptions{WriteIndented=true}));
Console.WriteLine("Installed diagnostic in isolated test copy only. Backup and hashes: "+backup);
