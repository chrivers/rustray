#[cfg(feature = "gui")]
use crate::types::Camera;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_symmetric, FiniteGeometry, Geometry};
use crate::material::Material;
use crate::point;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, HasTransform, Maxel, Point, Ray, Transform, Vector, Vectorx};

#[derive(Debug)]
pub struct Square<F: Float, M: Material<F>> {
    xfrm: Transform<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Square<F, M>);

#[cfg(feature = "gui")]
impl<F: Float, M: Material<F>> Interactive<F> for Square<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.mat.ui(ui)
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }
}

geometry_impl_sceneobject!(Square<F, M>, "Square");
geometry_impl_hastransform!(Square<F, M>);

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Square<F, M> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::HALF, F::HALF, F::ZERO);
    }
}

impl<F: Float, M: Material<F>> Geometry<F> for Square<F, M> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        if r.dir.z.is_zero() {
            return None;
        }

        let t = -r.pos.z / r.dir.z;

        if t <= F::BIAS2 {
            return None;
        }

        let mut p = r.extend(t);
        p.x += F::HALF;
        p.y += F::HALF;

        if !p.x.is_unit() {
            return None;
        }

        if !p.y.is_unit() {
            return None;
        }

        let normal = if r.dir.z.is_positive() {
            -Vector::UNIT_Z
        } else {
            Vector::UNIT_Z
        };

        Some(
            ray.hit_at(t, self, &self.mat)
                .with_normal(self.xfrm.nml(normal))
                .with_uv(point!(p.x, p.y)),
        )
    }
}

impl<F: Float, M: Material<F>> Square<F, M> {
    pub const ICON: &'static str = egui_phosphor::regular::SQUARE;

    pub fn new(xfrm: Matrix4<F>, mat: M) -> Self {
        let mut res = Self {
            mat,
            xfrm: Transform::new(xfrm),
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use std::hint::black_box;

    use cgmath::{Deg, Matrix4};
    use rand::Rng;
    use test::Bencher;

    use super::{Float, Geometry, Ray, Square, Vector, Vectorx};
    use crate::types::Color;

    type F = f64;

    fn square() -> Square<F, Color<F>> {
        let xfrm = Matrix4::from_translation(Vector::UNIT_Z)
            * Matrix4::from_angle_y(Deg(45.0))
            * Matrix4::from_scale(0.5);
        let sq = Square::new(xfrm, Color::BLACK);
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

    fn bench_square_intersect(
        bench: &mut Bencher,
        gendir: fn(idx: usize) -> Vector<F>,
        test: fn(ray: &Ray<F>, obj: &Square<F, Color<F>>) -> bool,
        check: fn(hits: usize, rays: usize),
    ) {
        const ITERATIONS: usize = 100;
        let mut ray = ray();
        let obj = square();
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

    fn bench_square_intersect_mixed(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Square<F, Color<F>>) -> bool,
    ) {
        bench_square_intersect(
            bench,
            |_idx| randdir(),
            test,
            |hits, rays| {
                assert_ne!(hits, 0);
                assert_ne!(hits, rays);
            },
        )
    }

    fn bench_square_intersect_never(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Square<F, Color<F>>) -> bool,
    ) {
        bench_square_intersect(
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

    fn bench_square_intersect_always(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Square<F, Color<F>>) -> bool,
    ) {
        bench_square_intersect(
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
        bench_square_intersect_mixed(bench, |ray, square| square.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the square

    #[bench]
    fn intersect_never(bench: &mut Bencher) {
        bench_square_intersect_never(bench, |ray, square| square.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the square

    #[bench]
    fn intersect_always(bench: &mut Bencher) {
        bench_square_intersect_always(bench, |ray, square| square.intersect(&ray).is_some());
    }
}
