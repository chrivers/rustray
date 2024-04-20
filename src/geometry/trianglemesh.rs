use std::num::NonZeroUsize;

use cgmath::{Matrix4, SquareMatrix};
use glam::Vec3;
use obj::Obj;
use rtbvh::{Aabb, Bounds, Builder, Bvh, Primitive};

#[cfg(feature = "gui")]
use crate::types::Camera;

use crate::geometry::{build_aabb_ranged, FiniteGeometry, Geometry, Triangle};
use crate::material::{BoxMaterial, Material};
use crate::sampler::Texel;
use crate::scene::{Interactive, SceneObject};
use crate::types::{
    BvhExt, Color, Float, HasTransform, MaterialId, MaterialLib, Maxel, RResult, Ray, Transform,
    Vector, Vectorx, RF,
};

#[derive(Debug)]
pub struct TriangleMesh<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: BoxMaterial<F>,
    pub tris: Vec<Triangle<F, M>>,
    bvh: Bvh,
    aabb: Aabb,
}

#[cfg(feature = "gui")]
impl<F: Float, M: Material<F>> Interactive<F> for TriangleMesh<F, M> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;
        if ui.button("Face normals").clicked() {
            crate::mesh::face_normals(&mut self.tris);
            res |= true;
        }
        if ui.button("Smooth normals").clicked() {
            crate::mesh::smooth_normals(&mut self.tris);
            res |= true;
        }
        res
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }

    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}

geometry_impl_sceneobject!(TriangleMesh<F, M>, "TriangleMesh");
geometry_impl_hastransform!(TriangleMesh<F, M>);
aabb_impl_fm!(TriangleMesh<F, M>);

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
        if !ray.flags.contains(RF::StopAtGroup) {
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
                mxl.with_normal(self.xfrm.nml(mxl.nml()))
            })
        } else {
            let center = self.xfrm.pos_inv(Vector::from_vec3(self.center()));
            Some(Maxel::new(
                center,
                -ray.dir,
                ray.lvl,
                self,
                self.mat.as_ref(),
                ray.flags,
            ))
        }
    }
}

impl<F: Float, M: Material<F>> TriangleMesh<F, M> {
    const ICON: &'static str = egui_phosphor::regular::POLYGON;

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

impl<F: Float + Texel> TriangleMesh<F, MaterialId> {
    pub fn load_obj(obj: Obj, lib: &mut MaterialLib<F>, pos: Vector<F>, scale: F) -> RResult<Self> {
        let tris = crate::format::obj::load(obj, lib, pos, scale)?;
        Ok(Self::new(tris, Matrix4::identity()))
    }
}
