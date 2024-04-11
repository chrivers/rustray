use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Phong<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
{
    pow: S1,
    ks: S2,
    kd: S3,
    _p: PhantomData<F>,
}

impl<F, S1, S2, S3> Phong<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    #[must_use]
    pub const fn new(pow: S1, ks: S2, kd: S3) -> Self {
        Self {
            pow,
            ks,
            kd,
            _p: PhantomData,
        }
    }
}

impl<F: Float + Texel> Phong<F, F, Color<F>, Color<F>> {
    #[must_use]
    pub fn white() -> Self {
        Self::new(F::from_u32(8), Color::WHITE, Color::WHITE)
    }

    #[must_use]
    pub fn black() -> Self {
        Self::new(F::from_u32(8), Color::WHITE, Color::BLACK)
    }
}

impl<F, S1, S2, S3> Material<F> for Phong<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let mut res = Color::BLACK;
        let uv = maxel.uv();

        let diff_color = self.kd.sample(uv);
        let spec_color = self.ks.sample(uv);
        let spec_pow = self.pow.sample(uv);

        for light in rt.get_lights() {
            let lixel = light.contribution(maxel, rt);

            let lambert = maxel.nml().dot(lixel.dir);

            if lambert < F::BIAS {
                continue;
            }

            res += (lixel.color * diff_color) * lambert;

            if spec_color.is_zero() {
                continue;
            }

            let refl_dir = lixel.dir.reflect(&maxel.nml());
            let spec_angle = refl_dir.dot(maxel.dir).max(F::ZERO);
            let specular = spec_angle.pow(spec_pow);

            res += lixel.color * spec_color * specular;
        }
        res
    }
}

#[cfg(feature = "gui")]
impl<F, S1, S2, S3> Interactive<F> for Phong<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.strong("Phong");
        ui.end_row();

        let mut res = false;
        res |= self.pow.ui(ui, "Power");
        res |= self.ks.ui(ui, "Specular");
        res |= self.kd.ui(ui, "Diffuse");
        res
    }
}

impl<F, S1, S2, S3> SceneObject<F> for Phong<F, S1, S2, S3>
where
    F: Float + Texel,
    S1: Sampler<F, F>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
{
    sceneobject_impl_body!("Phong");
}
