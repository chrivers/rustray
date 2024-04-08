use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Bumpmap<F, S1, S2, M>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F>,
{
    pow: S1,
    img: S2,
    mat: M,
    _p: PhantomData<F>,
}

impl<F, S1, S2, M> Bumpmap<F, S1, S2, M>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F>,
    Vector<F>: Texel,
{
    pub const fn new(pow: S1, img: S2, mat: M) -> Self {
        Self {
            pow,
            img,
            mat,
            _p: PhantomData,
        }
    }
}

impl<F, S1, S2, M> Material<F> for Bumpmap<F, S1, S2, M>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Vector<F>>,
    M: Material<F>,
    Vector<F>: Texel,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();
        let n = self.img.sample(uv);
        let pow = self.pow.sample(uv);

        let mut mxl = *maxel;

        let normal = mxl.nml();
        let (normalu, normalv) = normal.surface_tangents();
        let nx = normalu * n.x + normalv * n.y + normal * n.z / (pow + F::BIAS);

        mxl = mxl.with_normal(nx.normalize());

        self.mat.render(&mut mxl, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        self.mat.shadow(maxel, lixel)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        CollapsingHeader::new("Bumpmap")
            .default_open(true)
            .show(ui, |ui| {
                let mut res = false;
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        res |= Sampler::ui(&mut self.pow, ui, "Power");
                        res |= Sampler::ui(&mut self.img, ui, "Image");
                    });
                res |= self.mat.ui(ui);

                res
            })
            .body_returned
            .unwrap_or(false)
    }
}
