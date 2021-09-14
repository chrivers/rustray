use std::fmt::Debug;
use std::fmt::Display;
use num_traits::float::FloatConst;
use num::clamp;

pub trait Float : num::Float + FloatConst + num::Signed + Debug + Display + Sync
{
    const BIAS: Self;
    const HALF: Self;
    const TWO:  Self;
    fn from_i32(value: i32) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_float(value: f32) -> Self;
    fn non_zero(self) -> bool { self != Self::zero() }

    fn clamp(self, low: Self, high: Self) -> Self { clamp(self, low, high) }
}

impl Float for f32
{
    const BIAS: Self = 1e-4;
    const HALF: Self = 0.5;
    const TWO:  Self = 2.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self { value as Self }

    #[inline(always)]
    fn from_u32(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_float(value: f32) -> Self { value as Self }
}

impl Float for f64
{
    const BIAS: Self = 1e-10;
    const HALF: Self = 0.5;
    const TWO:  Self = 2.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self { value as Self }

    #[inline(always)]
    fn from_u32(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_float(value: f32) -> Self { value as Self }
}
