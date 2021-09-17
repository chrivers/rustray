use crate::traits::Float;
use crate::scene::*;
use crate::ray::{Ray, Hit, Maxel};
use crate::material::Material;

pub struct TestObject<'a, F: Float>
{
    pct: F,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> HitTarget<F> for TestObject<'a, F>
{
    fn resolve(&self, _hit: &Hit<F>) -> Maxel<F>
    {
        Maxel::zero(self.mat)
    }
}

impl<'a, F: Float> RayTarget<F> for TestObject<'a, F>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let rand = rand::random();
        if F::from_f32(rand) > self.pct {
            Some(ray.hit_at(F::zero(), self))
        } else {
            None
        }
    }

}

impl<'a, F: Float> TestObject<'a, F>
{
    pub fn new(pct: F, mat: &'a dyn Material<F=F>) -> TestObject<'a, F>
    {
        TestObject { pct, mat }
    }
}
