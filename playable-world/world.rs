use std::{io, path::Path};
pub fn height(x: f32, z: f32) -> f32 {
    if let Some(t) = crate::terrain::TILE.get() {
        t.height(x, z)
    } else {
        0.32 * (x * 0.16).sin() * (z * 0.13).sin()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    pub x: f32,
    pub z: f32,
    pub y: f32,
    pub vy: f32,
    pub wood: u32,
    pub health: f32,
    pub nodes: Vec<(f32, f32, bool)>,
    pub floors: Vec<(f32, f32)>,
    pub enemy: [f32; 2],
    pub enemy_hp: f32,
    pub attack_time: f32,
    pub attack_hit: bool,
    pub enemy_moving: bool,
    pub cooldown: f32,
    pub swing: f32,
    pub hurt: f32,
    pub hit_timer: f32,
    pub message: String,
}
impl World {
    pub fn new() -> Self {
        Self {
            x: spawn_x(),
            z: 0.,
            y: height(spawn_x(), 0.),
            vy: 0.,
            wood: 0,
            health: 100.,
            nodes: resource_nodes(),
            floors: vec![],
            enemy: [spawn_x()+5., 13.],
            enemy_hp: 60.,
            attack_time: 0.,
            attack_hit: false,
            enemy_moving: false,
            cooldown: 0.,
            swing: 0.,
            hurt: 0.,
            hit_timer: 0.,
            message: "Gather wood with E, build a floor with B. Watch the Greyling!".into(),
        }
    }
    pub fn ground(&self, x: f32, z: f32) -> f32 {
        let mut h = height(x, z);
        for &(a, b) in &self.floors {
            if (x - a).abs() < 1.0 && (z - b).abs() < 1.0 {
                h = h.max(floor_top(a, b));
            }
        }
        h
    }
    pub fn step(&mut self, dt: f32, forward: f32, side: f32, yaw: f32, sprint: bool, jump: bool) {
        let dt = dt.clamp(0., 0.05);
        self.cooldown = (self.cooldown - dt).max(0.);
        self.swing = (self.swing - dt).max(0.);
        self.hurt = (self.hurt - dt).max(0.);
        self.hit_timer = (self.hit_timer - dt).max(0.);
        self.attack_time = (self.attack_time - dt).max(0.);
        self.enemy_moving = false;
        if self.health <= 0. {
            return;
        }
        let mag = (forward * forward + side * side).sqrt().max(1.);
        let speed = if sprint { 6. } else { 3.6 };
        self.x = ((self.x + (yaw.sin() * forward + yaw.cos() * side) / mag * speed * dt).clamp(
            -crate::terrain::extent() + 2.,
            crate::terrain::extent() - 2.,
        ));
        self.z = ((self.z + (yaw.cos() * forward - yaw.sin() * side) / mag * speed * dt).clamp(
            -crate::terrain::extent() + 2.,
            crate::terrain::extent() - 2.,
        ));
        if let Some(obstacles) = OBSTACLES.get() {
            let t = crate::terrain::TILE.get().unwrap();
            for c in obstacles {
                let cx = c.center[0] - t.center[0];
                let cz = c.center[2] - t.center[1];
                if self.y + 1.65 < c.center[1] - c.size[1] * 0.5 - t.origin
                    || self.y > c.center[1] + c.size[1] * 0.5 - t.origin
                {
                    continue;
                }
                let hx = c.size[0] * 0.5 + 0.25;
                let hz = c.size[2] * 0.5 + 0.25;
                let dx = self.x - cx;
                let dz = self.z - cz;
                if dx.abs() < hx && dz.abs() < hz {
                    if hx - dx.abs() < hz - dz.abs() {
                        self.x = cx + if dx >= 0. { hx } else { -hx };
                    } else {
                        self.z = cz + if dz >= 0. { hz } else { -hz };
                    }
                }
            }
        }
        for i in 0..tree_count() {
            let (a, b) = tree_position(i);
            let dx = self.x - a;
            let dz = self.z - b;
            let d = dx.hypot(dz);
            let radius = tree_radius(i);
            if d < radius {
                self.x = a + dx / d.max(0.001) * radius;
                self.z = b + dz / d.max(0.001) * radius;
            }
        }
        let ground = self.ground(self.x, self.z);
        if jump && self.y <= ground + 0.02 {
            self.vy = 5.;
        }
        self.vy -= 13. * dt;
        self.y += self.vy * dt;
        if self.y < ground {
            self.y = ground;
            self.vy = 0.;
        }
        if self.enemy_hp > 0. {
            let dx = self.x - self.enemy[0];
            let dz = self.z - self.enemy[1];
            let d = (dx * dx + dz * dz).sqrt();
            if d < 10. && d > 1.5 && self.attack_time <= 0. {
                self.enemy_moving = true;
                self.enemy[0] += dx / d * 1.35 * dt;
                self.enemy[1] += dz / d * 1.35 * dt;
            }
            if d < 1.9 && self.hit_timer <= 0. && self.attack_time <= 0. {
                self.attack_time = 2.;
                self.attack_hit = false;
                self.hit_timer = 2.3;
            }
            if self.attack_time > 0. && self.attack_time < 1.5 && !self.attack_hit {
                self.attack_hit = true;
                if d < 2.2 {
                    self.health = (self.health - 8.).max(0.);
                    self.hurt = 0.3;
                    self.message = if self.health == 0. {
                        "You fell. Press N for a fresh sandbox."
                    } else {
                        "Greyling hit! Space to strike, or back away."
                    }
                    .into();
                }
            }
        }
    }
    pub fn target(&self, yaw: f32) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                let dx = n.0 - self.x;
                let dz = n.1 - self.z;
                let d = dx.hypot(dz);
                n.2 && d < 3.2 && (d < 0.6 || (dx * yaw.sin() + dz * yaw.cos()) / d > 0.8)
            })
            .min_by(|(_, a), (_, b)| {
                ((a.0 - self.x).hypot(a.1 - self.z))
                    .total_cmp(&((b.0 - self.x).hypot(b.1 - self.z)))
            })
            .map(|(i, _)| i)
    }
    pub fn gather(&mut self, yaw: f32) {
        if self.health <= 0. {
            return;
        }
        if let Some(i) = self.target(yaw) {
            self.nodes[i].2 = false;
            self.wood += 3;
            self.message = "+3 wood".into();
            return;
        }
        self.message = "Move closer to a wood pile (E).".into();
    }
    pub fn preview(&self, yaw: f32) -> (f32, f32, f32, bool, &'static str) {
        let x = ((self.x + yaw.sin() * 2.5) / 2.).round() * 2.;
        let z = ((self.z + yaw.cos() * 2.5) / 2.).round() * 2.;
        let y = floor_top(x, z);
        let low = [(-1., -1.), (-1., 1.), (1., -1.), (1., 1.)]
            .iter()
            .map(|(a, b)| height(x + a, z + b))
            .fold(f32::INFINITY, f32::min);
        let reason = if self.health <= 0. {
            "You have fallen"
        } else if x.abs() > crate::terrain::extent() - 4. || z.abs() > crate::terrain::extent() - 4.
        {
            "Outside clearing"
        } else if self.floors.contains(&(x, z)) {
            "Space occupied"
        } else if y - low > 1.2 {
            "Ground too steep"
        } else if (0..tree_count()).any(|i| {
            let (a, b) = tree_position(i);
            (a - x).abs() < 1.6 && (b - z).abs() < 1.6
        }) {
            "Tree in the way"
        } else if OBSTACLES.get().is_some_and(|cs| {
            cs.iter().any(|c| {
                let t = crate::terrain::TILE.get().unwrap();
                (x - (c.center[0] - t.center[0])).abs() < 1. + c.size[0] * 0.5
                    && (z - (c.center[2] - t.center[1])).abs() < 1. + c.size[2] * 0.5
            })
        }) {
            "Rock in the way"
        } else if self.wood < 2 {
            "Needs 2 wood"
        } else {
            "B: place wooden floor"
        };
        (x, y, z, reason == "B: place wooden floor", reason)
    }
    pub fn build(&mut self, yaw: f32) {
        let (x, _y, z, valid, reason) = self.preview(yaw);
        if !valid {
            self.message = reason.into();
            return;
        }
        self.floors.push((x, z));
        self.wood -= 2;
        self.message = "Wooden floor placed".into();
    }
    pub fn attack(&mut self, yaw: f32) {
        if self.health <= 0. || self.cooldown > 0. {
            return;
        }
        self.cooldown = 0.45;
        self.swing = 0.3;
        let dx = self.enemy[0] - self.x;
        let dz = self.enemy[1] - self.z;
        let d = dx.hypot(dz);
        if self.enemy_hp > 0. && d < 3. && (dx * yaw.sin() + dz * yaw.cos()) / d.max(0.01) > 0.15 {
            self.enemy_hp = (self.enemy_hp - 20.).max(0.);
            self.message = if self.enemy_hp == 0. {
                "Greyling defeated. Gather, build and save your camp."
            } else {
                "Hit!"
            }
            .into();
        } else {
            self.message = "Swing missed. Face the Greyling and move closer.".into();
        }
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut s = format!(
            "VALHEIM_LAB_SANDBOX_3 {}\n{} {} {} {} {} {} {} {}\n",
            crate::terrain::identity(),
            self.x,
            self.z,
            self.y,
            self.wood,
            self.health,
            self.enemy[0],
            self.enemy[1],
            self.enemy_hp
        );
        for &(x, z, active) in &self.nodes {
            s += &format!("node {x} {z} {}\n", active as u8);
        }
        for &(x, z) in &self.floors {
            s += &format!("floor {x} {z}\n");
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, s)?;
        if path.exists() {
            std::fs::copy(path, path.with_extension("bak"))?;
        }
        std::fs::rename(tmp, path)
    }
    pub fn load(path: &Path) -> io::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let bad = || io::Error::new(io::ErrorKind::InvalidData, "Invalid sandbox save");
        let mut lines = s.lines();
        if lines.next()
            != Some(format!("VALHEIM_LAB_SANDBOX_3 {}", crate::terrain::identity()).as_str())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Save belongs to a different terrain profile or older sandbox version",
            ));
        }
        let v = lines
            .next()
            .ok_or_else(bad)?
            .split_whitespace()
            .map(str::parse::<f32>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| bad())?;
        if v.len() != 8
            || v.iter().any(|v| !v.is_finite())
            || v[3] < 0.
            || v[3] > 100000.
            || v[4] < 0.
            || v[4] > 100.
            || v[7] < 0.
            || v[7] > 60.
        {
            return Err(bad());
        }
        let mut w = Self::new();
        w.x = v[0];
        w.z = v[1];
        w.y = v[2];
        w.wood = v[3] as u32;
        w.health = v[4];
        w.enemy = [v[5], v[6]];
        w.enemy_hp = v[7];
        w.nodes.clear();
        for l in lines {
            let p = l.split_whitespace().collect::<Vec<_>>();
            if p.len() < 3 {
                return Err(bad());
            }
            let x = p[1].parse::<f32>().map_err(|_| bad())?;
            let z = p[2].parse::<f32>().map_err(|_| bad())?;
            if !x.is_finite()
                || !z.is_finite()
                || x.abs() > crate::terrain::extent()
                || z.abs() > crate::terrain::extent()
            {
                return Err(bad());
            }
            match p[0] {
                "node" if p.len() == 4 => w.nodes.push((
                    x,
                    z,
                    match p[3] {
                        "1" => true,
                        "0" => false,
                        _ => return Err(bad()),
                    },
                )),
                "floor" if p.len() == 3 => w.floors.push((x, z)),
                _ => return Err(bad()),
            }
        }
        if w.x.abs() > crate::terrain::extent() - 2.
            || w.z.abs() > crate::terrain::extent() - 2.
            || w.y.abs() > 100.
        {
            return Err(bad());
        }
        w.message = "Sandbox loaded.".into();
        Ok(w)
    }
}
pub fn scenario(path: &Path) -> io::Result<()> {
    let mut w = World::new();
    for _ in 0..45 {
        w.step(1. / 60., 1., 0., 0., false, false);
    }
    assert!(w.z > 2.6 && w.z < 2.8);
    w.gather(0.);
    assert_eq!(w.wood, 3);
    assert!(!w.nodes[0].2);
    w.build(0.);
    assert_eq!(w.wood, 1);
    assert_eq!(w.floors.len(), 1);
    w.step(1. / 60., 0., 0., 0., false, true);
    assert!(w.y > w.ground(w.x, w.z));
    for _ in 0..90 {
        w.step(1. / 60., 0., 0., 0., false, false);
    }
    assert!((w.y - w.ground(w.x, w.z)).abs() < 0.001);
    let before = w.wood;
    w.build(0.);
    assert_eq!(w.wood, before);
    assert_eq!(w.floors.len(), 1);
    for i in 0..60 {
        w.step(1. / 60., 1., 0., 0., false, i == 30);
    }
    assert!(w.z > 6.);
    for _ in 0..60 {
        w.step(1. / 60., 0., 0., 0., false, false);
    }
    assert!((w.y - floor_top(spawn_x(), 6.)).abs() < 0.01);
    w.enemy = [w.x, w.z + 1.6];
    let hp = w.health;
    w.step(0.05, 0., 0., 0., false, false);
    assert_eq!(w.health, hp);
    for _ in 0..40 {
        w.step(1. / 60., 0., 0., 0., false, false);
    }
    assert!(w.health < hp);
    for _ in 0..3 {
        w.attack(0.);
        for _ in 0..30 {
            w.step(1. / 60., 0., 0., 0., false, false);
        }
    }
    assert_eq!(w.enemy_hp, 0.);
    w.save(path)?;
    let r = World::load(path)?;
    assert_eq!(w.x, r.x);
    assert_eq!(w.z, r.z);
    assert_eq!(w.wood, r.wood);
    assert_eq!(w.health, r.health);
    assert_eq!(w.nodes, r.nodes);
    assert_eq!(w.floors, r.floors);
    assert_eq!(w.enemy_hp, r.enemy_hp);
    println!("PASS movement, jump/landing, gather, resource cost, placement, enemy retaliation, three melee hits, death, save/load: {}",path.display());
    Ok(())
}

pub fn tree_position(i: usize) -> (f32, f32) {
    if let Some(p) = PLACEMENTS.get() {
        let p = &p[TREE_INDICES.get().unwrap()[i]];
        let t = crate::terrain::TILE.get().unwrap();
        return (p.position[0] - t.center[0], p.position[2] - t.center[1]);
    }
    let t = i as f32 * 2.399;
    let r = 18. + (i % 4) as f32 * 2.5;
    (t.sin() * r, t.cos() * r)
}
pub fn floor_top(x: f32, z: f32) -> f32 {
    [
        (-1., -1.),
        (-1., 0.),
        (-1., 1.),
        (0., -1.),
        (0., 0.),
        (0., 1.),
        (1., -1.),
        (1., 0.),
        (1., 1.),
    ]
    .iter()
    .map(|(a, b)| height(x + a, z + b))
    .fold(f32::NEG_INFINITY, f32::max)
        + 0.12
}

pub static PLACEMENTS: std::sync::OnceLock<Vec<crate::meadows_environment::Placement>> =
    std::sync::OnceLock::new();
static TREE_INDICES: std::sync::OnceLock<Vec<usize>> = std::sync::OnceLock::new();
pub fn init_environment(lab: &Path) -> crate::model::Result<()> {
    match crate::meadows_environment::load(lab) {
        Ok(p) => {
            println!("Loaded {} native environment placements", p.len());
            TREE_INDICES.set(p.iter().enumerate().filter(|(_, p)| is_tree(&p.prefab)).map(|(i, _)| i).collect()).ok();
            PLACEMENTS.set(p).ok();
            {
                let cs = crate::meadows_environment::load_colliders(lab)?;
                let cs = cs
                    .into_iter()
                    .filter(|c| {
                        matches!(
                            c.prefab.trim_end_matches("(Clone)"),
                            "Rock_3" | "Rock_4" | "Rock_7_meadows"
                        )
                    })
                    .collect::<Vec<_>>();
                println!("Loaded {} native collider AABB proxies", cs.len());
                OBSTACLES.set(cs).ok();
            }
        }
        Err(e) => {
            println!("Native placement cache unavailable: {e}");
            if crate::terrain::extent() > 32. {
                return Err(
                    format!("Complete native Meadows placement cache required: {e}").into(),
                );
            }
        }
    }
    Ok(())
}
pub fn tree_count() -> usize {
    TREE_INDICES
        .get()
        .map(|p| p.len())
        .unwrap_or(14)
}
pub fn placement_matrix(p: &crate::meadows_environment::Placement) -> [f32; 16] {
    let [x, y, z, w] = p.rotation;
    let s = p.scale;
    let t = crate::terrain::TILE.get().unwrap();
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
        p.position[0] - t.center[0],
        p.position[1] - t.origin,
        p.position[2] - t.center[1],
        1.,
    ]
}

// Explicit gameplay supplies, separate from native world vegetation placement.
fn resource_nodes() -> Vec<(f32, f32, bool)> {
    let mut nodes = vec![
        (0., 3., true),
        (2., 4., true),
        (-2., 5., true),
        (5., 1., true),
        (-5., 8., true),
        (8., -4., true),
    ];
    for n in &mut nodes {n.0+=spawn_x();}
    if crate::terrain::extent() > 32. {
        for z in [-64., 0., 64.] {
            for x in [-64., 0., 64.] {
                if x == 0. && z == 0. {
                    continue;
                }
                for (a, b) in [(0., 3.), (2., 4.), (-2., 5.)] {
                    nodes.push((x + a, z + b, true));
                }
            }
        }
    }
    nodes
}

pub static OBSTACLES: std::sync::OnceLock<Vec<crate::meadows_environment::Collider>> =
    std::sync::OnceLock::new();
pub fn area_scenario(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    assert!(crate::terrain::extent() > 90.);
    for z in [-64f32, 0., 64.] {
        for x in [-64f32, 0., 64.] {
            let mut w = World::new();
            w.x = x+if x==0.&&z==0.{spawn_x()}else{0.};
            w.z = z + 2.7;
            w.y = w.ground(w.x, w.z);
            w.enemy_hp = 0.;
            w.gather(0.);
            assert_eq!(w.wood, 3);
            let mut built = false;
            'search: for dz in -8..9 {
                for dx in -8..9 {
                    w.x = x + dx as f32;
                    w.z = z + dz as f32;
                    w.y = w.ground(w.x, w.z);
                    if w.preview(0.).3 {
                        w.build(0.);
                        built = true;
                        break 'search;
                    }
                }
            }
            assert!(built, "No valid floor placement in tile {},{}", x, z);
            assert_eq!(w.wood, 1);
            w.enemy = [w.x, w.z + 1.6];
            w.enemy_hp = 60.;
            w.cooldown = 0.;
            for _ in 0..3 {
                w.attack(0.);
                w.cooldown = 0.;
            }
            assert_eq!(w.enemy_hp, 0.);
            let path = dir.join(format!("tile_{x}_{z}.save"));
            w.save(&path)?;
            let r = World::load(&path)?;
            assert_eq!(r.nodes, w.nodes);
            assert_eq!(r.floors, w.floors);
            assert_eq!(r.wood, 1);
            assert_eq!(r.enemy_hp, 0.);
            println!("PASS AUTOMATED simulation fixture tile local center {x},{z}: gather, floor, combat, custom save/load");
        }
    }
    Ok(())
}

fn tree_radius(i: usize) -> f32 {
    PLACEMENTS
        .get()
        .and_then(|ps| TREE_INDICES.get().and_then(|indices| indices.get(i)).map(|&i| &ps[i]))
        .map(|p| 0.65 * p.scale[0].max(p.scale[2]))
        .unwrap_or(0.65)
}

fn is_tree(name: &str) -> bool {
    matches!(name.trim_end_matches("(Clone)"), "Beech1" | "Birch1")
}

// Chosen from inspected native placements to leave a usable starting building lane.
pub fn spawn_x()->f32 {if crate::terrain::extent()>32. {-8.}else{0.}}

/// Select an observable keyboard-test route against the actual loaded obstacles.
/// No collider is removed or bypassed: each candidate uses normal movement.
pub fn traversal_start() -> io::Result<(f32, f32)> {
    for z in (-60..=-36).step_by(2) {
        for x in (-60..=-36).step_by(2) {
            let mut w = World::new();
            w.x = x as f32;
            w.z = z as f32;
            w.y = w.ground(w.x, w.z);
            w.enemy_hp = 0.;
            let (mut minx, mut maxx, mut minz, mut maxz) = (w.x,w.x,w.z,w.z);
            for frame in 1..=1280 {
                let (forward, side) = match (frame-1)/320 {
                    0 => (1.,0.), 1 => (0.,1.), 2 => (-1.,0.), _ => (0.,-1.),
                };
                w.step(0.05, forward, side, 0., frame>1, frame>1 && frame%80==1);
                minx=minx.min(w.x); maxx=maxx.max(w.x);
                minz=minz.min(w.z); maxz=maxz.max(w.z);
            }
            if minx < -33. && maxx > 33. && minz < -33. && maxz > 33.
                && (w.x-x as f32).abs()<4. && (w.z-z as f32).abs()<4. {
                println!("Traversal preflight selected start {x},{z}; simulated bounds x={minx}..{maxx} z={minz}..{maxz}; collisions enabled");
                return Ok((x as f32,z as f32));
            }
        }
    }
    Err(io::Error::other("No clear 96m keyboard square found in loaded environment"))
}
