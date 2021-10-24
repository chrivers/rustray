use super::samp_util::*;
use super::Bilinear;
use super::Nearest;

pub trait SamplerExt<T>
{
    fn bilinear(self) -> Bilinear<T, Self>
    where
        Self: Sampler<u32, T> + Sized,
        T: Texel
    {
        Bilinear::new(self)
    }

    fn nearest(self) -> Nearest<T, Self>
    where
        Self: Sampler<u32, T> + Sized,
        T: Texel
    {
        Nearest::new(self)
    }
}

impl<T, S> SamplerExt<T> for S
where
    T: Texel,
    S: Sampler<u32, T>
{
}
