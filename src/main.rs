#[macro_use]
extern crate log;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use std::str::FromStr;

use log::LevelFilter;
use pest::Parser;

use rustray::format::sbt2::{Rule as Rule2, SbtBuilder, SbtParser2};
use rustray::sampler::Texel;
use rustray::scene::BoxScene;
use rustray::types::{Float, RResult, TimeSlice};

const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1200;

fn load_scene<F: Float + FromStr + Texel>(
    time: &mut TimeSlice,
    width: u32,
    height: u32,
) -> RResult<BoxScene<F>> {
    let name = env::args().last().unwrap();
    let path = Path::new(&name);
    info!("=={:=<60}==", format!("[ {:50} ]", name));
    let resdir = path.parent().unwrap();
    let mut file = File::open(path)?;

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

    let p = SbtParser2::<F>::parse(Rule2::program, &data).map_err(|err| err.with_path(&name))?;
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

    type F = f64;

    let mut time = TimeSlice::new("startup");
    let scene = load_scene::<F>(&mut time, WIDTH, HEIGHT)?;
    /* let scene = rustray::demoscene::construct_demo_scene::<F>(&mut time, WIDTH, HEIGHT)?; */

    info!(
        "Loaded scene\ncams={}\nobjs={}\nlights={}",
        scene.cameras.len(),
        scene.objects.len(),
        scene.lights.len()
    );

    rustray::frontend::cli::run(scene, WIDTH, HEIGHT)
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
