use super::mat_util::*;

#[derive(Copy, Clone)]
pub struct Bumpmap<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>>
{
    pow: F,
    img: S,
    mat: M,
}

impl<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>> Bumpmap<F, S, M>
{
    pub fn new(pow: F, img: S, mat: M) -> Self
    {
        Self { pow, img, mat }
    }
}

impl<F: Float, S: Sampler<F, Color<F>>, M: Material<F=F>> Material for Bumpmap<F, S, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let mut n = self.img.sample(maxel.uv);
        n.r -= F::HALF;
        n.g -= F::HALF;
        n.r *= F::TWO;
        n.g *= F::TWO;

        let mut mxl = *maxel;

        let nx =
            mxl.normalu * n.r +
            mxl.normalv * n.g +
            mxl.normal  * n.b / (self.pow + F::BIAS);

        mxl.normal = nx.normalized();

        self.mat.render(hit, &mxl, lights, rt, lvl)
    }
}
