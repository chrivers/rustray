use crate::traits::Float;
use crate::scene::*;
use crate::vector::Vector;
use crate::ray::{Ray, Hit, Maxel};
use crate::material::Material;

pub struct ChessPlane<'a, F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> RayTarget<F> for ChessPlane<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let (u, v);
        if self.dir1.x.non_zero() {
            u = hit.pos.x / self.dir1.x;
            v = if self.dir2.y.non_zero() {
                hit.pos.y / self.dir2.y
            } else {
                hit.pos.z / self.dir2.z
            }
        } else {
            u = hit.pos.y / self.dir1.y;
            v = if self.dir2.x.non_zero() {
                hit.pos.x / self.dir2.x
            } else {
                hit.pos.z / self.dir2.z
            }
        }
        Maxel::from_uv(u, v, self.normal, self.mat)
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, self))
    }

}

impl<'a, F: Float> ChessPlane<'a, F>
{
    pub fn new(pos: Vector<F>, dir1: Vector<F>, dir2: Vector<F>, mat: &'a dyn Material<F=F>) -> ChessPlane<'a, F>
    {
        ChessPlane { pos, dir1, dir2, normal: dir1.cross(dir2), mat }
    }
}
