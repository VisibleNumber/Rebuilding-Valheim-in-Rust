use super::*;
#[path = "tree.rs"]
pub mod tree;
pub struct Database {
    pub bundle: Bundle,
    pub objects: std::collections::HashMap<i64, (u64, usize, i32, i32)>,
    trees: std::collections::HashMap<i32, (Vec<u8>, Vec<u8>)>,
}
impl Database {
    pub fn open(bundle: &Path, metadata: &Path) -> R<Self> {
        let b = Bundle::open(bundle)?;
        let ix = fs::read_to_string(metadata.join("objects.tsv"))?;
        let mut objects = std::collections::HashMap::new();
        for line in ix.lines().skip(1) {
            let c = line.split('\t').collect::<Vec<_>>();
            objects.insert(
                c[0].parse()?,
                (c[1].parse()?, c[2].parse()?, c[3].parse()?, c[4].parse()?),
            );
        }
        let mut trees = std::collections::HashMap::new();
        for line in fs::read_to_string(metadata.join("types.tsv"))?
            .lines()
            .skip(1)
        {
            let c = line.split('\t').collect::<Vec<_>>();
            let ti: i32 = c[0].parse()?;
            let cl: i32 = c[1].parse()?;
            trees.insert(
                ti,
                (
                    fs::read(metadata.join(format!("tree-{ti}-{cl}.bin")))?,
                    fs::read(metadata.join(format!("strings-{ti}-{cl}.bin")))?,
                ),
            );
        }
        Ok(Self {
            bundle: b,
            objects,
            trees,
        })
    }
    pub fn read(&self, id: i64) -> R<tree::Value> {
        let &(off, len, _, ti) = self
            .objects
            .get(&id)
            .ok_or_else(|| bad(&format!("missing object {id}")))?;
        let data = self.bundle.read(self.bundle.nodes[0].1 + off, len)?;
        let (t, s) = &self.trees[&ti];
        tree::read(t, s, &data)
    }
}
pub fn object(bundle: &Path, metadata: &Path, id: i64) -> R<tree::Value> {
    let ix = fs::read_to_string(metadata.join("objects.tsv"))?;
    let line = ix
        .lines()
        .skip(1)
        .find(|l| l.split('\t').next().and_then(|s| s.parse::<i64>().ok()) == Some(id))
        .ok_or_else(|| bad("object not found"))?;
    let c = line.split('\t').collect::<Vec<_>>();
    let off: u64 = c[1].parse()?;
    let len: usize = c[2].parse()?;
    let class: i32 = c[3].parse()?;
    let ti: i32 = c[4].parse()?;
    let b = Bundle::open(bundle)?;
    let data = b.read(b.nodes[0].1 + off, len)?;
    let t = fs::read(metadata.join(format!("tree-{ti}-{class}.bin")))?;
    let s = fs::read(metadata.join(format!("strings-{ti}-{class}.bin")))?;
    tree::read(&t, &s, &data)
}
pub struct Bundle {
    path: std::path::PathBuf,
    blocks: Vec<(u64, u64, u32, u32, u16)>,
    pub nodes: Vec<(String, u64, u64)>,
}
impl Bundle {
    pub fn open(path: &Path) -> R<Self> {
        let mut f = File::open(path)?;
        let size = f.metadata()?.len();
        if string(&mut f)? != "UnityFS" {
            return Err(bad("not UnityFS"));
        }
        let v = u32be(&mut f)?;
        if v != 8 {
            return Err(bad("range reader requires observed UnityFS 8"));
        }
        string(&mut f)?;
        string(&mut f)?;
        if u64be(&mut f)? != size {
            return Err(bad("bundle size mismatch"));
        }
        let packed = u32be(&mut f)?;
        let raw = u32be(&mut f)?;
        let flags = u32be(&mut f)?;
        let p = (f.stream_position()? + 15) & !15;
        f.seek(SeekFrom::Start(if flags & 128 != 0 {
            size - packed as u64
        } else {
            p
        }))?;
        let meta = decode(bytes(&mut f, packed as usize)?, raw as usize, flags & 63)?;
        let mut at = if flags & 128 != 0 {
            p
        } else {
            f.stream_position()?
        };
        if flags & 512 != 0 {
            at = (at + 15) & !15;
        }
        let mut r = io::Cursor::new(meta);
        bytes(&mut r, 16)?;
        let n = u32be(&mut r)?;
        if n > 1_000_000 {
            return Err(bad("block count"));
        }
        let mut blocks = Vec::new();
        let mut offset = 0;
        for _ in 0..n {
            let raw = u32be(&mut r)?;
            let packed = u32be(&mut r)?;
            let codec = u16be(&mut r)?;
            blocks.push((offset, at, raw, packed, codec));
            offset += raw as u64;
            at += packed as u64;
        }
        let n = u32be(&mut r)?;
        if n > 1_000_000 {
            return Err(bad("node count"));
        }
        let mut nodes = Vec::new();
        for _ in 0..n {
            let off = u64be(&mut r)?;
            let len = u64be(&mut r)?;
            u32be(&mut r)?;
            let name = string(&mut r)?;
            if off.checked_add(len).is_none_or(|v| v > offset) {
                return Err(bad("invalid node range"));
            }
            nodes.push((name, off, len));
        }
        Ok(Self {
            path: path.to_owned(),
            blocks,
            nodes,
        })
    }
    pub fn read(&self, offset: u64, len: usize) -> R<Vec<u8>> {
        if len > LIMIT {
            return Err(bad("range exceeds 64MiB"));
        }
        let end = offset
            .checked_add(len as u64)
            .ok_or_else(|| bad("range overflow"))?;
        let mut out = Vec::with_capacity(len);
        let mut f = File::open(&self.path)?;
        for &(base, pos, raw, packed, codec) in &self.blocks {
            if base >= end {
                break;
            }
            if base + raw as u64 <= offset {
                continue;
            }
            f.seek(SeekFrom::Start(pos))?;
            let b = decode(
                bytes(&mut f, packed as usize)?,
                raw as usize,
                (codec & 63) as u32,
            )?;
            let a = offset.saturating_sub(base) as usize;
            let z = (end - base).min(raw as u64) as usize;
            out.extend_from_slice(&b[a..z]);
        }
        if out.len() != len {
            return Err(bad("incomplete requested range"));
        }
        Ok(out)
    }
}
struct Le<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Le<'a> {
    fn take(&mut self, n: usize) -> R<&'a [u8]> {
        let e = self
            .p
            .checked_add(n)
            .ok_or_else(|| bad("metadata overflow"))?;
        let b = self
            .b
            .get(self.p..e)
            .ok_or_else(|| bad("truncated metadata"))?;
        self.p = e;
        Ok(b)
    }
    fn byte(&mut self) -> R<u8> {
        Ok(self.take(1)?[0])
    }
    fn i32(&mut self) -> R<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn u32(&mut self) -> R<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn i64(&mut self) -> R<i64> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into()?))
    }
    fn z(&mut self) -> R<String> {
        let p = self.p;
        while self.byte()? != 0 {}
        Ok(String::from_utf8(self.b[p..self.p - 1].to_vec())?)
    }
    fn aligned_string(&mut self) -> R<String> {
        let n = self.i32()?;
        if !(0..=4096).contains(&n) {
            return Err(bad("invalid string length"));
        }
        let s = String::from_utf8(self.take(n as usize)?.to_vec())?;
        self.align();
        Ok(s)
    }
    fn align(&mut self) {
        self.p = (self.p + 3) & !3;
    }
}
pub fn inspect(path: &Path, out: &Path) -> R<()> {
    if out.exists() {
        return Err(bad("choose fresh output folder"));
    }
    fs::create_dir_all(out)?;
    let b = Bundle::open(path)?;
    let node = &b.nodes[0];
    let h = b.read(node.1, 48)?;
    if u32::from_be_bytes(h[8..12].try_into()?) != 22 || h[16] != 0 {
        return Err(bad("requires observed serialized v22 little-endian"));
    }
    let meta = u32::from_be_bytes(h[20..24].try_into()?) as usize;
    let data = u64::from_be_bytes(h[32..40].try_into()?);
    let all = b.read(node.1, 48 + meta)?;
    let mut r = Le { b: &all, p: 48 };
    let unity = r.z()?;
    let platform = r.i32()?;
    let trees = r.byte()? != 0;
    let nt = r.i32()?;
    if !(0..=100000).contains(&nt) {
        return Err(bad("type count"));
    }
    let mut types = Vec::new();
    let mut tf = File::create(out.join("types.tsv"))?;
    writeln!(tf, "index\tclass_id\tnodes\tstring_bytes")?;
    for ti in 0..nt {
        let class = r.i32()?;
        r.byte()?;
        r.take(2)?;
        if class == 114 {
            r.take(16)?;
        }
        r.take(16)?;
        types.push(class);
        let (mut nodes, mut strings) = (0, 0);
        if trees {
            nodes = r.i32()?;
            strings = r.i32()?;
            if nodes < 0 || strings < 0 {
                return Err(bad("negative tree size"));
            }
            let tree = r.take(nodes as usize * 32)?;
            let st = r.take(strings as usize)?;
            fs::write(out.join(format!("tree-{ti}-{class}.bin")), tree)?;
            fs::write(out.join(format!("strings-{ti}-{class}.bin")), st)?;
            let deps = r.i32()?;
            if deps < 0 {
                return Err(bad("negative dependencies"));
            }
            r.take(deps as usize * 4)?;
        }
        writeln!(tf, "{ti}\t{class}\t{nodes}\t{strings}")?;
    }
    let count = r.i32()?;
    if !(0..=5_000_000).contains(&count) {
        return Err(bad("object count"));
    }
    let mut objects = Vec::new();
    for _ in 0..count {
        r.align();
        let id = r.i64()?;
        let off = r.i64()?;
        let len = r.u32()?;
        let ti = r.i32()?;
        if off < 0 || ti < 0 || ti as usize >= types.len() {
            return Err(bad("bad object entry"));
        }
        objects.push((id, data + off as u64, len, types[ti as usize], ti));
    }
    let mut ix = File::create(out.join("objects.tsv"))?;
    writeln!(ix, "path_id\toffset\tsize\tclass_id\ttype_index")?;
    for o in &objects {
        writeln!(ix, "{}\t{}\t{}\t{}\t{}", o.0, o.1, o.2, o.3, o.4)?;
    }
    let scripts = r.i32()?;
    for _ in 0..scripts {
        r.i32()?;
        r.align();
        r.i64()?;
    }
    let ex = r.i32()?;
    let mut ef = File::create(out.join("externals.tsv"))?;
    writeln!(ef, "file_id\tpath")?;
    for i in 1..=ex {
        r.z()?;
        r.take(16)?;
        r.i32()?;
        writeln!(ef, "{i}\t{}", r.z()?)?;
    }
    let mut containers = File::create(out.join("container.tsv"))?;
    writeln!(containers, "asset_path\tfile_id\tpath_id")?;
    for &(id, off, len, class, _) in &objects {
        if class != 142 {
            continue;
        }
        let bytes = b.read(node.1 + off, len as usize)?;
        let mut ar = Le { b: &bytes, p: 0 };
        let name = ar.aligned_string()?;
        let preload = ar.i32()?;
        if preload < 0 {
            return Err(bad("preload count"));
        }
        ar.take(preload as usize * 12)?;
        let n = ar.i32()?;
        for _ in 0..n {
            let name = ar.aligned_string()?;
            ar.i32()?;
            ar.i32()?;
            let file = ar.i32()?;
            let target = ar.i64()?;
            writeln!(containers, "{name}\t{file}\t{target}")?;
        }
        println!("AssetBundle {name}, object {id}: {n} container entries");
    }
    fs::write(out.join("summary.txt"),format!("bundle={}\nnode={}\nunity={unity}\nplatform={platform}\nserialized_version=22\ntype_trees={trees}\nmetadata_bytes={meta}\ndata_offset={data}\ntypes={nt}\nobjects={count}\nexternals={ex}\n",path.display(),node.0))?;
    println!("Serialized metadata: Unity {unity}, trees={trees}, {nt} types, {count} objects, {ex} external files");
    Ok(())
}
