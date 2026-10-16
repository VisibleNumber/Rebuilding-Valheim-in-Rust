use crate::{
    model::{Result, Texture},
    probe::range::Database,
};
// Installed Heightmap_basematerial _DiffuseArrayTex. Private source payload is read in-place.
pub fn layers(db: &Database) -> Result<Vec<Texture>> {
    let v = db.read(-4418745866935824791)?;
    let n = |k: &str| v.get(k).unwrap().int();
    assert_eq!(
        (
            n("m_Format"),
            n("m_Width"),
            n("m_Height"),
            n("m_Depth"),
            n("m_MipCount")
        ),
        (108, 256, 256, 16, 1)
    );
    let stream = v.get("m_StreamData").unwrap();
    let path = stream.get("path").unwrap().text();
    let off = stream.get("offset").unwrap().int() as u64;
    let len = stream.get("size").unwrap().int() as usize;
    let node = db
        .bundle
        .nodes
        .iter()
        .find(|n| path.ends_with(&n.0))
        .ok_or("Terrain array resource missing")?;
    assert!(off + len as u64 <= node.2);
    let data = db.bundle.read(node.1 + off, len)?;
    assert_eq!(data.len(), 16 * 65536);
    Ok(data
        .chunks_exact(65536)
        .map(|b| Texture {
            width: 256,
            height: 256,
            mips: 1,
            format: 25,
            data: b.to_vec(),
        })
        .collect())
}
