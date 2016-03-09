#![allow(dead_code)]

use rand;

use traits::Float;
use scene::*;
use vector;
use vector::Vector;
use color::Color;
use light::Light;
use ray::Ray;

#[derive(Debug)]
pub struct TestObject<F: Float>
{
    pct: F,
}

impl<F: Float> RayTarget<F> for TestObject<F>
{
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>
    {
        Color::<F>::white()
    }

    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>
    {
        let rand = rand::random::<f32>();
        if F::from(rand).unwrap() > self.pct
        {
            Some(ray.pos)
        } else
        {
            None
        }
    }
}

impl<F: Float> TestObject<F>
{
    pub fn new(pct: F) -> TestObject<F>
    {
        TestObject { pct: pct }
    }
}
