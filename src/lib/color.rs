use derive_more::{Add, AddAssign, Sub};
use std::ops;

use crate::lib::float::{Float, Blended};

#[derive(Clone, Copy, Debug)]
#[derive(Add, AddAssign, Sub)]
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
    pub fn new(r: F, g: F, b: F) -> Color<F>
    {
        Color { r, g, b }
    }

    pub fn gray(c: F) -> Color<F>
    {
        Color { r: c, g: c, b: c }
    }

    pub fn black() -> Color<F>
    {
        let zero = F::zero();
        Color { r: zero, g: zero, b: zero }
    }

    pub fn white() -> Color<F>
    {
        let one = F::one();
        Color { r: one, g: one, b: one }
    }

    fn clamp(n: F) -> F
    {
        match n
        {
            n if n < F::zero() => F::zero(),
            n if n > F::one() => F::one(),
            n => n,
        }
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
            0 => Color::black(),
            n => {
                let sum = input.iter().fold(Color::black(), |a, &c| a + c);
                sum / F::from_usize(n)
            }
        }
    }

    pub fn to_array(&self) -> [u8; 3]
    {
        let clamped = self.clamped();
        let max = F::from_f32(255.0);
        [
            <u8 as num::traits::NumCast>::from((clamped.r * max).round()).unwrap_or(255),
            <u8 as num::traits::NumCast>::from((clamped.g * max).round()).unwrap_or(255),
            <u8 as num::traits::NumCast>::from((clamped.b * max).round()).unwrap_or(255),
        ]
    }
}

impl<F: Float> Blended<F> for Color<F>
{
    fn blended(&self, other: &Color<F>, pct: F) -> Color<F>
    {
        (*self * (F::one() - pct)) + (*other * pct)
    }
}