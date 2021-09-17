use crate::traits::Float;
use crate::point::Point;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::{Ray, Hit};
use crate::point::Point;

#[derive(Clone, Debug)]
pub struct Triangle<F: Float>
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
}

use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::Point3;

impl<F: Float> Bounded for Triangle<F> {

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

impl<F: Float> BHShape for Triangle<F> {

    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }

}

impl<F: Float> Triangle<F> {

    fn interpolate_normal(&self, u: F, v: F) -> Vector<F>
    {
        let w = F::one() - u - v;
        let normal =
            self.na * u +
            self.nb * v +
            self.nc * w;

        normal.normalized()
    }

}

impl<F: Float> RayTarget<F> for Triangle<F>
{
    fn resolve(&self, hit: &Hit<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let mut res = Color::black();

        let normal = self.interpolate_normal(hit.uv.x, hit.uv.y);

        let d = hit.dir;

        let refl = d.reflect(&normal);
        let reflection_coeff = d.cos_angle(refl);
        let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);

        let refr = hit.dir.refract(&normal, F::from_f32(1.13));
        let c_refr = rt.ray_trace(&Ray::new(hit.pos + refr * F::BIAS, refr), lvl + 1).unwrap_or_else(Color::black);

        let fr = hit.dir.fresnel(&normal, F::from_f32(1.13));
        res += c_refr.blended(&c_refl, fr);

        // for light in lights {
        //     // if rt.ray_shadow(hit, light) {
        //     //     continue
        //     // }
        //     let m = hit.pos.vector_to(light.pos);
        //     let light_color = light.color * Color::white();
        //     let reflection_coeff = normal.cos_angle(m);
        //     res += light_color * reflection_coeff / m.length().sqrt();
        // }
        res
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let (t, u, v) = ray.intersect_triangle(&self.a, &self.b, &self.c, &self.n)?;
        Some(ray.hit_at(t, point!(u, v), self))
    }

}

impl<F: Float> Triangle<F>
{
    pub fn new(a: Vector<F>, b: Vector<F>, c: Vector<F>, na: Vector<F>, nb: Vector<F>, nc: Vector<F>, ta: Point<F>, tb: Point<F>, tc: Point<F>) -> Triangle<F>
    {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);
        Triangle {
            a, b, c,
            na, nb, nc,
            ta, tb, tc,
            n: ab.cross(ac),
            ni: 0
        }
    }
}
