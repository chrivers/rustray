use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Bumpmap<F: Float, S1: Sampler<F, F>, S2: Sampler<F, Vector<F>>, M: Material<F=F>>
{
    pow: S1,
    img: S2,
    mat: M,
}

impl<F: Float, S1: Sampler<F, F>, S2: Sampler<F, Vector<F>>, M: Material<F=F>> Bumpmap<F, S1, S2, M>
{
    pub fn new(pow: S1, img: S2, mat: M) -> Self
    {
        Self { pow, img, mat }
    }
}

impl<'a, F: Float, S1: Sampler<F, F>, S2: Sampler<F, Vector<F>>, M: Material<F=F>> Material for Bumpmap<F, S1, S2, M>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[&dyn Light<F>], rt: &dyn RayTracer<F>) -> Color<F>
    {
        let n = self.img.sample(maxel.uv);
        let pow = self.pow.sample(maxel.uv);

        let mut mxl = *maxel;

        let nx =
            mxl.normalu * n.x +
            mxl.normalv * n.y +
            mxl.normal  * n.z / (pow + F::BIAS);

        mxl.normal = nx.normalize();

        self.mat.render(hit, &mxl, lights, rt)
    }

    fn shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>
    {
        self.mat.shadow(hit, maxel, light)
    }
}
