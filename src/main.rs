#![allow(unused_variables)]
#![allow(clippy::many_single_char_names)]
#![feature(box_syntax)]
#![feature(destructuring_assignment)]
#![feature(const_generics_defaults)]
#![feature(const_fn_trait_bound)]

#[macro_use]
extern crate log;

use std::fs::File;
use std::io::{Read, Seek, Cursor};

use image::{ColorType, DynamicImage, ImageBuffer, ImageFormat};
use image::png::PngEncoder;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use std::time::Instant;
use log::LevelFilter;
use obj::ObjData;
use zip::ZipArchive;

pub mod lib;
pub mod scene;
pub mod tracer;
pub mod material;
pub mod geometry;
pub mod sampler;
pub mod download;

use crate::lib::{Color, Point, Vector, Float, Camera, Light, Error};
use crate::geometry::{Sphere, Plane, Triangle, TriangleMesh};
use crate::scene::RayTarget;
#[allow(unused_imports)]
use crate::sampler::{Sampler, Bilinear, BilinearSampler};
#[allow(unused_imports)]
use crate::material::{ChessBoard, Mirror, Fresnel, Phong, ScaleUV, Blend, Texture, Bumpmap, ColorNormal, ColorUV, Matte, TextureSampler};
use crate::download::TextureDownloader;
use crate::download::{ACGDownloader, ACGQuality};

fn main() -> Result<(), Error>
{
    let mut logger = colog::builder();
    logger.filter(None, LevelFilter::Debug);
    logger.init();

    info!("rustray initialized");

    type F = f64;

    // const WIDTH:  usize = 1920;
    // const HEIGHT: usize = 1080;
    const WIDTH:  usize = 2560;
    const HEIGHT: usize = 1440;

    let camera = Camera::parametric(
        vec3!(10.0, 4.5, 10.0),
        vec3!(0.0, 1.0, 0.0),
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

    fn load_zip_tex<T: Read + Seek>(arch: &mut ZipArchive<T>, name: &str, format: ImageFormat) -> Result<DynamicImage, Error>
    {
        info!("  - {}", name);
        let mut file = arch.by_name(name)?;
        let mut data = vec![0u8; file.size().try_into().unwrap()];
        let sz = file.read_exact(&mut data).unwrap();
        let imgdata: Cursor<&[u8]> = Cursor::new(&data);

        extern crate image;
        Ok(image::load(imgdata, format)?)
    }

    fn load_tex3(dl: &ACGDownloader, name: &str) -> Result<(DynamicImage, DynamicImage, DynamicImage, Result<DynamicImage, Error>), Error>
    {
        info!("Loading texture archive [{}]", name);
        let zipfile = File::open(dl.download(name)?)?;
        let mut archive = zip::ZipArchive::new(zipfile)?;
        Ok((
            load_zip_tex(&mut archive, &format!("{}_1K_Color.png", name), ImageFormat::Png)?,
            load_zip_tex(&mut archive, &format!("{}_1K_NormalDX.png", name), ImageFormat::Png)?,
            load_zip_tex(&mut archive, &format!("{}_1K_Roughness.png", name), ImageFormat::Png)?,
            load_zip_tex(&mut archive, &format!("{}_1K_Metalness.png", name), ImageFormat::Png),
        ))
    }

    let dl = ACGDownloader::new("textures/download".into(), ACGQuality::PNG_1K)?;

    let (tex0a, tex0b, tex0r, tex0m) = load_tex3(&dl, "WoodFloor008")?;
    let (tex1a, tex1b, tex1r, tex1m) = load_tex3(&dl, "WoodFloor006")?;
    let (tex2a, tex2b, tex2r, tex2m) = load_tex3(&dl, "Wood069")?;

    let mat_sphere = Fresnel::new(1.6);
    let mat_white  = Phong::white();

    let mat_plane = ChessBoard::new(
        Bumpmap::new(0.5, tex0b.bilinear(), Phong::new(tex0r, tex0a.bilinear().texture())),
        Bumpmap::new(0.5, tex1b.bilinear(), Phong::new(tex1r, tex1a.bilinear().texture())));

    let mat_bmp2 = Bumpmap::new(0.5, tex2b.bilinear(), Phong::new(tex2r.clone(), tex2a.bilinear().texture()));

    let reader = File::open("models/teapot.obj").expect("Failed to open model file");
    let obj = ObjData::load_buf(reader).unwrap();
    let trimesh1 = TriangleMesh::load_obj(obj, vec3!(0.5, 0.0, 1.5), F::from_f32(1.0/6.0), &mat_bmp2);

    let plane1  = Plane::new(vec3!(  0.0,  0.0,   20.0), vec3!( -1.0, 0.0, 0.0), vec3!( 0.0, 1.0,  0.0), &mat_plane);
    let plane2  = Plane::new(vec3!(  0.0,  0.0,  - 0.0), vec3!(  1.0, 0.0, 0.0), vec3!( 0.0, 1.0,  0.0), &mat_plane);

    let plane3  = Plane::new(vec3!(  20.0,  0.0,   0.0), vec3!( 0.0, -1.0, 0.0), vec3!(0.0, 0.0,  1.0), &mat_plane);
    let plane4  = Plane::new(vec3!( - 0.0,  0.0,   0.0), vec3!( 0.0,  1.0, 0.0), vec3!(0.0, 0.0,  1.0), &mat_plane);

    let plane5  = Plane::new(vec3!(   0.0,  20.0,  0.0), vec3!(  0.0, 0.0,-1.0), vec3!(1.0, 0.0,  0.0), &mat_plane);
    let plane6  = Plane::new(vec3!(   0.0, - 0.0,  0.0), vec3!(  0.0, 0.0, 1.0), vec3!(1.0, 0.0,  0.0), &mat_plane);

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

    let sphere11 = Sphere::new(vec3!( 3.0, 3.0, 3.0), 2.0, &mat_sphere);

    let tri1 = Triangle::new(
        vec3!(1.0, 0.0, 3.0), vec3!(5.0, 5.0, 3.0), vec3!(5.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0),
        point!(0.0, 0.0), point!(0.0, 1.0), point!(1.0, 0.0),
        &mat_white
    );

    let tri2 = Triangle::new(
        vec3!(5.0, 5.0, 3.0), vec3!(1.0, 5.0, 3.0), vec3!(1.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0),
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
        // &sphere11,

        // &tri1,
        // &tri2,
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

    let buffer = File::create("output.png")?;
    let png = PngEncoder::new(buffer);
    png.encode(&img.into_raw(), WIDTH as u32, HEIGHT as u32, ColorType::Rgb8)?;

    Ok(())
}
