use super::geo_util::*;

#[derive(Debug)]
pub struct Cylinder<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    capped: bool,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Cylinder<F, M>);

impl<F: Float, M: Material<F>> Interactive<F> for Cylinder<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        egui::Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.capped, "Capped");
                ui.end_row();

                self.mat.ui(ui)
            })
            .inner
    }

    #[cfg(feature = "gui")]
    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        gizmo_ui(ui, camera, self, rect)
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for Cylinder<F, M> {
    fn get_name(&self) -> &str {
        "Cylinder"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> HasTransform<F> for Cylinder<F, M> {
    fn get_transform(&self) -> &Transform<F> {
        &self.xfrm
    }

    fn set_transform(&mut self, xfrm: &Transform<F>) {
        self.xfrm = *xfrm;
        self.recompute_aabb();
    }
}

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Cylinder<F, M> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_ranged(
            &self.xfrm,
            [-F::ONE, F::ONE],
            [-F::ONE, F::ONE],
            [F::ZERO, F::ONE],
        );
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Cylinder<F, M> {
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cylinder.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let self_height = F::ONE;
        let self_top_r = F::ONE;
        let self_bot_r = F::ONE;

        let bot_r = self_bot_r.abs().max(F::BIAS);
        let top_r = self_top_r.abs().max(F::BIAS);

        let mut beta = (top_r - bot_r) / self_height;

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
            gamma -= self_height;
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

        let (root1, root2) = crate::types::ray::quadratic2(a, b, c)?;

        /* test side 1 */
        if root1.is_positive() && (root1 < root) {
            let point = r.extend(root1);
            if point.z >= F::ZERO && point.z <= self_height {
                root = root1;
                normal = vec3!(-point.x, -point.y, F::TWO * beta2 * (point.z + gamma));
            }
        }

        /* test side 2 */
        if root2.is_positive() && (root2 < root) {
            let point = r.extend(root2);
            if point.z >= F::ZERO && point.z <= self_height {
                root = root2;
                normal = vec3!(point.x, point.y, -F::TWO * beta2 * (point.z + gamma));
            }
        }

        if self.capped {
            let t1 = (-p.z) / d.z;
            let t2 = (self_height - p.z) / d.z;
            let cap_normal = if d.z.is_positive() {
                -Vector::UNIT_Z
            } else {
                Vector::UNIT_Z
            };

            /* test bottom cap */
            if t1 <= F::ZERO && t1 < root {
                let p = r.extend(t1);
                if p.x * p.x + p.y * p.y <= self_bot_r * self_bot_r {
                    root = t1;
                    normal = cap_normal;
                }
            }

            /* test top cap */
            if t2 <= F::ZERO && t2 < root {
                let p = r.extend(t2);
                if p.x * p.x + p.y * p.y <= self_top_r * self_top_r {
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

impl<F: Float, M: Material<F>> Cylinder<F, M> {
    pub fn new(xfrm: Matrix4<F>, capped: bool, mat: M) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            capped,
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}

#[cfg(test)]
mod test {
    use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    use cgmath::{Matrix4, Zero};

    use super::{Cylinder, Geometry, Ray, Vector};
    use crate::types::Color;

    macro_rules! assert_vec {
        ($val:expr, $x:expr, $y:expr, $z:expr) => {
            assert_f64_near!($val.x, $x);
            assert_f64_near!($val.y, $y);
            assert_f64_near!($val.z, $z);
        };
    }

    #[test]
    fn test_cylinder1() {
        let c = Cylinder::new(Matrix4::from_scale(2.0), false, Color::WHITE);

        let r0 = Ray {
            pos: Vector::unit_x() * 4.0,
            dir: -Vector::unit_x(),
            lvl: 10,
            grp: 10,
            dbg: false,
        };
        let h0 = c.intersect(&r0).unwrap();
        assert_vec!(h0.pos, 2.0, 0.0, 0.0);
        assert_vec!(h0.dir, -1.0, 0.0, 0.0);

        let r1 = Ray {
            pos: Vector::zero(),
            dir: Vector::unit_x(),
            lvl: 10,
            grp: 10,
            dbg: false,
        };
        let h1 = c.intersect(&r1).unwrap();
        assert_vec!(h1.pos, 2.0, 0.0, 0.0);
        assert_vec!(h1.dir, 1.0, 0.0, 0.0);

        let r2 = Ray {
            pos: Vector::unit_x() * 1.99,
            dir: -Vector::unit_x(),
            lvl: 10,
            grp: 10,
            dbg: false,
        };
        let h2 = c.intersect(&r2).unwrap();
        assert_vec!(h2.pos, -2.0, 0.0, 0.0);
        assert_vec!(h2.dir, -1.0, 0.0, 0.0);
    }
}
