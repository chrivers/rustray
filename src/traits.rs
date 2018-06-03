use num;
use std::fmt::Debug;
use std::fmt::Display;

pub trait Float : num::Float + Debug + Display
{
    fn small_value() -> Self;
    fn from_int(value: u32) -> Self;
    fn from_float(value: f32) -> Self;
}

impl Float for f32
{
    #[inline(always)]
    fn small_value() -> Self { 1e-4 }

    #[inline(always)]
    fn from_int(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_float(value: f32) -> Self { value as Self }
}

impl Float for f64
{
    #[inline(always)]
    fn small_value() -> Self { 1e-10 }

    #[inline(always)]
    fn from_int(value: u32) -> Self { value as Self }

    #[inline(always)]
    fn from_float(value: f32) -> Self { value as Self }
}
