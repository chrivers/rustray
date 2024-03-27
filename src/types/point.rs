use crate::types::Float;
use derive_more::{Add, Mul, Sub};
use num::Num;

/**
Convenience macro to construct a [`Point<F>`] from input values.

```
use rustray::types::Point;
```

Shortens
```
# use rustray::types::Point;
# let x = 1.0f32;
# let y = 2.0f32;
let p: Point<f32> = Point::new(x, y);
```

 to

```
# use rustray::types::Point;
# use rustray::point;
# let x = 1.0f32;
# let y = 2.0f32;
let p: Point<f32> = point!(x, y);
```
*/
#[macro_export]
macro_rules! point {
    ($( $vals:expr ),+ $(,)?) => { Point::new( $($vals.into()),+ ) }
}

#[derive(Clone, Copy, Debug, Add, Sub, Mul)]
pub struct Point<F: Num> {
    pub x: F,
    pub y: F,
}

impl<F: Num + Copy> Point<F> {
    pub fn new(x: F, y: F) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    pub fn zero() -> Self {
        Self {
            x: F::zero(),
            y: F::zero(),
        }
    }
}

impl<F: Float> From<(F, F)> for Point<F> {
    fn from(val: (F, F)) -> Self {
        Self::new(val.0, val.1)
    }
}

impl<F: Float> From<[f32; 2]> for Point<F> {
    fn from(val: [f32; 2]) -> Self {
        Self::new(F::from_f32(val[0]), F::from_f32(val[1]))
    }
}
