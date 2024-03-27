use std::fmt::Debug;
use std::sync::Arc;

use num_traits::Num;

use crate::types::{Point, Float, Color};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F, T>
where
    Self: Debug + Send + Sync,
    F: Num,
    T: Texel,
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> T;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (u32, u32);

    fn dynsampler(self) -> DynSampler<F, T>
    where
        Self: Sized + 'static,
    {
        Arc::new(Box::new(self))
    }
}

pub type DynSampler<F, T> = Arc<Box<dyn Sampler<F, T>>>;

impl<'a, F: Num, T: Texel> Sampler<F, T> for Arc<Box<dyn Sampler<F, T> + 'a>> {
    fn sample(&self, uv: Point<F>) -> T {
        self.as_ref().sample(uv)
    }

    fn dimensions(&self) -> (u32, u32) {
        self.as_ref().dimensions()
    }
}

pub trait Texel: Debug + Send + Sync {}

impl Texel for f32 {}
impl Texel for f64 {}

/**
Blanket implementation: [`Sync`] + [`Copy`] types can sample themselves, returning
self as their value.

This is useful to make e.g. a [`crate::Float`] or [`crate::Color<F>`] a viable substitute for a real
texture sampler.
*/
impl<F: Float + Texel> Sampler<F, F> for F
where
    Self: Debug
{
    fn sample(&self, _uv: Point<F>) -> F {
        *self
    }

    fn dimensions(&self) -> (u32, u32) {
        (1, 1)
    }
}

impl<F: Float> Sampler<F, Color<F>> for Color<F> {
    fn sample(&self, _uv: Point<F>) -> Color<F> {
        *self
    }

    fn dimensions(&self) -> (u32, u32) {
        (1, 1)
    }
}

pub(crate) mod samp_util {
    pub use std::marker::PhantomData;

    pub use super::{Sampler, Texel};
    pub use crate::point;
    pub use crate::types::float::Lerp;
    pub use crate::types::{Color, Float, Point, Vector};

    pub use cgmath::{InnerSpace, VectorSpace};

    pub use num_traits::ToPrimitive;
}

pub mod bilinear;
pub mod heightnormal;
pub mod nearest;
pub mod normalmap;
pub mod perlin;
pub mod samplerext;
pub mod shinemap;
pub mod texture1;
pub mod texture3;
pub mod transform;

pub use bilinear::Bilinear;
pub use heightnormal::HeightNormal;
pub use nearest::Nearest;
pub use normalmap::NormalMap;
pub use perlin::Perlin;
pub use samplerext::SamplerExt;
pub use shinemap::ShineMap;
pub use transform::Adjust;
