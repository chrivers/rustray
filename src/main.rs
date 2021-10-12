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
use std::env;

use indicatif::ParallelProgressIterator;

use image::{Rgb, ColorType, ImageBuffer, GenericImageView};
use image::png::PngEncoder;

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

const WIDTH:  u32 = 1440;
const HEIGHT: u32 = 1440;

fn main() -> RResult<()>
{
    let mut time = TimeSlice::new("startup");

    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    type F = f32;

    let name = env::args().last().unwrap();
    let path = Path::new(&name);
    info!("---------- {:?} ----------", path);
    let resdir = path.parent().unwrap();
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    time.set("parse");
    let p = SbtParser::<F>::parse(Rule::program, &data)?;

    time.set("construct");
    let scene = SbtParser::<F>::parse_file(p, resdir, WIDTH, HEIGHT)?;

    /* let scene = demoscene::construct_demo_scene::<F>(&mut time, WIDTH, HEIGHT)?; */

    info!("Loaded scene\ncams={}\nobjs={}\nlights={}", scene.cameras.len(), scene.objects.len(), scene.lights.len());

    let img = draw_image(&mut time, Tracer::new(&scene, &scene.cameras[0]), WIDTH, HEIGHT)?;

    time.set("write");
    save_image("output.png", img)?;

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
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.bright.cyan/blue}] line {pos}/{len} ({eta})")
                .progress_chars("*>-")
        );
        pb
    }
}

fn save_image(name: &str, img: ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> RResult<()>
{
    let buffer = File::create(name)?;
    let png = PngEncoder::new(buffer);
    Ok(png.encode(img.inner(), img.width(), img.height(), ColorType::Rgb8)?)
}

fn draw_image<F: Float, T: RayTarget<F>, L: Light<F>>(time: &mut TimeSlice, tracer: Tracer<F, T, L>, width: u32, height: u32) -> RResult<ImageBuffer<Rgb<u8>, Vec<u8>>>
{
    let pb = pbar::init(height as u64);

    let mut img = ImageBuffer::new(width, height);

    time.set("render");

    let lines: Vec<_> = (0..height).into_par_iter().progress_with(pb).map(|y| {
        tracer.generate_span(y)
    }).collect();

    time.set("copy");

    for (y, line) in lines.iter().enumerate() {
        for (x, pixel) in line.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, image::Rgb(pixel.to_array()));
        }
    }

    Ok(img)
}
