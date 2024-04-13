use crate::sampler::{Bilinear, Nearest, Sampler, Texel};

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
}

impl<T: Texel, S: Sampler<u32, T>> SamplerExt<T> for S {}
