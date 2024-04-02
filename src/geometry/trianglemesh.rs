use super::geo_util::*;
use super::triangle::Triangle;

use obj::Obj;

use rtbvh::{Bounds, Builder, Bvh};
use std::num::NonZeroUsize;

use crate::material::DynMaterial;
use crate::sampler::Texel;
use crate::types::bvh::BvhExt;
use crate::types::result::RResult;

#[derive(Debug)]
pub struct TriangleMesh<F: Float, M: Material<F>> {
    pub tris: Vec<Triangle<F, M>>,
    bvh: Bvh,
}

impl<F: Float, M: Material<F>> Interactive<F> for TriangleMesh<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        for tri in &mut self.tris {
            tri.ui(ui);
        }
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for TriangleMesh<F, M> {
    fn get_name(&self) -> &str {
        "Triangle mesh"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> Primitive for TriangleMesh<F, M> {
    fn center(&self) -> Vec3 {
        self.bvh.bounds().center()
    }

    fn aabb(&self) -> Aabb {
        self.bvh.bounds()
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for TriangleMesh<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        self.bvh
            .nearest_intersection(ray, &self.tris, &mut F::max_value())
    }
}

impl<F: Float, M: Material<F>> TriangleMesh<F, M> {
    pub fn new(tris: Vec<Triangle<F, M>>) -> Self {
        debug!("building bvh for {} triangles..", tris.len());

        let aabbs: Vec<Aabb> = tris.iter().map(rtbvh::Primitive::aabb).collect();

        let bvh = Builder {
            aabbs: Some(&aabbs),
            primitives: &tris,
            primitives_per_leaf: NonZeroUsize::new(16),
        }
        /* .construct_spatial_sah().unwrap(); */
        .construct_binned_sah()
        .unwrap();
        /* .construct_locally_ordered_clustered().unwrap(); */

        Self { tris, bvh }
    }
}

impl<F: Float + Texel> TriangleMesh<F, DynMaterial<F>> {
    pub fn load_obj(obj: Obj, pos: Vector<F>, scale: F) -> RResult<Self> {
        let tris = crate::format::obj::load(obj, pos, scale)?;
        Ok(TriangleMesh::new(tris))
    }
}
