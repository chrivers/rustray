use super::mat_util::*;
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub struct Matte<F: Float + Texel, S: Sampler<F, F>, M: Material<F=F>>
{
    rays: u32, /* Number of rays to average over */
    src: S,    /* Surface Roughness Coefficient */
    mat: M,    /* Underlying material */
}

impl<F, S, M> Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F=F>,
{
    pub fn new(src: S, rays: u32, mat: M) -> Self
    {
        Self { src, rays, mat }
    }

}

impl<F, S, M> Material for Matte<F, S, M>
where
    F: Float + Texel,
    S: Sampler<F, F>,
    M: Material<F=F>,
    rand::distributions::Standard: rand::distributions::Distribution<F>
{
    type F = F;

    fn render(&self, maxel: &mut Maxel<F>, light: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let mut rng = rand::thread_rng();
        let mut col = Color::<F>::black();
        let mut mxl = *maxel;

        let uv = maxel.uv();
        let normal = mxl.nml();
        for _n in 0..self.rays {
            let src = self.src.sample(uv);
            let rx = (rng.gen::<F>() - F::HALF) * src;
            let ry = (rng.gen::<F>() - F::HALF) * src;
            let rz = (rng.gen::<F>() / F::TWO ) * (F::one() - src) + src;
            let (normalu, normalv) = normal.surface_tangents();
            mxl = mxl.with_normal((
                normal * rz +
                     normalu * rx +
                     normalv * ry)
                .normalize());

            col += self.mat.render(&mut mxl, light, rt);
        }
        col / F::from_u32(self.rays)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>
    {
        self.mat.shadow(maxel, light)
    }
}
