#[cfg(feature = "gui")]
use crate::types::Camera;

use cgmath::{InnerSpace, Matrix4};
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_ranged, FiniteGeometry, Geometry};
use crate::material::HasMaterial;
use crate::scene::{Interactive, SceneObject};
use crate::types::{self, Float, HasTransform, MaterialId, Maxel, Ray, Transform, Vector, Vectorx};
use crate::vec3;

#[derive(Debug)]
pub struct Cylinder<F: Float> {
    xfrm: Transform<F>,
    capped: bool,
    mat: MaterialId,
    aabb: Aabb,
}

aabb_impl_fm!(Cylinder<F>);

impl<F: Float> Interactive<F> for Cylinder<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;
        res |= ui.checkbox(&mut self.capped, "Capped").changed();
        ui.end_row();

        res |= Interactive::<F>::ui(&mut self.mat, ui);

        res
    }

    #[cfg(feature = "gui")]
    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }

    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}

geometry_impl_sceneobject!(Cylinder<F>, "Cylinder");
geometry_impl_hastransform!(Cylinder<F>);
geometry_impl_hasmaterial!(Cylinder<F>);

impl<F: Float> FiniteGeometry<F> for Cylinder<F> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_ranged(
            &self.xfrm,
            [-F::ONE, F::ONE],
            [-F::ONE, F::ONE],
            [F::ZERO, F::ONE],
        );
    }
}

impl<F: Float> Geometry<F> for Cylinder<F> {
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

        let (root1, root2) = types::quadratic2(a, b, c)?;

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
            if t1 > F::ZERO && t1 < root {
                let p = r.extend(t1);
                if p.x * p.x + p.y * p.y <= self_bot_r * self_bot_r {
                    root = t1;
                    normal = cap_normal;
                }
            }

            /* test top cap */
            if t2 > F::ZERO && t2 < root {
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

        Some(
            ray.hit_at(r.extend(root), root, self, self.mat)
                .with_normal(nml),
        )
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }
}

impl<F: Float> Cylinder<F> {
    pub const ICON: &'static str = egui_phosphor::regular::CYLINDER;

    pub fn new(xfrm: Matrix4<F>, capped: bool, mat: MaterialId) -> Self {
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
    use cgmath::Matrix4;

    use super::{Cylinder, Geometry, Ray, Vector};
    use crate::types::MaterialId;

    macro_rules! assert_vec {
        ($val:expr, $x:expr, $y:expr, $z:expr) => {
            assert_f64_near!($val.x, $x);
            assert_f64_near!($val.y, $y);
            assert_f64_near!($val.z, $z);
        };
    }

    #[test]
    fn test_cylinder1() {
        let c = Cylinder::new(Matrix4::from_scale(2.0), false, MaterialId::NULL);

        let r0 = Ray::new(Vector::UNIT_X * 4.0, -Vector::UNIT_X);
        let h0 = c.intersect(&r0).unwrap();
        assert_vec!(h0.pos, 2.0, 0.0, 0.0);
        assert_vec!(h0.dir, -1.0, 0.0, 0.0);

        let r1 = Ray::new(Vector::ZERO, Vector::UNIT_X);
        let h1 = c.intersect(&r1).unwrap();
        assert_vec!(h1.pos, 2.0, 0.0, 0.0);
        assert_vec!(h1.dir, 1.0, 0.0, 0.0);

        let r2 = Ray::new(Vector::UNIT_X * 1.99, -Vector::UNIT_X);
        let h2 = c.intersect(&r2).unwrap();
        assert_vec!(h2.pos, -2.0, 0.0, 0.0);
        assert_vec!(h2.dir, -1.0, 0.0, 0.0);
    }

    extern crate test;

    use std::hint::black_box;

    use cgmath::Deg;
    use rand::Rng;
    use test::Bencher;

    use super::{Float, Vectorx};

    type F = f64;

    fn cylinder() -> Cylinder<F> {
        let xfrm = Matrix4::from_translation(Vector::UNIT_Z)
            * Matrix4::from_angle_y(Deg(45.0))
            * Matrix4::from_scale(0.3);
        let sq = Cylinder::new(xfrm, true, MaterialId::NULL);
        black_box(sq)
    }

    fn ray() -> Ray<F> {
        let ray = Ray::<F>::new(-Vector::UNIT_Z * F::TWO, Vector::UNIT_Z);
        black_box(ray)
    }

    fn randdir() -> Vector<F> {
        let mut rng = rand::thread_rng();
        Vector::new(rng.gen::<F>() * 0.2 - 0.1, rng.gen::<F>() * 0.2 - 0.1, 1.0)
    }

    fn bench_cylinder_intersect(
        bench: &mut Bencher,
        gendir: fn(idx: usize) -> Vector<F>,
        test: fn(ray: &Ray<F>, obj: &Cylinder<F>) -> bool,
        check: fn(hits: usize, rays: usize),
    ) {
        const ITERATIONS: usize = 100;
        let mut ray = ray();
        let obj = cylinder();
        let dirs: Vec<_> = (0..ITERATIONS).map(gendir).collect();
        bench.iter(|| {
            let mut hits: usize = 0;
            for dir in &dirs {
                ray.dir = *dir;
                if test(&ray, &obj) {
                    hits += 1;
                }
            }
            check(hits, ITERATIONS);
        })
    }

    fn bench_cylinder_intersect_mixed(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Cylinder<F>) -> bool,
    ) {
        bench_cylinder_intersect(
            bench,
            |_idx| randdir(),
            test,
            |hits, rays| {
                assert_ne!(hits, 0);
                assert_ne!(hits, rays);
            },
        )
    }

    fn bench_cylinder_intersect_never(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Cylinder<F>) -> bool,
    ) {
        bench_cylinder_intersect(
            bench,
            |_idx| {
                let mut vec = randdir();
                vec.z = -vec.z;
                vec
            },
            test,
            |hits, _rays| {
                assert_eq!(hits, 0);
            },
        )
    }

    fn bench_cylinder_intersect_always(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Cylinder<F>) -> bool,
    ) {
        bench_cylinder_intersect(
            bench,
            |_idx| {
                let mut rng = rand::thread_rng();
                Vector::new(
                    rng.gen::<F>() * 0.01 - 0.005,
                    rng.gen::<F>() * 0.01 - 0.005,
                    1.0,
                )
            },
            test,
            |hits, rays| {
                assert_eq!(hits, rays);
            },
        )
    }

    // benchmark methods with a mix of hit or miss rays

    #[bench]
    fn intersect_mixed(bench: &mut Bencher) {
        bench_cylinder_intersect_mixed(bench, |ray, cylinder| cylinder.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the cylinder

    #[bench]
    fn intersect_never(bench: &mut Bencher) {
        bench_cylinder_intersect_never(bench, |ray, cylinder| cylinder.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the cylinder

    #[bench]
    fn intersect_always(bench: &mut Bencher) {
        bench_cylinder_intersect_always(bench, |ray, cylinder| cylinder.intersect(&ray).is_some());
    }
}
