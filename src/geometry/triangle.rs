use super::geo_util::*;

use std::fmt;
use std::fmt::Debug;

#[derive(Clone)]
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

    n: Vector<F>,
    ntan1: Vector<F>,
    ntan2: Vector<F>,

    ni: usize,

    mat: M,
}


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

use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::Point3;

impl<F: Float, M: Material<F=F>> Bounded for Triangle<F, M> {

    fn aabb(&self) -> AABB {
        let min = Point3::new(
            self.a.x.min(self.b.x.min(self.c.x)).to_f32().unwrap(),
            self.a.y.min(self.b.y.min(self.c.y)).to_f32().unwrap(),
            self.a.z.min(self.b.z.min(self.c.z)).to_f32().unwrap(),
        );
        let max = Point3::new(
            self.a.x.max(self.b.x.max(self.c.x)).to_f32().unwrap(),
            self.a.y.max(self.b.y.max(self.c.y)).to_f32().unwrap(),
            self.a.z.max(self.b.z.max(self.c.z)).to_f32().unwrap(),
        );
        AABB::with_bounds(min, max)
    }

}

impl<F: Float, M: Material<F=F>> BHShape for Triangle<F, M> {

    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }

}

impl<F: Float, M: Material<F=F>> Triangle<F, M> {

    fn interpolate_normal(&self, u: F, v: F) -> Vector<F>
    {
        let w = F::one() - u - v;
        let normal =
            self.na * w +
            self.nb * v +
            self.nc * u;

        normal.normalize()
    }

    fn interpolate_uv(&self, u: F, v: F) -> Point<F>
    {
        let w = F::one() - u - v;
        (self.ta * w) + (self.tb * v) + (self.tc * u)
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
        Maxel::new(uv, normal, self.ntan1, self.ntan2, &self.mat)
    }
}

impl<F: Float, M: Material<F=F> + Clone> RayTarget<F> for Triangle<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_triangle(&self.a, &self.b, &self.c)?;
        Some(ray.hit_at(t, self, None))
    }

}

impl<F: Float, M: Material<F=F>> Triangle<F, M>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(a: Vector<F>, b: Vector<F>, c: Vector<F>, na: Vector<F>, nb: Vector<F>, nc: Vector<F>, ta: Point<F>, tb: Point<F>, tc: Point<F>, mat: M) -> Self
    {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);

        let uv_ab = tb - ta;
        let uv_ac = tc - ta;

        let f = F::one() / (uv_ab.x * uv_ac.y - uv_ac.x * uv_ab.y);

        let ntan1 = ((ab * uv_ac.y) - (ac * uv_ab.y)) * f;
        let ntan2 = ((ac * uv_ab.x) - (ab * uv_ac.x)) * f;

        Triangle {
            a, b, c,
            na, nb, nc,
            ta, tb, tc,
            n: ab.cross(ac),
            ntan1: ntan1.normalize(),
            ntan2: ntan2.normalize(),
            ni: 0,
            mat,
        }
    }
}
