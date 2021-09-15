use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::{Ray, Hit};
use crate::point::Point;

#[derive(Debug)]
pub struct Sphere<F: Float>
{
    pos: Vector<F>,
    radius2: F,
}

impl<F: Float> RayTarget<F> for Sphere<F>
{

    fn resolve(&self, hit: &Hit<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let q = (hit.pos - self.pos).normalized();
        let u = q.y.acos() / (F::PI() * F::TWO);
        let v = (q.z.atan2(q.x) + F::PI()) / (F::PI() * F::TWO);

        let normal = self.pos.normal_to(hit.pos);

        let d = hit.dir.normalized();

        let mut res = Color::black();

        let refl = hit.dir.reflect(&normal);
        let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);

        let refr = hit.dir.refract(&normal, F::from_f32(1.8));
        let c_refr = rt.ray_trace(&Ray::new(hit.pos + refr * F::BIAS, refr), lvl + 1).unwrap_or_else(Color::black);

        let fr = hit.dir.fresnel(&normal, F::from_f32(1.8));
        res += c_refr.blended(&c_refl, fr);

        // for light in lights {
        //     if rt.ray_shadow(&hit, light) {
        //         continue
        //     }
        //     let m = hit.pos.vector_to(light.pos);
        //     let reflection_coeff = normal.cos_angle(m);
        //     res += Color::white() * light.color * reflection_coeff / m.length().sqrt();
        // }
        res
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_sphere(&self.pos, self.radius2)?;

        Some(ray.hit_at(t, Point::zero(), self))
    }
}

impl<F: Float> Sphere<F>
{
    pub fn new(pos: Vector<F>, _color: Color<F>, radius: F) -> Sphere<F>
    {
        Sphere { pos, radius2: radius * radius }
    }
}

impl<F: Float> HasPosition<F> for Sphere<F>
{
    fn get_position(&self) -> Vector<F> { self.pos }
    fn set_position(&mut self, value: Vector<F>) { self.pos = value }
}
