#![allow(unused_variables)]
use std::fs::File;
use std::io::{Read, Seek, Cursor};
use zip::ZipArchive;
use obj::ObjData;

use crate::lib::{Color, Point, Vector, Float, Camera, PointLight, RResult, TimeSlice};
use crate::lib::vector::Vectorx;
use crate::geometry::{Sphere, Plane, Triangle, TriangleMesh};
use crate::material::*;
use crate::download::{TextureDownloader, ACGDownloader, ACGQuality};
use crate::sampler::{BilinearSampler};

use crate::scene::{RayTarget, Light, Scene, BoxScene};

use crate::{point, vec3};

use image::{DynamicImage, ImageFormat};

pub fn construct_demo_scene<F: Float>(time: &mut TimeSlice, width: usize, height: usize) -> RResult<BoxScene<F>>
where
    F: 'static,
    f32: Into<F>,
{
    time.set("construct");

    let cameras = vec![
        Camera::parametric(
            vec3!(10.0, 4.5, 10.0),
            vec3!(0.0, 1.0, 0.0),
            Vector::identity_y(),
            F::from_f32(50.0),
            width,
            height,
        )
    ];

    let (h, l) = (0.8.into(), 0.2.into());
    let light1 = PointLight { pos: vec3!( 2.0, 2.0, 2.0 ), color: Color { r: h, g: h, b: h }, a: l, b: l, c: l };
    let light2 = PointLight { pos: vec3!( 2.0, 2.0, 7.0 ), color: Color { r: h, g: l, b: l }, a: l, b: l, c: l };
    let light3 = PointLight { pos: vec3!( 2.0, 7.0, 2.0 ), color: Color { r: l, g: h, b: l }, a: l, b: l, c: l };
    let light4 = PointLight { pos: vec3!( 7.0, 2.0, 2.0 ), color: Color { r: l, g: l, b: h }, a: l, b: l, c: l };
    let light5 = PointLight { pos: vec3!( 5.0, 5.0, 5.0 ), color: Color { r: h, g: h, b: h }, a: l, b: l, c: l };

    let lights: Vec<Box<dyn Light<F>>> = vec![
        box light1,
        box light2,
        box light3,
        box light4,
        box light5,
    ];

    fn load_zip_tex<T: Read + Seek>(time: &mut TimeSlice, arch: &mut ZipArchive<T>, name: &str, format: ImageFormat) -> RResult<DynamicImage>
    {
        info!("  - {}", name);
        time.set("zipload");
        let mut file = arch.by_name(name)?;
        let mut data = vec![0u8; file.size() as usize];
        let sz = file.read_exact(&mut data)?;
        let imgdata: Cursor<&[u8]> = Cursor::new(&data);

        Ok(image::load(imgdata, format)?)
    }

    fn load_tex3(time: &mut TimeSlice, dl: &ACGDownloader, name: &str) -> RResult<(DynamicImage, DynamicImage, DynamicImage, RResult<DynamicImage>)>
    {
        info!("Loading texture archive [{}]", name);
        time.set("download");
        let zipfile = File::open(dl.download(name)?)?;
        let mut archive = zip::ZipArchive::new(zipfile)?;
        Ok((
            load_zip_tex(time, &mut archive, &format!("{}_1K_Color.png", name), ImageFormat::Png)?,
            load_zip_tex(time, &mut archive, &format!("{}_1K_NormalDX.png", name), ImageFormat::Png)?,
            load_zip_tex(time, &mut archive, &format!("{}_1K_Roughness.png", name), ImageFormat::Png)?,
            load_zip_tex(time, &mut archive, &format!("{}_1K_Metalness.png", name), ImageFormat::Png),
        ))
    }

    let dl = ACGDownloader::new("textures/download".into(), ACGQuality::PNG_1K)?;

    let (tex0a, tex0b, tex0r, tex0m) = load_tex3(time, &dl, "WoodFloor008")?;
    let (tex1a, tex1b, tex1r, tex1m) = load_tex3(time, &dl, "WoodFloor006")?;
    let (tex2a, tex2b, tex2r, tex2m) = load_tex3(time, &dl, "Wood069")?;

    time.set("construct");
    let mat_sphere = Fresnel::new(1.6.into()).dynamic();
    let mat_white  = Phong::white().dynamic();

    let mat_plane = ChessBoard::new(
        Bumpmap::new(0.5.into(), tex0b.bilinear(), Phong::new(tex0r, tex0a.bilinear().texture())),
        Bumpmap::new(0.5.into(), tex1b.bilinear(), Phong::new(tex1r, tex1a.bilinear().texture()))).dynamic();

    let mat_bmp2 = Bumpmap::new(0.5.into(), tex2b.bilinear(), Phong::new(tex2r, tex2a.bilinear().texture())).dynamic();

    time.set("objload");
    let reader = File::open("models/teapot.obj").expect("Failed to open model file");
    let obj = ObjData::load_buf(reader)?;

    let trimesh1 = TriangleMesh::load_obj(obj, vec3!(0.5, 0.0, 1.5), F::from_f32(1.0/5.0), mat_bmp2);

    time.set("construct");
    let plane1   = Plane::new(vec3!( 0.0,  0.0, 20.0), vec3!(-1.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0), mat_plane.clone());
    let plane2   = Plane::new(vec3!( 0.0,  0.0,  0.0), vec3!( 1.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0), mat_plane.clone());
    let plane3   = Plane::new(vec3!(20.0,  0.0,  0.0), vec3!( 0.0,-1.0, 0.0), vec3!(0.0, 0.0, 1.0), mat_plane.clone());
    let plane4   = Plane::new(vec3!( 0.0,  0.0,  0.0), vec3!( 0.0, 1.0, 0.0), vec3!(0.0, 0.0, 1.0), mat_plane.clone());
    let plane5   = Plane::new(vec3!( 0.0, 20.0,  0.0), vec3!( 0.0, 0.0,-1.0), vec3!(1.0, 0.0, 0.0), mat_plane.clone());
    let plane6   = Plane::new(vec3!( 0.0,  0.0,  0.0), vec3!( 0.0, 0.0, 1.0), vec3!(1.0, 0.0, 0.0), mat_plane.clone());

    let sphere1  = Sphere::new(vec3!(1.0, 3.0, 5.0), 1.0.into(), mat_sphere.clone());
    let sphere2  = Sphere::new(vec3!(4.0, 1.0, 1.0), 1.0.into(), mat_sphere.clone());
    let sphere3  = Sphere::new(vec3!(2.0, 3.0, 9.0), 1.0.into(), mat_sphere.clone());
    let sphere4  = Sphere::new(vec3!(1.0, 5.0, 4.0), 1.0.into(), mat_sphere.clone());

    let sphere5  = Sphere::new(vec3!( 3.0, 3.0, 1.0),  1.0.into(), mat_sphere.clone());
    let sphere6  = Sphere::new(vec3!( 2.0, 2.0, 3.0),  2.0.into(), mat_sphere.clone());
    let sphere7  = Sphere::new(vec3!( 6.0, 6.0, 8.0),  1.0.into(), mat_sphere.clone());
    let sphere8  = Sphere::new(vec3!( 4.0, 4.0, -1.0), 3.0.into(), mat_sphere.clone());
    let sphere9  = Sphere::new(vec3!( 4.0, -1.0, 4.0), 3.0.into(), mat_sphere.clone());
    let sphere10 = Sphere::new(vec3!( -1.0, 4.0, 4.0), 3.0.into(), mat_sphere.clone());

    let sphere11 = Sphere::new(vec3!( 3.0, 3.0, 3.0), 2.0.into(), mat_sphere);

    let tri1 = Triangle::new(
        vec3!(1.0, 0.0, 3.0), vec3!(5.0, 5.0, 3.0), vec3!(5.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0),
        point!(0.0, 0.0), point!(0.0, 1.0), point!(1.0, 0.0),
        mat_white.clone()
    );

    let tri2 = Triangle::new(
        vec3!(5.0, 5.0, 3.0), vec3!(1.0, 5.0, 3.0), vec3!(1.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0), vec3!(0.0, 0.0, 1.0),
        point!(0.0, 0.0), point!(0.0, 1.0), point!(1.0, 0.0),
        mat_white
    );

    let objects: Vec<Box<dyn RayTarget<F>>> = vec![
        box plane2,
        box plane4,
        box plane6,

        // box plane1,
        // box plane3,
        // box plane5,

        // box sphere1,
        // box sphere2,
        // box sphere3,
        // box sphere4,
        // box sphere5,
        // box sphere6,
        // box sphere7,

        // box sphere8,
        // box sphere9,
        // box sphere10,
        // box sphere11,

        // box tri1,
        // box tri2,
        box trimesh1,
        // box trimesh2,
        // box trimesh3,
    ];

    Ok(Scene { cameras, objects, lights })
}
