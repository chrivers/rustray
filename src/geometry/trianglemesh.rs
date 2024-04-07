use super::geo_util::*;
use super::triangle::Triangle;

use cgmath::SquareMatrix;
use obj::Obj;

use rtbvh::{Bounds, Builder, Bvh};
use std::num::NonZeroUsize;

use crate::material::DynMaterial;
use crate::sampler::Texel;
use crate::types::bvh::BvhExt;
use crate::types::result::RResult;
use crate::types::Color;

#[derive(Debug)]
pub struct TriangleMesh<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: Box<dyn Material<F>>,
    pub tris: Vec<Triangle<F, M>>,
    bvh: Bvh,
    aabb: Aabb,
}

impl<F: Float, M: Material<F>> Interactive<F> for TriangleMesh<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, _ui: &mut egui::Ui) -> bool {
        false
    }

    #[cfg(feature = "gui")]
    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        gizmo_ui(ui, camera, self, rect)
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

impl<F: Float, M: Material<F>> HasTransform<F> for TriangleMesh<F, M> {
    fn get_transform(&self) -> &Transform<F> {
        &self.xfrm
    }

    fn set_transform(&mut self, xfrm: &Transform<F>) {
        self.xfrm = *xfrm;
        self.recompute_aabb();
    }
}

impl<F: Float, M: Material<F>> Primitive for TriangleMesh<F, M> {
    fn center(&self) -> Vec3 {
        self.aabb.center()
    }

    fn aabb(&self) -> Aabb {
        self.aabb
    }
}

impl<F: Float, M: Material<F>> FiniteGeometry<F> for TriangleMesh<F, M> {
    fn recompute_aabb(&mut self) {
        let bounds = self.bvh.bounds();

        let min = Vector::from_vec3(bounds.min);
        let max = Vector::from_vec3(bounds.max);

        self.aabb = build_aabb_ranged(&self.xfrm, [min.x, max.x], [min.y, max.y], [min.z, max.z]);
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for TriangleMesh<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        if ray.grp > 0 {
            let r = ray.xfrm_inv(&self.xfrm).enter_group()?;

            let maxel = self
                .bvh
                .nearest_intersection(&r, &self.tris, &mut F::max_value());

            /* FIXME: We have to transform maxel results backwards through our
             * xfrm, to avoid results in object space. This breaks the design
             * idea of maxel, which can calculate information on-demand (but
             * this would require access to the resulting Transform, which is
             * not currently available). */
            maxel.map(|mut mxl| {
                mxl.st();
                mxl.uv();
                mxl.pos = self.xfrm.pos(mxl.pos);
                mxl.dir = self.xfrm.dir(mxl.dir);
                mxl.with_normal(self.xfrm.nml(mxl.nml()).normalize())
            })
        } else {
            let center = self.xfrm.pos_inv(Vector::from_vec3(self.center()));
            Some(Maxel::new(
                center,
                -ray.dir,
                ray.lvl,
                self,
                self.mat.as_ref(),
                ray.dbg,
            ))
        }
    }
}

impl<F: Float, M: Material<F>> TriangleMesh<F, M> {
    pub fn new(tris: Vec<Triangle<F, M>>, xfrm: Matrix4<F>) -> Self {
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

        let mut res = Self {
            xfrm: Transform::new(xfrm),
            mat: Box::new(Color::BLACK),
            tris,
            bvh,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}

impl<F: Float + Texel> TriangleMesh<F, DynMaterial<F>> {
    pub fn load_obj(obj: Obj, pos: Vector<F>, scale: F) -> RResult<Self> {
        let tris = crate::format::obj::load(obj, pos, scale)?;
        Ok(TriangleMesh::new(tris, Matrix4::identity()))
    }
}
