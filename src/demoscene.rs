#![allow(unused_variables)]
use obj::Obj;
use rand::distributions::{Distribution, Standard};
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

use crate::download::{ACGDownloader, ACGQuality, TextureDownloader};
use crate::geometry::{FiniteGeometry, Geometry, Plane, Sphere, Triangle, TriangleMesh};
use crate::light::{Attenuation, Light, PointLight};
use crate::material::*;
use crate::sampler::{Adjust, NormalMap, Perlin, SamplerExt, Texel};
use crate::types::vector::Vectorx;
use crate::types::{Camera, Color, Float, MaterialLib, Point, RResult, TimeSlice, Vector};

use crate::scene::{BoxScene, Scene};

use crate::{point, vec3};

use image::{DynamicImage, ImageFormat};

fn point_light<F: Float>(pos: Vector<F>, color: Color<F>) -> impl Light<F>
where
    f32: Into<F>,
{
    let attn = Attenuation {
        a: (0.3).into(),
        b: (0.2).into(),
        c: (0.01).into(),
    };

    PointLight { attn, pos, color }
}

fn load_zip_tex<T: Read + Seek>(
    time: &mut TimeSlice,
    arch: &mut ZipArchive<T>,
    name: &str,
    format: ImageFormat,
) -> RResult<DynamicImage> {
    info!("  - {}", name);
    time.set("zipload");
    let mut file = arch.by_name(name)?;
    let mut data = vec![0u8; file.size() as usize];
    file.read_exact(&mut data)?;
    let imgdata: Cursor<&[u8]> = Cursor::new(&data);

    Ok(image::load(imgdata, format)?)
}

fn load_tex3(
    time: &mut TimeSlice,
    dl: &ACGDownloader,
    name: &str,
) -> RResult<(
    DynamicImage,
    DynamicImage,
    DynamicImage,
    RResult<DynamicImage>,
)> {
    info!("Loading texture archive [{}]", name);
    time.set("download");
    let zipfile = File::open(dl.download(name)?)?;
    let mut archive = ZipArchive::new(zipfile)?;
    Ok((
        load_zip_tex(
            time,
            &mut archive,
            &format!("{name}_1K_Color.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            time,
            &mut archive,
            &format!("{name}_1K_NormalDX.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            time,
            &mut archive,
            &format!("{name}_1K_Roughness.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            time,
            &mut archive,
            &format!("{name}_1K_Metalness.png"),
            ImageFormat::Png,
        ),
    ))
}

#[allow(clippy::too_many_lines)]
pub fn construct_demo_scene<F>(
    time: &mut TimeSlice,
    width: u32,
    height: u32,
) -> RResult<BoxScene<F>>
where
    F: Float + Texel,
    Standard: Distribution<F>,
    f32: Into<F>,
{
    time.set("construct");

    let mut materials = MaterialLib::new();

    let cameras = vec![Camera::parametric(
        vec3!(10.0, 4.5, 10.0),
        vec3!(0.0, 1.0, 0.0),
        Vector::UNIT_Y,
        F::from_f32(50.0),
        width,
        height,
    )];

    let (h, l) = (1.2.into(), 0.3.into());

    let light1 = point_light(vec3!(2.0, 2.0, 2.0), Color { r: h, g: h, b: h });
    let light2 = point_light(vec3!(2.0, 2.0, 7.0), Color { r: h, g: l, b: l });
    let light3 = point_light(vec3!(2.0, 7.0, 2.0), Color { r: l, g: h, b: l });
    let light4 = point_light(vec3!(7.0, 2.0, 2.0), Color { r: l, g: l, b: h });
    let light5 = point_light(vec3!(5.0, 5.0, 5.0), Color { r: h, g: h, b: h });

    let lights: Vec<Box<dyn Light<F>>> = vec![
        Box::new(light1),
        Box::new(light2),
        Box::new(light3),
        Box::new(light4),
        Box::new(light5),
    ];

    let dl = ACGDownloader::new("textures/download".into(), ACGQuality::PNG_1K)?;

    let (tex0a, tex0b, tex0r, tex0m) = load_tex3(time, &dl, "WoodFloor008")?;
    let (tex1a, tex1b, tex1r, tex1m) = load_tex3(time, &dl, "WoodFloor006")?;
    let (tex2a, tex2b, tex2r, tex2m) = load_tex3(time, &dl, "Wood069")?;

    time.set("construct");
    let mat_sphere = Fresnel::new(1.6.into(), Color::WHITE, Color::WHITE).dynamic();
    let mat_white = Phong::white().dynamic();

    let mat_plane = ScaleUV::new(
        (0.1).into(),
        (0.1).into(),
        ChessBoard::new(
            ChessBoardMode::UV,
            Bumpmap::new(
                BumpPower(F::HALF),
                NormalMap::new(tex0b.bilinear()),
                Phong::new(tex0r.bilinear(), Color::WHITE, tex0a.bilinear()),
            ),
            Bumpmap::new(
                BumpPower(F::HALF),
                NormalMap::new(tex1b.bilinear()),
                Phong::new(tex1r.bilinear(), Color::WHITE, tex1a.bilinear()),
            ),
        ),
    );

    let mat_bmp2 = Bumpmap::new(
        BumpPower(F::HALF),
        NormalMap::new(tex2b.bilinear()),
        Phong::new(tex2r.bilinear(), Color::WHITE, tex2a.bilinear()),
    );

    time.set("objload");
    let obj = Obj::load("models/teapot.obj")?;

    let trimesh1 = TriangleMesh::load_obj(
        obj,
        &mut materials,
        vec3!(0.5, 0.0, 1.5),
        F::from_f32(1.0 / 5.0),
    )?;

    time.set("construct");
    let plane1 = Plane::new(
        vec3!(0.0, 0.0, 20.0),
        vec3!(-1.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        mat_plane.clone(),
    );
    let plane2 = Plane::new(
        vec3!(0.0, 0.0, 0.0),
        vec3!(1.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        mat_plane.clone(),
    );
    let plane3 = Plane::new(
        vec3!(20.0, 0.0, 0.0),
        vec3!(0.0, -1.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        mat_plane.clone(),
    );
    let plane4 = Plane::new(
        vec3!(0.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        mat_plane.clone(),
    );
    let plane5 = Plane::new(
        vec3!(0.0, 20.0, 0.0),
        vec3!(0.0, 0.0, -1.0),
        vec3!(1.0, 0.0, 0.0),
        mat_plane.clone(),
    );
    let plane6 = Plane::new(
        vec3!(0.0, 0.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(1.0, 0.0, 0.0),
        mat_plane,
    );

    let mat_matte = ScaleUV::new(
        16.0.into(),
        16.0.into(),
        Matte::new(
            Adjust::new((0.1).into(), (0.00).into(), Perlin::new(3, 3)),
            8,
            Fresnel::new(0.1.into(), Color::WHITE, Color::WHITE),
        ),
    );

    let sphere1 = Sphere::place(vec3!(1.0, 3.0, 5.0), 1.0.into(), mat_sphere.clone());
    let sphere2 = Sphere::place(vec3!(4.0, 1.0, 1.0), 1.0.into(), mat_sphere.clone());
    let sphere3 = Sphere::place(vec3!(2.0, 3.0, 9.0), 1.0.into(), mat_sphere.clone());
    let sphere4 = Sphere::place(vec3!(1.0, 5.0, 4.0), 1.0.into(), mat_sphere.clone());

    let sphere5 = Sphere::place(vec3!(3.0, 3.0, 1.0), 1.0.into(), mat_sphere.clone());
    let sphere6 = Sphere::place(vec3!(2.0, 2.0, 3.0), 2.0.into(), mat_sphere.clone());
    let sphere7 = Sphere::place(vec3!(6.0, 6.0, 8.0), 1.0.into(), mat_sphere.clone());
    let sphere8 = Sphere::place(vec3!(4.0, 4.0, -1.0), 3.0.into(), mat_sphere.clone());
    let sphere9 = Sphere::place(vec3!(4.0, -1.0, 4.0), 3.0.into(), mat_sphere.clone());
    let sphere10 = Sphere::place(vec3!(-1.0, 4.0, 4.0), 3.0.into(), mat_sphere.clone());

    let sphere11 = Sphere::place(vec3!(3.0, 3.0, 3.0), 2.0.into(), mat_sphere);

    let tri1 = Triangle::new(
        vec3!(1.0, 0.0, 3.0),
        vec3!(5.0, 5.0, 3.0),
        vec3!(5.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(0.0, 0.0, 1.0),
        point!(0.0, 0.0),
        point!(0.0, 1.0),
        point!(1.0, 0.0),
        mat_white.clone(),
    );

    let tri2 = Triangle::new(
        vec3!(5.0, 5.0, 3.0),
        vec3!(1.0, 5.0, 3.0),
        vec3!(1.0, 0.0, 3.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(0.0, 0.0, 1.0),
        vec3!(0.0, 0.0, 1.0),
        point!(0.0, 0.0),
        point!(0.0, 1.0),
        point!(1.0, 0.0),
        mat_white,
    );

    let geometry: Vec<Box<dyn Geometry<F>>> = vec![
        Box::new(plane2),
        Box::new(plane4),
        Box::new(plane6),
        // Box::new(plane1),
        // Box::new(plane3),
        // Box::new(plane5),
    ];

    let objects: Vec<Box<dyn FiniteGeometry<F>>> = vec![
        // Box::new(sphere1),
        // Box::new(sphere2),
        // Box::new(sphere3),
        // Box::new(sphere4),
        // Box::new(sphere5),
        // Box::new(sphere6),
        // Box::new(sphere7),

        // Box::new(sphere8),
        // Box::new(sphere9),
        // Box::new(sphere10),
        // Box::new(sphere11),

        // Box::new(tri1),
        // Box::new(tri2),
        Box::new(trimesh1),
        // Box::new(trimesh2),
        // Box::new(trimesh3),
    ];

    Scene::new(cameras, objects, geometry, materials, lights)
}
