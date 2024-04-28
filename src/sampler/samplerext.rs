use crate::sampler::{Adjust, Bilinear, Nearest, Sampler, Texel};
use crate::types::Float;

pub trait SamplerExt<T: Texel> {
    fn bilinear(self) -> Bilinear<T, Self>
    where
        Self: Sampler<u32, T> + Sized,
    {
        Bilinear::new(self)
    }

    fn nearest(self) -> Nearest<T, Self>
    where
        Self: Sampler<u32, T> + Sized,
    {
        Nearest::new(self)
    }

    fn scale<F>(self, scale: F, offset: F) -> Adjust<F, T, Self>
    where
        F: Float,
        T: Texel<Ratio = F>,
        Self: Sampler<F, T> + Sized,
    {
        Adjust::new(scale, offset, self)
    }
}

impl<T: Texel, S: Sampler<u32, T>> SamplerExt<T> for S {}
