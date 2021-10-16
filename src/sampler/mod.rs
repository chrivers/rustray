use std::sync::Arc;

use crate::lib::{Float, Point};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F: Float, P> : Send + Sync
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> P;

    /** Read a raw sample value at position `uv` */
    fn raw_sample(&self, uv: Point<u32>) -> P;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (u32, u32);

    fn dynsampler<'a>(self) -> DynSampler<'a, F, P>
    where
        Self: Sized + 'a
    {
        Arc::new(Box::new(self) as Box<dyn Sampler<F, P> + 'a>)
    }
}

pub type DynSampler<'a, F, P> = Arc<Box<dyn Sampler<F, P> + 'a>>;

impl<F: Float, P> Sampler<F, P> for Arc<Box<dyn Sampler<F, P>>>
{
    fn sample(&self, uv: Point<F>) -> P
    {
        (**self.as_ref()).sample(uv)
    }

    fn raw_sample(&self, uv: Point<u32>) -> P
    {
        (**self.as_ref()).raw_sample(uv)
    }

    fn dimensions(&self) -> (u32, u32)
    {
        (**self.as_ref()).dimensions()
    }
}

pub trait Pixel: Send + Sync
{
}


/**
Blanket implementation: [`Sync`] + [`Copy`] types can sample themselves, returning
self as their value.

This is useful to make e.g. a [`Float`] or [`Color<F>`] a viable substitute for a real
texture sampler.
*/
impl<F: Float, T: Send + Sync + Copy> Sampler<F, T> for T
{
    fn sample(&self, _uv: Point<F>) -> T
    {
        *self
    }

    fn raw_sample(&self, _uv: Point<u32>) -> T
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
    pub use super::{Sampler, Pixel};

    pub use cgmath::VectorSpace;
}

pub mod texture1;
pub mod texture3;
pub mod bilinear;
pub mod transform;
pub mod perlin;
pub mod heightnormal;

pub use bilinear::{Bilinear, BilinearSampler};
pub use perlin::Perlin;
pub use heightnormal::HeightNormal;
