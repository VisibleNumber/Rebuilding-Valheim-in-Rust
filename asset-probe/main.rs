//! Read-only UnityFS metadata probe. No game payloads are written to disk.
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};
pub mod range;
type R<T> = Result<T, Box<dyn std::error::Error>>;
const LIMIT: usize = 64 * 1024 * 1024;
fn bad(s: &str) -> Box<dyn std::error::Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn bytes(r: &mut impl Read, n: usize) -> R<Vec<u8>> {
    if n > LIMIT {
        return Err(bad("metadata/block exceeds 64 MiB bound"));
    }
    let mut b = vec![0; n];
    r.read_exact(&mut b)?;
    Ok(b)
}
fn u16be(r: &mut impl Read) -> R<u16> {
    let mut b = [0; 2];
    r.read_exact(&mut b)?;
    Ok(u16::from_be_bytes(b))
}
fn u32be(r: &mut impl Read) -> R<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_be_bytes(b))
}
fn u64be(r: &mut impl Read) -> R<u64> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_be_bytes(b))
}
fn string(r: &mut impl Read) -> R<String> {
    let mut b = Vec::new();
    loop {
        let mut c = [0];
        r.read_exact(&mut c)?;
        if c[0] == 0 {
            break;
        }
        if b.len() > 4096 {
            return Err(bad("unterminated string"));
        }
        b.push(c[0]);
    }
    Ok(String::from_utf8(b)?)
}
fn length(src: &[u8], p: &mut usize, mut n: usize) -> R<usize> {
    if n == 15 {
        loop {
            let b = *src.get(*p).ok_or_else(|| bad("truncated LZ4 length"))? as usize;
            *p += 1;
            n = n.checked_add(b).ok_or_else(|| bad("length overflow"))?;
            if b != 255 {
                break;
            }
        }
    }
    Ok(n)
}
fn lz4(src: &[u8], expected: usize) -> R<Vec<u8>> {
    if expected > LIMIT {
        return Err(bad("LZ4 output too large"));
    }
    let mut out = Vec::with_capacity(expected);
    let mut p = 0;
    while p < src.len() {
        let token = src[p];
        p += 1;
        let n = length(src, &mut p, (token >> 4) as usize)?;
        if p.checked_add(n).is_none_or(|end| end > src.len()) || out.len() + n > expected {
            return Err(bad("invalid literal span"));
        }
        out.extend_from_slice(&src[p..p + n]);
        p += n;
        if p == src.len() {
            break;
        }
        let lo = *src.get(p).ok_or_else(|| bad("missing offset"))? as usize;
        let hi = *src.get(p + 1).ok_or_else(|| bad("missing offset"))? as usize;
        p += 2;
        let back = lo | (hi << 8);
        if back == 0 || back > out.len() {
            return Err(bad("invalid LZ4 back reference"));
        }
        let n = length(src, &mut p, (token & 15) as usize)?
            .checked_add(4)
            .ok_or_else(|| bad("match overflow"))?;
        if out.len() + n > expected {
            return Err(bad("LZ4 exceeds declared output"));
        }
        for _ in 0..n {
            let c = out[out.len() - back];
            out.push(c);
        }
    }
    if out.len() != expected {
        return Err(bad("LZ4 size mismatch"));
    }
    Ok(out)
}
fn decode(src: Vec<u8>, expected: usize, codec: u32) -> R<Vec<u8>> {
    match codec {
        0 if src.len() == expected => Ok(src),
        2 | 3 => lz4(&src, expected),
        _ => Err(bad(&format!("unsupported compression {codec}"))),
    }
}
struct Block {
    raw: u32,
    packed: u32,
    codec: u16,
}
fn probe(path: &Path, out: &mut impl Write) -> R<()> {
    let mut f = File::open(path)?;
    let actual = f.metadata()?.len();
    if string(&mut f)? != "UnityFS" {
        return Err(bad("not UnityFS"));
    }
    let version = u32be(&mut f)?;
    if !(6..=8).contains(&version) {
        return Err(bad("unreviewed UnityFS version"));
    }
    let player = string(&mut f)?;
    let editor = string(&mut f)?;
    let declared = u64be(&mut f)?;
    let packed = u32be(&mut f)?;
    let raw = u32be(&mut f)?;
    let flags = u32be(&mut f)?;
    if declared != actual {
        return Err(bad("bundle file-size mismatch"));
    }
    if flags & 0x40 == 0 {
        return Err(bad("separate block/directory metadata unsupported"));
    }
    if version >= 7 {
        let p = f.stream_position()?;
        f.seek(SeekFrom::Start((p + 15) & !15))?;
    }
    let header_end = f.stream_position()?;
    if flags & 0x80 != 0 {
        f.seek(SeekFrom::Start(
            actual
                .checked_sub(packed as u64)
                .ok_or_else(|| bad("metadata outside file"))?,
        ))?;
    }
    let meta = decode(bytes(&mut f, packed as usize)?, raw as usize, flags & 63)?;
    let mut data_start = if flags & 0x80 != 0 {
        header_end
    } else {
        f.stream_position()?
    };
    if flags & 0x200 != 0 {
        data_start = (data_start + 15) & !15;
    }
    let mut r = io::Cursor::new(meta);
    bytes(&mut r, 16)?;
    let n = u32be(&mut r)?;
    if n > 1_000_000 {
        return Err(bad("too many blocks"));
    }
    let mut blocks = Vec::new();
    let mut total_raw = 0u64;
    let mut total_packed = 0u64;
    for _ in 0..n {
        let b = Block {
            raw: u32be(&mut r)?,
            packed: u32be(&mut r)?,
            codec: u16be(&mut r)?,
        };
        total_raw = total_raw
            .checked_add(b.raw as u64)
            .ok_or_else(|| bad("raw size overflow"))?;
        total_packed += b.packed as u64;
        blocks.push(b);
    }
    let data_end = data_start
        .checked_add(total_packed)
        .ok_or_else(|| bad("packed size overflow"))?;
    let limit = if flags & 0x80 != 0 {
        actual - packed as u64
    } else {
        actual
    };
    if data_end > limit {
        return Err(bad("payload blocks exceed file"));
    }
    let nodes = u32be(&mut r)?;
    if nodes > 1_000_000 {
        return Err(bad("too many directory nodes"));
    }
    // Decode only the first block to confirm that a serialized-file header is accessible.
    f.seek(SeekFrom::Start(data_start))?;
    let prefix = if let Some(b) = blocks.first() {
        decode(
            bytes(&mut f, b.packed as usize)?,
            b.raw as usize,
            (b.codec & 63) as u32,
        )?
    } else {
        Vec::new()
    };
    for _ in 0..nodes {
        let offset = u64be(&mut r)?;
        let size = u64be(&mut r)?;
        let node_flags = u32be(&mut r)?;
        let name = string(&mut r)?;
        if offset.checked_add(size).is_none_or(|end| end > total_raw) {
            return Err(bad("node exceeds uncompressed data"));
        }
        let serialized = if offset == 0 && prefix.len() >= 20 {
            u32::from_be_bytes(prefix[8..12].try_into().unwrap()).to_string()
        } else {
            String::new()
        };
        writeln!(out,"{}\t{version}\t{player}\t{editor}\t{declared}\t{flags:#x}\t{n}\t{nodes}\t{total_raw}\t{offset}\t{size}\t{node_flags:#x}\t{name}\t{serialized}",path.file_name().unwrap().to_string_lossy())?;
    }
    Ok(())
}
fn main() -> R<()> {
    let a: Vec<_> = env::args().collect();
    if a.len() == 6 && a[1] == "--object" {
        let v = range::object(Path::new(&a[2]), Path::new(&a[3]), a[4].parse()?)?;
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&a[5])?;
        write!(out, "{}", v.json())?;
        return Ok(());
    }
    let a: Vec<_> = env::args_os().collect();
    if a.len() == 4 && a[1] == "--inspect-bundle" {
        return range::inspect(Path::new(&a[2]), Path::new(&a[3]));
    }
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 3 {
        return Err(bad(
            "Usage: asset-probe <installed SoftRef/Bundles directory> <NEW report.tsv>",
        ));
    }
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    writeln!(out,"bundle\tunityfs_version\tplayer\teditor\tfile_size\tflags\tblocks\tnodes\traw_size\tnode_offset\tnode_size\tnode_flags\tnode_name\tfirst_node_serialized_version")?;
    let mut paths = fs::read_dir(&args[1])?
        .map(|x| x.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut count = 0;
    let mut failed = 0;
    for p in paths.into_iter().filter(|p| p.is_file()) {
        match probe(&p, &mut out) {
            Ok(()) => count += 1,
            Err(e) => {
                failed += 1;
                eprintln!("{}: {e}", p.display());
            }
        }
    }
    println!("Read {count} bundles; {failed} failed. Metadata only; no payload exported.");
    if failed > 0 {
        return Err(bad("one or more bundles failed inspection"));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlap_match() {
        assert_eq!(lz4(&[0x11, b'a', 1, 0], 6).unwrap(), b"aaaaaa");
    }
    #[test]
    fn rejects_truncated_and_zero_backref() {
        for b in [&[0xf0][..], &[0x00, 0, 0][..], &[0x10, b'a', 1][..]] {
            assert!(lz4(b, 16).is_err());
        }
    }
    #[test]
    fn rejects_output_overrun() {
        assert!(lz4(&[0x11, b'a', 1, 0], 5).is_err());
    }
    #[test]
    fn literal_block() {
        assert_eq!(lz4(&[0x30, b'a', b'b', b'c'], 3).unwrap(), b"abc");
    }
}
