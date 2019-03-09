#![allow(unused_variables)]
#![feature(box_syntax)]

extern crate num;
extern crate image;
extern crate rand;

use std::fs::File;
use image::ColorType;
use image::png::PNGEncoder;
use image::ImageBuffer;

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
use crate::plane::Plane;
use crate::chessplane::ChessPlane;
use crate::testobj::TestObject;

fn main() {
    let buffer = File::create("output.png").unwrap();
    let png = PNGEncoder::new(buffer);

    let pos = Vector::new(-5.0, 5.0, -10.0);
    let camera = camera::Camera::new(
        pos,
        pos.vector_to(Vector::new(0.0, 0.0, 0.0)),
        Vector::new(12.0, 0.0, 0.0),
        Vector::new(0.0, 12.0, 0.0),
        512,
        512
    );

    let light1 = Light { pos: Vector::new(0.0, 0.0, 0.0), color: Color::<f32> { r: 2.0, g: 0.0, b: 0.0 } };
    let light2 = Light { pos: Vector::new(5.0, 0.0, 5.0), color: Color::<f32> { r: 0.0, g: 2.0, b: 0.0 } };
    let light3 = Light { pos: Vector::new(-5.0, 5.0, 5.0), color: Color::<f32> { r: 0.0, g: 0.0, b: 4.0 } };
    let lights = vec![
        light1,
        light2,
        light3,
    ];
    let testobj = TestObject::new(0.9f32);
    let plane1  = Plane::new(Vector::new(0.0, -1.0, 0.0), Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0), Color::<f32> { r: 1.0, g: 1.0, b: 1.0 });
    let plane2  = ChessPlane::new(Vector::new(0.0, -1.0, 0.0), Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0), Color::<f32> { r: 1.0, g: 1.0, b: 1.0 });
    let sphere1 = Sphere::new(Vector::new(0.0, 1.0, 6.0), Color::<f32>::white(), 1.0);
    let sphere2 = Sphere::new(Vector::new(4.0, -2.0, 1.0), Color::<f32>::white(), 2.0);
    let sphere3 = Sphere::new(Vector::new(-4.0, -2.0, 9.0), Color::<f32>::white(), 3.0);
    let objects: Vec<Box<dyn RayTarget<f32>>> = vec![
        box plane2,
        box sphere1,
        box sphere2,
        box sphere3,
    ];

    let tracer = tracer::Tracer::new(
        camera,
        objects,
        lights,
    );

    const WIDTH: u32 = 512;
    const HEIGHT: u32 = 512;
    let mut img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(WIDTH, HEIGHT);
    tracer.render_image::<_, _, u8>(&mut img);
    png.encode(&img.into_raw(), WIDTH, HEIGHT, ColorType::RGB(8)).expect("Failed to encode");

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
