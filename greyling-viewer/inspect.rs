#[path="../asset-probe/main.rs"]mod probe;
use probe::range::{Database,tree::Value as V};
use std::{fs,io::Write,path::Path,collections::{HashSet,VecDeque}};
fn ptr(v:&V)->i64{assert_eq!(v.get("m_FileID").unwrap().int(),0,"external ref requires separate database");v.get("m_PathID").unwrap().int()}
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a=std::env::args().collect::<Vec<_>>();let db=Database::open(Path::new(&a[1]),Path::new(&a[2]))?;let out=Path::new(&a[3]);if out.exists(){return Err("choose fresh output".into());}fs::create_dir_all(out)?;
 let mut q=VecDeque::from([1067265541275873022i64]);let mut seen=HashSet::new();let mut map=fs::File::create(out.join("graph.tsv"))?;writeln!(map,"path_id\tclass\tname")?;
 while let Some(id)=q.pop_front(){if id==0||!seen.insert(id){continue;}let cl=db.objects[&id].2;
  if ![1,4,23,33,95,137,43,21,74,91,221,90,28].contains(&cl){writeln!(map,"{id}\t{cl}\t(component not decoded in visual traversal)")?;continue;}
  let v=db.read(id).map_err(|e|format!("Object {id} class {cl}: {e}"))?;let name=v.get("m_Name").map(|v|v.text()).unwrap_or("");writeln!(map,"{id}\t{cl}\t{name}")?;fs::write(out.join(format!("{id}-{cl}.json")),v.json())?;
  if cl==1{for c in v.get("m_Component").unwrap().array(){q.push_back(ptr(c.get("component").unwrap()));}}
  if cl==4 {q.push_back(ptr(v.get("m_GameObject").unwrap()));for c in v.get("m_Children").unwrap().array(){q.push_back(ptr(c));}}
  for field in ["m_Mesh","m_Controller","m_Avatar"]{if let Some(p)=v.get(field){if p.get("m_FileID").is_some_and(|v|v.int()==0){q.push_back(ptr(p));}}}
  for field in ["m_Materials","m_AnimationClips"]{if let Some(p)=v.get(field){for p in p.array(){if p.get("m_FileID").unwrap().int()==0{q.push_back(ptr(p));}}}}
 }
 println!("Visited {} referenced objects",seen.len());Ok(())
}
