use super::mat_util::*;

use std::marker::PhantomData;

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

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        self.img.sample(maxel.uv)
    }
}
