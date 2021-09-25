use super::mat_util::*;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Matte<F: Float, M: Material, S: Sampler<F, F>>
{
    rays: u32, // Number of rays to average over
    src: S,    // Surface Roughness Coefficient
    mat: M,    // Underlying material
    _p: PhantomData<F>
}

impl<F: Float, M: Material, S: Sampler<F, F>> Matte<F, M, S>
{
    pub fn new(src: S, rays: u32, mat: M) -> Self
    {
        Self { src, rays, mat, _p: PhantomData {} }
    }

}

impl<F: Float, M: Material<F=F>, S: Sampler<F, F>> Material for Matte<F, M, S>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &[Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
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

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &Light<F>, rt: &dyn RayTracer<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light, rt)
    }
}
