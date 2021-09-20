use std::fmt::Debug;
use std::fmt::Display;
use num_traits::{float::FloatConst,NumAssignOps};
use num::{clamp, pow};
use std::ops::{Add, Mul};

pub trait Float : num::Float + NumAssignOps + pow::Pow<Self, Output=Self> + FloatConst + num::Signed + Debug + Display + Sync
{
    const BIAS: Self;
    const HALF: Self;
    const TWO:  Self;
    const FOUR: Self;
    fn from_i32(value: i32) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_f32(value: f32) -> Self;
    fn non_zero(self) -> bool { self != Self::zero() }

    fn clamp(self, low: Self, high: Self) -> Self { clamp(self, low, high) }
}

pub trait Blended<F>
where
    F: Float,
    Self: Mul<F, Output=Self> + Add<Output=Self> + Copy + Sized
{
    fn blended(&self, other: &Self, pct: F) -> Self
    {
        (*self * (F::one() - pct)) + (*other * pct)
    }
}

impl Float for f32
{
    const BIAS: Self = 1e-4;
    const HALF: Self = 0.5;
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
    const BIAS: Self = 1e-10;
    const HALF: Self = 0.5;
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
