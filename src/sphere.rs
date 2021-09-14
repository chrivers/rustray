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
    radius: F,
}

impl<F: Float> RayTarget<F> for Sphere<F>
{
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>
    {
        let m = hit.vector_to(light.pos);
        let normal = self.pos.vector_to(*hit);
        let light_color = light.color * self.color;
        let reflection_coeff = F::max(normal.cos_angle(m), (-normal).cos_angle(m));
        light_color * reflection_coeff / m.length().sqrt()
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        let dir = ray.pos - self.pos;

        let f0 = F::zero();
        let f2 = F::from_float(2.0);
        let f4 = F::from_float(4.0);

        let a = ray.dir.dot(ray.dir);

        let b = ray.dir.dot(dir) * f2;
        let c = dir.dot(dir) - self.radius * self.radius;

        let d = b * b - f4 * a * c;

        if d.is_negative()
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
    pub fn new(pos: Vector<F>, _color: Color<F>, radius: F) -> Sphere<F>
    {
        Sphere { pos, radius }
    }
}
