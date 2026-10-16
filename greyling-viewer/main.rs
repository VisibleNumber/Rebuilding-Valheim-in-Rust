#[path="../asset-probe/main.rs"]mod probe;
mod model;
mod window;
fn main()->model::Result<()>{let lab=std::env::current_dir()?;let m=model::load(&lab)?;println!("{}",m.describe());let args=std::env::args().collect::<Vec<_>>();let animation=model::Animation::load(&lab,&m)?;let pose=args.iter().position(|s|s=="--pose").map(|i|args[i+1].parse::<usize>()).transpose()?;
 if let Some(i)=pose{if animation.as_ref().is_none_or(|a|i>=a.frames.len()){return Err("pose index unavailable".into());}}
 if let Some(i)=args.iter().position(|s|s=="--dump"){m.dump(std::path::Path::new(&args[i+1]),pose.map(|i|&animation.as_ref().unwrap().frames[i]))?;}
 if args.iter().any(|s|s=="--check"){return Ok(());}let capture=args.iter().position(|s|s=="--capture").map(|i|std::path::Path::new(&args[i+1]));let controls=args.iter().position(|s|s=="--controls-test").map(|i|std::path::Path::new(&args[i+1]));let yaw=args.iter().position(|s|s=="--yaw").map(|i|args[i+1].parse::<f32>()).transpose()?.unwrap_or(0.);window::run(&m,animation.as_ref(),pose,capture,yaw,controls)}
