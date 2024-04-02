use super::geo_util::*;

use std::fmt;
use std::fmt::Debug;

use rtbvh::SpatialTriangle;

#[derive(Clone, Debug)]
pub struct Triangle<F: Float, M: Material<F>> {
    a: Vector<F>,
    b: Vector<F>,
    c: Vector<F>,

    na: Vector<F>,
    nb: Vector<F>,
    nc: Vector<F>,

    ta: Point<F>,
    tb: Point<F>,
    tc: Point<F>,

    edge1: Vector<F>,
    edge2: Vector<F>,
    area2: F,

    aabb: Aabb,

    mat: M,
}

impl<F: Float, M: Material<F>> Interactive<F> for Triangle<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.mat.ui(ui);
    }
}

impl<F: Float, M: Material<F>> SceneObject<F> for Triangle<F, M> {
    fn get_name(&self) -> &str {
        "Triangle"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F>> SpatialTriangle for Triangle<F, M> {
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

impl<F: Float, M: Material<F>> FiniteGeometry<F> for Triangle<F, M> {
    fn recompute_aabb(&mut self) {
        let mut aabb = Aabb::empty();
        aabb.grow(self.a.into_vec3());
        aabb.grow(self.b.into_vec3());
        aabb.grow(self.c.into_vec3());
        self.aabb = aabb;
    }
}

aabb_impl_fm!(Triangle<F, M>);

impl<F: Float, M: Material<F>> fmt::Display for Triangle<F, M> {
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

impl<F: Float, M: Material<F>> Triangle<F, M> {
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

impl<F: Float, M: Material<F>> Geometry<F> for Triangle<F, M> {
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
}

impl<F: Float, M: Material<F>> Triangle<F, M> {
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
        mat: M,
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
