use super::geo_util::*;

use std::fmt;
use std::fmt::Debug;

use rtbvh::SpatialTriangle;

#[derive(Clone, Debug)]
pub struct Triangle<F: Float, M: Material<F = F>> {
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

impl<F: Float, M: Material<F = F>> Interactive for Triangle<F, M> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.mat.ui(ui);
    }
}

impl<F: Float, M: Material<F = F>> SceneObject for Triangle<F, M> {
    fn get_name(&self) -> &str {
        "Triangle"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float, M: Material<F = F>> SpatialTriangle for Triangle<F, M> {
    fn vertex0(&self) -> Vec3 {
        self.a.into_vector3()
    }
    fn vertex1(&self) -> Vec3 {
        self.b.into_vector3()
    }
    fn vertex2(&self) -> Vec3 {
        self.c.into_vector3()
    }
}

aabb_impl_fm!(Triangle<F, M>);

impl<F: Float, M: Material<F = F>> fmt::Display for Triangle<F, M> {
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

impl<F: Float, M: Material<F = F>> Triangle<F, M> {
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

impl<F: Float, M: Material<F = F> + Clone> Geometry<F> for Triangle<F, M> {
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

impl<F: Float, M: Material<F = F>> Triangle<F, M> {
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
        let mut aabb = Aabb::empty();
        aabb.grow(a.into_vector3());
        aabb.grow(b.into_vector3());
        aabb.grow(c.into_vector3());

        let edge1 = b - a;
        let edge2 = c - a;
        let area2 = edge1.cross(edge2).magnitude();

        Self {
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
            aabb,
            mat,
        }
    }
}
