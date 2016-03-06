#![allow(dead_code)]

use num::Float;

#[derive(Clone, Copy)]
struct Point<F: Float>
{
    x: F,
    y: F,
}

impl<F: Float> Point<F>
{
    fn line_to(&self, other: Point<F>) -> Point<F>
    {
        Point
        {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    fn pos_det(&self, other: Point<F>) -> bool
    {
        self.x * other.y - self.y * other.x > F::zero()
    }
}
