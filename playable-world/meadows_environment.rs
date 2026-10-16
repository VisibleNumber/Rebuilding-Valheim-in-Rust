use std::{fs, path::Path};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub struct Placement {
    pub prefab: String,
    /// Original world position exported by Unity.
    pub position: [f32; 3],
    /// Original Unity quaternion in x, y, z, w order.
    pub rotation: [f32; 4],
    /// Original Unity lossy scale.
    pub scale: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct NativeCollider {
    pub prefab: String,
    pub part_path: String,
    pub collider_type: String,
    pub layer_id: i32,
    pub layer_name: String,
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub is_trigger: bool,
    pub enabled: bool,
    pub shape_detail: String,
}

#[derive(Clone, Debug)]
pub struct Collider {
    pub prefab: String,
    pub part_path: String,
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub kind: String,
    pub layer_id: i32,
    pub layer_name: String,
}

fn parse(path: &Path) -> Result<Vec<Placement>> {
    let content = fs::read_to_string(path)?;
    let mut rows = content.lines();
    let header = rows.next().ok_or("empty native placement file")?;
    let fields = header.split('\t').enumerate().map(|(i, s)| (s, i)).collect::<std::collections::HashMap<_, _>>();
    let col = |name: &str| -> Result<usize> { fields.get(name).copied().ok_or_else(|| format!("missing column {name}").into()) };
    let (prefab, x, y, z, qx, qy, qz, qw, sx, sy, sz) = (
        col("prefab")?, col("world_x")?, col("world_y")?, col("world_z")?,
        col("qx")?, col("qy")?, col("qz")?, col("qw")?,
        col("scale_x")?, col("scale_y")?, col("scale_z")?,
    );
    let mut placements = Vec::new();
    for (line_no, line) in rows.enumerate() {
        if line.trim().is_empty() { continue; }
        let f = line.split('\t').collect::<Vec<_>>();
        let number = |i: usize| -> Result<f32> { Ok(f.get(i).ok_or("short placement row")?.parse()?) };
        let position = [number(x)?, number(y)?, number(z)?];
        let rotation = [number(qx)?, number(qy)?, number(qz)?, number(qw)?];
        let scale = [number(sx)?, number(sy)?, number(sz)?];
        if !position.iter().chain(rotation.iter()).chain(scale.iter()).all(|v| v.is_finite())
            || scale.iter().any(|v| *v <= 0.)
            || (rotation.iter().map(|v| v * v).sum::<f32>() - 1.).abs() > 0.01
        {
            return Err(format!("invalid transform on row {}", line_no + 2).into());
        }
        placements.push(Placement {
            prefab: f.get(prefab).ok_or("missing prefab value")?.to_string(),
            position,
            rotation,
            scale,
        });
    }
    Ok(placements)
}

/// Read a known native placement TSV. Absolute coordinates remain intact so
/// the terrain owner can choose its own local-origin conversion.
pub fn load_from_tsv(path: &Path) -> Result<Vec<Placement>> { parse(path) }

/// Read native collider observations. `center` and `size` are world AABBs;
/// using them for collision is a conservative proxy, not exact PhysX parity.
pub fn load_colliders_from_tsv(path: &Path) -> Result<Vec<NativeCollider>> {
    let content = fs::read_to_string(path)?;
    let mut rows = content.lines();
    let header = rows.next().ok_or("empty native collider file")?;
    let fields = header.split('\t').enumerate().map(|(i, s)| (s, i)).collect::<std::collections::HashMap<_, _>>();
    let col = |name: &str| -> Result<usize> { fields.get(name).copied().ok_or_else(|| format!("missing collider column {name}").into()) };
    let (prefab, part, kind, layer_id, layer_name, cx, cy, cz, sx, sy, sz, trigger, enabled, detail) = (
        col("prefab")?, col("part_path")?, col("type")?, col("layer_id")?, col("layer_name")?, col("world_center_x")?, col("world_center_y")?, col("world_center_z")?,
        col("size_x")?, col("size_y")?, col("size_z")?, col("is_trigger")?, col("enabled")?, col("shape_detail")?,
    );
    let mut result = Vec::new();
    for (line_no, line) in rows.enumerate() {
        if line.trim().is_empty() { continue; }
        let f = line.split('\t').collect::<Vec<_>>();
        let number = |i: usize| -> Result<f32> { Ok(f.get(i).ok_or("short collider row")?.parse()?) };
        let center = [number(cx)?, number(cy)?, number(cz)?];
        let size = [number(sx)?, number(sy)?, number(sz)?];
        if !center.iter().chain(size.iter()).all(|v| v.is_finite()) || size.iter().any(|v| *v < 0.) {
            return Err(format!("invalid collider bounds on row {}", line_no + 2).into());
        }
        result.push(NativeCollider {
            prefab: f.get(prefab).ok_or("missing collider prefab")?.to_string(),
            part_path: f.get(part).ok_or("missing collider path")?.to_string(),
            collider_type: f.get(kind).ok_or("missing collider type")?.to_string(),
            layer_id: f.get(layer_id).ok_or("missing collider layer")?.parse()?,
            layer_name: f.get(layer_name).ok_or("missing collider layer name")?.to_string(),
            center,
            size,
            is_trigger: parse_bool(f.get(trigger).ok_or("missing trigger flag")?)?,
            enabled: parse_bool(f.get(enabled).ok_or("missing enabled flag")?)?,
            shape_detail: f.get(detail).ok_or("missing collider shape")?.to_string(),
        });
    }
    Ok(result)
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid Boolean value {value}").into()),
    }
}

fn cache_directory(lab: &Path) -> Result<std::path::PathBuf> {
    for pointer in [
        lab.join("private-cache/environment/patch-current.txt"),
        lab.join("private-cache/terrain/environment/patch-current.txt"),
        lab.join("private-cache/terrain/patch-current.txt"),
        lab.join("private-cache/terrain/native-current.txt"),
    ] {
        if pointer.is_file() {
            let directory = fs::read_to_string(&pointer)?;
            let directory = std::path::PathBuf::from(directory.trim());
            if complete_cache(&directory) { return Ok(directory); }
            let nested = directory.join("environment");
            if complete_cache(&nested) { return Ok(nested); }
        }
    }
    Err("native environment placement cache pointer not found".into())
}

fn complete_cache(directory: &Path) -> bool {
    if !directory.join("environment.tsv").is_file() && !directory.join("placements.tsv").is_file() { return false; }
    let status = match fs::read_to_string(directory.join("generation-status.txt")) { Ok(v) => v, Err(_) => return false };
    if status.contains("Native environment generation failed") || !status.contains("Native zones generated: 9/9") { return false; }
    let capture = match fs::read_to_string(directory.join("environment-capture.txt")) { Ok(v) => v, Err(_) => return false };
    capture.lines().find_map(|line| line.strip_prefix("Rows: ").and_then(|n| n.parse::<usize>().ok())).is_some_and(|n| n > 0)
}

/// Load enabled, non-trigger colliders as conservative native-world AABBs.
pub fn load_colliders(lab: &Path) -> Result<Vec<Collider>> {
    let dir = cache_directory(lab)?;
    let path = dir.join("environment-colliders.tsv");
    Ok(load_colliders_from_tsv(&path)?.into_iter().filter(|c| c.enabled && !c.is_trigger).map(|c| Collider {
        prefab: c.prefab,
        part_path: c.part_path,
        center: c.center,
        size: c.size,
        kind: c.collider_type,
        layer_id: c.layer_id,
        layer_name: c.layer_name,
    }).collect())
}

/// Resolve the newest native cache pointer produced by the diagnostic helper.
pub fn load(lab: &Path) -> Result<Vec<Placement>> {
    let directory = cache_directory(lab)?;
    let placement_file = directory.join("environment.tsv");
    if placement_file.is_file() { return parse(&placement_file); }
    parse(&directory.join("placements.tsv"))
}
