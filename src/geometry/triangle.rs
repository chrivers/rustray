use crate::math::{Vector, Float, Point};
use crate::scene::*;
use crate::math::ray::{Ray, Hit, Maxel};
use crate::material::Material;

use std::fmt;
use std::fmt::Display;

#[derive(Clone)]
pub struct Triangle<'a, F: Float>
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

    mat: &'a dyn Material<F=F>
}


impl<'a, F: Float> std::fmt::Display for Triangle<'a, F>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Triangle {{ a:")?; Display::fmt(&self.a, f)?;
        write!(f, ", b:")?;  Display::fmt(&self.b, f)?;
        write!(f, ", c:")?;  Display::fmt(&self.c, f)?;
        write!(f, ", na:")?; Display::fmt(&self.na, f)?;
        write!(f, ", nb:")?; Display::fmt(&self.nb, f)?;
        write!(f, ", nc:")?; Display::fmt(&self.nc, f)?;
        f.write_str("}")
    }
}

use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::Point3;

impl<'a, F: Float> Bounded for Triangle<'a, F> {

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

impl<'a, F: Float> BHShape for Triangle<'a, F> {

    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }

}

impl<'a, F: Float> Triangle<'a, F> {

    fn interpolate_normal(&self, u: F, v: F) -> Vector<F>
    {
        let w = F::one() - u - v;
        let normal =
            self.na * w +
            self.nb * v +
            self.nc * u;

        normal.normalized()
    }

    fn interpolate_uv(&self, u: F, v: F) -> Point<F>
    {
        let w = F::one() - u - v;
        (self.ta * w) + (self.tb * v) + (self.tc * u)
    }

}

impl<'a, F: Float> HitTarget<F> for Triangle<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let c1 = (self.b - self.a).cross(hit.pos - self.b);
        let c2 = (self.c - self.a).cross(hit.pos - self.c);
        let area2 = self.n.length();
        let u = c1.length() / area2;
        let v = c2.length() / area2;

        let normal = self.interpolate_normal(u, v);
        let uv = self.interpolate_uv(u, v);
        Maxel::new(uv, normal, self.ntan1, self.ntan2, self.mat)
    }
}

impl<'a, F: Float> RayTarget<F> for Triangle<'a, F>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_triangle(&self.a, &self.b, &self.c, &self.n)?;
        Some(ray.hit_at(t, self))
    }

}

impl<'a, F: Float> Triangle<'a, F>
{
    pub fn new(a: Vector<F>, b: Vector<F>, c: Vector<F>, na: Vector<F>, nb: Vector<F>, nc: Vector<F>, ta: Point<F>, tb: Point<F>, tc: Point<F>, mat: &'a dyn Material<F=F>) -> Triangle<'a, F>
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
            ntan1: ntan1.normalized(),
            ntan2: ntan2.normalized(),
            ni: 0,
            mat
        }
    }
}
