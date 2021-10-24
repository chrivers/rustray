use std::fmt::Debug;
use std::sync::Arc;

use num_traits::Num;

use crate::lib::{Float, Point};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F, T>
where
    Self: Debug + Send + Sync,
    F: Num,
    T: Texel
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> T;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (u32, u32);

    fn dynsampler<'a>(self) -> DynSampler<'a, F, T>
    where
        Self: Sized + 'a
    {
        Arc::new(Box::new(self) as Box<dyn Sampler<F, T> + 'a>)
    }
}

pub type DynSampler<'a, F, T> = Arc<Box<dyn Sampler<F, T> + 'a>>;

impl<F: Num, T: Texel> Sampler<F, T> for Arc<Box<dyn Sampler<F, T>>>
{
    fn sample(&self, uv: Point<F>) -> T
    {
        (*self.as_ref()).sample(uv)
    }

    fn dimensions(&self) -> (u32, u32)
    {
        (*self.as_ref()).dimensions()
    }
}

pub trait Texel: Debug + Send + Sync
{
}

impl Texel for f32 {}
impl Texel for f64 {}

/**
Blanket implementation: [`Sync`] + [`Copy`] types can sample themselves, returning
self as their value.

This is useful to make e.g. a [`Float`] or [`Color<F>`] a viable substitute for a real
texture sampler.
*/
impl<N: Num, T: Texel + Copy> Sampler<N, T> for T
{
    fn sample(&self, _uv: Point<N>) -> T
    {
        *self
    }

    fn dimensions(&self) -> (u32, u32)
    {
        (1, 1)
    }
}

pub(crate) mod samp_util {
    pub use std::marker::PhantomData;

    pub use crate::{vec3, point};
    pub use crate::lib::{Vector, Float, Point, Color};
    pub use crate::lib::float::Lerp;
    pub use super::{Sampler, Texel};

    pub use cgmath::{VectorSpace, InnerSpace};

    pub use num_traits::ToPrimitive;
    pub use num_traits::Num;
}

pub mod texture1;
pub mod texture3;
pub mod nearest;
pub mod bilinear;
pub mod transform;
pub mod perlin;
pub mod heightnormal;
pub mod normalmap;
pub mod shinemap;
pub mod samplerext;

pub use transform::Adjust;
pub use nearest::Nearest;
pub use bilinear::Bilinear;
pub use perlin::Perlin;
pub use heightnormal::HeightNormal;
pub use normalmap::NormalMap;
pub use shinemap::ShineMap;
pub use samplerext::SamplerExt;
