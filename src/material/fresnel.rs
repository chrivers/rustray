use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Fresnel<F: Float, S: Sampler<F, F>>
{
    ior: S,
    _p: PhantomData<F>
}

impl<F: Float, S: Sampler<F, F>> Fresnel<F, S>
{
    pub fn new(ior: S) -> Self
    {
        Self { ior, _p: PhantomData {} }
    }

}

impl<F: Float, S: Sampler<F, F>> Material for Fresnel<F, S>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, _light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let ior = self.ior.sample(maxel.uv);

        let refl = hit.reflected_ray(&maxel.normal);
        let c_refl = rt.ray_trace(&refl).unwrap_or_else(Color::black);

        let refr = hit.refracted_ray(&maxel.normal, ior);
        let c_refr = rt.ray_trace(&refr).unwrap_or_else(Color::black);

        let fr = hit.dir.fresnel(&maxel.normal, ior);

        c_refr.lerp(c_refl, fr)
    }
}
