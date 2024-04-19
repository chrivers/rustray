use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
use pest::Parser;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use image::{ColorType, ImageBuffer, Rgb};

use crate::format::sbt2::{Rule as Rule2, SbtBuilder, SbtParser2};
use crate::sampler::Texel;
use crate::scene::{BoxScene, RayTracer};
use crate::tracer::Tracer;
use crate::types::{Error, Float, RResult, TimeSlice};

mod pbar {
    use indicatif::{ProgressBar, ProgressStyle};

    pub fn init(range: u64) -> ProgressBar {
        let pb = ProgressBar::new(range);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.bright.cyan/blue}] line {pos}/{len} ({eta})").unwrap()
                .progress_chars("*>-")
        );
        pb
    }
}

fn load_scene<F>(time: &mut TimeSlice, path: &Path) -> RResult<BoxScene<F>>
where
    F: Float + FromStr + Texel,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let name = path
        .to_str()
        .ok_or(Error::ParseError("Invalid UTF-8 filename".into()))?;

    info!("=={:=<60}==", format!("[ {:50} ]", name));
    let resdir = path.parent().unwrap();
    let mut file = File::open(path)?;

    time.set("read");

    /* Option 1: Scene from .ply file */
    /* let scene = PlyParser::<F>::parse_file(&mut file, resdir, WIDTH, HEIGHT)?; */

    /* Option 2: Scene from .ray file */

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    /* /\* Option 2a: Scene from .ray file (old parser) *\/ */
    /* let p = SbtParser::parse(Rule::program, &data).map_err(|err| err.with_path(&name))?; */

    /* time.set("construct"); */
    /* let scene = SbtParser::parse_file(p, resdir, width, height)?; */

    /* Option 2b: Scene from .ray file (new parser) */

    time.set("parse");
    let p = SbtParser2::parse(Rule2::program, &data).map_err(|err| err.with_path(name))?;

    time.set("ast");
    /* SbtParser2::<F>::dump(p.clone())?; */
    let p = SbtParser2::ast(p)?;

    /* info!("AST {:#?}", p); */
    time.set("build");
    let scene = SbtBuilder::new(resdir).build(p)?;

    /* Option 3: Scene from built-in constructor */
    /* use rustray::demoscene; */
    /* let scene = demoscene::construct_demo_scene::<F>(&mut time, width, height)?; */

    Ok(scene)
}

fn draw_image<F: Float>(
    time: &mut TimeSlice,
    tracer: &Tracer<F>,
    width: u32,
    height: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let pb = pbar::init(u64::from(height));

    let mut img = ImageBuffer::new(width, height);

    time.set("render");

    let camera = &tracer.scene().cameras[0];

    let indices = 0..height;

    #[cfg(feature = "rayon")]
    let indices = indices.into_par_iter();

    let lines: Vec<_> = indices
        .progress_with(pb)
        .map(|y| tracer.render_span(camera, width, height, y, 1))
        .collect();

    time.set("copy");

    for (y, line) in lines.iter().enumerate() {
        for (x, pixel) in line.pixels.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, Rgb(pixel.to_array()));
        }
    }

    img
}

pub fn run<F>(input: &Path, width: u32, height: u32, output: PathBuf) -> RResult<()>
where
    F: Float + FromStr + From<f32>,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let mut time = TimeSlice::new("startup");
    let scene = load_scene::<F>(&mut time, input)?;
    /* let scene = rustray::demoscene::construct_demo_scene::<F>(&mut time, cli.width, cli.height)?; */

    info!(
        "Loaded scene\ncams={}\nobjs={}\nlights={}",
        scene.cameras.len(),
        scene.objects.len(),
        scene.lights.len()
    );

    let img = draw_image(&mut time, &Tracer::new(&scene), width, height);

    time.set("write");
    image::save_buffer(output, &img, img.width(), img.height(), ColorType::Rgb8)?;

    info!("render complete");
    time.stop();
    time.show();
    Ok(())
}
