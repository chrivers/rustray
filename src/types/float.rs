use num::{clamp, pow};
use num_traits::{float::FloatConst, NumAssignOps};
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

use cgmath::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "gui")]
pub trait FloatReq: num::Float + num::Signed + egui::emath::Numeric {}

#[cfg(feature = "gui")]
impl<T: num::Float + num::Signed + egui::emath::Numeric> FloatReq for T {}

#[cfg(not(feature = "gui"))]
pub trait FloatReq: num::Float + num::Signed + 'static {}

#[cfg(not(feature = "gui"))]
impl<T: num::Float + num::Signed + 'static> FloatReq for T {}

pub trait Float
where
    Self: Debug,
    Self: Display,
    Self: Send,
    Self: Sync,
    Self: AbsDiffEq<Epsilon = Self>,
    Self: FloatConst,
    Self: Lerp<Ratio = Self>,
    Self: NumAssignOps,
    Self: RelativeEq,
    Self: UlpsEq,
    Self: FloatReq,
    Self: pow::Pow<Self, Output = Self>,
{
    const BIAS: Self; // Basic offset to account for numerical imprecision
    const BIAS2: Self; // Used for shadow rays
    const BIAS3: Self; // Used for reflected rays
    const BIAS4: Self; // Used for refracted rays
    const ZERO: Self;
    const HALF: Self;
    const ONE: Self;
    const TWO: Self;
    const FOUR: Self;
    fn from_i32(value: i32) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_usize(value: usize) -> Self;
    fn from_f32(value: f32) -> Self;
    #[cfg(not(feature = "gui"))]
    fn from_f64(value: f64) -> Self;

    fn non_zero(self) -> bool {
        self != Self::ZERO
    }

    #[must_use]
    fn clamp(self, low: Self, high: Self) -> Self {
        clamp(self, low, high)
    }

    fn is_unit(self) -> bool {
        (self >= Self::ZERO) && (self <= Self::ONE)
    }

    #[cfg(not(feature = "gui"))]
    fn to_f64(self) -> f64;
}

pub trait Lerp
where
    Self: Add<Output = Self> + Sub<Output = Self> + Mul<Self::Ratio, Output = Self> + Sized + Copy,
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

impl Float for f32 {
    const BIAS: Self = 1e-7;
    const BIAS2: Self = 1e-6;
    const BIAS3: Self = 1e-5;
    const BIAS4: Self = 1e-4;
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const FOUR: Self = 4.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline(always)]
    #[cfg(not(feature = "gui"))]
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    #[inline(always)]
    #[cfg(not(feature = "gui"))]
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Float for f64 {
    const BIAS: Self = 1e-10;
    const BIAS2: Self = 1e-9;
    const BIAS3: Self = 1e-7;
    const BIAS4: Self = 1e-5;
    const ZERO: Self = 0.0;
    const HALF: Self = 0.5;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
    const FOUR: Self = 4.0;

    #[inline(always)]
    fn from_i32(value: i32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }

    #[inline(always)]
    fn from_f32(value: f32) -> Self {
        value as Self
    }

    #[inline(always)]
    #[cfg(not(feature = "gui"))]
    fn from_f64(value: f64) -> Self {
        value
    }

    #[inline(always)]
    #[cfg(not(feature = "gui"))]
    fn to_f64(self) -> f64 {
        self as f64
    }
}
