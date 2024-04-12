#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use std::str::FromStr;

use log::LevelFilter;
use pest::Parser as PestParser;

use rustray::format::sbt2::{Rule as Rule2, SbtBuilder, SbtParser2};
use rustray::sampler::Texel;
use rustray::scene::BoxScene;
use rustray::types::{Error, Float, RResult, TimeSlice};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = 1440)]
    width: u32,

    #[arg(short, long, default_value_t = 1200)]
    height: u32,

    #[arg(value_name = "input")]
    input: PathBuf,

    #[arg(short, long, value_name = "output")]
    output: Option<PathBuf>,
}

fn load_scene<F>(
    time: &mut TimeSlice,
    path: PathBuf,
    width: u32,
    height: u32,
) -> RResult<BoxScene<F>>
where
    F: Float + FromStr + Texel,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let name = path
        .to_str()
        .ok_or(Error::ParseError("Invalid UTF-8 filename"))?;

    info!("=={:=<60}==", format!("[ {:50} ]", name));
    let resdir = path.parent().unwrap();
    let mut file = File::open(&path)?;

    time.set("parse");

    /* Option 1: Scene from .ply file */
    /* let scene = PlyParser::<F>::parse_file(&mut file, resdir, WIDTH, HEIGHT)?; */

    /* Option 2: Scene from .ray file */

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    /* /\* Option 2a: Scene from .ray file (old parser) *\/ */
    /* let p = SbtParser::<F>::parse(Rule::program, &data).map_err(|err| err.with_path(&name))?; */

    /* time.set("construct"); */
    /* let scene = SbtParser::<F>::parse_file(p, resdir, width, height)?; */

    /* Option 2b: Scene from .ray file (new parser) */

    let p = SbtParser2::<F>::parse(Rule2::program, &data).map_err(|err| err.with_path(name))?;
    time.set("ast");
    /* SbtParser2::<F>::dump(p.clone())?; */
    let p = SbtParser2::<F>::ast(p)?;
    /* info!("AST {:#?}", p); */
    time.set("build");
    let scene = SbtBuilder::new(width, height, resdir).build(p)?;

    /* Option 3: Scene from built-in constructor */
    /* use rustray::demoscene; */
    /* let scene = demoscene::construct_demo_scene::<F>(&mut time, width, height)?; */

    Ok(scene)
}

fn main() -> RResult<()> {
    let mut logger = colog::default_builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    let cli = Cli::parse();

    type F = f64;

    let mut time = TimeSlice::new("startup");
    let scene = load_scene::<F>(&mut time, cli.input, cli.width, cli.height)?;
    /* let scene = rustray::demoscene::construct_demo_scene::<F>(&mut time, cli.width, cli.height)?; */

    info!(
        "Loaded scene\ncams={}\nobjs={}\nlights={}",
        scene.cameras.len(),
        scene.objects.len(),
        scene.lights.len()
    );

    #[cfg(feature = "gui")]
    return rustray::frontend::gui::run(scene, cli.width, cli.height);

    #[cfg(not(feature = "gui"))]
    return rustray::frontend::cli::run(
        scene,
        cli.width,
        cli.height,
        cli.output
            .unwrap_or_else(|| PathBuf::from_str("output.png").unwrap()),
    );
}

#[cfg(test)]
mod test {
    use rustray::demoscene;
    use rustray::types::TimeSlice;

    #[test]
    fn test_demoscene() {
        colog::init();
        let mut time = TimeSlice::new("test");
        demoscene::construct_demo_scene::<f32>(&mut time, 640, 480).unwrap();
    }
}
