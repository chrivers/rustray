use cgmath::{InnerSpace, Matrix4};
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_ranged, FiniteGeometry, Geometry};
use crate::material::Material;
use crate::scene::{Interactive, SceneObject};
use crate::types::{self, Float, HasTransform, Maxel, Ray, Transform, Vector, Vectorx};
use crate::vec3;

#[cfg(feature = "gui")]
use crate::types::Camera;

#[derive(Debug)]
pub struct Cone<F: Float, M: Material<F>> {
    height: F,
    top_r: F,
    bot_r: F,
    capped: bool,
    mat: M,
    xfrm: Transform<F>,
    aabb: Aabb,
}

aabb_impl_fm!(Cone<F, M>);

#[cfg(feature = "gui")]
impl<F: Float, M: Material<F>> Interactive<F> for Cone<F, M> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use egui::{Slider, Widget};
        let mut res = false;

        res |= Slider::new(&mut self.top_r, F::ZERO..=F::from_u32(10))
            .clamp_to_range(false)
            .smallest_positive(0.01)
            .text("Top radius")
            .ui(ui)
            .changed();
        ui.end_row();

        res |= Slider::new(&mut self.bot_r, F::ZERO..=F::from_u32(10))
            .clamp_to_range(false)
            .smallest_positive(0.01)
            .text("Bottom radius")
            .ui(ui)
            .changed();
        ui.end_row();

        res |= Slider::new(&mut self.height, F::ZERO..=F::from_u32(10))
            .clamp_to_range(false)
            .smallest_positive(0.01)
            .text("Height")
            .ui(ui)
            .changed();
        ui.end_row();

        res |= ui.checkbox(&mut self.capped, "Capped").changed();
        ui.end_row();

        res |= self.mat.ui(ui);

        if res {
            self.recompute_aabb();
        }

        res
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }
}

geometry_impl_sceneobject!(Cone<F, M>, "Cone");
geometry_impl_hastransform!(Cone<F, M>);

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Cone<F, M> {
    fn recompute_aabb(&mut self) {
        let m = self.bot_r.max(self.top_r);
        self.aabb = build_aabb_ranged(&self.xfrm, [-m, m], [-m, m], [F::ZERO, self.height]);
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Cone<F, M> {
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cone.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let bot_r = self.bot_r.abs().max(F::BIAS);
        let top_r = self.top_r.abs().max(F::BIAS);

        let mut beta = (top_r - bot_r) / self.height;

        if beta.abs() < F::BIAS {
            beta = F::BIAS;
        }

        let mut gamma;
        gamma = if beta.is_negative() {
            top_r / beta
        } else {
            bot_r / beta
        };

        if gamma.is_negative() {
            gamma -= self.height;
        }

        let mut normal = Vector::UNIT_X;

        let p = r.pos;
        let d = r.dir;

        let beta2 = beta * beta;

        let pzg = p.z + gamma;

        let a = d.x * d.x + d.y * d.y - beta2 * d.z * d.z;
        let b = F::TWO * (p.x * d.x + p.y * d.y - beta2 * pzg * d.z);
        let c = p.x * p.x + p.y * p.y - beta2 * pzg * pzg;

        let mut root = F::max_value();

        let (root1, root2) = types::quadratic2(a, b, c)?;

        /* test side 1 */
        if root1.is_positive() && (root1 < root) {
            let point = r.extend(root1);
            if point.z >= F::ZERO && point.z <= self.height {
                root = root1;
                normal = vec3!(-point.x, -point.y, F::TWO * beta2 * (point.z + gamma));
            }
        }

        /* test side 2 */
        if root2.is_positive() && (root2 < root) {
            let point = r.extend(root2);
            if point.z >= F::ZERO && point.z <= self.height {
                root = root2;
                normal = vec3!(point.x, point.y, -F::TWO * beta2 * (point.z + gamma));
            }
        }

        if self.capped {
            let t1 = (-p.z) / d.z;
            let t2 = (self.height - p.z) / d.z;
            let cap_normal = if d.z.is_positive() {
                -Vector::UNIT_Z
            } else {
                Vector::UNIT_Z
            };

            /* test bottom cap */
            if t1 > F::ZERO && t1 < root {
                let p = r.extend(t1);
                if p.x * p.x + p.y * p.y <= self.bot_r * self.bot_r {
                    root = t1;
                    normal = cap_normal;
                }
            }

            /* test top cap */
            if t2 > F::ZERO && t2 < root {
                let p = r.extend(t2);
                if p.x * p.x + p.y * p.y <= self.top_r * self.top_r {
                    root = t2;
                    normal = cap_normal;
                }
            }
        }

        if root == F::max_value() {
            return None;
        }

        let nml = self.xfrm.nml(normal.normalize());

        Some(ray.hit_at(root, self, &self.mat).with_normal(nml))
    }
}

impl<F: Float, M: Material<F>> Cone<F, M> {
    pub fn new(height: F, top_r: F, bot_r: F, capped: bool, xfrm: Matrix4<F>, mat: M) -> Self {
        let mut res = Self {
            height,
            top_r,
            bot_r,
            capped,
            mat,
            xfrm: Transform::new(xfrm),
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}
