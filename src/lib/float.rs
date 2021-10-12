use std::fmt::Debug;
use std::fmt::Display;
use num_traits::{float::FloatConst,NumAssignOps};
use num::{clamp, pow};
use std::ops::{Add, Sub, Mul};

use cgmath::{RelativeEq, AbsDiffEq, UlpsEq};

pub trait Float : num::Float + NumAssignOps + pow::Pow<Self, Output=Self> + FloatConst + num::Signed + Lerp + Debug + Display + Send + Sync + RelativeEq + AbsDiffEq<Epsilon = Self> + UlpsEq
{
    const BIAS: Self;
    const ZERO: Self;
    const HALF: Self;
    const ONE:  Self;
    const TWO:  Self;
    const FOUR: Self;
    fn from_i32(value: i32) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_f32(value: f32) -> Self;
    fn non_zero(self) -> bool { self != Self::zero() }

    fn clamp(self, low: Self, high: Self) -> Self { clamp(self, low, high) }
}

pub trait Lerp
where
    Self: Add<Output=Self>
    + Sub<Output=Self>
    + Mul<Self::Ratio, Output=Self>
    + Sized + Copy
{
    type Ratio: Float;

    #[inline]
    fn lerp(self, other: Self, amount: Self::Ratio) -> Self {
        self + ((other - self) * amount)
    }
}

impl Lerp for f32 {
    type Ratio = Self;
}
impl Lerp for f64 {
    type Ratio = Self;
}

impl Float for f32
{
    const BIAS: Self = 1e-6;
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;
    const ONE:  Self = 1.0;
    const TWO:  Self = 2.0;
    const FOUR: Self = 4.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self { value as Self }

    #[inline(always)]
    fn from_u32(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_usize(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn from_f32(value: f32) -> Self { value as Self }
}

impl Float for f64
{
    const BIAS: Self = 1e-9;
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;
    const ONE:  Self = 1.0;
    const TWO:  Self = 2.0;
    const FOUR: Self = 4.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self { value as Self }

    #[inline(always)]
    fn from_u32(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_usize(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn from_f32(value: f32) -> Self { value as Self }
}
