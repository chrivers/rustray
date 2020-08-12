#![allow(dead_code)]

use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::Ray;

#[derive(Debug)]
pub struct Sphere<F: Float>
{
    pos: Vector<F>,
    color: Color<F>,
    radius: F,
}

impl<F: Float> RayTarget<F> for Sphere<F>
{
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>
    {
        let m = hit.vector_to(light.pos);
        let normal = self.pos.vector_to(*hit);
        let light_color = light.color * self.color;
        let reflection_coeff = F::max(normal.cos_angle(m), (normal * (-F::one())).cos_angle(m));
        light_color * reflection_coeff / m.length().sqrt()
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        let xd = ray.pos.x - self.pos.x;
        let yd = ray.pos.y - self.pos.y;
        let zd = ray.pos.z - self.pos.z;
        let _0 = F::zero();
        let _2 = F::from(2.0).unwrap();
        let _4 = F::from(4.0).unwrap();

        let a =
            ray.dir.x * ray.dir.x +
            ray.dir.y * ray.dir.y +
            ray.dir.z * ray.dir.z;

        let b =
            _2 *
            (ray.dir.x * xd +
             ray.dir.y * yd +
             ray.dir.z * zd);
        let c =
            xd * xd +
            yd * yd +
            zd * zd -
            self.radius * self.radius;

        let d = b * b - _4 * a * c;

        if d < F::zero()
        {
            return None;
        }

        let twice_a = _2 * a;
        let t1 = (-b + d.sqrt()) / twice_a;
        let t2 = (-b - d.sqrt()) / twice_a;

        let t = match true
        {
            _ if t1 < _0 => t2,
            _ if t2 < _0 => t1,
            _ => F::min(t1, t2),
        };

        if t < F::epsilon()
        {
            None
        } else
        {
            Some(ray.extend(t))
        }
    }
}

impl<F: Float> Sphere<F>
{
    pub fn new(pos: Vector<F>, color: Color<F>, radius: F) -> Sphere<F>
    {
        Sphere { pos: pos, color: color, radius: radius }
    }
}
