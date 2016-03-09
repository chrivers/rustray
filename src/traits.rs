use num;
use std::fmt::Debug;
use std::fmt::Display;

pub trait Float : num::Float + Debug + Display
{
    fn epsilon() -> Self;
    fn small_value() -> Self;
}

impl Float for f32
{
    #[inline(always)]
    fn epsilon() -> Self { 1e-10 }

    #[inline(always)]
    fn small_value() -> Self { 1e-4 }
}

impl Float for f64
{
    #[inline(always)]
    fn epsilon() -> Self { 1e-10 }

    #[inline(always)]
    fn small_value() -> Self { 1e-10 }
}
