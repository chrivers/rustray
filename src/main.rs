#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::env;

use indicatif::ParallelProgressIterator;

use image::{Rgb, ColorType, ImageBuffer};

use rayon::iter::{ParallelIterator, IntoParallelIterator};
use log::LevelFilter;
use pest::Parser;

use rustray::types::{Float, RResult, TimeSlice};
use rustray::types::result::Error;
use rustray::geometry::{Geometry, FiniteGeometry};
use rustray::scene::Light;
use rustray::format::sbt2::{SbtParser2, Rule as Rule2, SbtBuilder};
use rustray::tracer::Tracer;

const WIDTH:  u32 = 1440;
const HEIGHT: u32 = 1440;

fn main() -> RResult<()>
{
    match runmain() {
        Ok(()) => { },
        Err(Error::IOError(err)) => {
            error!("Error: {}", err)
        }
        Err(Error::PestError(err)) => {
            error!("Error: {}", err);
        }
        Err(Error::PestError2(err)) => {
            error!("Error: {}", err);
        }
        Err(err) => {
            error!("Error: {:#?}", err)
        }
    }
    Ok(())
}

fn runmain() -> RResult<()> {
    let mut time = TimeSlice::new("startup");

    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    type F = f32;

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
    /* let scene = SbtParser::<F>::parse_file(p, resdir, WIDTH, HEIGHT)?; */

    /* Option 2b: Scene from .ray file (new parser) */

    let p = SbtParser2::<F>::parse(Rule2::program, &data).map_err(|err| err.with_path(&name))?;
    time.set("ast");
    /* SbtParser2::<F>::new().dump(p.clone())?; */
    let p = SbtParser2::<F>::ast(p)?;
    /* info!("AST {:#?}", p); */
    time.set("build");
    let scene = SbtBuilder::new(WIDTH, HEIGHT, resdir).build(p)?;

    /* Option 3: Scene from built-in constructor */

    /* let scene = demoscene::construct_demo_scene::<F>(&mut time, WIDTH, HEIGHT)?; */

    info!("Loaded scene\ncams={}\nobjs={}\nlights={}", scene.cameras.len(), scene.objects.len(), scene.lights.len());

    let img = draw_image(&mut time, Tracer::new(&scene), WIDTH, HEIGHT)?;

    time.set("write");
    image::save_buffer("output.png", &img, img.width(), img.height(), ColorType::Rgb8)?;

    info!("render complete");
    time.stop();
    time.show();
    Ok(())
}

mod pbar {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn init(range: u64) -> ProgressBar
    {
        let pb = ProgressBar::new(range);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.bright.cyan/blue}] line {pos}/{len} ({eta})").unwrap()
                .progress_chars("*>-")
        );
        pb
    }
}

fn draw_image<F, B, G, L>(time: &mut TimeSlice, tracer: Tracer<'_, F, B, G, L>, width: u32, height: u32) -> RResult<ImageBuffer<Rgb<u8>, Vec<u8>>>
where
    F: Float,
    B: FiniteGeometry<F>,
    G: Geometry<F>,
    L: Light<F>
{
    let pb = pbar::init(height as u64);

    let mut img = ImageBuffer::new(width, height);

    time.set("render");

    let camera = &tracer.scene().cameras[0];

    let lines: Vec<_> = (0..height).into_par_iter().progress_with(pb).map(|y| {
        tracer.generate_span(camera, y)
    }).collect();

    time.set("copy");

    for (y, line) in lines.iter().enumerate() {
        for (x, pixel) in line.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, Rgb(pixel.to_array()));
        }
    }

    Ok(img)
}
