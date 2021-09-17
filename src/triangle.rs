use crate::traits::Float;
use crate::point::Point;
use crate::scene::*;
use crate::vector::Vector;
use crate::ray::{Ray, Hit, Maxel};
use crate::material::Material;

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

    ni: usize,

    mat: &'a dyn Material<F=F>
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
            self.na * u +
            self.nb * v +
            self.nc * w;

        normal.normalized()
    }

    fn interpolate_uv(&self, u: F, v: F) -> Point<F>
    {
        let w = F::one() - u - v;
        (self.ta * u) + (self.tb * v) + (self.tc * w)
    }

}

impl<'a, F: Float> RayTarget<F> for Triangle<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let c1 = (self.c - self.b).cross(hit.pos - self.b);
        let c2 = (self.a - self.c).cross(hit.pos - self.c);
        let area2 = self.n.length();
        let u = c1.length() / area2;
        let v = c2.length() / area2;

        let normal = self.interpolate_normal(u, v);
        let uv = self.interpolate_uv(u, v);
        Maxel::from_uv(uv.x, uv.y, normal, self.mat)
    }

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
        Triangle {
            a, b, c,
            na, nb, nc,
            ta, tb, tc,
            n: ab.cross(ac),
            ni: 0,
            mat
        }
    }
}
