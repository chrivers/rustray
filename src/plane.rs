use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::Ray;

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
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>
    {
        let m = hit.vector_to(light.pos);
        let normal = self.dir2.crossed(self.dir1);
        let light_color = light.color * self.color;
        // let reflection_coeff = F::max(normal.cos_angle(m), (normal * (-F::one())).cos_angle(m));
        let reflection_coeff = normal.cos_angle(m);
        light_color * reflection_coeff
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        ray_hit_plane(&self.pos, &self.dir1, &self.dir2, ray)
    }

}

impl<F: Float> Plane<F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, color: Color<F>) -> Plane<F>
    {
        Plane { pos, dir1, dir2, color }
    }
}

pub fn ray_hit_plane<F>(pos: &Vector<F>, dir1: &Vector<F>, dir2: &Vector<F>, ray: &Ray<F>) -> Option<Vector<F>>
    where F: Float
{
    let abc = dir1.crossed(*dir2);
    let d = abc.dot(*pos);
    let t = (-abc.dot(ray.pos) + d) / abc.dot(ray.dir);

    if t < F::epsilon() {
        None
    } else {
        Some(ray.extend(t))
    }
}
