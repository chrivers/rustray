use super::samp_util::*;

use num_traits::ToPrimitive;
use cgmath::InnerSpace;

pub struct HeightNormal<F: Float, S: Sampler<F, F>>
{
    delta: F,
    sampler: S,
}

impl<F: Float, S: Sampler<F, F>> HeightNormal<F, S>
{
    pub fn new(delta: F, sampler: S) -> Self
    {
        Self { delta, sampler }
    }
}

impl<F: Float, S: Sampler<F, F>> Sampler<F, Vector<F>> for HeightNormal<F, S>
{
    fn sample(&self, uv: Point<F>) -> Vector<F>
    {
        let d = self.delta;
        let a = self.sampler.sample(point!(uv.x - d, uv.y    ));
        let b = self.sampler.sample(point!(uv.x + d, uv.y    ));
        let c = self.sampler.sample(point!(uv.x,     uv.y - d));
        let d = self.sampler.sample(point!(uv.x,     uv.y + d));
        let n1 = (a - b).clamp(-F::ONE, F::ONE) / F::TWO + F::HALF;
        let n2 = (c - d).clamp(-F::ONE, F::ONE) / F::TWO + F::HALF;
        Vector::new(
            n1,
            n2,
            F::ONE,
        ).normalize()
    }

    fn raw_sample(&self, uv: Point<u32>) -> Vector<F>
    {
        Vector::new(F::ZERO, F::ZERO, self.sampler.raw_sample(uv))
    }

    fn dimensions(&self) -> (u32, u32)
    {
        self.sampler.dimensions()
    }
}
