use super::mat_util::*;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Matte<F: Float, S: Sampler<F, F>, M: Material<F=F>>
{
    rays: u32, /* Number of rays to average over */
    src: S,    /* Surface Roughness Coefficient */
    mat: M,    /* Underlying material */
}

impl<F: Float, S: Sampler<F, F>, M: Material<F=F>> Matte<F, S, M>
{
    pub fn new(src: S, rays: u32, mat: M) -> Self
    {
        Self { src, rays, mat }
    }

}

impl<F: Float, S: Sampler<F, F>, M: Material<F=F>> Material for Matte<F, S, M>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Box<dyn Light<F>>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut rng = rand::thread_rng();
        let mut col = Color::<F>::black();
        let mut mxl = *maxel;

        for _n in 0..self.rays {
            let src = self.src.sample(maxel.uv);
            let rx = (rng.gen::<F>() - F::HALF) * src;
            let ry = (rng.gen::<F>() - F::HALF) * src;
            let rz = (rng.gen::<F>() / F::TWO ) * (F::one() - src) + src;
            mxl.normal = (
                maxel.normal * rz +
                    maxel.normalu * rx +
                    maxel.normalv * ry)
                .normalize();

            col += self.mat.render(hit, &mxl, light, rt);
        }
        col / F::from_u32(self.rays)
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light, rt)
    }
}
