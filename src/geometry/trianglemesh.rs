use crate::{vec3, point};
use crate::math::{Vector, Float, Point};
use crate::scene::*;
use crate::math::ray::{Ray, Hit};
use crate::material::Material;

use super::triangle::Triangle;

use std::io::Read;

use obj::{ObjData, ObjError};

use bvh::bvh::BVH;
use bvh::{Point3, Vector3};

pub struct TriangleMesh<'a, F: Float>
{
    tris: Vec<Triangle<'a, F>>,
    bvh: BVH,
}

impl<'a, F: Float> RayTarget<F> for TriangleMesh<'a, F>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = bvh::ray::Ray::new(
            Point3::new(
                ray.pos.x.to_f32()?,
                ray.pos.y.to_f32()?,
                ray.pos.z.to_f32()?,
            ),
            Vector3::new(
                ray.dir.x.to_f32()?,
                ray.dir.y.to_f32()?,
                ray.dir.z.to_f32()?,
            )
        );
        let aabbs = self.bvh.traverse(&r, &self.tris);

        let mut dist = F::max_value();
        let mut hit: Option<Hit<F>> = None;
        for t in &aabbs {
            if let Some(curhit) = t.intersect(ray)
            {
                let curdist = ray.pos.length_to(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }
        hit
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

impl<'a, F: Float> TriangleMesh<'a, F>
{
    pub fn new(mut tris: Vec<Triangle<'a, F>>) -> TriangleMesh<'a, F>
    {
        let bvh = BVH::build(&mut tris);

        TriangleMesh {
            tris,
            bvh,
        }
    }

    pub fn load_obj<R: Read>(read: &mut R, pos: Vector<F>, scale: F, mat: &'a dyn Material<F=F>) -> Result<TriangleMesh<'a, F>, ObjError>
    {
        let obj = ObjData::load_buf(read)?;

        let mut tris: Vec<Triangle<F>> = vec![];
        for o in &obj.objects {
            for g in &o.groups {
                for poly in &g.polys {
                    for n in 1..(poly.0.len() - 1) {
                        /* FIXME: .unwrap() is a terrible when loading data from a file */
                        let tri = Triangle::new (
                            a2vec(&obj.position[poly.0[0].0]) * scale + pos,
                            a2vec(&obj.position[poly.0[n].0]) * scale + pos,
                            a2vec(&obj.position[poly.0[n+1].0]) * scale + pos,

                            a2vec(&obj.normal[poly.0[0].2.unwrap()]).normalized(),
                            a2vec(&obj.normal[poly.0[n].2.unwrap()]).normalized(),
                            a2vec(&obj.normal[poly.0[n+1].2.unwrap()]).normalized(),

                            a2point(&obj.texture[poly.0[0].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n].1.unwrap()]),
                            a2point(&obj.texture[poly.0[n+1].1.unwrap()]),

                            mat,
                        );
                        tris.push(tri);
                    }
                }
            }
        }

        info!("loaded .obj [index: {}, normal: {}, uv: {}, face: {}]", obj.position.len(), obj.normal.len(), obj.texture.len(), tris.len());

        Ok(TriangleMesh::new(tris))
    }

}
