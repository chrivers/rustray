use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Bumpmap<
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F = F>,
> where
    Vector<F>: Texel,
{
    pow: S1,
    img: S2,
    mat: M,
}

impl<F, S1, S2, M> Bumpmap<F, S1, S2, M>
where
    F: Float + Texel + crate::types::float::Lerp<Ratio = F>,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F = F>,
    Vector<F>: Texel,
{
    pub fn new(pow: S1, img: S2, mat: M) -> Self {
        Self { pow, img, mat }
    }
}

impl<F: Float + Texel, S1: Sampler<F, F>, S2: Sampler<F, Vector<F>>, M: Material<F = F>> Material
    for Bumpmap<F, S1, S2, M>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F = F>,
    Vector<F>: Texel,
{
    type F = F;

    fn render(
        &self,
        maxel: &mut Maxel<F>,
        lights: &[&dyn Light<F>],
        rt: &dyn RayTracer<F>,
    ) -> Color<F> {
        let uv = maxel.uv();
        let n = self.img.sample(uv);
        let pow = self.pow.sample(uv);

        let mut mxl = *maxel;

        let normal = mxl.nml();
        let (normalu, normalv) = normal.surface_tangents();
        let nx = normalu * n.x + normalv * n.y + normal * n.z / (pow + F::BIAS);

        mxl = mxl.with_normal(nx.normalize());

        self.mat.render(&mut mxl, lights, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>> {
        self.mat.shadow(maxel, light)
    }
}
