use derive_more::{Add, AddAssign, Sub, Rem};
use std::ops;
use cgmath::VectorSpace;
use std::iter::Sum;
use num_traits::Zero;

use crate::lib::float::{Float, Lerp};

#[derive(Clone, Copy, Debug)]
#[derive(Add, AddAssign, Sub, Rem)]
pub struct Color<F: Float>
{
    pub r: F,
    pub g: F,
    pub b: F,
}

impl<F: Float> ops::Mul<F> for Color<F>
{
    type Output = Color<F>;

    fn mul(self, other: F) -> Color<F>
    {
        Color
        {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}

impl<F: Float> ops::Mul<Color<F>> for Color<F>
{
    type Output = Color<F>;

    fn mul(self, other: Color<F>) -> Color<F>
    {
        Color
        {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }
}

impl<F: Float> ops::Div for Color<F>
{
    type Output = Color<F>;

    fn div(self, other: Color<F>) -> Color<F>
    {
        Color
        {
            r: self.r / other.r,
            g: self.g / other.g,
            b: self.b / other.b,
        }
    }
}

impl<F: Float> ops::Div<F> for Color<F>
{
    type Output = Color<F>;

    fn div(self, other: F) -> Color<F>
    {
        Color
        {
            r: self.r / other,
            g: self.g / other,
            b: self.b / other,
        }
    }
}

impl<F: Float> Color<F>
{
    pub const fn new(r: F, g: F, b: F) -> Color<F>
    {
        Color { r, g, b }
    }

    pub const fn gray(c: F) -> Color<F>
    {
        Self::new(c, c, c)
    }

    pub const fn black() -> Color<F>
    {
        Self::gray(F::ZERO)
    }

    pub const fn white() -> Color<F>
    {
        Self::gray(F::ONE)
    }

    fn clamp(n: F) -> F
    {
        n.max(F::ZERO).min(F::ONE)
    }

    pub fn clamped(self) -> Color<F>
    {
        let r = Color::clamp(self.r);
        let g = Color::clamp(self.g);
        let b = Color::clamp(self.b);
        Color { r, g, b }
    }

    pub fn mixed(input: &[Color<F>]) -> Color<F>
    {
        match input.len() {
            0 => Color::zero(),
            n => input.iter().copied().sum::<Color<F>>() / F::from_usize(n)
        }
    }

    pub fn to_array(&self) -> [u8; 3]
    {
        let clamped = self.clamped();
        let max = F::from_u32(u8::MAX as u32);
        [
            <u8 as num::traits::NumCast>::from((clamped.r * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.g * max).round()).unwrap_or(u8::MAX),
            <u8 as num::traits::NumCast>::from((clamped.b * max).round()).unwrap_or(u8::MAX),
        ]
    }
}

impl<F: Float> Zero for Color<F>
{
    fn zero() -> Color<F>
    {
        Self::black()
    }

    fn is_zero(&self) -> bool
    {
        self.r.is_zero() &&
        self.g.is_zero() &&
        self.b.is_zero()
    }
}

impl<F: Float> Sum for Color<F> {
    fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(Color::zero(), |a, ref c| a + *c)
    }
}

impl<F: Float> VectorSpace for Color<F>
{
    type Scalar = F;
}

impl<F: Float> Lerp for Color<F> {
    type Ratio = F;
}
