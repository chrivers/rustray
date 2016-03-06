#![allow(dead_code)]

use vector::Vector;
use color::Color;
use light::Light;
use num::Float;

struct Sphere<F: Float>
{
    pos: Vector<F>,
    color: Color<F>,
    radius: F,
}

impl<F: Float> Sphere<F>
{
    fn trace(&self, hit: Vector<F>, light: Light<F>) -> Color<F>
    {
        let m = hit.vector_to(light.pos);
        let normal = self.pos.vector_to(hit);
        let light_color = light.color * self.color;
        let reflection_coeff = F::max(normal.cos_angle(m), (normal * (-F::one())).cos_angle(m));
        light_color * reflection_coeff
    }
}
