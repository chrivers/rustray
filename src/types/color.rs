use std::fmt::{self, Debug};
use std::iter::Sum;
use std::ops;

use cgmath::VectorSpace;
use derive_more::{Add, AddAssign, Rem, Sub};
use num::NumCast;
use num_traits::Zero;

use crate::sampler::Texel;
use crate::scene::Interactive;
use crate::types::float::{Float, Lerp};

#[derive(Clone, Copy, Add, AddAssign, Sub, Rem, PartialEq, Eq)]
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

#[cfg(feature = "gui")]
impl<F: Float> Interactive<F> for Color<F> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        crate::frontend::gui::controls::color(ui, self, "Color")
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

impl<F: Float> ops::Mul<Self> for Color<F> {
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
    pub const BLACK: Self = Self::gray(F::ZERO);
    pub const WHITE: Self = Self::gray(F::ONE);

    #[must_use]
    pub const fn new(r: F, g: F, b: F) -> Self {
        Self { r, g, b }
    }

    #[must_use]
    pub const fn gray(c: F) -> Self {
        Self::new(c, c, c)
    }

    #[must_use]
    pub const fn black() -> Self {
        Self::gray(F::ZERO)
    }

    #[must_use]
    pub const fn white() -> Self {
        Self::gray(F::ONE)
    }

    #[must_use]
    pub fn clamped(self) -> Self {
        let r = self.r.clamp(F::ZERO, F::ONE);
        let g = self.g.clamp(F::ZERO, F::ONE);
        let b = self.b.clamp(F::ZERO, F::ONE);
        Self { r, g, b }
    }

    pub fn mixed(input: &[Self]) -> Self {
        match input.len() {
            0 => Self::zero(),
            n => input.iter().copied().sum::<Self>() / F::from_usize(n),
        }
    }

    pub fn to_array(&self) -> [u8; 3] {
        let clamped = self.clamped();
        let max = F::from_u32(u8::MAX as u32);
        [
            <u8 as NumCast>::from((clamped.r * max).round()).unwrap_or(u8::MAX),
            <u8 as NumCast>::from((clamped.g * max).round()).unwrap_or(u8::MAX),
            <u8 as NumCast>::from((clamped.b * max).round()).unwrap_or(u8::MAX),
        ]
    }

    pub fn to_array4(&self) -> [u8; 4] {
        let clamped = self.clamped();
        let max = F::from_u32(u8::MAX as u32);
        [
            <u8 as NumCast>::from((clamped.r * max).round()).unwrap_or(u8::MAX),
            <u8 as NumCast>::from((clamped.g * max).round()).unwrap_or(u8::MAX),
            <u8 as NumCast>::from((clamped.b * max).round()).unwrap_or(u8::MAX),
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
        iter.fold(Self::zero(), |a, ref c| a + *c)
    }
}

impl<F: Float> VectorSpace for Color<F> {
    type Scalar = F;
}

impl<F: Float> Lerp for Color<F> {
    type Ratio = F;
}

impl<F: Float> AsRef<Self> for Color<F> {
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
            color.r.to_f32().unwrap_or_default(),
            color.g.to_f32().unwrap_or_default(),
            color.b.to_f32().unwrap_or_default(),
        ]
    }
}

#[cfg(feature = "gui")]
impl<F: Float> From<Color<F>> for egui::Color32 {
    fn from(color: Color<F>) -> Self {
        let rgb = color.to_array();
        Self::from_rgb(rgb[0], rgb[1], rgb[2])
    }
}
