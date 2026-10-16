use crate::{
    model::{self, ident, mul, norm, point, Material, Result, Texture, Vertex, M},
    probe::range::{tree::Value as V, Database},
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
fn g<'a>(v: &'a V, k: &str) -> &'a V {
    v.get(k)
        .unwrap_or_else(|| panic!("missing static asset field {k}"))
}
fn ptr(v: &V) -> i64 {
    assert_eq!(g(v, "m_FileID").int(), 0, "external static asset reference");
    g(v, "m_PathID").int()
}
pub struct StaticMaterial {
    pub base: Material,
    pub cutoff: f32,
    pub uv: [f32; 4],
}
pub struct StaticPart {
    pub name: String,
    pub mesh: i64,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: i64,
}
pub struct Asset {
    pub name: String,
    pub parts: Vec<StaticPart>,
    pub min: [f32; 3],
    pub max: [f32; 3],
}
pub struct Environment {
    pub assets: Vec<Asset>,
    pub materials: HashMap<i64, StaticMaterial>,
    pub grass: Texture,
    pub ground_layers: Vec<Texture>,
    pub ground_uv: f32,
}
fn normal(m: M, v: [f32; 3]) -> [f32; 3] {
    let a = [m[0], m[1], m[2]];
    let b = [m[4], m[5], m[6]];
    let c = [m[8], m[9], m[10]];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let x = cross(b, c);
    let y = cross(c, a);
    let z = cross(a, b);
    let det = a[0] * x[0] + a[1] * x[1] + a[2] * x[2];
    norm(std::array::from_fn(|i| {
        (x[i] * v[0] + y[i] * v[1] + z[i] * v[2]) / det
    }))
}
fn floats(v: &V, name: &str, default: f32) -> f32 {
    g(g(v, "m_SavedProperties"), "m_Floats")
        .array()
        .iter()
        .find(|p| g(p, "first").text() == name)
        .map(|p| g(p, "second").float())
        .unwrap_or(default)
}
fn decode_mesh(
    db: &Database,
    id: i64,
    matrix: M,
    name: &str,
    mats: &[V],
    materials: &mut HashMap<i64, StaticMaterial>,
) -> Result<Vec<StaticPart>> {
    let m = db.read(id)?;
    if g(&m, "m_MeshCompression").int() != 0 {
        return Err("static compressed mesh unsupported".into());
    }
    let data = g(&m, "m_VertexData");
    let n = g(data, "m_VertexCount").int() as usize;
    let channels = g(data, "m_Channels").array();
    let raw = g(data, "m_DataSize").bytes();
    if g(g(&m, "m_StreamData"), "size").int() != 0 {
        return Err("static streamed vertex data unsupported".into());
    }
    let mut strides = [0usize; 16];
    for ch in channels {
        let d = g(ch, "dimension").int() as usize & 15;
        if d == 0 {
            continue;
        }
        if g(ch, "format").int() != 0 {
            return Err("static non-Float32 vertex declaration unsupported".into());
        }
        let s = g(ch, "stream").int() as usize;
        strides[s] = strides[s].max(g(ch, "offset").int() as usize + d * 4);
    }
    let mut offsets = [0usize; 16];
    let mut end = 0;
    for s in 0..16 {
        offsets[s] = end;
        end = (end + n * strides[s] + 15) & !15;
    }
    if end != raw.len() {
        return Err("static vertex stream size mismatch".into());
    }
    let read = |i: usize, c: usize, k: usize| {
        let ch = &channels[c];
        let s = g(ch, "stream").int() as usize;
        let off = offsets[s] + i * strides[s] + g(ch, "offset").int() as usize + k * 4;
        f32::from_le_bytes(raw[off..off + 4].try_into().unwrap())
    };
    let vertices = (0..n)
        .map(|i| Vertex {
            p: point(matrix, std::array::from_fn(|k| read(i, 0, k))),
            n: normal(matrix, std::array::from_fn(|k| read(i, 1, k))),
            uv: std::array::from_fn(|k| read(i, 4, k)),
            weights: [0.; 4],
            joints: [0; 4],
        })
        .collect::<Vec<_>>();
    let index = g(&m, "m_IndexBuffer").bytes();
    let isize = if g(&m, "m_IndexFormat").int() == 0 {
        2
    } else {
        4
    };
    let mut parts = Vec::new();
    for (sub, s) in g(&m, "m_SubMeshes").array().iter().enumerate() {
        if g(s, "topology").int() != 0 {
            return Err("nontriangle static mesh".into());
        }
        let first = g(s, "firstByte").int() as usize;
        let count = g(s, "indexCount").int() as usize;
        let base = g(s, "baseVertex").int() as u32;
        let indices = index[first..first + count * isize]
            .chunks_exact(isize)
            .map(|v| {
                base + if isize == 2 {
                    u16::from_le_bytes(v.try_into().unwrap()) as u32
                } else {
                    u32::from_le_bytes(v.try_into().unwrap())
                }
            })
            .collect::<Vec<_>>();
        if indices.iter().any(|i| *i as usize >= n) {
            return Err("static index outside vertices".into());
        }
        let mat = ptr(mats.get(sub).ok_or("missing submesh material")?);
        if !materials.contains_key(&mat) {
            let v = db.read(mat)?;
            let base = model::material(db, mat)?;
            let mut uv = [1., 1., 0., 0.];
            for p in g(g(&v, "m_SavedProperties"), "m_TexEnvs").array() {
                if g(p, "first").text() == "_MainTex" {
                    let e = g(p, "second");
                    uv = [
                        g(g(e, "m_Scale"), "x").float(),
                        g(g(e, "m_Scale"), "y").float(),
                        g(g(e, "m_Offset"), "x").float(),
                        g(g(e, "m_Offset"), "y").float(),
                    ];
                }
            }
            let cutoff = if base.name.starts_with("Rocks") {
                0.
            } else {
                floats(&v, "_Cutoff", 0.)
            };
            materials.insert(
                mat,
                StaticMaterial {
                    base,
                    // Native Custom/StaticRock has no alpha-test property; alpha is not transparency.
                    cutoff,
                    uv,
                },
            );
        }
        parts.push(StaticPart {
            name: format!("{name}#{sub}"),
            mesh: id,
            vertices: vertices.clone(),
            indices,
            material: mat,
        });
    }
    Ok(parts)
}
fn walk(
    db: &Database,
    tid: i64,
    parent: M,
    path: &str,
    root: bool,
    active: bool,
    selected: &mut HashSet<i64>,
    excluded: &mut HashSet<i64>,
    renders: &mut Vec<(i64, i64, M, String)>,
    materials: &mut HashMap<i64, StaticMaterial>,
) -> Result<()> {
    let t = db.read(tid)?;
    let go = db.read(ptr(g(&t, "m_GameObject")))?;
    let name = g(&go, "m_Name").text();
    let path = if root {
        String::new()
    } else if path.is_empty() {
        name.into()
    } else {
        format!("{path}/{name}")
    };
    let matrix = if root {
        ident()
    } else {
        mul(parent, model::trs(&t))
    };
    let active = active && g(&go, "m_IsActive").int() != 0;
    if !active {
        return Ok(());
    }
    let mut filter = None;
    let mut renderer = None;
    for c in g(&go, "m_Component").array() {
        let id = ptr(g(c, "component"));
        match db.objects[&id].2 {
            33 => filter = Some(id),
            23 => renderer = Some(id),
            205 => {
                let lod = db.read(id)?;
                for (i, l) in g(&lod, "m_LODs").array().iter().enumerate() {
                    for r in g(l, "renderers").array() {
                        let id = ptr(g(r, "renderer"));
                        if i == 0 {
                            selected.insert(id);
                        } else {
                            excluded.insert(id);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if active {
        if let (Some(mesh), Some(renderer)) = (filter, renderer) {
            if g(&db.read(renderer)?, "m_Enabled").int() != 0 {
                renders.push((renderer, mesh, matrix, path.clone()));
            }
        }
    }
    for c in g(&t, "m_Children").array() {
        walk(
            db,
            ptr(c),
            matrix,
            &path,
            false,
            active,
            selected,
            excluded,
            renders,
            materials,
        )?;
    }
    Ok(())
}
pub fn load(lab: &Path) -> Result<Environment> {
    let db = Database::open(
        &lab.join("../common/Valheim/valheim_Data/StreamingAssets/SoftRef/Bundles/c4210710"),
        &lab.join("private-cache/greyling/serialized-c421-001"),
    )?;
    let mut materials = HashMap::new();
    let mut assets = Vec::new();
    for (name, id) in [
        ("Beech1", -3545795938103434052),
        ("Wood", 4592554523128777832),
        ("wood_floor", 2057548747335490449),
        ("Rock_7_meadows", 3667914353002387897),
        ("Bush01", -5936258400036428677),
        ("Rock_3", -8183402743714650253),
        ("Rock_4", 394639990998974457),
        ("Beech_small1", 630817206670373515),
        ("Beech_small2", -2172994950688176601),
        ("Birch1", 3544524336112848001),
    ] {
        assets.push(load_asset(&db, name, id, &mut materials)?);
    }
    // Observed installed Heightmap material legacy _MainTex, not the full Texture2DArray biome shader.
    let terrain = db.read(-1531801132878023527)?;
    let grass = model::texture(&db, 180843422271574645)?;
    Ok(Environment {
        assets,
        materials,
        grass,
        ground_layers: crate::terrain_material::layers(&db)?,
        ground_uv: floats(&terrain, "_UVScale", 0.5),
    })
}
impl Asset {
    pub fn dump(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        let mut w = std::fs::File::create(path)?;
        writeln!(w, "part,index,x,y,z,nx,ny,nz")?;
        for p in &self.parts {
            for (i, v) in p.vertices.iter().enumerate() {
                writeln!(
                    w,
                    "{},{},{},{},{},{},{},{}",
                    p.name, i, v.p[0], v.p[1], v.p[2], v.n[0], v.n[1], v.n[2]
                )?;
            }
        }
        Ok(())
    }
}

impl Environment {
    pub fn reference_map(&self, path: &Path) -> Result<()> {
        use std::io::Write;
        let mut w = std::fs::File::create(path)?;
        writeln!(w,"asset\tpart\tmesh_path_id\tmaterial_path_id\tmaterial\tcutoff\tvertices\ttriangles\ttexture_width\ttexture_height\ttexture_format")?;
        for a in &self.assets {
            for p in &a.parts {
                let m = &self.materials[&p.material];
                let t = m.base.texture.as_ref();
                writeln!(
                    w,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    a.name,
                    p.name,
                    p.mesh,
                    p.material,
                    m.base.name,
                    m.cutoff,
                    p.vertices.len(),
                    p.indices.len() / 3,
                    t.map(|t| t.width).unwrap_or(0),
                    t.map(|t| t.height).unwrap_or(0),
                    t.map(|t| t.format).unwrap_or(0)
                )?;
            }
        }
        Ok(())
    }
}

pub(crate) fn load_asset(
    db: &Database,
    name: &str,
    id: i64,
    materials: &mut HashMap<i64, StaticMaterial>,
) -> Result<Asset> {
    let go = db.read(id)?;
    let tid = g(&go, "m_Component")
        .array()
        .iter()
        .map(|c| ptr(g(c, "component")))
        .find(|id| db.objects[id].2 == 4)
        .ok_or("prefab transform missing")?;
    let (mut selected, mut excluded, mut renders) = (HashSet::new(), HashSet::new(), Vec::new());
    walk(
        db,
        tid,
        ident(),
        "",
        true,
        true,
        &mut selected,
        &mut excluded,
        &mut renders,
        materials,
    )?;
    let mut parts = Vec::new();
    for (rid, mesh, matrix, path) in renders {
        if excluded.contains(&rid) && !selected.contains(&rid) {
            continue;
        }
        let r = db.read(rid)?;
        let mesh = ptr(g(&db.read(mesh)?, "m_Mesh"));
        parts.extend(decode_mesh(
            db,
            mesh,
            matrix,
            &path,
            g(&r, "m_Materials").array(),
            materials,
        )?);
    }
    let (mut min, mut max) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for p in &parts {
        for v in &p.vertices {
            for k in 0..3 {
                min[k] = min[k].min(v.p[k]);
                max[k] = max[k].max(v.p[k]);
            }
        }
    }
    println!(
        "Original {name}: {} submeshes bounds {:?} -> {:?}",
        parts.len(),
        min,
        max
    );
    Ok(Asset {
        name: name.into(),
        parts,
        min,
        max,
    })
}
