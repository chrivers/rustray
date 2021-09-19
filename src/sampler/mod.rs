use crate::math::{Vector, Float, Point, Color};
use crate::math::ray::{Ray, Hit, Maxel};

/** Trait for sampling values from datasource (textures, etc)
 */
pub trait Sampler<F: Float, S> : Sync
{
    /** Sample a single value at position `uv` */
    fn sample(&self, uv: Point<F>) -> S;

    /** Return (`width`, `height`) dimensions of sampler */
    fn dimensions(&self) -> (usize, usize);
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

    fn dimensions(&self) -> (usize, usize)
    {
        (1, 1)
    }
}

pub mod texture1;

pub use texture1::Texture1;
