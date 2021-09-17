use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::ray::{Ray, Hit, Maxel};
use crate::material::Material;

pub struct Sphere<'a, F: Float>
{
    pos: Vector<F>,
    radius2: F,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> RayTarget<F> for Sphere<'a, F>
{

    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let q = (hit.pos - self.pos).normalized();
        let u = q.y.acos() / (F::PI() * F::TWO);
        let v = (q.z.atan2(q.x) + F::PI()) / (F::PI() * F::TWO);

        let normal = self.pos.normal_to(hit.pos);

        Maxel::from_uv(u, v, normal, self.mat)
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_sphere(&self.pos, self.radius2)?;

        Some(ray.hit_at(t, self))
    }

}

impl<'a, F: Float> Sphere<'a, F>
{
    pub fn new(pos: Vector<F>, radius: F, mat: &'a dyn Material<F=F>) -> Sphere<'a, F>
    {
        Sphere { pos, radius2: radius * radius, mat }
    }
}

impl<'a, F: Float> HasPosition<F> for Sphere<'a, F>
{
    fn get_position(&self) -> Vector<F> { self.pos }
    fn set_position(&mut self, value: Vector<F>) { self.pos = value }
}
