use crate::traits::Float;
use crate::vector::Vector;
use crate::point::Point;
use crate::scene::RayTarget;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
}

pub struct Hit<'a, F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub normal: Vector<F>,
    pub uv: Point<F>,
    pub obj: &'a dyn RayTarget<F>,
}

impl<F: Float> Ray<F>
{
    pub fn new(pos: Vector<F>, dir: Vector<F>) -> Ray<F>
    {
        Ray { pos, dir }
    }

    pub fn length_to(self, other: Vector<F>) -> F
    {
        self.dir.cross(self.pos.vector_to(other)).length() / self.dir.length()
    }

    pub fn extend(self, scale: F) -> Vector<F>
    {
        self.pos + self.dir * scale
    }

    pub fn hit_at(self, ext: F, normal: Vector<F>, uv: Point<F>, obj: &dyn RayTarget<F>) -> Hit<'_, F>
    {
        Hit { pos: self.extend(ext), dir: self.dir, normal, uv, obj }
    }
}
