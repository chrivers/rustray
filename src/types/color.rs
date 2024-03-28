use cgmath::VectorSpace;
use derive_more::{Add, AddAssign, Rem, Sub};
use num_traits::Zero;
use std::fmt::{self, Debug};
use std::iter::Sum;
use std::ops;

use crate::sampler::Texel;

use crate::types::float::{Float, Lerp};

#[derive(Clone, Copy, Add, AddAssign, Sub, Rem)]
pub struct Color<F: Float> {
    pub r: F,
    pub g: F,
    pub b: F,
}

impl<F: Float> Texel for Color<F> {}

impl<F: Float> Debug for Color<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color(")?;
        Debug::fmt(&self.r, f)?;
        write!(f, ", ")?;
        Debug::fmt(&self.g, f)?;
        write!(f, ", ")?;
        Debug::fmt(&self.b, f)?;
        f.write_str(")")
    }
}

impl<F: Float> ops::Mul<F> for Color<F> {
    type Output = Self;

    fn mul(self, other: F) -> Self {
        Self {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}

impl<F: Float> ops::Mul<Color<F>> for Color<F> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

impl<F: Float> ops::Div for Color<F> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            r: self.r / other.r,
            g: self.g / other.g,
            b: self.b / other.b,
        }
    }
}

impl<F: Float> ops::Div<F> for Color<F> {
    type Output = Self;

    fn div(self, other: F) -> Self {
        Self {
            r: self.r / other,
            g: self.g / other,
            b: self.b / other,
        }
    }
}

impl<F: Float> Color<F> {
    pub const fn new(r: F, g: F, b: F) -> Self {
        Self { r, g, b }
    }

    pub const fn gray(c: F) -> Self {
        Self::new(c, c, c)
    }

    pub const fn black() -> Self {
        Self::gray(F::ZERO)
    }

    pub const fn white() -> Self {
        Self::gray(F::ONE)
    }

    pub fn clamped(self) -> Self {
        let r = self.r.clamp(F::ZERO, F::ONE);
        let g = self.g.clamp(F::ZERO, F::ONE);
        let b = self.b.clamp(F::ZERO, F::ONE);
        Self { r, g, b }
    }

    pub fn mixed(input: &[Color<F>]) -> Self {
        match input.len() {
            0 => Self::zero(),
            n => input.iter().copied().sum::<Self>() / F::from_usize(n),
        }
    }

    pub fn to_array(&self) -> [u8; 3] {
        let clamped = self.clamped();
        let max = F::from_u32(u8::MAX as u32);
        [
            <u8 as num::traits::NumCast>::from((clamped.r * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.g * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.b * max).round()).unwrap_or(u8::MAX),
        ]
    }

    pub fn to_array4(&self) -> [u8; 4] {
        let clamped = self.clamped();
        let max = F::from_u32(u8::MAX as u32);
        [
            <u8 as num::traits::NumCast>::from((clamped.r * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.g * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.b * max).round()).unwrap_or(u8::MAX),
            255,
        ]
    }
}

impl<F: Float> Zero for Color<F> {
    fn zero() -> Self {
        Self::black()
    }

    fn is_zero(&self) -> bool {
        self.r.is_zero() && self.g.is_zero() && self.b.is_zero()
    }
}

impl<F: Float> Sum for Color<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Color::zero(), |a, ref c| a + *c)
    }
}

impl<F: Float> VectorSpace for Color<F> {
    type Scalar = F;
}

impl<F: Float> Lerp for Color<F> {
    type Ratio = F;
}

impl<F: Float> AsRef<Color<F>> for Color<F> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<F: Float> From<[f32; 3]> for Color<F> {
    fn from(val: [f32; 3]) -> Self {
        Self::new(
            F::from_f32(val[0]),
            F::from_f32(val[1]),
            F::from_f32(val[2]),
        )
    }
}

impl<F: Float> From<Color<F>> for [f32; 3] {
    fn from(color: Color<F>) -> Self {
        [
            color.r.to_f32().unwrap(),
            color.g.to_f32().unwrap(),
            color.b.to_f32().unwrap(),
        ]
    }
}
