use std::num::NonZeroUsize;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::{Aabb, Bounds, Builder, Bvh, Primitive};

#[cfg(feature = "gui")]
use crate::types::Camera;

use crate::geometry::{build_aabb_ranged, FiniteGeometry, Geometry, Triangle};
use crate::material::HasMaterial;
use crate::scene::{Interactive, SceneObject};
use crate::types::{BvhExt, Float, HasTransform, Maxel, Ray, Transform, Vector, Vectorx, RF};

#[derive(Debug)]
pub struct TriangleMesh<F: Float> {
    xfrm: Transform<F>,
    pub tris: Vec<Triangle<F>>,
    bvh: Bvh,
    aabb: Aabb,
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for TriangleMesh<F> {
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

geometry_impl_sceneobject!(TriangleMesh<F>, "TriangleMesh");
geometry_impl_hastransform!(TriangleMesh<F>);
aabb_impl_fm!(TriangleMesh<F>);

impl<F: Float> FiniteGeometry<F> for TriangleMesh<F> {
    fn recompute_aabb(&mut self) {
        let bounds = self.bvh.bounds();

        let min = Vector::from_vec3(bounds.min);
        let max = Vector::from_vec3(bounds.max);

        self.aabb = build_aabb_ranged(&self.xfrm, [min.x, max.x], [min.y, max.y], [min.z, max.z]);
    }
}

impl<F: Float> Geometry<F> for TriangleMesh<F> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        if ray.flags.contains(RF::StopAtGroup) {
            let center = self.xfrm.pos_inv(Vector::from_vec3(self.center()));
            return Some(ray.synthetic_hit(center, self));
        }

        let r = ray.xfrm_inv(&self.xfrm);

        self.bvh
            .nearest_intersection(&r, &self.tris, &mut F::max_value())
            .map(|mut mxl| {
                /* FIXME: We have to make maxel cache all results here, to avoid results
                 * in object space. This breaks the design idea of maxel, which can
                 * calculate information on-demand (but this would require access to the
                 * resulting Transform, which is not currently available). */
                mxl.st();
                mxl.uv();
                mxl.nml();

                /* Transform maxel from object space */
                mxl.xfrm(&self.xfrm)
            })
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        None
    }
}

impl<F: Float> TriangleMesh<F> {
    const ICON: &'static str = egui_phosphor::regular::POLYGON;

    pub fn new(tris: Vec<Triangle<F>>, xfrm: Matrix4<F>) -> Self {
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
            tris,
            bvh,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}
