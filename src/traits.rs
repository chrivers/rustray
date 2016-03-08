use num;
use std::fmt::Debug;
use std::fmt::Display;

pub trait Float : num::Float + Debug + Display
{
    fn epsilon() -> Self;
}

impl Float for f32
{
    #[inline(always)]
    fn epsilon() -> Self { 1e-10 }
}

impl Float for f64
{
    #[inline(always)]
    fn epsilon() -> Self { 1e-10 }
}
