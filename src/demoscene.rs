#![allow(unused_variables)]
use std::fs::File;
use std::io::{Cursor, Read, Seek};

use image::{DynamicImage, ImageFormat};
use obj::Obj;
use rand::distributions::{Distribution, Standard};
use zip::ZipArchive;

use crate::download::{ACGDownloader, ACGQuality, TextureDownloader};
use crate::geometry::{Plane, Sphere, Triangle};
use crate::light::{Attenuation, PointLight};
use crate::material::{
    BumpPower, Bumpmap, ChessBoard, ChessBoardMode, Fresnel, Matte, Phong, ScaleUV,
};
use crate::sampler::{Adjust, NormalMap, Perlin, SamplerExt, Texel};
use crate::scene::BoxScene;
use crate::types::{Camera, Color, Float, Point, RResult, Vector, Vectorx};
use crate::{point, vec3};

fn load_zip_tex<T: Read + Seek>(
    arch: &mut ZipArchive<T>,
    name: &str,
    format: ImageFormat,
) -> RResult<DynamicImage> {
    info!("  - {}", name);
    let mut file = arch.by_name(name)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    let imgdata: Cursor<&[u8]> = Cursor::new(&data);

    Ok(image::load(imgdata, format)?)
}

fn load_tex3(
    dl: &ACGDownloader,
    name: &str,
) -> RResult<(
    DynamicImage,
    DynamicImage,
    DynamicImage,
    RResult<DynamicImage>,
)> {
    info!("Loading texture archive [{}]", name);
    let zipfile = File::open(dl.download(name)?)?;
    let mut archive = ZipArchive::new(zipfile)?;
    Ok((
        load_zip_tex(
            &mut archive,
            &format!("{name}_1K_Color.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            &mut archive,
            &format!("{name}_1K_NormalDX.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            &mut archive,
            &format!("{name}_1K_Roughness.png"),
            ImageFormat::Png,
        )?,
        load_zip_tex(
            &mut archive,
            &format!("{name}_1K_Metalness.png"),
            ImageFormat::Png,
        ),
    ))
}

#[allow(clippy::too_many_lines)]
pub fn construct_demo_scene<F>(scene: &mut BoxScene<F>) -> RResult<()>
where
    F: Float + Texel,
    Standard: Distribution<F>,
    f32: Into<F>,
{
    scene.add_camera(Camera::parametric(
        vec3!(10.0, 4.5, 10.0),
        vec3!(0.0, 1.0, 0.0),
        Vector::UNIT_Y,
        F::from_f32(50.0),
    ));

    let (h, l) = (1.2.into(), 0.3.into());

    let attn = Attenuation {
        a: (0.3).into(),
        b: (0.2).into(),
        c: (0.01).into(),
    };

    scene.add_light(PointLight::new(
        vec3!(2.0, 2.0, 2.0),
        attn,
        Color::new(h, h, h),
    ));
    scene.add_light(PointLight::new(
        vec3!(2.0, 2.0, 7.0),
        attn,
        Color::new(h, l, l),
    ));
    scene.add_light(PointLight::new(
        vec3!(2.0, 7.0, 2.0),
        attn,
        Color::new(l, h, l),
    ));
    scene.add_light(PointLight::new(
        vec3!(7.0, 2.0, 2.0),
        attn,
        Color::new(l, l, h),
    ));
    scene.add_light(PointLight::new(
        vec3!(5.0, 5.0, 5.0),
        attn,
        Color::new(h, h, h),
    ));

    let dl = ACGDownloader::new("textures/download".into(), ACGQuality::PNG_1K)?;

    let (tex0a, tex0b, tex0r, tex0m) = load_tex3(&dl, "WoodFloor008")?;
    let (tex1a, tex1b, tex1r, tex1m) = load_tex3(&dl, "WoodFloor006")?;
    let (tex2a, tex2b, tex2r, tex2m) = load_tex3(&dl, "Wood069")?;

    let mat_sphere = scene.materials.insert(Box::new(Fresnel::new(
        1.6.into(),
        Color::WHITE,
        Color::WHITE,
    )));
    let mat_white = scene.materials.insert(Box::new(Phong::white()));

    let mat_plane = scene.materials.insert(Box::new(ScaleUV::new(
        (0.1).into(),
        (0.1).into(),
        ChessBoard::new(
            ChessBoardMode::UV,
            Bumpmap::new(
                BumpPower(F::HALF),
                NormalMap::new(tex0b.bilinear()),
                Phong::new(
                    Color::BLACK,
                    tex0a.bilinear(),
                    Color::WHITE,
                    tex0r.bilinear(),
                ),
            ),
            Bumpmap::new(
                BumpPower(F::HALF),
                NormalMap::new(tex1b.bilinear()),
                Phong::new(
                    Color::BLACK,
                    tex1a.bilinear(),
                    Color::WHITE,
                    tex1r.bilinear(),
                ),
            ),
        ),
    )));

    let mat_bmp2 = Bumpmap::new(
        BumpPower(F::HALF),
        NormalMap::new(tex2b.bilinear()),
        Phong::new(
            Color::BLACK,
            tex2a.bilinear(),
            Color::WHITE,
            tex2r.bilinear(),
        ),
    );

    let obj = Obj::load("models/teapot.obj")?;

    crate::format::obj::load(obj, scene)?;

    let plane1 = Plane::new(
        vec3!(0.0, 0.0, 20.0),
        vec3!(-1.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        mat_plane,
    );
    let plane2 = Plane::new(
        vec3!(0.0, 0.0, 0.0),
        vec3!(1.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        mat_plane,
    );
    let plane3 = Plane::new(
        vec3!(20.0, 0.0, 0.0),
        vec3!(0.0, -1.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        mat_plane,
    );
    let plane4 = Plane::new(
        vec3!(0.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
        vec3!(0.0, 0.0, 1.0),
        mat_plane,
    );
    let plane5 = Plane::new(
        vec3!(0.0, 20.0, 0.0),
        vec3!(0.0, 0.0, -1.0),
        vec3!(1.0, 0.0, 0.0),
        mat_plane,
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

    let sphere1 = Sphere::place(vec3!(1.0, 3.0, 5.0), 1.0.into(), mat_sphere);
    let sphere2 = Sphere::place(vec3!(4.0, 1.0, 1.0), 1.0.into(), mat_sphere);
    let sphere3 = Sphere::place(vec3!(2.0, 3.0, 9.0), 1.0.into(), mat_sphere);
    let sphere4 = Sphere::place(vec3!(1.0, 5.0, 4.0), 1.0.into(), mat_sphere);

    let sphere5 = Sphere::place(vec3!(3.0, 3.0, 1.0), 1.0.into(), mat_sphere);
    let sphere6 = Sphere::place(vec3!(2.0, 2.0, 3.0), 2.0.into(), mat_sphere);
    let sphere7 = Sphere::place(vec3!(6.0, 6.0, 8.0), 1.0.into(), mat_sphere);
    let sphere8 = Sphere::place(vec3!(4.0, 4.0, -1.0), 3.0.into(), mat_sphere);
    let sphere9 = Sphere::place(vec3!(4.0, -1.0, 4.0), 3.0.into(), mat_sphere);
    let sphere10 = Sphere::place(vec3!(-1.0, 4.0, 4.0), 3.0.into(), mat_sphere);

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
        mat_white,
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

    scene.add_geometry(plane2);
    scene.add_geometry(plane4);
    scene.add_geometry(plane6);
    // scene.add_geometry(plane1);
    // scene.add_geometry(plane3);
    // scene.add_geometry(plane5);

    // scene.add_object(sphere1);
    // scene.add_object(sphere2);
    // scene.add_object(sphere3);
    // scene.add_object(sphere4);
    // scene.add_object(sphere5);
    // scene.add_object(sphere6);
    // scene.add_object(sphere7);

    // scene.add_object(sphere8);
    // scene.add_object(sphere9);
    // scene.add_object(sphere10);
    // scene.add_object(sphere11);

    // scene.add_object(tri1);
    // scene.add_object(tri2);
    /* scene.add_object(trimesh1); */
    // scene.add_object(trimesh2);
    // scene.add_object(trimesh3);

    Ok(())
}
