use std::fmt::{Debug, Display};
use std::ops::{Div, DivAssign, Mul, MulAssign, Rem, RemAssign};
use std::str::FromStr;

use cgmath::{AbsDiffEq, RelativeEq, UlpsEq};
use derive_more::{Add, AddAssign, Neg, Sub, SubAssign};
use num::{Num, NumCast, One, Signed, ToPrimitive, Zero};
use num_traits::{ConstOne, ConstZero, FloatConst, Pow};

use crate::types::{Float, Lerp};

#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign, PartialEq, Eq, PartialOrd, Neg)]
pub struct FP<const P: u8>(i64);

#[cfg(feature = "gui")]
use egui::emath::Numeric;

#[cfg(feature = "gui")]
impl<const P: u8> Numeric for FP<P> {
    const INTEGRAL: bool = false;

    const MIN: Self = Self(i64::MIN);

    const MAX: Self = Self(i64::MAX);

    fn to_f64(self) -> f64 {
        self.into_f64()
    }

    fn from_f64(num: f64) -> Self {
        num.into()
    }
}

impl<const P: u8> FP<P> {
    #[must_use]
    pub const fn int(&self) -> i64 {
        self.0 >> P
    }

    const SCALING: i64 = (1i64 << P);
    const MASK: i64 = Self::SCALING - 1;

    #[must_use]
    pub fn into_f32(&self) -> f32 {
        self.into_f64() as f32
    }

    #[must_use]
    pub fn into_f64(&self) -> f64 {
        (self.0 as f64) / (Self::SCALING as f64)
    }
}

impl<const P: u8> ToPrimitive for FP<P> {
    fn to_i64(&self) -> Option<i64> {
        Some(self.int())
    }

    fn to_u64(&self) -> Option<u64> {
        if self.0 >= 0 {
            Some((self.0 >> P) as u64)
        } else {
            None
        }
    }
}

impl<const P: u8> Default for FP<P> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const P: u8> Display for FP<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_f32())
    }
}

impl<const P: u8> Debug for FP<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.into_f32())
    }
}

impl<const P: u8> RemAssign for FP<P> {
    fn rem_assign(&mut self, _rhs: Self) {
        todo!()
    }
}

impl<const P: u8> Num for FP<P> {
    type FromStrRadixErr = ();

    fn from_str_radix(_str: &str, _radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        todo!()
    }
}

impl<const P: u8> Signed for FP<P> {
    fn abs(&self) -> Self {
        Self(self.0.abs())
    }

    fn abs_sub(&self, _other: &Self) -> Self {
        todo!()
    }

    fn signum(&self) -> Self {
        todo!()
    }

    fn is_positive(&self) -> bool {
        todo!()
    }

    fn is_negative(&self) -> bool {
        todo!()
    }
}

impl<const P: u8> Lerp for FP<P> {
    type Ratio = Self;
}

impl<const P: u8> DivAssign for FP<P> {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<const P: u8> MulAssign for FP<P> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<const P: u8> Rem for FP<P> {
    type Output = Self;

    fn rem(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl<const P: u8> Div for FP<P> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        (self.into_f64() / rhs.into_f64()).into()
    }
}

impl<const P: u8> From<i64> for FP<P> {
    fn from(value: i64) -> Self {
        Self(value << P)
    }
}

impl<const P: u8> From<i32> for FP<P> {
    fn from(value: i32) -> Self {
        Into::<i64>::into(value).into()
    }
}

impl<const P: u8> From<u32> for FP<P> {
    fn from(value: u32) -> Self {
        Into::<i64>::into(value).into()
    }
}

impl<const P: u8> From<usize> for FP<P> {
    fn from(value: usize) -> Self {
        (value as i64).into()
    }
}

impl<const P: u8> From<f32> for FP<P> {
    fn from(value: f32) -> Self {
        /* FIXME: f32 */
        Self((Into::<f64>::into(value) * Self::SCALING as f64) as i64)
    }
}

impl<const P: u8> From<f64> for FP<P> {
    fn from(value: f64) -> Self {
        Self((value * Self::SCALING as f64) as i64)
    }
}

impl<const P: u8> One for FP<P> {
    fn one() -> Self {
        1.into()
    }
}

impl<const P: u8> Mul for FP<P> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.into_f64() * rhs.into_f64()).into()
    }
}

impl<const P: u8> ConstZero for FP<P> {
    const ZERO: Self = Self(0);
}

impl<const P: u8> ConstOne for FP<P> {
    const ONE: Self = Self(1i64 << P);
}

impl<const P: u8> Zero for FP<P> {
    fn zero() -> Self {
        Self(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl<const P: u8> Pow<Self> for FP<P> {
    type Output = Self;

    fn pow(self, rhs: Self) -> Self::Output {
        (self.into_f64().pow(rhs.into_f64())).into()
    }
}

impl<const P: u8> NumCast for FP<P> {
    fn from<T: num::ToPrimitive>(n: T) -> Option<Self> {
        n.to_f64().map(Self::from_f64)
    }
}

impl<const P: u8> FromStr for FP<P> {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl<const P: u8> FloatConst for FP<P> {
    #[doc = "Return Euler’s number."]
    fn E() -> Self {
        todo!()
    }

    #[doc = "Return `1.0 / π`."]
    fn FRAC_1_PI() -> Self {
        todo!()
    }

    #[doc = "Return `1.0 / sqrt(2.0)`."]
    fn FRAC_1_SQRT_2() -> Self {
        todo!()
    }

    #[doc = "Return `2.0 / π`."]
    fn FRAC_2_PI() -> Self {
        todo!()
    }

    #[doc = "Return `2.0 / sqrt(π)`."]
    fn FRAC_2_SQRT_PI() -> Self {
        todo!()
    }

    #[doc = "Return `π / 2.0`."]
    fn FRAC_PI_2() -> Self {
        todo!()
    }

    #[doc = "Return `π / 3.0`."]
    fn FRAC_PI_3() -> Self {
        todo!()
    }

    #[doc = "Return `π / 4.0`."]
    fn FRAC_PI_4() -> Self {
        todo!()
    }

    #[doc = "Return `π / 6.0`."]
    fn FRAC_PI_6() -> Self {
        todo!()
    }

    #[doc = "Return `π / 8.0`."]
    fn FRAC_PI_8() -> Self {
        todo!()
    }

    #[doc = "Return `ln(10.0)`."]
    fn LN_10() -> Self {
        todo!()
    }

    #[doc = "Return `ln(2.0)`."]
    fn LN_2() -> Self {
        todo!()
    }

    #[doc = "Return `log10(e)`."]
    fn LOG10_E() -> Self {
        todo!()
    }

    #[doc = "Return `log2(e)`."]
    fn LOG2_E() -> Self {
        todo!()
    }

    #[doc = "Return Archimedes’ constant `π`."]
    fn PI() -> Self {
        core::f64::consts::PI.into()
    }

    #[doc = "Return `sqrt(2.0)`."]
    fn SQRT_2() -> Self {
        todo!()
    }
}

impl<const P: u8> num_traits::Float for FP<P> {
    fn nan() -> Self {
        todo!()
    }

    fn infinity() -> Self {
        todo!()
    }

    fn neg_infinity() -> Self {
        todo!()
    }

    fn neg_zero() -> Self {
        todo!()
    }

    fn min_value() -> Self {
        todo!()
    }

    fn min_positive_value() -> Self {
        todo!()
    }

    fn max_value() -> Self {
        Self(i64::MAX)
    }

    fn is_nan(self) -> bool {
        todo!()
    }

    fn is_infinite(self) -> bool {
        todo!()
    }

    fn is_finite(self) -> bool {
        todo!()
    }

    fn is_normal(self) -> bool {
        todo!()
    }

    fn classify(self) -> std::num::FpCategory {
        todo!()
    }

    fn floor(self) -> Self {
        todo!()
    }

    fn ceil(self) -> Self {
        todo!()
    }

    fn round(self) -> Self {
        self.into_f64().round().into()
        /* if self.frac().0 >= 0x8000_0000 { */
        /*     Self(self.int() << 32) */
        /* } else { */
        /*     Self((self.int() + 1) << 32) */
        /* } */
    }

    fn trunc(self) -> Self {
        Self(self.0 & !Self::MASK)
    }

    fn fract(self) -> Self {
        Self(self.0 & Self::MASK)
    }

    fn abs(self) -> Self {
        Self(self.0.abs())
    }

    fn signum(self) -> Self {
        todo!()
    }

    fn is_sign_positive(self) -> bool {
        todo!()
    }

    fn is_sign_negative(self) -> bool {
        todo!()
    }

    fn mul_add(self, _a: Self, _b: Self) -> Self {
        todo!()
    }

    fn recip(self) -> Self {
        todo!()
    }

    fn powi(self, _n: i32) -> Self {
        todo!()
    }

    fn powf(self, _n: Self) -> Self {
        todo!()
    }

    fn sqrt(self) -> Self {
        self.into_f64().sqrt().into()
    }

    fn exp(self) -> Self {
        todo!()
    }

    fn exp2(self) -> Self {
        todo!()
    }

    fn ln(self) -> Self {
        todo!()
    }

    fn log(self, _base: Self) -> Self {
        todo!()
    }

    fn log2(self) -> Self {
        todo!()
    }

    fn log10(self) -> Self {
        todo!()
    }

    fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    fn abs_sub(self, _other: Self) -> Self {
        todo!()
    }

    fn cbrt(self) -> Self {
        todo!()
    }

    fn hypot(self, _other: Self) -> Self {
        todo!()
    }

    fn sin(self) -> Self {
        todo!()
    }

    fn cos(self) -> Self {
        todo!()
    }

    fn tan(self) -> Self {
        self.into_f64().tan().into()
    }

    fn asin(self) -> Self {
        todo!()
    }

    fn acos(self) -> Self {
        self.into_f64().acos().into()
    }

    fn atan(self) -> Self {
        self.into_f64().atan().into()
    }

    fn atan2(self, other: Self) -> Self {
        self.into_f64().atan2(other.into_f64()).into()
    }

    fn sin_cos(self) -> (Self, Self) {
        let (sin, cos) = self.into_f64().sin_cos();
        (sin.into(), cos.into())
    }

    fn exp_m1(self) -> Self {
        todo!()
    }

    fn ln_1p(self) -> Self {
        todo!()
    }

    fn sinh(self) -> Self {
        todo!()
    }

    fn cosh(self) -> Self {
        todo!()
    }

    fn tanh(self) -> Self {
        todo!()
    }

    fn asinh(self) -> Self {
        todo!()
    }

    fn acosh(self) -> Self {
        todo!()
    }

    fn atanh(self) -> Self {
        todo!()
    }

    fn integer_decode(self) -> (u64, i16, i8) {
        todo!()
    }
}

impl<const P: u8> UlpsEq for FP<P> {
    fn default_max_ulps() -> u32 {
        todo!()
    }

    fn ulps_eq(&self, _other: &Self, _epsilon: Self::Epsilon, _max_ulps: u32) -> bool {
        todo!()
    }
}

impl<const P: u8> AbsDiffEq for FP<P> {
    type Epsilon = Self;

    fn default_epsilon() -> Self::Epsilon {
        todo!()
    }

    fn abs_diff_eq(&self, _other: &Self, _epsilon: Self::Epsilon) -> bool {
        todo!()
    }
}

impl<const P: u8> RelativeEq for FP<P> {
    fn default_max_relative() -> Self::Epsilon {
        todo!()
    }

    fn relative_eq(
        &self,
        _other: &Self,
        _epsilon: Self::Epsilon,
        _max_relative: Self::Epsilon,
    ) -> bool {
        todo!()
    }
}

impl<const P: u8> Float for FP<P> {
    const BIAS: Self = Self(Self::ONE.0 / 1_000_000);
    const BIAS2: Self = Self(Self::ONE.0 / 100_000);
    const BIAS3: Self = Self(Self::ONE.0 / 10_000);
    const BIAS4: Self = Self(Self::ONE.0 / 1_000);

    /* const BIAS: Self = Self(1); */
    /* const BIAS2: Self = Self(0x10); */
    /* const BIAS3: Self = Self(0x20); */
    /* const BIAS4: Self = Self(0x100); */

    /* const BIAS: Self = Self(1); */
    /* const BIAS2: Self = Self(0x10); */
    /* const BIAS3: Self = Self(0x100); */
    /* const BIAS4: Self = Self(0x1000); */

    const HALF: Self = Self(1i64 << (P - 1));

    const TWO: Self = Self(1i64 << (P + 1));

    const FOUR: Self = Self(1i64 << (P + 2));

    fn from_i32(value: i32) -> Self {
        value.into()
    }

    fn from_u32(value: u32) -> Self {
        value.into()
    }

    fn from_usize(value: usize) -> Self {
        value.into()
    }

    fn from_f32(value: f32) -> Self {
        value.into()
    }

    #[cfg(not(feature = "gui"))]
    fn from_f64(value: f64) -> Self {
        value.into()
    }

    #[cfg(not(feature = "gui"))]
    fn to_f64(self) -> f64 {
        self.into_f64()
    }
}

#[cfg(test)]
mod test {
    use super::FP as FPG;
    use crate::types::Float;
    use num_traits::{ConstOne as _, ConstZero as _, Float as _, FloatConst as _, Pow as _};

    use assert_float_eq::{afe_is_f32_near, afe_near_error_msg, assert_f32_near};

    type FP = FPG<32>;

    macro_rules! assert_eq_5dec {
        ($a:expr, $b:expr) => {
            assert_eq!(($a * 10000.into()).round(), $b.into());
        };
    }

    #[test]
    fn mask() {
        type FP8 = FPG<8>;
        assert_eq!(FP8::MASK, 255);
        assert_eq!(FP8::SCALING, 0x100);
    }

    #[test]
    fn consts() {
        assert_eq!(FP::ZERO.0, 0);
        assert_eq!(FP::ZERO.into_f32(), 0f32);
        assert_eq!(FP::ZERO.into_f64(), 0f64);

        assert_eq!(FP::ONE.into_f32(), 1f32);
        assert_eq!(FP::ONE.into_f64(), 1f64);

        assert_eq!(FP::TWO.into_f32(), 2f32);
        assert_eq!(FP::TWO.into_f64(), 2f64);

        assert_eq!(FP::FOUR.into_f32(), 4f32);
        assert_eq!(FP::FOUR.into_f64(), 4f64);
    }

    #[test]
    fn zero() {
        let zero = FP::ZERO;
        assert_eq!(zero.0, 0);
    }

    #[test]
    fn add() {
        assert_eq!(FP::HALF + FP::HALF, FP::ONE);
        assert_eq!(FP::ONE + FP::ONE, FP::TWO);
        assert_eq!(FP::TWO + FP::TWO, FP::FOUR);
    }

    #[test]
    fn int() {
        assert_eq!(FP::ZERO.int(), 0);
        assert_eq!(FP::ONE.int(), 1);
        assert_eq!(FP::TWO.int(), 2);
        assert_eq!(FP::FOUR.int(), 4);
        assert_eq!(FP::from_u32(1337).int(), 1337);
    }

    #[test]
    fn fract() {
        assert_eq!(FP::ZERO.fract(), FP::ZERO);
        assert_eq!(FP::ONE.fract(), FP::ZERO);
        assert_eq!(FP::TWO.fract(), FP::ZERO);
        assert_eq!(FP::ONE / FP::TWO, FP::HALF);
        assert_eq!((FP::from_u32(9) / FP::FOUR).fract(), FP::HALF / FP::TWO);
    }

    #[test]
    fn abs() {
        let one = FP::ONE;
        assert_eq!(one.abs(), FP::ONE);

        let negone = -FP::ONE;
        assert_eq!(negone.abs(), FP::ONE);

        let negtwo = -FP::TWO;
        assert_eq!(negtwo.abs(), FP::TWO);
    }

    #[test]
    fn mul() {
        assert_eq!(FP::TWO * FP::ZERO, FP::ZERO);
        assert_eq!(FP::TWO * FP::ONE, FP::TWO);
        assert_eq!(FP::TWO * FP::TWO, FP::FOUR);

        let mut num = FP::TWO;
        num *= FP::TWO;
        assert_eq!(num, FP::FOUR);
    }

    #[test]
    fn div() {
        assert_eq!(FP::TWO / FP::ONE, FP::TWO);
        assert_eq!(FP::TWO / FP::TWO, FP::ONE);
        assert_eq!(FP::FOUR / FP::TWO, FP::TWO);

        let mut num = FP::FOUR;
        num /= FP::TWO;
        assert_eq!(num, FP::TWO);
    }

    /* #[test] */
    /* #[should_panic] */
    /* fn div_0() { */
    /*     dbg!(FP::ONE / FP::ZERO); */
    /*     let _ = FP::ONE / FP::ZERO; */
    /* } */

    #[test]
    fn pi() {
        let pi = FP::PI();
        assert_eq_5dec!(pi, 3_1416);
    }

    #[test]
    fn ord() {
        assert!(FP::ZERO < FP::HALF);
        assert!(FP::HALF < FP::ONE);
        assert!(FP::ONE < FP::TWO);
        assert!(FP::TWO < FP::FOUR);

        assert!(FP::BIAS <= FP::BIAS2);
        assert!(FP::BIAS2 <= FP::BIAS3);
        assert!(FP::BIAS3 <= FP::BIAS4);
    }

    #[test]
    fn trunc() {
        let three = FP::ONE + FP::ONE + FP::ONE;
        assert_eq!(FP::from_u32(3), three);

        let one_half = three / FP::TWO;
        assert_eq!(FP::from_f32(1.5), one_half);

        let one = one_half.trunc();
        assert_eq!(one, FP::ONE);
    }

    #[test]
    fn neg() {
        assert_eq!((-FP::ZERO).int(), 0);
        assert_eq!((-FP::ONE).int(), -1);
        assert_eq!((--FP::ONE).int(), 1);
    }

    #[test]
    fn pow() {
        assert_eq!(FP::ZERO.pow(FP::ONE), FP::ZERO);
        assert_eq!(FP::ONE.pow(FP::ONE), FP::ONE);
        assert_eq!(FP::TWO.pow(FP::TWO), FP::FOUR);
        assert_eq!(FP::FOUR.pow(FP::TWO).int(), 16);
    }

    #[test]
    fn sqrt() {
        let four = FP::FOUR;
        assert_eq!(four.sqrt(), FP::TWO);
        let four = FP::ZERO;
        assert_eq!(four.sqrt(), FP::ZERO);
    }

    #[test]
    fn min() {
        assert_eq!(FP::ONE.min(FP::TWO), FP::ONE);
        assert_eq!(FP::TWO.min(FP::ONE), FP::ONE);
    }

    #[test]
    fn max() {
        assert_eq!(FP::ONE.max(FP::TWO), FP::TWO);
        assert_eq!(FP::TWO.max(FP::ONE), FP::TWO);
    }

    #[test]
    fn sincos() {
        let pi = FP::PI();
        let four = FP::FOUR;
        let six = FP::from_f32(6.0);

        let (sin, cos) = (pi / four).sin_cos();
        assert_f32_near!(sin.into_f32(), cos.into_f32());
        assert_eq_5dec!(sin, 0_7071);

        let sin = (pi / six).sin_cos().0;
        assert_eq_5dec!(sin, 0_5000);
    }

    #[test]
    fn tan() {
        let pi = FP::PI();
        let four = FP::FOUR;
        let eight = FP::FOUR * FP::TWO;

        let tan = (pi / four).tan();
        assert_eq_5dec!(tan, 1_0000);

        let tan = (pi / eight).tan();
        assert_eq_5dec!(tan, 0_4142);
    }

    #[test]
    fn atan() {
        let pi = FP::PI();
        let four = FP::FOUR;

        assert_f32_near!(FP::ZERO.atan().into_f32(), 0f32);

        assert_f32_near!(FP::ONE.atan().into_f32(), (pi / four).into_f32());
    }

    #[test]
    fn atan2() {
        /* let pi = FP::PI(); */
        /* let four = FP::FOUR; */

        /* let atan = FP::ONE.atan(); */
        /* assert_f32_near!(atan.into_f32(), (pi / four).into_f32()); */
    }
}
