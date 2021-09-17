use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Fresnel<F: Float>
{
    ior: F
}

impl<F: Float> Fresnel<F>
{
    pub fn new(ior: F) -> Self
    {
        Self { ior }
    }

}

impl<F: Float> Material for Fresnel<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let d = hit.dir.normalized();

        let refl = hit.dir.reflect(&maxel.normal);
        let c_refl = rt.ray_trace(&Ray::new(hit.pos + refl * F::BIAS, refl), lvl + 1).unwrap_or_else(Color::black);

        let refr = hit.dir.refract(&maxel.normal, self.ior);
        let c_refr = rt.ray_trace(&Ray::new(hit.pos + refr * F::BIAS, refr), lvl + 1).unwrap_or_else(Color::black);

        let fr = hit.dir.fresnel(&maxel.normal, self.ior);

        c_refr.blended(&c_refl, fr)
    }
}
