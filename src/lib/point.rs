use crate::lib::Float;
use std::convert::From;
use num::Num;
use derive_more::{Add, Sub, Mul};

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
    ($( $vals:expr ),+ $(,)?) => { Point::new( $($vals.into()),+ ) }
}

#[derive(Clone, Copy, Debug)]
#[derive(Add, Sub, Mul)]
pub struct Point<F: Num>
{
    pub x: F,
    pub y: F,
}

impl<F: Num + Copy> Point<F>
{
    pub fn new(x: F, y: F) -> Point<F>
    {
        Point { x, y }
    }

    pub fn zero() -> Point<F>
    {
        Point { x: F::zero(), y: F::zero() }
    }
}

impl<F: Float> From<(F, F)> for Point<F>
{
    fn from(val: (F, F)) -> Self {
        Point::new(val.0, val.1)
    }
}

impl<F: Float> From<[f32; 2]> for Point<F>
{
    fn from(val: [f32; 2]) -> Self {
        Point::new(F::from_f32(val[0]), F::from_f32(val[1]))
    }
}
