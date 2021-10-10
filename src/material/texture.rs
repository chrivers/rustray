use super::mat_util::*;

use std::marker::PhantomData;

pub trait TextureSampler<F: Float>
where
    Self: Sampler<F, Color<F>> + Sized
{
    fn texture(self) -> Texture<F, Self> {
        Texture::new(self)
    }
}

impl<F: Float, T: Sampler<F, Color<F>>> TextureSampler<F> for T
{

}

#[derive(Copy, Clone)]
pub struct Texture<F: Float, S: Sampler<F, Color<F>>>
{
    _f: PhantomData<F>,
    img: S
}

impl<F: Float, S: Sampler<F, Color<F>>> Texture<F, S>
{
    pub fn new(img: S) -> Self
    {
        Self { _f: PhantomData::<F> {}, img }
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> Material for Texture<F, S>
{
    type F = F;

    fn render(&self, _hit: &Hit<F>, maxel: &Maxel<F>, _lights: &[Box<dyn Light<F>>], _rt: &dyn RayTracer<F>) -> Color<F>
    {
        self.img.sample(maxel.uv)
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> AsRef<Texture<F, S>> for Texture<F, S>
{
    fn as_ref(&self) -> &Self {
        self
    }
}
