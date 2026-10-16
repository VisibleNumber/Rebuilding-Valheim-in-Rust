use crate::probe::range::{tree::Value as V, Database};
use std::{collections::HashMap, path::Path};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type M = [f32; 16];
pub fn ident() -> M {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
}
pub fn mul(a: M, b: M) -> M {
    let mut c = [0.; 16];
    for j in 0..4 {
        for i in 0..4 {
            for k in 0..4 {
                c[j * 4 + i] += a[k * 4 + i] * b[j * 4 + k];
            }
        }
    }
    c
}
pub fn point(m: M, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| m[i] * p[0] + m[4 + i] * p[1] + m[8 + i] * p[2] + m[12 + i])
}
pub fn direction(m: M, p: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| m[i] * p[0] + m[4 + i] * p[1] + m[8 + i] * p[2])
}
pub fn norm(v: [f32; 3]) -> [f32; 3] {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-20);
    v.map(|x| x / n)
}
fn get<'a>(v: &'a V, k: &str) -> &'a V {
    v.get(k).unwrap_or_else(|| panic!("missing {k}"))
}
fn ptr(v: &V) -> i64 {
    assert_eq!(get(v, "m_FileID").int(), 0);
    get(v, "m_PathID").int()
}
fn xyz(v: &V) -> [f32; 3] {
    [
        get(v, "x").float(),
        get(v, "y").float(),
        get(v, "z").float(),
    ]
}
pub(crate) fn trs(v: &V) -> M {
    let t = xyz(get(v, "m_LocalPosition"));
    let s = xyz(get(v, "m_LocalScale"));
    let q = get(v, "m_LocalRotation");
    let [x, y, z] = xyz(q);
    let w = get(q, "w").float();
    [
        (1. - 2. * (y * y + z * z)) * s[0],
        2. * (x * y + z * w) * s[0],
        2. * (x * z - y * w) * s[0],
        0.,
        2. * (x * y - z * w) * s[1],
        (1. - 2. * (x * x + z * z)) * s[1],
        2. * (y * z + x * w) * s[1],
        0.,
        2. * (x * z + y * w) * s[2],
        2. * (y * z - x * w) * s[2],
        (1. - 2. * (x * x + y * y)) * s[2],
        0.,
        t[0],
        t[1],
        t[2],
        1.,
    ]
}
#[derive(Clone)]
pub struct Bone {
    pub id: i64,
    pub parent: i64,
    pub path: String,
    pub world: M,
}
#[derive(Clone)]
pub struct Vertex {
    pub p: [f32; 3],
    pub n: [f32; 3],
    pub uv: [f32; 2],
    pub weights: [f32; 4],
    pub joints: [usize; 4],
}
pub struct Part {
    pub name: String,
    pub mesh_id: i64,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bind: Vec<M>,
    pub bones: Vec<i64>,
    pub material: i64,
    pub active: bool,
}
pub struct Texture {
    pub width: i32,
    pub height: i32,
    pub mips: i32,
    pub format: i32,
    pub data: Vec<u8>,
}
pub struct Material {
    pub name: String,
    pub color: [f32; 4],
    pub emission: [f32; 3],
    pub texture: Option<Texture>,
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}
pub struct Model {
    pub parts: Vec<Part>,
    pub bones: HashMap<i64, Bone>,
    pub materials: HashMap<i64, Material>,
    pub min: [f32; 3],
    pub max: [f32; 3],
}
fn walk(
    db: &Database,
    tid: i64,
    parent: M,
    path: &str,
    active: bool,
    bones: &mut HashMap<i64, Bone>,
    renderers: &mut Vec<(i64, bool)>,
) -> Result<()> {
    let t = db.read(tid)?;
    let go = db.read(ptr(get(&t, "m_GameObject")))?;
    let name = get(&go, "m_Name").text();
    let path = if path.is_empty() {
        String::new()
    } else if path == "/" {
        name.into()
    } else {
        format!("{path}/{name}")
    };
    let root = tid == 8935594583187484414;
    let mat = if root { ident() } else { mul(parent, trs(&t)) };
    let active = active && get(&go, "m_IsActive").int() != 0;
    bones.insert(
        tid,
        Bone {
            id: tid,
            parent: ptr(get(&t, "m_Father")),
            path: path.clone(),
            world: mat,
        },
    );
    for c in get(&go, "m_Component").array() {
        let id = ptr(get(c, "component"));
        if db.objects[&id].2 == 137 {
            renderers.push((id, active));
        }
    }
    for child in get(&t, "m_Children").array() {
        walk(
            db,
            ptr(child),
            mat,
            if root { "/" } else { &path },
            active,
            bones,
            renderers,
        )?;
    }
    Ok(())
}
pub(crate) fn texture(db: &Database, id: i64) -> Result<Texture> {
    let v = db.read(id)?;
    let s = get(&v, "m_StreamData");
    let off = get(s, "offset").int() as u64;
    let len = get(s, "size").int() as usize;
    let path = get(s, "path").text();
    let data = if len > 0 {
        let n = db
            .bundle
            .nodes
            .iter()
            .find(|n| path.ends_with(&n.0))
            .ok_or("texture resource node missing")?;
        if off + len as u64 > n.2 {
            return Err("streamed texture outside resource".into());
        }
        db.bundle.read(n.1 + off, len)?
    } else {
        get(&v, "image data").bytes().to_vec()
    };
    Ok(Texture {
        width: get(&v, "m_Width").int() as i32,
        height: get(&v, "m_Height").int() as i32,
        mips: get(&v, "m_MipCount").int() as i32,
        format: get(&v, "m_TextureFormat").int() as i32,
        data,
    })
}
pub(crate) fn material(db: &Database, id: i64) -> Result<Material> {
    let v = db.read(id)?;
    let props = get(&v, "m_SavedProperties");
    let mut m = Material {
        name: get(&v, "m_Name").text().into(),
        color: [1.; 4],
        emission: [0.; 3],
        texture: None,
        hue: 0.,
        saturation: 0.,
        value: 0.,
    };
    for c in get(props, "m_Colors").array() {
        let key = get(c, "first").text();
        let val = get(c, "second");
        let color = [
            get(val, "r").float(),
            get(val, "g").float(),
            get(val, "b").float(),
            get(val, "a").float(),
        ];
        if key == "_Color" {
            m.color = color;
        }
        if key == "_EmissionColor" {
            m.emission = color[..3].try_into()?;
        }
    }
    for c in get(props, "m_Floats").array() {
        let val = get(c, "second").float();
        match get(c, "first").text() {
            "_Hue" => m.hue = val,
            "_Saturation" => m.saturation = val,
            "_Value" => m.value = val,
            _ => {}
        }
    }
    for c in get(props, "m_TexEnvs").array() {
        if get(c, "first").text() == "_MainTex" {
            let id = ptr(get(get(c, "second"), "m_Texture"));
            if id != 0 {
                m.texture = Some(texture(db, id)?);
            }
        }
    }
    Ok(m)
}
pub fn load(lab: &Path) -> Result<Model> {
    let bundle =
        lab.join("../common/Valheim/valheim_Data/StreamingAssets/SoftRef/Bundles/c4210710");
    let meta = lab.join("private-cache/greyling/serialized-c421-001");
    let db = Database::open(&bundle, &meta)?;
    let mut bones = HashMap::new();
    let mut renderers = Vec::new();
    walk(
        &db,
        8935594583187484414,
        ident(),
        "",
        true,
        &mut bones,
        &mut renderers,
    )?;
    let mut model = Model {
        parts: Vec::new(),
        bones,
        materials: HashMap::new(),
        min: [f32::INFINITY; 3],
        max: [f32::NEG_INFINITY; 3],
    };
    for (rid, active) in renderers {
        let r = db.read(rid)?;
        let go = db.read(ptr(get(&r, "m_GameObject")))?;
        let mesh_id = ptr(get(&r, "m_Mesh"));
        let m = db.read(mesh_id)?;
        let data = get(&m, "m_VertexData");
        let n = get(data, "m_VertexCount").int() as usize;
        if get(&m, "m_MeshCompression").int() != 0 {
            return Err("compressed vertices not implemented".into());
        }
        let channels = get(data, "m_Channels").array();
        let raw = get(data, "m_DataSize").bytes();
        let mut stride = [0usize; 16];
        for c in channels {
            let dim = get(c, "dimension").int() as usize & 15;
            if dim == 0 {
                continue;
            }
            let stream = get(c, "stream").int() as usize;
            let format = get(c, "format").int();
            if format != 0 && format != 10 {
                return Err(format!("unimplemented vertex format {format}").into());
            }
            stride[stream] = stride[stream].max(get(c, "offset").int() as usize + dim * 4);
        }
        let mut offsets = [0usize; 16];
        let mut end = 0;
        for s in 0..16 {
            offsets[s] = end;
            end = (end + n * stride[s] + 15) & !15;
        }
        if end != raw.len() {
            return Err(format!("stream size mismatch {end}/{}", raw.len()).into());
        }
        let read = |i: usize, ch: usize, k: usize| -> u32 {
            let c = &channels[ch];
            if k >= get(c, "dimension").int() as usize & 15 {
                return 0;
            }
            let s = get(c, "stream").int() as usize;
            let off = offsets[s] + i * stride[s] + get(c, "offset").int() as usize + 4 * k;
            u32::from_le_bytes(raw[off..off + 4].try_into().unwrap())
        };
        let implicit_weight = get(&channels[12], "dimension").int() == 0
            && get(&channels[13], "dimension").int() == 1;
        let mut vertices = Vec::new();
        for i in 0..n {
            vertices.push(Vertex {
                p: std::array::from_fn(|k| f32::from_bits(read(i, 0, k))),
                n: std::array::from_fn(|k| f32::from_bits(read(i, 1, k))),
                uv: std::array::from_fn(|k| f32::from_bits(read(i, 4, k))),
                weights: std::array::from_fn(|k| {
                    if implicit_weight {
                        if k == 0 {
                            1.
                        } else {
                            0.
                        }
                    } else {
                        f32::from_bits(read(i, 12, k))
                    }
                }),
                joints: std::array::from_fn(|k| read(i, 13, k) as usize),
            });
        }
        let index = get(&m, "m_IndexBuffer").bytes();
        let format = get(&m, "m_IndexFormat").int();
        let indices = match format {
            0 => index
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes(c.try_into().unwrap()) as u32)
                .collect::<Vec<_>>(),
            1 => index
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
                .collect(),
            _ => return Err("index format unsupported".into()),
        };
        for sub in get(&m, "m_SubMeshes").array() {
            if get(sub, "topology").int() != 0 || get(sub, "baseVertex").int() != 0 {
                return Err("submesh topology/base vertex unsupported".into());
            }
        }
        if indices.len() % 3 != 0 || indices.iter().any(|i| *i as usize >= n) {
            return Err("invalid triangle index".into());
        }
        let bind = get(&m, "m_BindPose")
            .array()
            .iter()
            .map(|v| std::array::from_fn(|i| get(v, &format!("e{}{}", i % 4, i / 4)).float()))
            .collect::<Vec<M>>();
        let bs = get(&r, "m_Bones")
            .array()
            .iter()
            .map(ptr)
            .collect::<Vec<_>>();
        if bind.len() != bs.len() {
            return Err("bind/bone count differs".into());
        }
        for v in &vertices {
            if (v.weights.iter().sum::<f32>() - 1.).abs() > 0.0001
                || v.joints.iter().any(|j| *j >= bs.len())
            {
                return Err("invalid skin weights/indices".into());
            }
        }
        let mats = get(&r, "m_Materials").array();
        if mats.len() != 1 {
            return Err("multiple materials not implemented for this slice".into());
        }
        let mat = ptr(&mats[0]);
        if !model.materials.contains_key(&mat) {
            model.materials.insert(mat, material(&db, mat)?);
        }
        let part = Part {
            name: get(&go, "m_Name").text().into(),
            mesh_id,
            vertices,
            indices,
            bind,
            bones: bs,
            material: mat,
            active,
        };
        if active {
            for (p, _) in model.skin(&part, None) {
                for k in 0..3 {
                    model.min[k] = model.min[k].min(p[k]);
                    model.max[k] = model.max[k].max(p[k]);
                }
            }
        }
        model.parts.push(part);
    }
    Ok(model)
}
impl Model {
    pub fn dump(&self, path: &Path, pose: Option<&HashMap<i64, M>>) -> Result<()> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
        writeln!(f, "part,index,x,y,z,nx,ny,nz")?;
        for part in &self.parts {
            if !part.active {
                continue;
            }
            for (i, (p, n)) in self.skin(part, pose).iter().enumerate() {
                writeln!(
                    f,
                    "{},{i},{},{},{},{},{},{}",
                    part.name, p[0], p[1], p[2], n[0], n[1], n[2]
                )?;
            }
        }
        Ok(())
    }
    pub fn skin(&self, part: &Part, pose: Option<&HashMap<i64, M>>) -> Vec<([f32; 3], [f32; 3])> {
        let matrices = part
            .bones
            .iter()
            .zip(&part.bind)
            .map(|(id, bind)| {
                mul(
                    pose.and_then(|p| p.get(id))
                        .copied()
                        .unwrap_or(self.bones[id].world),
                    *bind,
                )
            })
            .collect::<Vec<_>>();
        part.vertices
            .iter()
            .map(|v| {
                let (mut p, mut n) = ([0.; 3], [0.; 3]);
                for k in 0..4 {
                    if v.weights[k] == 0. {
                        continue;
                    }
                    let mat = matrices[v.joints[k]];
                    let a = point(mat, v.p);
                    let b = direction(mat, v.n);
                    for j in 0..3 {
                        p[j] += v.weights[k] * a[j];
                        n[j] += v.weights[k] * b[j];
                    }
                }
                (p, norm(n))
            })
            .collect()
    }
    pub fn describe(&self) -> String {
        let mut s = format!(
            "Direct installed-asset decoding; no Unity export\nBounds {:?} -> {:?}\n",
            self.min, self.max
        );
        for p in &self.parts {
            s += &format!(
                "{} mesh={} active={} vertices={} triangles={} bones={} material={}\n",
                p.name,
                p.mesh_id,
                p.active,
                p.vertices.len(),
                p.indices.len() / 3,
                p.bones.len(),
                self.materials[&p.material].name
            );
        }
        s
    }
}
pub struct Animation {
    pub duration: f32,
    pub frames: Vec<HashMap<i64, M>>,
}
impl Animation {
    pub fn load(lab: &Path, model: &Model) -> Result<Option<Self>> {
        Self::load_clip(lab, model, "idle")
    }
    pub fn load_clip(lab: &Path, model: &Model, clip: &str) -> Result<Option<Self>> {
        let current = lab.join("private-cache/greyling/native-current.txt");
        if !current.exists() {
            return Ok(None);
        }
        let dir = std::fs::read_to_string(current)?;
        let dir = Path::new(dir.trim());
        if !dir.join("SUCCESS.txt").exists() {
            return Err("Native pose cache has no successful reference marker".into());
        }
        let paths = std::fs::read_to_string(dir.join("pose-paths.txt"))?;
        let paths = paths.lines().collect::<Vec<_>>();
        let file = dir.join(format!("native-{clip}-poses.bin"));
        if !file.exists() {
            return Ok(None);
        }
        let data = std::fs::read(file)?;
        if data.len() < 16 || &data[..4] != b"GPO1" {
            return Err("pose cache header".into());
        }
        let count = u32::from_le_bytes(data[4..8].try_into()?) as usize;
        let bones = u32::from_le_bytes(data[8..12].try_into()?) as usize;
        let duration = f32::from_le_bytes(data[12..16].try_into()?);
        if count < 2
            || count > 2000
            || bones != paths.len()
            || bones > 500
            || !duration.is_finite()
            || duration <= 0.
            || duration > 60.
            || data.len() != 16 + count * bones * 64
        {
            return Err("pose cache size/count mismatch".into());
        }
        let ids = paths
            .iter()
            .map(|path| {
                model
                    .bones
                    .iter()
                    .find(|(_, b)| b.path == *path)
                    .map(|(id, _)| *id)
            })
            .collect::<Vec<_>>();
        let mut frames = Vec::new();
        for f in 0..count {
            let mut frame = HashMap::new();
            for b in 0..bones {
                if let Some(id) = ids[b] {
                    let mut mat = [0.; 16];
                    for k in 0..16 {
                        let p = 16 + ((f * bones + b) * 16 + k) * 4;
                        mat[k] = f32::from_le_bytes(data[p..p + 4].try_into()?);
                        if !mat[k].is_finite() {
                            return Err("nonfinite pose matrix".into());
                        }
                    }
                    frame.insert(id, mat);
                }
            }
            for part in &model.parts {
                for bone in &part.bones {
                    if !frame.contains_key(bone) {
                        return Err("pose missing a skin bone".into());
                    }
                }
            }
            frames.push(frame);
        }
        Ok(Some(Self { duration, frames }))
    }
}
