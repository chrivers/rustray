use crate::traits::Float;
use crate::vector::Vector;

#[derive(Clone, Copy, Debug)]
pub struct Ray<F: Float>
{
    pub pos: Vector<F>,
    pub dir: Vector<F>,
}

impl<F: Float> Ray<F>
{
    pub fn new(pos: Vector<F>, dir: Vector<F>) -> Ray<F>
    {
        Ray { pos, dir }
    }

    pub fn length_to(self, other: Vector<F>) -> F
    {
        self.dir.crossed(self.pos.vector_to(other)).length() / self.dir.length()
    }

    pub fn extend(self, scale: F) -> Vector<F>
    {
        self.pos + self.dir * scale
    }
}
