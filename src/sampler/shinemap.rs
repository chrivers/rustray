use super::samp_util::*;

#[derive(Copy, Clone, Debug)]
pub struct ShineMap<F: Float + Texel, S: Sampler<F, F>>
{
    sampler: S,
    scale: F,
}

impl<F: Float + Texel, S: Sampler<F, F>> ShineMap<F, S>
{
    pub fn new(sampler: S, scale: F) -> Self
    {
        Self { sampler, scale }
    }
}

impl<F: Float + Texel, S: Sampler<F, F>> Sampler<F, F> for ShineMap<F, S>
{
    fn sample(&self, uv: Point<F>) -> F
    {
        self.sampler.sample(uv) * self.scale
    }

    fn dimensions(&self) -> (u32, u32)
    {
        self.sampler.dimensions()
    }
}
