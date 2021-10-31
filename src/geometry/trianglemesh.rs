use super::geo_util::*;
use super::triangle::Triangle;

use std::path::Path;

use obj::Obj;
use obj::ObjMaterial;

use rtbvh::{Bvh, Builder, Bounds};
use std::num::NonZeroUsize;

use crate::lib::Color;
use crate::lib::bvh::BvhExt;
use crate::material::{Smart, DynMaterial};
use crate::sampler::{Sampler, SamplerExt, Texel, DynSampler};

#[derive(Debug)]
pub struct TriangleMesh<F: Float, M: Material<F=F>>
{
    pub tris: Vec<Triangle<F, M>>,
    bvh: Bvh,
}

impl<F: Float, M: Material<F=F>> Primitive for TriangleMesh<F, M>
{
    fn center(&self) -> Vec3 {
        self.bvh.bounds().center()
    }

    fn aabb(&self) -> Aabb {
        self.bvh.bounds()
    }
}

impl<F: Float, M: Material<F=F> + Clone> Geometry<F> for TriangleMesh<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        self.bvh.nearest_intersection(ray, &self.tris, &mut F::max_value())
    }
}

fn a2point<F: Float>(p: &[f32; 2]) -> Point<F> {
    point!(F::from_f32(p[0]),
           F::from_f32(p[1]))
}

fn a2vec<F: Float>(p: &[f32; 3]) -> Vector<F> {
    vec3!(F::from_f32(p[0]),
          F::from_f32(p[1]),
          F::from_f32(p[2]))
}

impl<F: Float, M: Material<F=F> + Clone> TriangleMesh<F, M>
{
    pub fn new(tris: Vec<Triangle<F, M>>) -> Self
    {
        debug!("building bvh for {} triangles..", tris.len());

        let aabbs = tris
            .iter()
            .map(|t| t.aabb())
            .collect::<Vec<Aabb>>();

        let bvh = Builder {
            aabbs: Some(&aabbs),
            primitives: &tris,
            primitives_per_leaf: NonZeroUsize::new(16),
        }
        /* .construct_spatial_sah().unwrap(); */
        .construct_binned_sah().unwrap();
        /* .construct_locally_ordered_clustered().unwrap(); */

        TriangleMesh {
            tris,
            bvh,
        }
    }
}

fn sampler<'a, F: Float + 'a>(val: Option<[f32; 3]>) -> DynSampler<'a, F, Color<F>>
{
    match val {
        None => Color::white().dynsampler(),
        Some([r, g, b]) => Color::new(
            F::from_f32(r),
            F::from_f32(g),
            F::from_f32(b),
        ).dynsampler(),
    }
}

fn obj_sampler<'a, F: Float + 'a>(resdir: &Path, map: &Option<String>, col: &Option<[f32; 3]>) -> DynSampler<'a, F, Color<F>>
{
    match map {
        Some(ref kd) => {
            info!("loading [{}]", kd);
            image::open(resdir.join(kd)).map(|m| m.bilinear().dynsampler()).unwrap_or_else(
                |_| {
                    warn!("Missing texture [{}]", kd);
                    Color::white().dynsampler()
                }
            )
            /* image::open(obj.path.join(kd)).map(|m| Bilinear::new(m).dynsampler()).unwrap_or_else(|_| Color::white().dynsampler()) */
        }
        None => sampler(*col),
    }
}

impl<'a, F: Float + Texel + 'static> TriangleMesh<F, DynMaterial<'a, F>>
{
    pub fn load_obj(mut obj: Obj, pos: Vector<F>, scale: F) -> Self
    {
        let mut corner = Vector::new(
            F::max_value(),
            F::max_value(),
            F::max_value()
        );

        for o in &obj.data.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 0..(poly.0.len() - 1) {
                        corner.x = corner.x.min(F::from_f32(obj.data.position[poly.0[n].0][0]));
                        corner.y = corner.y.min(F::from_f32(obj.data.position[poly.0[n].0][1]));
                        corner.z = corner.z.min(F::from_f32(obj.data.position[poly.0[n].0][2]));
                    }
                }
            }
        }

        let mut tris = vec![];
        obj.load_mtls().unwrap();
        /* info!("mats: {:#?}", obj.data.material_libs); */
        for o in &obj.data.objects {
            for (_i, g) in o.groups.iter().enumerate() {
                let mat = if let Some(ObjMaterial::Mtl(ref omat)) = g.material {
                    let ke = obj_sampler(&obj.path, &omat.map_ke, &omat.ke);
                    let kd = obj_sampler(&obj.path, &omat.map_kd, &omat.kd);
                    let ks = obj_sampler(&obj.path, &omat.map_ks, &omat.ks);
                    /* let kt = obj_sampler(&obj.path, &omat.map_kt, &omat.kt); */
                    /* let kr = obj_sampler(&obj.path, &omat.map_refl, &omat.kr); */
                    /* let ns = obj_sampler(&obj.path, &omat.map_ns, F::from_f32(omat.ns.unwrap_or(1.0)) */

                    Smart::new(
                        F::from_f32(omat.ni.unwrap_or(1.0)),
                        F::from_f32(omat.ns.unwrap_or(1.0)),
                        ke,
                        kd,
                        ks,
                        Color::black().dynsampler(),
                        Color::white().dynsampler()
                    ).dynamic()
                } else {
                    Color::white().dynamic()
                };

                for poly in &g.polys {
                    if !obj.data.texture.is_empty() && poly.0[0].1.is_none() {
                        continue
                    }
                    for n in 1..(poly.0.len() - 1) {
                        let a = (a2vec(&obj.data.position[poly.0[0  ].0]) - corner) * scale + pos;
                        let b = (a2vec(&obj.data.position[poly.0[n  ].0]) - corner) * scale + pos;
                        let c = (a2vec(&obj.data.position[poly.0[n+1].0]) - corner) * scale + pos;

                        /* FIXME: .unwrap() is a terrible when loading data from a file */
                        let (na, nb, nc) = if obj.data.normal.is_empty() {
                            let n = (a-b).cross(a-c);
                            (
                                n, n, n
                            )
                        } else {
                            (
                                a2vec(&obj.data.normal[poly.0[0].2.unwrap()]).normalize(),
                                a2vec(&obj.data.normal[poly.0[n].2.unwrap()]).normalize(),
                                a2vec(&obj.data.normal[poly.0[n+1].2.unwrap()]).normalize(),
                            )
                        };

                        let (ta, tb, tc) = if obj.data.texture.is_empty() {
                            (
                                Point::zero(),
                                Point::zero(),
                                Point::zero(),
                            )
                        } else {
                            (
                                a2point(&obj.data.texture[poly.0[0].1.unwrap()]),
                                a2point(&obj.data.texture[poly.0[n].1.unwrap()]),
                                a2point(&obj.data.texture[poly.0[n+1].1.unwrap()]),
                            )
                        };

                        let tri = Triangle::new (
                            a, b, c,
                            na, nb, nc,
                            ta, tb, tc,
                            mat.clone(),
                        );
                        tris.push(tri);
                    }
                }
            }
        }

        info!("loaded .obj [index: {}, normal: {}, uv: {}, face: {}]", obj.data.position.len(), obj.data.normal.len(), obj.data.texture.len(), tris.len());

        TriangleMesh::new(tris)
    }

}
