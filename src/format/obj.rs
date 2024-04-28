use std::collections::HashMap;
use std::path::Path;

use obj::{Obj, ObjMaterial};

use cgmath::{InnerSpace, Matrix4, SquareMatrix};

use crate::geometry::{FiniteGeometry, Group, Triangle, TriangleMesh};
use crate::material::{BoxMaterial, BumpPower, Bumpmap, Fresnel, Phong, Smart};
use crate::sampler::{NormalMap, Sampler, SamplerExt, Texel};
use crate::scene::BoxScene;
use crate::types::{Color, Float, MaterialId, NamedObject, Point, RResult, Vector, Vectorx};

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

pub fn load_material<F: Float>(resdir: &Path, omat: &obj::Material) -> BoxMaterial<F> {
    let phong = Phong::new()
        .with_ke(omat.ke.map_or(Color::BLACK, Color::from))
        .with_kd(omat.kd.map_or(Color::WHITE, Color::from))
        .with_ks(omat.ks.map_or(Color::BLACK, Color::from))
        .with_pow(F::from_f32(omat.ns.unwrap_or(8.0)))
        .with_ke_map(obj_sampler3(resdir, &omat.map_ke))
        .with_kd_map(obj_sampler3(resdir, &omat.map_kd))
        .with_ks_map(obj_sampler3(resdir, &omat.map_ks))
        .with_pow_map(obj_sampler1(resdir, &omat.map_ns))
        .with_ambient(omat.ka.map_or(Color::BLACK, Into::into));

    let fresnel = Fresnel::new(
        omat.ni.map_or(F::ONE, F::from_f32),
        omat.tf.map_or(Color::BLACK, Color::from),
        Color::BLACK,
    );

    let smart = NamedObject::new(omat.name.clone(), Smart::make(phong, fresnel));

    if omat.map_bump.is_some() {
        let bumpmap = NormalMap::new(obj_sampler3(resdir, &omat.map_bump).unwrap());
        let bump = Bumpmap::new(BumpPower(F::HALF), bumpmap, smart);
        Box::new(bump)
    } else {
        Box::new(smart)
    }
}

pub fn load<F: Float + Texel>(mut obj: Obj, scene: &mut BoxScene<F>) -> RResult<()> {
    let mut corner = Vector::new(F::max_value(), F::max_value(), F::max_value());

    obj.load_mtls()?;
    let position = &obj.data.position;
    let objects = &obj.data.objects;
    let texture = &obj.data.texture;
    let normal: &[[f32; 3]] = &obj.data.normal;

    let mut total = Vector::ZERO;
    let mut avgc = 0;

    for o in objects {
        for g in &o.groups {
            for poly in &g.polys {
                for n in 0..(poly.0.len() - 1) {
                    let point = Vector::from_f32s(position[poly.0[n].0]);
                    total += point;
                    corner = corner.min(&point);
                    avgc += 1;
                }
            }
        }
    }

    let offset = total / F::from_u32(avgc);

    let mut hashmat: HashMap<&str, MaterialId> = HashMap::new();

    let mut faces = 0;
    let mut meshes = 0;
    let mut groups = 0;

    for o in &obj.data.objects {
        info!("Object: {}", o.name);
        let mut geos: Vec<Box<dyn FiniteGeometry<F>>> = vec![];
        for g in &o.groups {
            let mut tris = vec![];
            info!("  group: {}", g.name);
            let mat = if let Some(ObjMaterial::Mtl(ref omat)) = g.material {
                *hashmat.entry(&omat.name).or_insert_with(|| {
                    let mat = load_material(&obj.path, omat);
                    scene.materials.insert(mat)
                })
            } else {
                scene.materials.default()
            };

            for poly in &g.polys {
                for n in 1..(poly.0.len() - 1) {
                    let a = Vector::from_f32s(position[poly.0[0].0]) - offset;
                    let b = Vector::from_f32s(position[poly.0[n].0]) - offset;
                    let c = Vector::from_f32s(position[poly.0[n + 1].0]) - offset;

                    let (na, nb, nc) = {
                        let na = poly.0[0].2;
                        let nb = poly.0[n].2;
                        let nc = poly.0[n + 1].2;
                        if let (Some(a), Some(b), Some(c)) = (na, nb, nc) {
                            (
                                Vector::from_f32s(normal[a]).normalize(),
                                Vector::from_f32s(normal[b]).normalize(),
                                Vector::from_f32s(normal[c]).normalize(),
                            )
                        } else {
                            let n = (a - b).cross(a - c);
                            (n, n, n)
                        }
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
                    faces += 1;
                }
            }
            if !tris.is_empty() {
                let mesh = TriangleMesh::new(tris, Matrix4::identity());
                geos.push(Box::new(NamedObject::new(g.name.clone(), mesh)));
                meshes += 1;
            }
        }
        if !geos.is_empty() {
            let grp = Group::new(geos, Matrix4::identity());
            scene.add_object(NamedObject::new(o.name.clone(), grp));
            groups += 1;
        }
    }

    info!(
        "loaded obj file: {meshes} meshe(s) in {groups} group(s) [index: {}, normal: {}, uv: {}, face: {}]",
        position.len(),
        obj.data.normal.len(),
        obj.data.texture.len(),
        faces,
    );

    Ok(())
}
