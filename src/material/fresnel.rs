use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Fresnel<F: Float + Texel, S: Sampler<F, F>>
{
    ior: S,
    _p: PhantomData<F>
}

impl<F: Float + Texel, S: Sampler<F, F>> Fresnel<F, S>
{
    pub fn new(ior: S) -> Self
    {
        Self { ior, _p: PhantomData {} }
    }

}

impl<F: Float + Texel, S: Sampler<F, F>> Material for Fresnel<F, S>
{
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, _light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let ior = self.ior.sample(maxel.uv());

        let refl = maxel.reflected_ray();
        let c_refl = rt.ray_trace(&refl).unwrap_or_else(Color::black);

        let refr = maxel.refracted_ray(ior);
        let c_refr = rt.ray_trace(&refr).unwrap_or_else(Color::black);

        let fr = maxel.dir.fresnel(&maxel.nml(), ior);

        c_refr.lerp(c_refl, fr)
    }
}
