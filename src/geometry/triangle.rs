use super::geo_util::*;

use std::fmt;
use std::fmt::Debug;

use rtbvh::SpatialTriangle;

#[derive(Clone, Debug)]
pub struct Triangle<F: Float, M: Material<F=F>>
{
    a: Vector<F>,
    b: Vector<F>,
    c: Vector<F>,

    na: Vector<F>,
    nb: Vector<F>,
    nc: Vector<F>,

    ta: Point<F>,
    tb: Point<F>,
    tc: Point<F>,

    aabb: Aabb,

    mat: M,
}

impl<F: Float, M: Material<F=F>> SpatialTriangle for Triangle<F, M>
{
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

impl<F: Float, M: Material<F=F>> std::fmt::Display for Triangle<F, M>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Triangle {{ a:")?; Debug::fmt(&self.a, f)?;
        write!(f, ", b:")?;  Debug::fmt(&self.b, f)?;
        write!(f, ", c:")?;  Debug::fmt(&self.c, f)?;
        write!(f, ", na:")?; Debug::fmt(&self.na, f)?;
        write!(f, ", nb:")?; Debug::fmt(&self.nb, f)?;
        write!(f, ", nc:")?; Debug::fmt(&self.nc, f)?;
        f.write_str("}")
    }
}

impl<F: Float, M: Material<F=F>> Triangle<F, M> {

    fn interpolate_normal(&self, u: F, v: F) -> Vector<F>
    {
        let w = F::ONE - u - v;
        let normal =
            self.na * w +
            self.nb * u +
            self.nc * v;

        normal.normalize()
    }

    fn interpolate_uv(&self, u: F, v: F) -> Point<F>
    {
        let w = F::ONE - u - v;
        (self.ta * w) + (self.tb * u) + (self.tc * v)
    }

}

impl<F: Float, M: Material<F=F> + Clone> HitTarget<F> for Triangle<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let c1 = (self.b - self.a).cross(hit.pos - self.b);
        let c2 = (self.c - self.a).cross(hit.pos - self.c);
        let area2 = self.n.magnitude();
        let u = c1.magnitude() / area2;
        let v = c2.magnitude() / area2;

        let normal = self.interpolate_normal(u, v);
        let uv = self.interpolate_uv(u, v);

        Maxel::new(uv, normal, &self.mat)
    }
}

impl<F: Float, M: Material<F=F> + Clone> Geometry<F> for Triangle<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_triangle(&self.a, &self.b, &self.c)?;
        Some(ray.hit_at(t, self))
    }

}

impl<F: Float, M: Material<F=F>> Triangle<F, M>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: Vector<F>, b: Vector<F>, c: Vector<F>, na: Vector<F>, nb: Vector<F>, nc: Vector<F>, ta: Point<F>, tb: Point<F>, tc: Point<F>, mat: M) -> Self
    {
        let mut aabb = Aabb::empty();
        aabb.grow(a.into_vector3());
        aabb.grow(b.into_vector3());
        aabb.grow(c.into_vector3());

        Triangle {
            a, b, c,
            na, nb, nc,
            ta, tb, tc,
            aabb,
            mat,
        }
    }
}
