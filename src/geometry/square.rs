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
pub struct Square<F: Float> {
    xfrm: Transform<F>,
    mat: MaterialId,
    aabb: Aabb,
}

aabb_impl_fm!(Square<F>);

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Square<F> {
    #[cfg(feature = "gui")]
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

geometry_impl_sceneobject!(Square<F>, "Square");
geometry_impl_hastransform!(Square<F>);
geometry_impl_hasmaterial!(Square<F>);

impl<F: Float> FiniteGeometry<F> for Square<F> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::HALF, F::HALF, F::ZERO);
    }
}

impl<F: Float> Geometry<F> for Square<F> {
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
            ray.hit_at(r.extend(t), t, self, self.mat)
                .with_normal(self.xfrm.nml(normal))
                .with_uv(point!(p.x, p.y)),
        )
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }
}

impl<F: Float> Square<F> {
    pub const ICON: &'static str = egui_phosphor::regular::SQUARE;

    pub fn new(xfrm: Matrix4<F>, mat: MaterialId) -> Self {
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
    use crate::types::MaterialId;

    type F = f64;

    fn square() -> Square<F> {
        let xfrm = Matrix4::from_translation(Vector::UNIT_Z)
            * Matrix4::from_angle_y(Deg(45.0))
            * Matrix4::from_scale(0.5);
        let sq = Square::new(xfrm, MaterialId::NULL);
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
        test: fn(ray: &Ray<F>, obj: &Square<F>) -> bool,
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

    fn bench_square_intersect_mixed(bench: &mut Bencher, test: fn(&Ray<F>, &Square<F>) -> bool) {
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

    fn bench_square_intersect_never(bench: &mut Bencher, test: fn(&Ray<F>, &Square<F>) -> bool) {
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

    fn bench_square_intersect_always(bench: &mut Bencher, test: fn(&Ray<F>, &Square<F>) -> bool) {
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
