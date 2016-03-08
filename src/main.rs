extern crate num;
extern crate image;
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
pub mod light;
pub mod tracer;
use color::Color;
use vector::Vector;
use light::Light;
use sphere::Sphere;
use scene::RayTarget;

fn main() {
    let buffer = File::create("output.png").unwrap();
    let png = PNGEncoder::new(buffer);

    let camera = camera::Camera::new(
        Vector::new(0.0, 10.0, -10.0),
        Vector::new(-5.0, -5.0, 0.0),
        Vector::new(0.0, 10.0, 0.0),
        Vector::new(10.0, 0.0, 0.0),
        512,
        512
    );

    let lights = vec![Box::new(Light { pos: Vector::new(0.0, 0.0, 0.0), color: Color::<f32> { r: 1.0, g: 0.0, b: 0.0 } })];
    let sphere = Sphere::new(Vector::new(0.0, 0.0, 6.0), Color::<f32> { r: 1.0, g: 1.0, b: 1.0 }, 1.0);
    let objects = vec![Box::new(sphere) as Box<RayTarget<f32>>];

    let tracer = tracer::Tracer::new(
        camera,
        objects,
        lights,
    );

    let mut img = ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(32, 32);
    tracer.render_image::<_, _, u8>(&mut img);
    png.encode(&img.into_raw(), 32, 32, ColorType::RGB(8)).expect("Failed to encode");

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
