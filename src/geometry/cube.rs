#[cfg(feature = "gui")]
use crate::types::Camera;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_symmetric, FiniteGeometry, Geometry};
use crate::material::Material;
use crate::point;
use crate::scene::{Interactive, SceneObject};
use crate::types::transform::{HasTransform, Transform};
use crate::types::{Float, Maxel, Point, Ray, Vector, Vectorx};

#[derive(Debug)]
pub struct Cube<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Cube<F, M>);

#[cfg(feature = "gui")]
impl<F: Float, M: Material<F>> Interactive<F> for Cube<F, M> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.mat.ui(ui)
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::frontend::gui::gizmo_ui(ui, camera, self, rect)
    }
}

geometry_impl_sceneobject!(Cube<F, M>, "Cube");
geometry_impl_hastransform!(Cube<F, M>);

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Cube<F, M> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::HALF, F::HALF, F::HALF);
    }
}

impl<F: Float, M: Material<F>> Cube<F, M> {
    const NORMALS: [Vector<F>; 3] = [Vector::UNIT_X, Vector::UNIT_Y, Vector::UNIT_Z];
}

impl<F: Float, M: Material<F>> Geometry<F> for Cube<F, M> {
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
            ray.hit_at(best_t, self, &self.mat)
                .with_normal(self.xfrm.nml(normal))
                .with_uv(uv),
        )
    }
}

impl<F: Float, M: Material<F>> Cube<F, M> {
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}
