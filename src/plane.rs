use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::{Ray, Hit};
use crate::point::Point;

#[derive(Debug)]
pub struct Plane<F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    color: Color<F>,
}

impl<F: Float> RayTarget<F> for Plane<F>
{
    fn resolve(&self, hit: &Hit<F>, lights: &[Light<F>], _rt: &dyn RayTracer<F>, _lvl: u32) -> Color<F>
    {
        let normal = self.dir2.cross(self.dir1);

        let mut res = Color::black();
        for light in lights {
            let m = hit.pos.vector_to(light.pos);
            let light_color = light.color * self.color;
            let reflection_coeff = normal.cos_angle(m);
            res += light_color * reflection_coeff / m.length().sqrt();
        }
        res
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, Point::zero(), self))
    }

}

impl<F: Float> Plane<F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, color: Color<F>) -> Plane<F>
    {
        Plane { pos, dir1, dir2, color }
    }
}
