#![allow(dead_code)]

use num::Float;

#[derive(Clone, Copy)]
pub struct Point<F: Float>
{
    pub x: F,
    pub y: F,
}

impl<F: Float> Point<F>
{
    pub fn new(x: F, y: F) -> Point<F>
    {
        Point { x, y }
    }

    pub fn line_to(&self, other: Point<F>) -> Point<F>
    {
        Point
        {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    pub fn pos_det(&self, other: Point<F>) -> bool
    {
        self.x * other.y - self.y * other.x > F::zero()
    }
}
