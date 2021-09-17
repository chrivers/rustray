use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::ray::{Ray, Hit, Maxel};
use crate::material::Material;

pub struct Plane<'a, F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> RayTarget<F> for Plane<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        Maxel::zero(self.mat)
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, self))
    }

}

impl<'a, F: Float> Plane<'a, F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, mat: &'a dyn Material<F=F>) -> Plane<'a, F>
    {
        Plane { pos, dir1, dir2, mat }
    }
}
