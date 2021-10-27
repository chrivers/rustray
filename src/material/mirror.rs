use super::mat_util::*;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct Mirror<F, S>
where
    F: Float + Texel,
    S: Sampler<F, F>,
{
    refl: S,
    _p: PhantomData<F>
}

impl<F: Float + Texel, S: Sampler<F, F>> Mirror<F, S>
{
    pub fn new(refl: S) -> Self
    {
        Self { refl, _p: PhantomData {} }
    }
}

impl<F: Float + Texel, S: Sampler<F, F>> Material for Mirror<F, S>
{
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, _light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let refl = maxel.reflected_ray();
        let c_refl = rt.ray_trace(&refl).unwrap_or_else(Color::black);
        c_refl * self.refl.sample(maxel.uv())
    }
}
