use std::path::Path;

use obj::{Obj, ObjMaterial};

use cgmath::{InnerSpace, Matrix4, SquareMatrix};

use crate::geometry::{Triangle, TriangleMesh};
use crate::material::{BoxMaterial, BumpPower, Bumpmap, Fresnel, Phong, Smart};
use crate::sampler::{NormalMap, Sampler, SamplerExt, Texel};
use crate::scene::BoxScene;
use crate::types::{Color, Float, Point, RResult, Vector, Vectorx};

fn obj_sampler1<F: Float + Texel>(
    resdir: &Path,
    map: &Option<String>,
) -> Option<impl Sampler<F, F>> {
    map.as_ref().map(|kd| {
        image::open(resdir.join(kd)).map_or_else(
            |_| {
                warn!("Missing texture [{}]", kd);
                F::ONE.dynsampler()
            },
            |img| {
                info!("Loading [{}]", kd);
                img.bilinear().dynsampler()
            },
        )
    })
}

fn obj_sampler3<F: Float + Texel>(
    resdir: &Path,
    map: &Option<String>,
) -> Option<impl Sampler<F, Color<F>>> {
    map.as_ref().map(|kd| {
        image::open(resdir.join(kd)).map_or_else(
            |_| {
                warn!("Missing texture [{}]", kd);
                Color::WHITE.dynsampler()
            },
            |img| {
                info!("Loading [{}]", kd);
                img.bilinear().dynsampler()
            },
        )
    })
}

pub fn load<F: Float + Texel>(mut obj: Obj, scene: &mut BoxScene<F>) -> RResult<()> {
    let mut corner = Vector::new(F::max_value(), F::max_value(), F::max_value());

    obj.load_mtls()?;
    let position = &obj.data.position;
    let objects = &obj.data.objects;
    let texture = &obj.data.texture;
    let normal: &[[f32; 3]] = &obj.data.normal;

    for o in objects {
        for g in &o.groups {
            for poly in &g.polys {
                for n in 0..(poly.0.len() - 1) {
                    corner = corner.min(&Vector::from_f32s(position[poly.0[n].0]));
                }
            }
        }
    }

    let mut mat0 = None;

    let mut tris = vec![];
    /* info!("mats: {:#?}", obj.data.material_libs); */
    for o in &obj.data.objects {
        for g in &o.groups {
            let mat = if let Some(ObjMaterial::Mtl(ref omat)) = g.material {
                let phong = Phong::new()
                    .with_ke(omat.ke.map_or(Color::BLACK, Color::from))
                    .with_kd(omat.kd.map_or(Color::WHITE, Color::from))
                    .with_ks(omat.ks.map_or(Color::BLACK, Color::from))
                    .with_pow(F::from_f32(omat.ns.unwrap_or(8.0)))
                    .with_ke_map(obj_sampler3(&obj.path, &omat.map_ke))
                    .with_kd_map(obj_sampler3(&obj.path, &omat.map_kd))
                    .with_ks_map(obj_sampler3(&obj.path, &omat.map_ks))
                    .with_pow_map(obj_sampler1(&obj.path, &omat.map_ns))
                    .with_ambient(omat.ka.map_or(Color::BLACK, Into::into));

                let fresnel = Fresnel::new(
                    F::from_f32(omat.ni.unwrap_or(1.0)),
                    omat.tf.map_or(Color::BLACK, Color::from),
                    Color::BLACK,
                );

                let smart = Smart::make(phong, fresnel);

                let res: BoxMaterial<F> = if omat.map_bump.is_some() {
                    let bumpmap = NormalMap::new(obj_sampler3(&obj.path, &omat.map_bump).unwrap());
                    let bump = Bumpmap::new(BumpPower(F::HALF), bumpmap, smart);
                    Box::new(bump)
                } else {
                    Box::new(smart)
                };
                scene.materials.insert(res)
            } else {
                *mat0.get_or_insert_with(|| {
                    let mat = Box::new(Phong::white());
                    scene.materials.insert(mat)
                })
            };

            for poly in &g.polys {
                for n in 1..(poly.0.len() - 1) {
                    let a = Vector::from_f32s(position[poly.0[0].0]) - corner;
                    let b = Vector::from_f32s(position[poly.0[n].0]) - corner;
                    let c = Vector::from_f32s(position[poly.0[n + 1].0]) - corner;

                    /* FIXME: .unwrap() is a terrible when loading data from a file */
                    let (na, nb, nc) = if normal.is_empty() {
                        let n = (a - b).cross(a - c);
                        (n, n, n)
                    } else {
                        (
                            Vector::from_f32s(normal[poly.0[0].2.unwrap()]).normalize(),
                            Vector::from_f32s(normal[poly.0[n].2.unwrap()]).normalize(),
                            Vector::from_f32s(normal[poly.0[n + 1].2.unwrap()]).normalize(),
                        )
                    };

                    let (mut ta, mut tb, mut tc) = {
                        let tpa = poly.0[0].1;
                        let tpb = poly.0[n].1;
                        let tpc = poly.0[n + 1].1;
                        match (tpa, tpb, tpc) {
                            (Some(a), Some(b), Some(c)) => {
                                (texture[a].into(), texture[b].into(), texture[c].into())
                            }
                            _ => (Point::ZERO, Point::ZERO, Point::ZERO),
                        }
                    };
                    ta.y = F::ONE - ta.y;
                    tb.y = F::ONE - tb.y;
                    tc.y = F::ONE - tc.y;

                    let tri = Triangle::new(a, b, c, na, nb, nc, ta, tb, tc, mat);
                    tris.push(tri);
                }
            }
        }
    }

    info!(
        "loaded .obj [index: {}, normal: {}, uv: {}, face: {}]",
        position.len(),
        obj.data.normal.len(),
        obj.data.texture.len(),
        tris.len()
    );

    let mesh = TriangleMesh::new(tris, Matrix4::identity());
    scene.add_object(mesh);

    Ok(())
}
