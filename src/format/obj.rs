use std::path::Path;

use obj::{Obj, ObjMaterial};

use cgmath::InnerSpace;

use crate::geometry::Triangle;
use crate::material::{BoxMaterial, BumpPower, Bumpmap, Phong, Smart};
use crate::sampler::{NormalMap, Sampler, SamplerExt, Texel};
use crate::types::{Color, Float, MaterialLib, Point, RResult, Vector, Vectorx};

fn obj_sampler<F: Float + Texel>(
    resdir: &Path,
    map: &Option<String>,
    col: &Option<[f32; 3]>,
) -> impl Sampler<F, Color<F>> {
    map.as_ref().map_or_else(
        || col.map_or(Color::BLACK, Color::from).dynsampler(),
        |kd| {
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
        },
    )
}

pub fn load<F: Float + Texel>(
    mut obj: Obj,
    lib: &mut MaterialLib<F>,
    pos: Vector<F>,
    scale: F,
) -> RResult<Vec<Triangle<F>>> {
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
                let ni = F::from_f32(omat.ni.unwrap_or(1.0));
                let ns = F::from_f32(omat.ns.unwrap_or(1.0));
                let ke = obj_sampler(&obj.path, &omat.map_ke, &omat.ke);
                let kd = obj_sampler(&obj.path, &omat.map_kd, &omat.kd);
                let ks = obj_sampler(&obj.path, &omat.map_ks, &omat.ks);
                let tf = obj_sampler(&obj.path, &None, &omat.tf);
                /* let kt = obj_sampler(&obj.path, &omat.map_kt, &omat.kt); */
                /* let kr = obj_sampler(&obj.path, &omat.map_refl, &omat.kr); */
                /* let ns = obj_sampler(&obj.path, &omat.map_ns, F::from_f32(omat.ns.unwrap_or(1.0)) */

                let smart = Smart::new(ni, ns, ke, kd, ks, tf, Color::BLACK)
                    .with_ambient(omat.ka.map_or(Color::BLACK, Into::into));

                let res: BoxMaterial<F> = if omat.map_bump.is_some() {
                    let bumpmap = NormalMap::new(obj_sampler(&obj.path, &omat.map_bump, &None));
                    let bump = Bumpmap::new(BumpPower(F::HALF), bumpmap, smart);
                    Box::new(bump)
                } else {
                    Box::new(smart)
                };
                lib.insert(res)
            } else {
                *mat0.get_or_insert_with(|| {
                    let mat = Box::new(Phong::white());
                    lib.insert(mat)
                })
            };

            for poly in &g.polys {
                if !texture.is_empty() && poly.0[0].1.is_none() {
                    continue;
                }

                for n in 1..(poly.0.len() - 1) {
                    let a = (Vector::from_f32s(position[poly.0[0].0]) - corner) * scale + pos;
                    let b = (Vector::from_f32s(position[poly.0[n].0]) - corner) * scale + pos;
                    let c = (Vector::from_f32s(position[poly.0[n + 1].0]) - corner) * scale + pos;

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

                    let (mut ta, mut tb, mut tc) = if texture.is_empty() {
                        (Point::ZERO, Point::ZERO, Point::ZERO)
                    } else {
                        (
                            texture[poly.0[0].1.unwrap()].into(),
                            texture[poly.0[n].1.unwrap()].into(),
                            texture[poly.0[n + 1].1.unwrap()].into(),
                        )
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
    Ok(tris)
}
