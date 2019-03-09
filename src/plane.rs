#![allow(dead_code)]

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
        Plane { pos: pos, dir1: dir1, dir2: dir2, color: color }
    }
}

pub fn ray_hit_plane<F>(pos: &Vector<F>, dir1: &Vector<F>, dir2: &Vector<F>, ray: &Ray<F>) -> Option<Vector<F>>
    where F: Float
{
    let v1 = *pos;
    let v2 = v1 - *dir1;
    let v3 = v1 - *dir2;
    let a = (v2.y - v1.y) * (v3.z - v1.z) - (v3.y - v1.y) * (v2.z - v1.z);
    let b = (v2.z - v1.z) * (v3.x - v1.x) - (v3.z - v1.z) * (v2.x - v1.x);
    let c = (v2.x - v1.x) * (v3.y - v1.y) - (v3.x - v1.x) * (v2.y - v1.y);
    let d = (-a * v1.x) + (-b * v1.y) + (-c * v1.z);
    let t = -(a * ray.pos.x + b * ray.pos.y + c * ray.pos.z + d) / (a * ray.dir.x + b * ray.dir.y + c * ray.dir.z);

    if t < F::epsilon()
    {
        None
    } else
    {
        Some(ray.extend(t))
    }
}
