#[cfg(feature = "gui")]
use crate::types::Camera;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::Aabb;

use crate::geometry::{build_aabb_symmetric, FiniteGeometry, Geometry};
use crate::material::HasMaterial;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, HasTransform, MaterialId, Maxel, Ray, Transform, Vector, Vectorx};

#[derive(Debug)]
pub struct Sphere<F: Float> {
    xfrm: Transform<F>,
    mat: MaterialId,
    aabb: Aabb,
}

aabb_impl_fm!(Sphere<F>);

impl<F: Float> Interactive<F> for Sphere<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        Interactive::<F>::ui(&mut self.mat, ui)
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

geometry_impl_sceneobject!(Sphere<F>, "Sphere");
geometry_impl_hastransform!(Sphere<F>);
geometry_impl_hasmaterial!(Sphere<F>);

impl<F: Float> FiniteGeometry<F> for Sphere<F> {
    fn recompute_aabb(&mut self) {
        self.aabb = build_aabb_symmetric(&self.xfrm, F::ONE, F::ONE, F::ONE);
    }
}

impl<F: Float> Geometry<F> for Sphere<F> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let r = ray.xfrm_inv(&self.xfrm);

        let result = r.intersect_unit_sphere()?;
        let intersect = r.extend(result);

        Some(ray.hit_at(intersect, result, self, self.mat))
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }

    fn normal(&self, maxel: &mut Maxel<F>) -> Vector<F> {
        self.xfrm.nml(maxel.hit)
    }

    fn uv(&self, maxel: &mut Maxel<F>) -> crate::types::Point<F> {
        maxel.hit.polar_uv().into()
    }
}

impl<F: Float> Sphere<F> {
    pub const ICON: &'static str = egui_phosphor::regular::CIRCLE;

    pub fn new(xfrm: Matrix4<F>, mat: MaterialId) -> Self {
        let mut res = Self {
            xfrm: Transform::new(xfrm),
            mat,
            aabb: Aabb::empty(),
        };
        res.recompute_aabb();
        res
    }

    pub fn place(pos: Vector<F>, radius: F, mat: MaterialId) -> Self {
        let scale = Matrix4::from_scale(radius);
        let xlate = Matrix4::from_translation(pos);
        Self::new(xlate * scale, mat)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use std::hint::black_box;

    use cgmath::{Deg, Matrix4};
    use rand::Rng;
    use test::Bencher;

    use super::{Float, Geometry, Ray, Sphere, Vector, Vectorx};
    use crate::types::MaterialId;

    type F = f64;

    fn sphere() -> Sphere<F> {
        let xfrm = Matrix4::from_translation(Vector::UNIT_Z)
            * Matrix4::from_angle_y(Deg(45.0))
            * Matrix4::from_scale(0.3);
        let sq = Sphere::new(xfrm, MaterialId::NULL);
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

    fn bench_sphere_intersect(
        bench: &mut Bencher,
        gendir: fn(idx: usize) -> Vector<F>,
        test: fn(ray: &Ray<F>, obj: &Sphere<F>) -> bool,
        check: fn(hits: usize, rays: usize),
    ) {
        const ITERATIONS: usize = 100;
        let mut ray = ray();
        let obj = sphere();
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

    fn bench_sphere_intersect_mixed(bench: &mut Bencher, test: fn(&Ray<F>, &Sphere<F>) -> bool) {
        bench_sphere_intersect(
            bench,
            |_idx| randdir(),
            test,
            |hits, rays| {
                assert_ne!(hits, 0);
                assert_ne!(hits, rays);
            },
        )
    }

    fn bench_sphere_intersect_never(bench: &mut Bencher, test: fn(&Ray<F>, &Sphere<F>) -> bool) {
        bench_sphere_intersect(
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

    fn bench_sphere_intersect_always(bench: &mut Bencher, test: fn(&Ray<F>, &Sphere<F>) -> bool) {
        bench_sphere_intersect(
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
        bench_sphere_intersect_mixed(bench, |ray, sphere| sphere.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the sphere

    #[bench]
    fn intersect_never(bench: &mut Bencher) {
        bench_sphere_intersect_never(bench, |ray, sphere| sphere.intersect(&ray).is_some());
    }

    // benchmark methods for rays that miss the sphere

    #[bench]
    fn intersect_always(bench: &mut Bencher) {
        bench_sphere_intersect_always(bench, |ray, sphere| sphere.intersect(&ray).is_some());
    }
}
