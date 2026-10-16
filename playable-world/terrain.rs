use std::{io, path::Path, sync::OnceLock};
pub struct Tile {
    pub width: usize,
    pub scale: f32,
    pub center: [f32; 2],
    pub origin: f32,
    pub seed: i32,
    pub version: i32,
    pub heights: Vec<f32>,
}
pub static TILE: OnceLock<Tile> = OnceLock::new();
impl Tile {
    pub fn load(path: &Path) -> io::Result<Self> {
        let b = std::fs::read(path)?;
        let bad = || io::Error::new(io::ErrorKind::InvalidData, "Invalid native terrain cache");
        if b.len() < 32 || &b[..4] != b"TER1" {
            return Err(bad());
        }
        let u = |o| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let f = |o| f32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let width = u(4) as usize;
        let scale = f(8);
        if (width != 64 && width != 192)
            || scale != 1.
            || b.len() != 32 + (width + 1) * (width + 1) * 4
        {
            return Err(bad());
        }
        let heights = (32..b.len()).step_by(4).map(f).collect::<Vec<_>>();
        if heights.iter().any(|x| !x.is_finite()) || !f(20).is_finite() {
            return Err(bad());
        }
        Ok(Self {
            width,
            scale,
            center: [f(12), f(16)],
            origin: f(20),
            seed: u(24) as i32,
            version: u(28) as i32,
            heights,
        })
    }
    pub fn vertex(&self, i: usize) -> [f32; 3] {
        [
            (i % (self.width + 1)) as f32 * self.scale - self.width as f32 * 0.5,
            self.heights[i],
            (i / (self.width + 1)) as f32 * self.scale - self.width as f32 * 0.5,
        ]
    }
    pub fn indices(&self) -> Vec<usize> {
        let mut out = Vec::new();
        let n = self.width + 1;
        for z in 0..self.width {
            for x in 0..self.width {
                let a = z * n + x;
                out.extend([a, a + n, a + 1, a + 1, a + n, a + n + 1]);
            }
        }
        out
    }
    pub fn height(&self, x: f32, z: f32) -> f32 {
        let half = self.width as f32 * 0.5;
        let fx = (x + half).clamp(0., self.width as f32 - 0.00002);
        let fz = (z + half).clamp(0., self.width as f32 - 0.00002);
        let ix = fx.floor() as usize;
        let iz = fz.floor() as usize;
        let u = fx - ix as f32;
        let v = fz - iz as f32;
        let a = self.heights[iz * (self.width + 1) + ix];
        let b = self.heights[iz * (self.width + 1) + ix + 1];
        let c = self.heights[(iz + 1) * (self.width + 1) + ix];
        let d = self.heights[(iz + 1) * (self.width + 1) + ix + 1];
        (if u + v <= 1. {
            a + (b - a) * u + (c - a) * v
        } else {
            d + (c - d) * (1. - u) + (b - d) * (1. - v)
        }) - self.origin
    }
    pub fn verify(&self, reference: &Path) -> io::Result<()> {
        let s = std::fs::read_to_string(reference.join("native-vertices.csv"))?;
        let mut max = 0f32;
        let mut count = 0;
        for line in s.lines().skip(1) {
            let p = line.split(',').collect::<Vec<_>>();
            let i = p[0].parse::<usize>().unwrap();
            let a = self.vertex(i);
            for j in 0..3 {
                let b = p[j + 1].parse::<f32>().unwrap();
                max = max.max((a[j] - b).abs());
            }
            count += 1;
        }
        assert_eq!(count, 4225);
        assert!(max < 0.00001);
        let native = std::fs::read_to_string(reference.join("native-indices.txt"))?
            .lines()
            .map(|s| s.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(native, self.indices());
        for z in -30..30 {
            for x in -30..30 {
                let i = (z + 32) as usize * 65 + (x + 32) as usize;
                assert!(
                    (self.height(x as f32, z as f32) - (self.heights[i] - self.origin)).abs()
                        < 1e-5
                );
            }
        }
        let mut ray_error = 0f32;
        let mut rays = 0;
        for l in std::fs::read_to_string(reference.join("native-raycasts.csv"))?
            .lines()
            .skip(1)
        {
            let v = l
                .split(',')
                .map(|s| s.parse::<f32>().unwrap())
                .collect::<Vec<_>>();
            let (x, z, h) = if v.len() == 5 {
                (v[2] - self.center[0], v[3] - self.center[1], v[4])
            } else {
                (v[0], v[1], v[2])
            };
            ray_error = ray_error.max((self.height(x, z) + self.origin - h).abs());
            rays += 1;
        }
        assert_eq!(rays, 100);
        assert!(ray_error < 0.0001);
        println!(
            "PASS 100 native PhysX raycast surface checks, max error={}m",
            ray_error
        );
        println!("PASS {} native Heightmap vertices, {} exact index matches, 3600 collision/grid checks; max vertex error={}m; seed={} version={} center={:?}",count,native.len(),max,self.seed,self.version,self.center);
        Ok(())
    }
}
pub fn init(lab: &Path) -> io::Result<()> {
    let patch = lab.join("private-cache/terrain/patch-current.txt");
    if patch.exists() {
        let dir = std::fs::read_to_string(patch)?;
        let t = Tile::load_patch(Path::new(dir.trim()))?;
        TILE.set(t)
            .map_err(|_| io::Error::new(io::ErrorKind::AlreadyExists, "Terrain initialized"))?;
        return Ok(());
    }
    let pointer = lab.join("private-cache/terrain/native-current.txt");
    if pointer.exists() {
        let dir = std::fs::read_to_string(pointer)?;
        let t = Tile::load(&Path::new(dir.trim()).join("tile.bin"))?;
        println!("Native terrain tile: seed={} center={:?}; Unity-generated heights / Rust mesh and collision",t.seed,t.center);
        TILE.set(t).map_err(|_| {
            io::Error::new(io::ErrorKind::AlreadyExists, "Terrain already initialized")
        })?;
    }
    Ok(())
}

pub fn identity() -> String {
    match TILE.get() {
        Some(t) => format!(
            "native:{}:{}:{}:{}:{}",
            t.seed, t.version, t.center[0], t.center[1], t.width
        ),
        None => "custom-001".into(),
    }
}

pub fn extent() -> f32 {
    TILE.get().map(|t| t.width as f32 * 0.5).unwrap_or(32.)
}
impl Tile {
    pub fn load_patch(dir: &Path) -> io::Result<Self> {
        let manifest = std::fs::read_to_string(dir.join("patch.tsv"))?;
        let mut tiles = Vec::new();
        for line in manifest.lines().skip(1).filter(|l| !l.trim().is_empty()) {
            let file = line.split('\t').next().unwrap();
            let path = dir.join(file);
            tiles.push(Self::load(&if path.is_dir() {
                path.join("tile.bin")
            } else {
                path
            })?);
        }
        assert_eq!(tiles.len(), 9);
        let minx = tiles
            .iter()
            .map(|t| t.center[0])
            .fold(f32::INFINITY, f32::min);
        let minz = tiles
            .iter()
            .map(|t| t.center[1])
            .fold(f32::INFINITY, f32::min);
        let center = [minx + 64., minz + 64.];
        let middle = tiles
            .iter()
            .find(|t| t.center == center)
            .expect("Missing central tile");
        let (origin, seed, version) = (middle.origin, middle.seed, middle.version);
        let mut heights = vec![f32::NAN; 193 * 193];
        let mut overlaps = 0;
        let mut error = 0f32;
        let mut cells = std::collections::HashSet::new();
        for t in tiles {
            assert_eq!((t.seed, t.version), (seed, version));
            let dx = t.center[0] - minx;
            let dz = t.center[1] - minz;
            assert!(
                dx >= 0.
                    && dz >= 0.
                    && dx <= 128.
                    && dz <= 128.
                    && dx % 64. == 0.
                    && dz % 64. == 0.
            );
            assert!(cells.insert((dx as i32, dz as i32)));
            for z in 0..65 {
                for x in 0..65 {
                    let dst = (dz as usize + z) * 193 + dx as usize + x;
                    let h = t.heights[z * 65 + x];
                    if heights[dst].is_finite() {
                        overlaps += 1;
                        error = error.max((heights[dst] - h).abs());
                    }
                    heights[dst] = h;
                }
            }
        }
        assert!(heights.iter().all(|h| h.is_finite()));
        assert!(error <= 0.00001, "Terrain seams disagree {}", error);
        println!(
            "PASS nine native tiles stitched: {} shared edge/corner samples max error={}m",
            overlaps, error
        );
        Ok(Self {
            width: 192,
            scale: 1.,
            center,
            origin,
            seed,
            version,
            heights,
        })
    }
}

pub fn verify_patch(dir: &Path) -> io::Result<()> {
    let patch = Tile::load_patch(dir)?;
    let manifest = std::fs::read_to_string(dir.join("patch.tsv"))?;
    let mut checks = 0;
    let mut max = 0f32;
    for line in manifest.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let file = line.split('\t').next().unwrap();
        let path = dir.join(file);
        let path = if path.is_dir() {
            path.join("tile.bin")
        } else {
            path
        };
        let t = Tile::load(&path)?;
        t.verify(path.parent().unwrap())?;
        for z in -30..31 {
            for x in -30..31 {
                let a = t.height(x as f32 + 0.17, z as f32 + 0.23);
                let b = patch.height(
                    x as f32 + 0.17 + t.center[0] - patch.center[0],
                    z as f32 + 0.23 + t.center[1] - patch.center[1],
                );
                max = max.max((a - b).abs());
                checks += 1;
            }
        }
    }
    assert!(max < 0.0001);
    // Fixed timestep locomotion crossing every internal edge, with jumping and landing.
    let mut crossings = 0;
    for axis in 0..2 {
        for seam in [-32f32, 32.] {
            for along in [-64f32, 0., 64.] {
                let mut w = crate::world::World::new();
                w.enemy_hp = 0.;
                if axis == 0 {
                    w.x = seam - 2.;
                    w.z = along;
                } else {
                    w.x = along;
                    w.z = seam - 2.;
                }
                w.y = w.ground(w.x, w.z);
                let yaw = if axis == 0 {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.
                };
                for f in 0..100 {
                    let old = w.y;
                    w.step(1. / 60., 1., 0., yaw, false, f == 25);
                    assert!(w.y >= w.ground(w.x, w.z) - 0.001);
                    assert!((w.y - old).abs() < 0.6, "Abrupt walking elevation");
                }
                assert!(if axis == 0 {
                    w.x > seam + 2.
                } else {
                    w.z > seam + 2.
                });
                for _ in 0..100 {
                    w.step(1. / 60., 0., 0., yaw, false, false);
                }
                assert!((w.y - w.ground(w.x, w.z)).abs() < 0.001);
                crossings += 1;
            }
        }
    }
    println!("PASS {checks} stitched/tile fractional heights max={max}m; {crossings} automated movement/jump/landing boundary crossings");
    Ok(())
}
