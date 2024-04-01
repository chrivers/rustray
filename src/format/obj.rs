use std::path::Path;

use obj::{Obj, ObjMaterial};

use cgmath::InnerSpace;

use crate::geometry::Triangle;
use crate::material::{Bumpmap, DynMaterial, Material, Phong, Smart};
use crate::sampler::{DynSampler, NormalMap, Sampler, SamplerExt, Texel};
use crate::types::result::RResult;
use crate::types::vector::Vectorx;
use crate::types::{Color, Float, Point, Vector};

fn obj_sampler<F: Float + Texel>(
    resdir: &Path,
    map: &Option<String>,
    col: &Option<[f32; 3]>,
) -> DynSampler<F, Color<F>> {
    match map {
        Some(ref kd) => {
            info!("loading [{}]", kd);
            image::open(resdir.join(kd))
                .map(|m| m.bilinear().dynsampler())
                .unwrap_or_else(|_| {
                    warn!("Missing texture [{}]", kd);
                    Color::WHITE.dynsampler()
                })
        }
        None => col
            .map(Color::from)
            .unwrap_or(Color::BLACK)
            .dynsampler(),
    }
}

pub fn load<F: Float + Texel>(
    mut obj: Obj,
    pos: Vector<F>,
    scale: F,
) -> RResult<Vec<Triangle<F, DynMaterial<F>>>> {
    let mut corner = Vector::new(F::max_value(), F::max_value(), F::max_value());

    for o in &obj.data.objects {
        for g in &o.groups {
            for poly in &g.polys {
                let pos = &obj.data.position;
                for n in 0..(poly.0.len() - 1) {
                    corner.x = corner.x.min(F::from_f32(pos[poly.0[n].0][0]));
                    corner.y = corner.y.min(F::from_f32(pos[poly.0[n].0][1]));
                    corner.z = corner.z.min(F::from_f32(pos[poly.0[n].0][2]));
                }
            }
        }
    }

    let mut tris = vec![];
    obj.load_mtls()?;
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

                let smart = Smart::new(ni, ns, ke, kd, ks, tf, Color::white());

                if omat.map_bump.is_some() {
                    let bumpmap = NormalMap::new(obj_sampler(&obj.path, &omat.map_bump, &None));
                    let bump = Bumpmap::new(F::ONE, bumpmap, smart);
                    bump.dynamic()
                } else {
                    smart.dynamic()
                }
            } else {
                Phong::white().dynamic()
            };

            for poly in &g.polys {
                if !obj.data.texture.is_empty() && poly.0[0].1.is_none() {
                    continue;
                }
                let data = &obj.data;
                for n in 1..(poly.0.len() - 1) {
                    let dpos = &data.position;
                    let a = (Vector::from_f32s(dpos[poly.0[0].0]) - corner) * scale + pos;
                    let b = (Vector::from_f32s(dpos[poly.0[n].0]) - corner) * scale + pos;
                    let c = (Vector::from_f32s(dpos[poly.0[n + 1].0]) - corner) * scale + pos;

                    /* FIXME: .unwrap() is a terrible when loading data from a file */
                    let (na, nb, nc) = if obj.data.normal.is_empty() {
                        let n = (a - b).cross(a - c);
                        (n, n, n)
                    } else {
                        (
                            Vector::from_f32s(data.normal[poly.0[0].2.unwrap()]).normalize(),
                            Vector::from_f32s(data.normal[poly.0[n].2.unwrap()]).normalize(),
                            Vector::from_f32s(data.normal[poly.0[n + 1].2.unwrap()]).normalize(),
                        )
                    };

                    let (mut ta, mut tb, mut tc) = if obj.data.texture.is_empty() {
                        (Point::zero(), Point::zero(), Point::zero())
                    } else {
                        (
                            Point::from(data.texture[poly.0[0].1.unwrap()]),
                            Point::from(data.texture[poly.0[n].1.unwrap()]),
                            Point::from(data.texture[poly.0[n + 1].1.unwrap()]),
                        )
                    };
                    ta.y = F::ONE - ta.y;
                    tb.y = F::ONE - tb.y;
                    tc.y = F::ONE - tc.y;

                    let tri = Triangle::new(a, b, c, na, nb, nc, ta, tb, tc, mat.clone());
                    tris.push(tri);
                }
            }
        }
    }

    info!(
        "loaded .obj [index: {}, normal: {}, uv: {}, face: {}]",
        obj.data.position.len(),
        obj.data.normal.len(),
        obj.data.texture.len(),
        tris.len()
    );
    Ok(tris)
}
