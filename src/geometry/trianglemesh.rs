use super::geo_util::*;
use super::triangle::Triangle;

use obj::Obj;

use rtbvh::{Bvh, Builder, Bounds};
use std::num::NonZeroUsize;

use crate::types::result::RResult;
use crate::types::bvh::BvhExt;
use crate::material::DynMaterial;
use crate::sampler::Texel;

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

impl<'a, F: Float + Texel + 'static> TriangleMesh<F, DynMaterial<'a, F>>
{
    pub fn load_obj(obj: Obj, pos: Vector<F>, scale: F) -> RResult<Self>
    {
        let tris = crate::format::obj::load(obj, pos, scale)?;
        Ok(TriangleMesh::new(tris))
    }

}
