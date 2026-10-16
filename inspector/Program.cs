using Mono.Cecil;
using System.Security.Cryptography;
using System.Text.Json;

if (args.Length == 4 && args[0] == "method") {
 using var a=AssemblyDefinition.ReadAssembly(args[1]);
 var t=a.MainModule.Types.Single(t=>t.FullName==args[2]);
 Console.WriteLine($"Assembly SHA256 {Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(args[1])))} MVID {a.MainModule.Mvid}");
 foreach(var f in t.Fields) Console.WriteLine($"FIELD {f.Attributes} {f.FullName} constant={f.Constant}");
 foreach(var m in t.Methods.Where(m=>args[3]=="*" || m.Name==args[3])) {
  Console.WriteLine($"METHOD {m.Attributes} {m.FullName} token={m.MetadataToken} RVA={m.RVA}");
  if(m.HasBody) foreach(var i in m.Body.Instructions) Console.WriteLine(i);
 }
 return 0;
}
if (args.Length != 2) { Console.Error.WriteLine("Usage: Inspector <game folder> <new output folder>"); return 2; }
string root = Path.GetFullPath(args[0]), output = Path.GetFullPath(args[1]);
if (!File.Exists(Path.Combine(root,"valheim.exe"))) throw new Exception("Valheim executable missing");
if (Directory.Exists(output)) throw new Exception("Output exists; choose a fresh directory to preserve evidence");
Directory.CreateDirectory(output);
string Hash(string p) { using var s=File.OpenRead(p); return Convert.ToHexString(SHA256.HashData(s)); }
var files=Directory.GetFiles(root,"*",SearchOption.AllDirectories).Order().Select(p=>new {path=Path.GetRelativePath(root,p),size=new FileInfo(p).Length,sha256=Hash(p)}).ToArray();
File.WriteAllText(Path.Combine(output,"inventory.json"),JsonSerializer.Serialize(new {schema=1,tool="valheim-inspector-0.1",utc=DateTime.UtcNow,root,files},new JsonSerializerOptions{WriteIndented=true}));
var selected=new HashSet<string>("Version,WorldGenerator,Heightmap,TerrainComp,TerrainModifier,Character,Player,Humanoid,Attack,HitData,BaseAI,MonsterAI,AnimalAI,Pathfinding,Piece,WearNTear,ZNet,ZNetScene,ZDO,ZDOMan,ZNetView,ZRpc,ZRoutedRpc,ZSteamSocket,ZPlayFabSocket,PlayerProfile,World,Game,Pickable,Destructible,TreeBase".Split(','));
using var index=new StreamWriter(Path.Combine(output,"methods.tsv"));
index.WriteLine("assembly\tmvid\ttype\tmethod\ttoken\trva\til_sha256\tsignature");
foreach(var path in Directory.GetFiles(Path.Combine(root,"valheim_Data","Managed"),"*.dll")) {
 using var asm=AssemblyDefinition.ReadAssembly(path);
 foreach(var t in asm.MainModule.Types) {
  foreach(var m in t.Methods) {
   string body=m.HasBody?string.Join("\n",m.Body.Instructions.Select(i=>i.ToString())):"";
   index.WriteLine($"{Path.GetFileName(path)}\t{asm.MainModule.Mvid}\t{t.FullName}\t{m.Name}\t0x{m.MetadataToken.ToUInt32():X8}\t{m.RVA}\t{Convert.ToHexString(SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(body)))}\t{m.FullName}");
  }
  if(!selected.Contains(t.Name)) continue;
  using var detail=new StreamWriter(Path.Combine(output,Path.GetFileNameWithoutExtension(path)+"-"+t.Name+".il.txt"));
  detail.WriteLine($"LOCAL EVIDENCE ONLY | assembly SHA256 {Hash(path)} | MVID {asm.MainModule.Mvid}\n{t.FullName} : {t.BaseType}");
  foreach(var f in t.Fields) detail.WriteLine($"FIELD {f.Attributes} {f.FullName} constant={f.Constant}");
  foreach(var m in t.Methods) { detail.WriteLine($"\nMETHOD {m.Attributes} {m.FullName} token={m.MetadataToken} RVA={m.RVA}"); if(m.HasBody) foreach(var i in m.Body.Instructions) detail.WriteLine(i); }
 }
}
using var bundles=new StreamWriter(Path.Combine(output,"container-headers.tsv"));
bundles.WriteLine("path\tfirst_96_bytes_hex\tprintable");
foreach(var f in files.Where(f=>f.path.Contains("Bundles") || f.path.EndsWith(".assets") || f.path.EndsWith("globalgamemanagers"))) {
 using var s=File.OpenRead(Path.Combine(root,f.path)); byte[] b=new byte[Math.Min(96,s.Length)]; s.ReadExactly(b);
 bundles.WriteLine($"{f.path}\t{Convert.ToHexString(b)}\t{new string(b.Select(c=>c>=32&&c<127?(char)c:'.').ToArray())}");
}
Console.WriteLine($"Inventoried {files.Length} files; managed methods and container headers recorded in {output}. No game code executed.");
return 0;

