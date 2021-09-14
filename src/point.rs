use num::Float;

#[macro_export]
macro_rules! point {
    ($( $vals:expr ),+) => { Point::new( $($vals),+ ) }
}

#[derive(Clone, Copy, Debug)]
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

    pub fn zero() -> Point<F>
    {
        Point { x: F::zero(), y: F::zero() }
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
