use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::color::Color;
use crate::light::Light;
use crate::ray::Ray;

#[derive(Debug)]
pub struct TestObject<F: Float>
{
    pct: F,
}

impl<F: Float> RayTarget<F> for TestObject<F>
{
    fn trace(&self, _hit: &Vector<F>, _light: &Light<F>) -> Color<F>
    {
        Color::white()
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        let rand = rand::random();
        if F::from_float(rand) > self.pct {
            Some(ray.pos)
        } else {
            None
        }
    }
}

impl<F: Float> TestObject<F>
{
    pub fn new(pct: F) -> TestObject<F>
    {
        TestObject { pct }
    }
}
