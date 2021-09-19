use super::mat_util::*;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Mirror<F: Float, S: Sampler<F, F>>
{
    refl: S,
    _p: PhantomData<F>
}

impl<F: Float, S: Sampler<F, F>> Mirror<F, S>
{
    pub fn new(refl: S) -> Self
    {
        Self { refl, _p: PhantomData {} }
    }
}

impl<F: Float, S: Sampler<F, F>> Material for Mirror<F, S>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let refl = hit.dir.reflect(&maxel.normal);
        let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);
        c_refl * self.refl.sample(maxel.uv)
    }
}
