use super::geo_util::*;
use super::triangle::Triangle;

use std::path::Path;

use obj::Obj;
use obj::ObjMaterial;
use obj::ObjData;

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
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
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
            .collect::<Vec<rtbvh::Aabb>>();

        let bvh = Builder {
            aabbs: Some(aabbs.as_slice()),
            primitives: tris.as_slice(),
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

    pub fn load_obj(obj: ObjData, pos: Vector<F>, scale: F, mat: M) -> Self
    {
        let mut corner = Vector::new(
            F::max_value(),
            F::max_value(),
            F::max_value()
        );

        for o in &obj.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 0..(poly.0.len() - 1) {
                        corner.x = corner.x.min(F::from_f32(obj.position[poly.0[n].0][0]));
                        corner.y = corner.y.min(F::from_f32(obj.position[poly.0[n].0][1]));
                        corner.z = corner.z.min(F::from_f32(obj.position[poly.0[n].0][2]));
                    }
                }
            }
        }

        let mut tris: Vec<Triangle<F, M>> = vec![];
        for o in &obj.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 1..(poly.0.len() - 1) {
                        /* FIXME: .unwrap() is a terrible when loading data from a file */
                        let tri = Triangle::new (
                            (a2vec(&obj.position[poly.0[0  ].0]) - corner) * scale + pos,
                            (a2vec(&obj.position[poly.0[n  ].0]) - corner) * scale + pos,
                            (a2vec(&obj.position[poly.0[n+1].0]) - corner) * scale + pos,

                            a2vec(&obj.normal[poly.0[0].2.unwrap()]).normalize(),
                            a2vec(&obj.normal[poly.0[n].2.unwrap()]).normalize(),
                            a2vec(&obj.normal[poly.0[n+1].2.unwrap()]).normalize(),

                            a2point(&obj.texture[poly.0[0].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n+1].1.unwrap()]),

                            mat.clone(),
                        );
                        tris.push(tri);
                    }
                }
            }
        }

        info!("loaded .obj [index: {}, normal: {}, uv: {}, face: {}]", obj.position.len(), obj.normal.len(), obj.texture.len(), tris.len());

        TriangleMesh::new(tris)
    }

}
