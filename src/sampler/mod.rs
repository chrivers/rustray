use crate::lib::{Float, Point};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F: Float, P> : Sync
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> P;

    /** Read a raw sample value at position `uv` */
    fn raw_sample(&self, uv: Point<u32>) -> P;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (u32, u32);
}

/**
Blanket implementation: [`Sync`] + [`Copy`] types can sample themselves, returning
self as their value.

This is useful to make e.g. a [`Float`] or [`Color<F>`] a viable substitute for a real
texture sampler.
*/
impl<F: Float, T: Sync + Copy> Sampler<F, T> for T
{
    fn sample(&self, uv: Point<F>) -> T
    {
        *self
    }

    fn raw_sample(&self, uv: Point<u32>) -> T
    {
        *self
    }

    fn dimensions(&self) -> (u32, u32)
    {
        (1, 1)
    }
}

pub(crate) mod samp_util {
    /* These are convenience re-imports for modules, so skip warnings */
    #![allow(unused_imports)]
    pub use crate::{vec3, point};
    pub use crate::lib::{Vector, Float, Point, Color};
    pub use super::Sampler;
}

pub mod texture1;
pub mod texture3;
pub mod bilinear;

pub use texture1::Texture1;
pub use texture3::Texture3;
pub use bilinear::Bilinear;
