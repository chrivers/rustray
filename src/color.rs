#![allow(dead_code)]

use std::ops::{Add, Sub, Mul};
use num::Float;

#[derive(Clone, Copy)]
struct Color<F: Float>
{
    r: F,
    g: F,
    b: F,
}

impl<F: Float> Add for Color<F>
{
    type Output = Color<F>;

    fn add(self, other: Color<F>) -> Color<F>
    {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl<F: Float> Sub for Color<F>
{
    type Output = Color<F>;

    fn sub(self, other: Color<F>) -> Color<F>
    {
        Color {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl<F: Float> Mul<F> for Color<F>
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

impl<F: Float> Mul for Color<F>
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

impl<F: Float> Color<F>
{
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
        Color { r: r, g: r, b: r }
    }
}