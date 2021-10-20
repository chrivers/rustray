use super::samp_util::*;

use num_traits::ToPrimitive;
use cgmath::InnerSpace;

#[derive(Copy, Clone, Debug)]
pub struct NormalMap<F: Float, S: Sampler<F, Color<F>>>
{
    sampler: S,
    _p: PhantomData<F>,
}

impl<F: Float, S: Sampler<F, Color<F>>> NormalMap<F, S>
{
    pub fn new(sampler: S) -> Self
    {
        Self { sampler, _p: PhantomData {} }
    }

    pub fn color_to_vector(col: &Color<F>) -> Vector<F>
    {
        let mut n = *col;
        n.r -= F::HALF;
        n.g -= F::HALF;
        n.r *= F::TWO;
        n.g *= F::TWO;
        Vector::new(n.r, n.g, n.b).normalize()
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> Sampler<F, Vector<F>> for NormalMap<F, S>
{
    fn sample(&self, uv: Point<F>) -> Vector<F>
    {
        Self::color_to_vector(&self.sampler.sample(uv))
    }

    fn raw_sample(&self, uv: Point<u32>) -> Vector<F>
    {
        Self::color_to_vector(&self.sampler.raw_sample(uv))
    }

    fn dimensions(&self) -> (u32, u32)
    {
        self.sampler.dimensions()
    }
}
