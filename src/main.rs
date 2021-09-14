#![allow(unused_variables)]
#![allow(clippy::many_single_char_names)]
#![feature(box_syntax)]
#![feature(destructuring_assignment)]
#![feature(const_generics_defaults)]

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

pub mod traits;
pub mod color;
pub mod vector;
pub mod point;
pub mod camera;
pub mod ray;
pub mod scene;
pub mod sphere;
pub mod plane;
pub mod chessplane;
pub mod light;
pub mod tracer;
pub mod testobj;

use crate::color::Color;
use crate::vector::Vector;
use crate::light::Light;
use crate::sphere::Sphere;
use crate::scene::RayTarget;
use crate::chessplane::ChessPlane;
use crate::testobj::TestObject;

fn main() {
    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    info!("rustray initialized");

    const WIDTH:  usize = 1920;
    const HEIGHT: usize = 1080;
    let scaling = max(WIDTH, HEIGHT) as f32 * 1.5;

    let camera = camera::Camera::parametric(
        Vector::new(5.0, 4.0, -15.0),
        Vector::new(0.0, 0.0, 10.0),
        (50.0f32).to_radians(),
        WIDTH,
        HEIGHT,
    );

    // let camera = camera::Camera::raw(
    //     Vector::new(0.0, 10.0, -30.0),
    //     Vector::new(0.0, 0.0, 10.0),
    //     Vector::new((WIDTH as f32) / scaling, 0.0, 0.0),
    //     Vector::new(0.0, (HEIGHT as f32) / scaling, 0.0),
    // );

    let light1 = Light { pos: vec3!(0.0, 2.0, 0.0), color: Color { r: 2.0, g: 0.0, b: 0.0 } };
    let light2 = Light { pos: vec3!(5.0, 2.0, 5.0), color: Color { r: 0.0, g: 2.0, b: 0.0 } };
    let light3 = Light { pos: vec3!(-5.0, 2.0, 5.0), color: Color { r: 0.0, g: 0.0, b: 4.0 } };
    let light4 = Light { pos: vec3!(0.0, 8.0, 0.0), color: Color { r: 0.0, g: 0.0, b: 8.0 } };

    let lights = vec![
        light1,
        light2,
        light3,
        light4,
    ];

    let testobj = TestObject::new(0.99f32);
    let plane1  = ChessPlane::new(vec3!(0.0,  6.0, 0.0), vec3!(-1.0, 0.0, 0.0), vec3!(0.0, 0.5, -1.0), Color::white());
    let plane2  = ChessPlane::new(vec3!(0.0,  0.0, 0.0), vec3!( 1.0, 0.0, 0.0), vec3!(0.0, 0.0,  1.0), Color::white());
    let sphere1 = Sphere::new(vec3!(0.0, 3.0, 5.0), Color::white(), 1.0);
    let sphere2 = Sphere::new(vec3!(4.0, 0.0, 1.0), Color::white(), 2.0);
    let sphere3 = Sphere::new(vec3!(-4.0, 0.0, 9.0), Color::white(), 3.0);

    let objects: Vec<&dyn RayTarget<f32>> = vec![
        &plane1,
        &plane2,
        &sphere1,
        &sphere2,
        &sphere3,
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

    let bar = ProgressBar::new(HEIGHT as u64);
    let lines: Vec<_> = (0..HEIGHT).into_par_iter().map(|y| {
        pb.inc(1);
        tracer.generate_span(y as u32)
    }).collect();

    let time_b = Instant::now();

    for y in 0..HEIGHT {
        assert_eq!(lines[y].len(), WIDTH as usize);
        for x in 0..WIDTH as usize {
            let colors = lines[y][x].to_array();
            img.put_pixel(x as u32, y as u32, image::Rgb(colors));
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

    //Construct a new ImageBuffer with the specified width and height.

    // //Construct a new by repeated calls to the supplied closure.
    // let img = ImageBuffer::from_fn(512, 512, |x, y| {
    //     let dx: i32 = x as i32 - 256;
    //     let dy: i32 = y as i32 - 256;
    //     let n = ((dx*dx + dy*dy*4) as f32).sqrt() as i32;
    //     match n
    //     {
    //         _ if n < 0 => image::Rgb([255, 255, 255]),
    //         _ if n > 255 => image::Rgb([0, 0, 0]),
    //         _ => {
    //             let n = 255 - n as u8;
    //             image::Rgb([n, n, n])
    //         }
    //     }
    //     // if x % 2 == 0 {
    //     //     image::Luma([(y % 128) as u8])
    //     // } else {
    //     //     image::Luma([255u8])
    //     // }
    // });

    // png.encode(&img.into_raw(), img.width(), img.height(), ColorType::RGB(8)).expect("Failed to encode");
    // Change this to OpenGL::V2_1 if not working.
    // let opengl = OpenGL::V3_2;
    // let window: PistonWindow =
    //     WindowSettings::new("Hello Piston!", [640, 480])
    //     .opengl(opengl)
    //     .exit_on_esc(true)
    //     .build()
    //     .unwrap();
    // for e in window {
    //     e.draw_2d(|c, g| {
    //         clear([1.0; 4], g);
    //         rectangle([1.0, 0.0, 0.0, 1.0], // red
    //                   [0.0, 0.0, 100.0, 100.0],
    //                   c.transform, g);
    //     });
    // }
}
