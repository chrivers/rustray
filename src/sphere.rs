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
        let f0 = F::zero();
        let f2 = F::from_float(2.0);
        let f4 = F::from_float(4.0);

        let a =
            ray.dir.x * ray.dir.x +
            ray.dir.y * ray.dir.y +
            ray.dir.z * ray.dir.z;

        let b =
            f2 *
            (ray.dir.x * xd +
             ray.dir.y * yd +
             ray.dir.z * zd);
        let c =
            xd * xd +
            yd * yd +
            zd * zd -
            self.radius * self.radius;

        let d = b * b - f4 * a * c;

        if d < F::zero()
        {
            return None;
        }

        let twice_a = f2 * a;
        let dr = d.sqrt();
        let t = if dr < b {
            -b - dr
        } else if -dr < b {
            -b + dr
        } else {
            F::min(-b + dr, -b - dr)
        };

        let t = t / twice_a;

        if t < F::epsilon() {
            None
        } else {
            Some(ray.extend(t))
        }
    }
}

impl<F: Float> Sphere<F>
{
    pub fn new(pos: Vector<F>, color: Color<F>, radius: F) -> Sphere<F>
    {
        Sphere { pos, color, radius }
    }
}
