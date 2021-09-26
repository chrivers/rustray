#![allow(unused_variables)]
#![allow(clippy::many_single_char_names)]
#![feature(box_syntax)]
#![feature(destructuring_assignment)]
#![feature(const_generics_defaults)]
#![feature(const_fn_trait_bound)]

#[macro_use]
extern crate log;

use std::fs::File;
use image::ColorType;
use image::png::PngEncoder;
use image::{ImageBuffer};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use std::cmp::max;
use std::time::Instant;
use log::LevelFilter;

pub mod lib;
pub mod scene;
pub mod tracer;
pub mod material;
pub mod geometry;
pub mod sampler;
pub mod download;

use crate::lib::{Color, Point, Vector, Float, Camera, Light};
use crate::geometry::{Sphere, Plane, Triangle, TriangleMesh};
use crate::scene::RayTarget;
use crate::material::{ChessBoard, Mirror, Fresnel, Phong, ScaleUV, Blend, Texture, Bumpmap};

fn main() {
    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    info!("rustray initialized");

    type F = f64;

    const WIDTH:  usize = 1920;
    const HEIGHT: usize = 1080;
    let scaling = max(WIDTH, HEIGHT) as f32 * 1.5;

    let camera = camera::Camera::parametric(
        vec3!(12.0, 8.0, 16.0),
        vec3!(0.0, 2.0, 0.0),
        (90.0 as F).to_radians(),
        WIDTH,
        HEIGHT,
    );

    let (h, l) = (0.8, 0.2);
    let light1 = Light { pos: vec3!( 2.0, 2.0, 2.0 ), color: Color { r: h, g: h, b: h } };
    let light2 = Light { pos: vec3!( 2.0, 2.0, 7.0 ), color: Color { r: h, g: l, b: l } };
    let light3 = Light { pos: vec3!( 2.0, 7.0, 2.0 ), color: Color { r: l, g: h, b: l } };
    let light4 = Light { pos: vec3!( 7.0, 2.0, 2.0 ), color: Color { r: l, g: l, b: h } };
    let light5 = Light { pos: vec3!( 5.0, 5.0, 5.0 ), color: Color { r: h, g: h, b: h } };

    let lights = vec![
        light1,
        light2,
        light3,
        light4,
        light5,
    ];

    let mat_mirror: Mirror<F> = Mirror::new(1.0);
    let mat_white = Color::white();
    let mat_smooth = Fresnel::new(1.6);
    let mat_sphere = Fresnel::new(1.9);

    let mat_plane = ChessBoard::new(Phong::black(), Phong::white());
    let mat_sphere2 = ChessBoard::new(Phong::new(2.0, Color::black()), Color::white());
    let mat_sphere = ScaleUV::new(2.0, 2.0, Blend::new(mat_sphere2, mat_mirror, 0.8));

    let mat_phong = Phong::<F, _>::new(2.0, Color::<F>::white());

    let tex0 = image::open("textures/stone-albedo.png").unwrap();
    let tex1 = image::open("textures/stone-normal.png").unwrap();
    let mat_tex = Phong::<F, _>::new(2.0, Texture::<F, _>::new(tex0));
    let mat_bmp = ScaleUV::new(1.0, 1.0, Bumpmap::new(0.05, tex1, mat_tex));
    // let mat_bmp = ScaleUV::new(2.1, 2.1, Bumpmap::new(1.0, tex1, mat_tex));
    // let mat_sphere = mat_bmp;
    // let mat_sphere = Phong::<F, _>::new(mat_tex);

    // let mat_plane = mat_tex.clone();

    // let mut reader = File::open("models/teapot.obj").expect("Failed to open model file");
    let mut reader = File::open("models/Porsche_911_GT2.obj").expect("Failed to open model file");
    let trimesh1 = TriangleMesh::load_obj(&mut reader, vec3!(4.0, 2.0, 6.0), F::from_f32(2.0/1.0), &mat_bmp).unwrap();

    let plane1  = Plane::new(vec3!(  0.0,  0.0,   20.0), vec3!( -1.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0), &mat_plane);
    let plane2  = Plane::new(vec3!(  0.0,  0.0,  - 0.0), vec3!( 1.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0), &mat_plane);

    let plane3  = Plane::new(vec3!(  20.0,  0.0,   0.0), vec3!( 0.0, -1.0, 0.0), vec3!(0.0, 0.0, 1.0), &mat_plane);
    let plane4  = Plane::new(vec3!( - 0.0,  0.0,   0.0), vec3!( 0.0, 1.0, 0.0), vec3!(0.0, 0.0, 1.0), &mat_plane);

    let plane5  = Plane::new(vec3!(   0.0,  20.0,  0.0), vec3!( 1.0, 0.0, 0.0), vec3!(0.0, 0.0, 1.0), &mat_plane);
    let plane6  = Plane::new(vec3!(   0.0, - 0.0,  0.0), vec3!( 1.0, 0.0, 0.0), vec3!(0.0, 0.0, -1.0), &mat_plane);

    let sphere1 = Sphere::new(vec3!(1.0, 3.0, 5.0), 1.0, &mat_sphere);
    let sphere2 = Sphere::new(vec3!(4.0, 1.0, 1.0), 1.0, &mat_sphere);
    let sphere3 = Sphere::new(vec3!(2.0, 3.0, 9.0), 1.0, &mat_sphere);
    let sphere4 = Sphere::new(vec3!(1.0, 5.0, 4.0), 1.0, &mat_sphere);

    let sphere5 = Sphere::new(vec3!( 3.0, 3.0, 1.0), 1.0, &mat_sphere);
    let sphere6 = Sphere::new(vec3!( 2.0, 2.0, 3.0), 2.0, &mat_sphere);
    let sphere7 = Sphere::new(vec3!( 6.0, 6.0, 8.0), 1.0, &mat_sphere);
    let sphere8 = Sphere::new(vec3!( 4.0, 4.0, -1.0), 3.0, &mat_sphere);
    let sphere9 = Sphere::new(vec3!( 4.0, -1.0, 4.0), 3.0, &mat_sphere);
    let sphere10 = Sphere::new(vec3!( -1.0, 4.0, 4.0), 3.0, &mat_sphere);

    let tri1 = Triangle::new(
        vec3!(3.0, 1.0, 1.0), vec3!(1.0, 5.0, 1.0), vec3!(1.0, 1.0, 7.0),
        vec3!(3.0, 1.0, 1.0), vec3!(1.0, 5.0, 1.0), vec3!(1.0, 1.0, 7.0),
        point!(0.0, 0.0), point!(0.0, 1.0), point!(1.0, 0.0),
        &mat_white
    );

    let objects: Vec<&dyn RayTarget<F>> = vec![
        &plane2,
        &plane4,
        &plane6,

        &plane1,
        &plane3,
        &plane5,

        // &sphere1,
        // &sphere2,
        // &sphere3,
        // &sphere4,
        // &sphere5,
        // &sphere6,
        // &sphere7,

        // &sphere8,
        // &sphere9,
        // &sphere10,

        // &tri1,
        &trimesh1,
        // &trimesh2,
        // &trimesh3,
    ];

    let tracer = tracer::Tracer::new(
        camera,
        objects,
        lights,
    );

    let time_a = Instant::now();

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

    let time_b = Instant::now();

    for (y, line) in lines.iter().enumerate().take(HEIGHT) {
        assert_eq!(line.len(), WIDTH);
        for (x, pixel) in line.iter().enumerate().take(WIDTH) {
            img.put_pixel(x as u32, y as u32, image::Rgb(pixel.to_array()));
        }
    }
    pb.finish();

    let time_c = Instant::now();
    info!("render complete");
    debug!("  render time:  {:.2?} ms", (time_b - time_a).as_micros() as f32 / 1000f32);
    debug!("  copy time:    {:.2?} ms", (time_c - time_b).as_micros() as f32 / 1000f32);

    let buffer = File::create("output.png").unwrap();
    let png = PngEncoder::new(buffer);
    png.encode(&img.into_raw(), WIDTH as u32, HEIGHT as u32, ColorType::Rgb8).expect("Failed to encode");
}
