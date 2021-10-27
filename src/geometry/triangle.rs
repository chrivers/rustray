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

impl<F: Float, M: Material<F=F> + Clone> Geometry<F> for Triangle<F, M>
{

    fn st(&self, hit: &mut Maxel<F>) -> Point<F>
    {
        let edge1 = self.b - self.a;
        let edge2 = self.c - self.a;

        let c1 = edge1.cross(hit.pos - self.b);
        let c2 = edge2.cross(hit.pos - self.c);
        let area2 = edge1.cross(edge2).magnitude();
        let s = c2.magnitude() / area2;
        let t = c1.magnitude() / area2;

        point!(s, t)
    }

    fn normal(&self, hit: &mut Maxel<F>) -> Vector<F>
    {
        let st = hit.st();
        self.interpolate_normal(st.x, st.y)
    }

    fn uv(&self, hit: &mut Maxel<F>) -> Point<F>
    {
        let st = hit.st();
        self.interpolate_uv(st.x, st.y)
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        let t = ray.intersect_triangle4(&self.a, &self.b, &self.c)?;
        Some(ray.hit_at(t, self, &self.mat))
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
