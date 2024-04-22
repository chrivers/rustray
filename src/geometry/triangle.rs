use std::fmt::{self, Debug};

use cgmath::InnerSpace;
use glam::Vec3;
use rtbvh::{Aabb, SpatialTriangle};

use crate::geometry::{FiniteGeometry, Geometry};
use crate::material::HasMaterial;
use crate::point;
use crate::scene::{Interactive, SceneObject};
use crate::types::{Float, MaterialId, Maxel, Point, Ray, Vector, Vectorx};

#[derive(Clone, Debug)]
pub struct Triangle<F: Float> {
    pub(crate) a: Vector<F>,
    pub(crate) b: Vector<F>,
    pub(crate) c: Vector<F>,

    pub(crate) na: Vector<F>,
    pub(crate) nb: Vector<F>,
    pub(crate) nc: Vector<F>,

    ta: Point<F>,
    tb: Point<F>,
    tc: Point<F>,

    pub(crate) edge1: Vector<F>,
    pub(crate) edge2: Vector<F>,
    pub(crate) area2: F,

    aabb: Aabb,

    mat: MaterialId,
}

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Triangle<F> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        Interactive::<F>::ui(&mut self.mat, ui)
    }

    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}

geometry_impl_sceneobject!(Triangle<F>, "Triangle");
geometry_impl_hasmaterial!(Triangle<F>);

impl<F: Float> SpatialTriangle for Triangle<F> {
    fn vertex0(&self) -> Vec3 {
        self.a.into_vec3()
    }
    fn vertex1(&self) -> Vec3 {
        self.b.into_vec3()
    }
    fn vertex2(&self) -> Vec3 {
        self.c.into_vec3()
    }
}

impl<F: Float> FiniteGeometry<F> for Triangle<F> {
    fn recompute_aabb(&mut self) {
        let mut aabb = Aabb::empty();
        aabb.grow(self.a.into_vec3());
        aabb.grow(self.b.into_vec3());
        aabb.grow(self.c.into_vec3());
        self.aabb = aabb;
    }
}

aabb_impl_fm!(Triangle<F>);

impl<F: Float> fmt::Display for Triangle<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Triangle {{ pos: [{a:?} {b:?} {c:?}], nml: [{na:?} {nb:?} {nc:?}], tex: [{ta:?} {tb:?} {tc:?}] }}",
            a = self.a,
            b = self.b,
            c = self.c,
            na = self.na,
            nb = self.nb,
            nc = self.nc,
            ta = self.ta,
            tb = self.tb,
            tc = self.tc,
        )
    }
}

impl<F: Float> Triangle<F> {
    fn interpolate_normal(&self, u: F, v: F) -> Vector<F> {
        let w = F::ONE - u - v;
        let normal = self.na * w + self.nb * u + self.nc * v;

        normal.normalize()
    }

    fn interpolate_uv(&self, u: F, v: F) -> Point<F> {
        let w = F::ONE - u - v;
        (self.ta * w) + (self.tb * u) + (self.tc * v)
    }
}

impl<F: Float> Geometry<F> for Triangle<F> {
    fn st(&self, hit: &mut Maxel<F>) -> Point<F> {
        let c1 = self.edge1.cross(hit.pos - self.b);
        let c2 = self.edge2.cross(hit.pos - self.c);
        let s = c2.magnitude() / self.area2;
        let t = c1.magnitude() / self.area2;

        point!(s, t)
    }

    fn normal(&self, hit: &mut Maxel<F>) -> Vector<F> {
        let st = hit.st();
        self.interpolate_normal(st.x, st.y)
    }

    fn uv(&self, hit: &mut Maxel<F>) -> Point<F> {
        let st = hit.st();
        self.interpolate_uv(st.x, st.y)
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let t = ray.intersect_triangle4(&self.edge1, &self.edge2, &self.a)?;
        Some(ray.hit_at(t, self, &self.mat))
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        Some(self)
    }
}

impl<F: Float> Triangle<F> {
    const ICON: &'static str = egui_phosphor::regular::TRIANGLE;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a: Vector<F>,
        b: Vector<F>,
        c: Vector<F>,
        na: Vector<F>,
        nb: Vector<F>,
        nc: Vector<F>,
        ta: Point<F>,
        tb: Point<F>,
        tc: Point<F>,
        mat: MaterialId,
    ) -> Self {
        let edge1 = b - a;
        let edge2 = c - a;
        let area2 = edge1.cross(edge2).magnitude();

        let mut res = Self {
            a,
            b,
            c,
            na,
            nb,
            nc,
            ta,
            tb,
            tc,
            edge1,
            edge2,
            area2,
            mat,
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

    use rand::Rng;
    use test::Bencher;

    use crate::{geometry::Triangle, types::MaterialId};

    use super::{Float, Point, Ray, Vector, Vectorx};

    type F = f64;

    fn triangle() -> Triangle<F> {
        let a = Vector::new(0.0, -1.0, 10.0);
        let b = Vector::new(-1.0, 1.0, 10.0);
        let c = Vector::new(1.0, 1.0, 10.0);
        let tri = Triangle::new(
            a,
            b,
            c,
            Vector::ZERO,
            Vector::ZERO,
            Vector::ZERO,
            Point::ZERO,
            Point::ZERO,
            Point::ZERO,
            MaterialId(0),
        );
        black_box(tri)
    }

    fn ray() -> Ray<F> {
        let ray = Ray::<F>::new(-Vector::UNIT_Z * F::TWO, Vector::UNIT_Z);
        black_box(ray)
    }

    fn randdir() -> Vector<F> {
        let mut rng = rand::thread_rng();
        Vector::new(rng.gen::<F>() * 0.2 - 0.1, rng.gen::<F>() * 0.2 - 0.1, 1.0)
    }

    fn bench_triangle_intersect<T>(
        bench: &mut Bencher,
        gendir: fn(idx: usize) -> Vector<F>,
        test: fn(ray: &Ray<F>, tri: &Triangle<F>) -> Option<T>,
        check: fn(hits: usize, rays: usize),
    ) {
        const ITERATIONS: usize = 100;
        let mut ray = ray();
        let tri = triangle();
        let dirs: Vec<_> = (0..ITERATIONS).map(gendir).collect();
        bench.iter(|| {
            let mut hits: usize = 0;
            for dir in &dirs {
                ray.dir = *dir;
                if test(&ray, &tri).is_some() {
                    hits += 1;
                }
            }
            check(hits, ITERATIONS);
        })
    }

    fn bench_triangle_intersect_mixed<T>(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Triangle<F>) -> Option<T>,
    ) {
        bench_triangle_intersect(
            bench,
            |_idx| randdir(),
            test,
            |hits, rays| {
                assert_ne!(hits, 0);
                assert_ne!(hits, rays);
            },
        )
    }

    fn bench_triangle_intersect_never<T>(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Triangle<F>) -> Option<T>,
    ) {
        bench_triangle_intersect(
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

    fn bench_triangle_intersect_always<T>(
        bench: &mut Bencher,
        test: fn(&Ray<F>, &Triangle<F>) -> Option<T>,
    ) {
        bench_triangle_intersect(
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
    fn intersect_mixed1(bench: &mut Bencher) {
        bench_triangle_intersect_mixed(bench, |ray, tri| {
            ray.intersect_triangle(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_mixed2(bench: &mut Bencher) {
        bench_triangle_intersect_mixed(bench, |ray, tri| {
            ray.intersect_triangle2(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_mixed3(bench: &mut Bencher) {
        bench_triangle_intersect_mixed(bench, |ray, tri| {
            ray.intersect_triangle3(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_mixed4(bench: &mut Bencher) {
        bench_triangle_intersect_mixed(bench, |ray, tri| {
            ray.intersect_triangle4(&tri.edge1, &tri.edge2, &tri.a)
        });
    }

    // benchmark methods for rays that miss the triangle

    #[bench]
    fn intersect_never1(bench: &mut Bencher) {
        bench_triangle_intersect_never(bench, |ray, tri| {
            ray.intersect_triangle(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_never2(bench: &mut Bencher) {
        bench_triangle_intersect_never(bench, |ray, tri| {
            ray.intersect_triangle2(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_never3(bench: &mut Bencher) {
        bench_triangle_intersect_never(bench, |ray, tri| {
            ray.intersect_triangle3(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_never4(bench: &mut Bencher) {
        bench_triangle_intersect_never(bench, |ray, tri| {
            ray.intersect_triangle4(&tri.edge1, &tri.edge2, &tri.a)
        });
    }

    // benchmark methods for rays that miss the triangle

    #[bench]
    fn intersect_always1(bench: &mut Bencher) {
        bench_triangle_intersect_always(bench, |ray, tri| {
            ray.intersect_triangle(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_always2(bench: &mut Bencher) {
        bench_triangle_intersect_always(bench, |ray, tri| {
            ray.intersect_triangle2(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_always3(bench: &mut Bencher) {
        bench_triangle_intersect_always(bench, |ray, tri| {
            ray.intersect_triangle3(&tri.a, &tri.b, &tri.c)
        });
    }

    #[bench]
    fn intersect_always4(bench: &mut Bencher) {
        bench_triangle_intersect_always(bench, |ray, tri| {
            ray.intersect_triangle4(&tri.edge1, &tri.edge2, &tri.a)
        });
    }
}
