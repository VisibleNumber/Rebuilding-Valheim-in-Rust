unsafe fn upload_texture(t: Option<&crate::model::Texture>) -> Result<u32> {
    let mut id = 0;
    glGenTextures(1, &mut id);
    glBindTexture(0xde1, id);
    for p in [0x2800, 0x2801] {
        glTexParameteri(0xde1, p, 0x2601);
    }
    for p in [0x2802, 0x2803] {
        glTexParameteri(0xde1, p, 0x2901);
    }
    if let Some(t) = t {
        let (format, block) = match t.format {
            25 => (0x8e8c, 16),
            10 => (0x83f1, 8),
            12 => (0x83f3, 16),
            _ => {
                return Err(
                    format!("Environment texture format {} not implemented", t.format).into(),
                )
            }
        };
        let n = (((t.width + 3) / 4) * ((t.height + 3) / 4) * block) as usize;
        if n > t.data.len() {
            return Err("environment texture truncated".into());
        }
        let upload = glfn!(
            "glCompressedTexImage2D",
            unsafe extern "system" fn(u32, i32, u32, i32, i32, i32, i32, *const c_void)
        );
        upload(
            0xde1,
            0,
            format,
            t.width,
            t.height,
            0,
            n as i32,
            t.data.as_ptr() as _,
        );
    } else {
        glTexImage2D(
            0xde1,
            0,
            0x1908,
            1,
            1,
            0,
            0x1908,
            0x1401,
            [255u8; 4].as_ptr() as _,
        );
    }
    if glGetError() != 0 {
        return Err("environment texture upload failed".into());
    }
    Ok(id)
}
unsafe fn draw_asset(
    asset: &crate::environment::Asset,
    env: &crate::environment::Environment,
    shader: &Shader,
    textures: &std::collections::HashMap<i64, u32>,
    highlight: bool,
) {
    (shader.use_program)(shader.program);
    (shader.one)(shader.loc("unlit"), 0.);
    for p in &asset.parts {
        let m = &env.materials[&p.material];
        glBindTexture(0xde1, textures[&p.material]);
        (shader.four)(
            shader.loc("tint"),
            m.base.color[0] * if highlight { 1.2 } else { 1. },
            m.base.color[1] * if highlight { 1.2 } else { 1. },
            m.base.color[2],
            m.base.color[3],
        );
        (shader.three)(
            shader.loc("emission"),
            if highlight { 0.13 } else { m.base.emission[0] },
            if highlight { 0.10 } else { m.base.emission[1] },
            m.base.emission[2],
        );
        (shader.three)(
            shader.loc("hsv"),
            m.base.hue,
            m.base.saturation,
            m.base.value,
        );
        (shader.one)(shader.loc("cutoff"), m.cutoff);
        static LISTS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<usize, u32>>> =
            std::sync::OnceLock::new();
        let key = p as *const crate::environment::StaticPart as usize;
        let lists = LISTS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        let known = lists.lock().unwrap().get(&key).copied();
        if let Some(id) = known {
            glCallList(id);
        } else {
            let id = glGenLists(1);
            glNewList(id, 0x1300);
            glBegin(4);
            for &i in &p.indices {
                let v = &p.vertices[i as usize];
                glTexCoord2f(v.uv[0] * m.uv[0] + m.uv[2], v.uv[1] * m.uv[1] + m.uv[3]);
                glNormal3f(v.n[0], v.n[1], v.n[2]);
                glVertex3f(v.p[0], v.p[1], v.p[2]);
            }
            glEnd();
            glEndList();
            lists.lock().unwrap().insert(key, id);
            glCallList(id);
        }
    }
    (shader.use_program)(0);
}
unsafe fn draw_ground(env: &crate::environment::Environment, shader: &Shader, texture: u32) {
    (shader.use_program)(shader.program);
    glBindTexture(0xde1, texture);
    (shader.four)(shader.loc("tint"), 1., 1., 1., 1.);
    (shader.three)(shader.loc("emission"), 0., 0., 0.);
    (shader.three)(shader.loc("hsv"), 0., 0., 0.);
    (shader.one)(shader.loc("unlit"), 0.);
    (shader.one)(shader.loc("cutoff"), 0.);
    static GROUND_LIST: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    if let Some(id) = GROUND_LIST.get() {
        glCallList(*id);
    } else {
        let id = glGenLists(1);
        glNewList(id, 0x1300);
        glBegin(4);
        let center = crate::terrain::TILE
            .get()
            .map(|t| t.center)
            .unwrap_or([0.; 2]);
        let extent = crate::terrain::extent() as i32;
        for x in -extent..extent {
            for z in -extent..extent {
                for (a, b) in [(0, 0), (0, 1), (1, 0), (1, 0), (0, 1), (1, 1)] {
                    let x = (x + a) as f32;
                    let z = (z + b) as f32;
                    let n = norm([
                        crate::world::height(x - 0.1, z) - crate::world::height(x + 0.1, z),
                        0.2,
                        crate::world::height(x, z - 0.1) - crate::world::height(x, z + 0.1),
                    ]);
                    glNormal3f(n[0], n[1], n[2]);
                    glTexCoord2f(
                        (x + center[0]) * env.ground_uv,
                        (z + center[1]) * env.ground_uv,
                    );
                    glVertex3f(x, crate::world::height(x, z), z);
                }
            }
        }
        glEnd();
        glEndList();
        GROUND_LIST.set(id).ok();
        glCallList(id);
    }
    (shader.use_program)(0);
}

fn resource_matrix(i: usize, x: f32, z: f32, min: f32) -> [f32; 16] {
    let a = (45. + i as f32 * 61.).to_radians();
    let (dx, dz) = (a.sin(), a.cos());
    let forward = norm([
        dx,
        (crate::world::height(x + dx, z + dz) - crate::world::height(x - dx, z - dz)) * 0.5,
        dz,
    ]);
    let up = norm([
        crate::world::height(x - 0.1, z) - crate::world::height(x + 0.1, z),
        0.2,
        crate::world::height(x, z - 0.1) - crate::world::height(x, z + 0.1),
    ]);
    let right = norm(cross(up, forward));
    let up = cross(forward, right);
    [
        right[0],
        right[1],
        right[2],
        0.,
        up[0],
        up[1],
        up[2],
        0.,
        forward[0],
        forward[1],
        forward[2],
        0.,
        x,
        crate::world::height(x, z) - min + 0.02,
        z,
        1.,
    ]
}

// Bounded approximation: flat opaque surface at inspected Heightmap material _WaterLevel=30.
unsafe fn draw_water() {
    if let Some(t) = crate::terrain::TILE.get() {
        if t.heights.iter().all(|h| *h >= 30.) {
            return;
        }
        let e = crate::terrain::extent();
        let y = 30. - t.origin;
        glDisable(0xde1);
        glColor4f(0.18, 0.40, 0.51, 1.);
        glNormal3f(0., 1., 0.);
        glBegin(7);
        for (x, z) in [(-e, -e), (-e, e), (e, e), (e, -e)] {
            glVertex3f(x, y, z);
        }
        glEnd();
        glColor4f(1., 1., 1., 1.);
    }
}
