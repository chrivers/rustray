use crate::traits::Float;
use crate::scene::*;
use crate::color::Color;
use crate::light::Light;
use crate::ray::{Ray, Hit};
use crate::point::Point;

#[derive(Debug)]
pub struct TestObject<F: Float>
{
    pct: F,
}

impl<F: Float> RayTarget<F> for TestObject<F>
{
    fn resolve(&self, _hit: &Hit<F>, _light: &[Light<F>], _rt: &dyn RayTracer<F>, _lvl: u32) -> Color<F>
    {
        Color::white()
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let rand = rand::random();
        if F::from_f32(rand) > self.pct {
            Some(ray.hit_at(F::zero(), Point::zero(), self))
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
