use num::Float;
use std::ops::{Add, Sub, Mul};

/**
Convenience macro to construct a [`Point<F>`] from input values.

Shortens
```
Point::new(x, y)
```

 to

```
point!(x, y)
```
*/
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
}

impl<F: Float> Mul<F> for Point<F>
{
    type Output = Point<F>;

    fn mul(self, other: F) -> Point<F>
    {
        Point
        {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl<F: Float> Add for Point<F>
{
    type Output = Point<F>;

    fn add(self, other: Point<F>) -> Point<F>
    {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<F: Float> Sub for Point<F>
{
    type Output = Point<F>;

    fn sub(self, other: Point<F>) -> Point<F>
    {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
