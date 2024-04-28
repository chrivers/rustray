use std::ops::Div;

use derive_more::{Add, Mul, Sub};
use num::Num;
use num_traits::ConstZero;

use crate::types::Float;

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

impl<F: Num + Copy + ConstZero> Point<F> {
    pub const ZERO: Self = Self {
        x: F::ZERO,
        y: F::ZERO,
    };

    #[must_use]
    pub const fn new(x: F, y: F) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn dot(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: F::ZERO,
            y: F::ZERO,
        }
    }
}

impl<F: Float> Div for Point<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        point!(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<F: Float> Div<F> for Point<F> {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        point!(self.x / rhs, self.y / rhs)
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

impl<F: Float> From<(u32, u32)> for Point<F> {
    fn from(val: (u32, u32)) -> Self {
        Self::new(F::from_u32(val.0), F::from_u32(val.1))
    }
}

#[cfg(feature = "gui")]
impl<F: Float> From<egui::emath::Pos2> for Point<F> {
    fn from(val: egui::emath::Pos2) -> Self {
        Self::new(F::from_f32(val[0]), F::from_f32(val[1]))
    }
}

#[cfg(feature = "gui")]
impl<F: Float> From<Point<F>> for egui::emath::Pos2 {
    fn from(val: Point<F>) -> Self {
        Self {
            x: val.x.to_f64() as f32,
            y: val.y.to_f64() as f32,
        }
    }
}

#[cfg(feature = "gui")]
impl<F: Float> From<egui::Vec2> for Point<F> {
    fn from(val: egui::Vec2) -> Self {
        Self::new(F::from_f32(val[0]), F::from_f32(val[1]))
    }
}
