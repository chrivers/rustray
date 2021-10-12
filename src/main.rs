#![allow(clippy::many_single_char_names)]
#![feature(box_syntax)]
#![feature(destructuring_assignment)]
#![feature(const_generics_defaults)]
#![feature(const_fn_trait_bound)]

#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use image::{ColorType, ImageBuffer};
use image::png::PngEncoder;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use log::LevelFilter;
use pest::Parser;

pub mod lib;
pub mod scene;
pub mod tracer;
pub mod material;
pub mod geometry;
pub mod sampler;
pub mod download;
pub mod format;
pub mod demoscene;

#[allow(unused_imports)]
use crate::lib::{Color, Point, Vector, Float, Camera, PointLight, RResult, TimeSlice};
#[allow(unused_imports)]
use crate::lib::vector::Vectorx;
#[allow(unused_imports)]
use crate::geometry::{Sphere, Plane, Triangle, TriangleMesh, Cone, Cylinder};
#[allow(unused_imports)]
use crate::scene::{RayTarget, Light, Scene, BoxScene};
#[allow(unused_imports)]
use crate::sampler::{Sampler, Bilinear, BilinearSampler, DynSampler};
#[allow(unused_imports)]
use crate::material::*;
#[allow(unused_imports)]
use crate::download::{TextureDownloader, ACGDownloader, ACGQuality};
#[allow(unused_imports)]
use crate::format::sbt::{SbtParser, Rule};
#[allow(unused_imports)]
use crate::tracer::Tracer;

const WIDTH:  usize = 1440;
const HEIGHT: usize = 1440;

fn main() -> RResult<()>
{
    let mut time = TimeSlice::new("startup");

    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    type F = f32;

    let name = std::env::args().last().unwrap();
    let path = Path::new(&name);
    info!("---------- {:?} ----------", path);
    let resdir = path.parent().unwrap();
    let mut file = std::fs::File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    time.set("parse");
    let p = SbtParser::<F>::parse(Rule::program, &data)?;

    time.set("construct");
    let scene = SbtParser::<F>::parse_file(p, resdir, WIDTH, HEIGHT)?;

    /* let scene = demoscene::construct_demo_scene::<F>(&mut time, WIDTH, HEIGHT)?; */

    info!("Loaded scene\ncams={}\nobjs={}\nlights={}", scene.cameras.len(), scene.objects.len(), scene.lights.len());
    draw_image(time, Tracer::new(&scene, &scene.cameras[0]))
}


fn draw_image<F: Float, T: RayTarget<F>, L: Light<F>>(mut time: TimeSlice, tracer: Tracer<F, T, L>) -> RResult<()>
{
    time.set("render");

    let pb = ProgressBar::new(HEIGHT as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.bright.cyan/blue}] line {pos}/{len} ({eta})")
            .progress_chars("*>-")
    );

    let mut img = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);

    let lines: Vec<_> = (0..HEIGHT).into_par_iter().map(|y| {
        pb.inc(1);
        tracer.generate_span(y as u32)
    }).collect();

    time.set("copy");

    for (y, line) in lines.iter().enumerate().take(HEIGHT) {
        assert_eq!(line.len(), WIDTH);
        for (x, pixel) in line.iter().enumerate().take(WIDTH) {
            img.put_pixel(x as u32, y as u32, image::Rgb(pixel.to_array()));
        }
    }
    pb.finish();

    time.set("write");

    let buffer = File::create("output.png")?;
    let png = PngEncoder::new(buffer);
    png.encode(&img.into_raw(), WIDTH as u32, HEIGHT as u32, ColorType::Rgb8)?;

    info!("render complete");
    time.stop();
    time.show();

    Ok(())
}
