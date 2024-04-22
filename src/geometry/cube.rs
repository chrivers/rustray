#[cfg(feature = "gui")]
use crate::types::Camera;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_symmetric, FiniteGeometry, Geometry};
use crate::material::HasMaterial;
use crate::point;
use crate::scene::{Interactive, SceneObject};
use crate::types::{
    Float, HasTransform, MaterialId, Maxel, Point, Ray, Transform, Vector, Vectorx,
};

#[derive(Debug)]
pub struct Cube<F: Float> {
    xfrm: Transform<F>,
    mat: MaterialId,
    aabb: Aabb,
}

aabb_impl_fm!(Cube<F>);

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Cube<F> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        Interactive::<F>::ui(&mut self.mat, ui)
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }

    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}

geometry_impl_sceneobject!(Cube<F>, "Cube");
geometry_impl_hastransform!(Cube<F>);
geometry_impl_hasmaterial!(Cube<F>);

impl<F: Float> FiniteGeometry<F> for Cube<F> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::HALF, F::HALF, F::HALF);
    }
}

impl<F: Float> Cube<F> {
    const NORMALS: [Vector<F>; 3] = [Vector::UNIT_X, Vector::UNIT_Y, Vector::UNIT_Z];
}

impl<F: Float> Geometry<F> for Cube<F> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let p = r.pos;
        let d = r.dir;

        let mut best_t = F::max_value();
        let mut best = None;

        for it in 0..6 {
            let mod0 = it % 3;

            if d[mod0].is_zero() {
                continue;
            }

            let t = (F::from_usize(it / 3) - F::HALF - p[mod0]) / d[mod0];

            if t < F::BIAS2 || t > best_t {
                continue;
            }

            let mod1 = (it + 1) % 3;
            let mod2 = (it + 2) % 3;
            let x = p[mod1] + t * d[mod1];
            let y = p[mod2] + t * d[mod2];

            let half = -F::HALF..F::HALF;
            if half.contains(&x) && half.contains(&y) && best_t > t {
                best_t = t;
                best = Some(it);
            }
        }

        let best = best?;

        let normal = if best < 3 {
            -Self::NORMALS[best % 3]
        } else {
            Self::NORMALS[best % 3]
        };

        let i1 = (best + 1) % 3;
        let i2 = (best + 2) % 3;
        let min = i1.min(i2);
        let max = i1.max(i2);

        let isec = r.extend(best_t);
        let uv = point!(F::HALF - isec[min], F::HALF - isec[max]);

        Some(
            ray.hit_at(best_t, self, self.mat)
                .with_normal(self.xfrm.nml(normal))
                .with_uv(uv),
        )
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }
}

impl<F: Float> Cube<F> {
    pub const ICON: &'static str = egui_phosphor::regular::CUBE;

    pub fn new(xfrm: Matrix4<F>, mat: MaterialId) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}
