mod environment;
mod meadows_environment;
#[path = "../greyling-viewer/model.rs"]
mod model;
#[path = "../asset-probe/main.rs"]
mod probe;
mod terrain;
mod terrain_material;
mod window;
mod world;
fn main() -> model::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if let Some(i) = args.iter().position(|s| s == "--verify-patch") {
        terrain::TILE
            .set(terrain::Tile::load_patch(std::path::Path::new(
                &args[i + 1],
            ))?)
            .ok();
    } else {
        terrain::init(&std::env::current_dir()?)?;
    }
    if !args.iter().any(|s| {
        matches!(
            s.as_str(),
            "--verify-patch" | "--verify-terrain" | "--probe-prefab" | "--asset-dump"
        )
    }) {
        world::init_environment(&std::env::current_dir()?)?;
    }
    if let Some(i) = args.iter().position(|s| s == "--verify-patch") {
        terrain::verify_patch(std::path::Path::new(&args[i + 1]))?;
        return Ok(());
    }
    if let Some(i) = args.iter().position(|s| s == "--verify-terrain") {
        let dir = std::path::Path::new(&args[i + 1]);
        terrain::Tile::load(&dir.join("tile.bin"))?.verify(dir)?;
        return Ok(());
    }
    if args.iter().any(|s| s == "--plan-traverse") {
        world::traversal_start()?;
        return Ok(());
    }
    if let Some(i) = args.iter().position(|s| s == "--area-scenario") {
        world::area_scenario(std::path::Path::new(&args[i + 1]))?;
        return Ok(());
    }
    if let Some(i) = args.iter().position(|s| s == "--scenario") {
        world::scenario(std::path::Path::new(
            args.get(i + 1).ok_or("save path required")?,
        ))?;
        return Ok(());
    }
    if let Some(i) = args.iter().position(|s| s == "--verify-save") {
        let w = world::World::load(std::path::Path::new(&args[i + 1]))?;
        assert_eq!(w.floors.len(), 1);
        assert_eq!(w.wood, 1);
        assert_eq!(w.enemy_hp, 0.);
        assert!(!w.nodes[0].2);
        println!(
            "PASS separate-process persistence: position={},{} wood={} floors={} enemy_hp={}",
            w.x,
            w.z,
            w.wood,
            w.floors.len(),
            w.enemy_hp
        );
        return Ok(());
    }
    let lab = std::env::current_dir()?;
    if let Some(i) = args.iter().position(|s| s == "--probe-prefab") {
        let db = probe::range::Database::open(
            &lab.join("../common/Valheim/valheim_Data/StreamingAssets/SoftRef/Bundles/c4210710"),
            &lab.join("private-cache/greyling/serialized-c421-001"),
        )?;
        let mut materials = std::collections::HashMap::new();
        let a = environment::load_asset(&db, &args[i + 1], args[i + 2].parse()?, &mut materials)?;
        println!("PASS {} parts {} materials", a.parts.len(), materials.len());
        return Ok(());
    }
    let env = environment::load(&lab)?;
    if let Some(ps)=world::PLACEMENTS.get() {let mut omitted=std::collections::BTreeMap::<String,usize>::new();let mut selected=0;for p in ps {if env.assets.iter().any(|a|a.name==p.prefab.trim_end_matches("(Clone)")){selected+=1;}else{*omitted.entry(p.prefab.clone()).or_default()+=1;}}println!("Native environment selection: {selected}/{} cached instances; omitted unsupported prefabs: {:?}",ps.len(),omitted);}

    if let Some(i) = args.iter().position(|s| s == "--asset-dump") {
        let out = std::path::Path::new(&args[i + 1]);
        std::fs::create_dir_all(out)?;
        env.reference_map(&out.join("reference-map.tsv"))?;
        for a in &env.assets {
            a.dump(&out.join(format!("{}.csv", a.name)))?;
        }
        return Ok(());
    }
    let m = model::load(&lab)?;
    let animation = model::Animation::load(&lab, &m)?;
    let walk = model::Animation::load_clip(&lab, &m, "walk")?;
    let attack = model::Animation::load_clip(&lab, &m, "attack")?;
    if let Some(i) = args.iter().position(|s| s == "--dump-motion") {
        let a = match args[i + 1].as_str() {
            "walk" => walk.as_ref(),
            "attack" => attack.as_ref(),
            _ => animation.as_ref(),
        }
        .ok_or("motion unavailable")?;
        let frame = args[i + 2].parse::<usize>()?;
        m.dump(std::path::Path::new(&args[i + 3]), Some(&a.frames[frame]))?;
        return Ok(());
    }
    let capture = args
        .iter()
        .position(|s| s == "--capture")
        .map(|i| std::path::Path::new(&args[i + 1]));
    let controls = args
        .iter()
        .position(|s| s == "--playtest" || s == "--traverse")
        .map(|i| std::path::Path::new(&args[i + 1]));
    window::run(
        &env,
        &m,
        animation.as_ref(),
        walk.as_ref(),
        attack.as_ref(),
        None,
        capture,
        0.,
        controls,
    )
}
