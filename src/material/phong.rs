use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Phong<F, S1, S2, S3, S4>
where
    F: Float + Texel,
    S1: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, F>,
{
    ke: S1,
    kd: S2,
    ks: S3,
    pow: S4,
    ambient: Color<F>,
}

impl<F, S1, S2, S3, S4> Phong<F, S1, S2, S3, S4>
where
    F: Float + Texel,
    S1: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, F>,
{
    #[must_use]
    pub const fn new(ke: S1, kd: S2, ks: S3, pow: S4) -> Self {
        Self {
            ke,
            kd,
            ks,
            pow,
            ambient: Color::BLACK,
        }
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self { ambient, ..self }
    }
}

impl<F: Float + Texel> Phong<F, Color<F>, Color<F>, Color<F>, F> {
    #[must_use]
    pub fn white() -> Self {
        Self {
            ke: Color::BLACK,
            kd: Color::WHITE,
            ks: Color::WHITE,
            pow: F::from_u32(8),
            ambient: Color::BLACK,
        }
    }

    #[must_use]
    pub fn black() -> Self {
        Self {
            ke: Color::BLACK,
            kd: Color::BLACK,
            ks: Color::WHITE,
            pow: F::from_u32(8),
            ambient: Color::BLACK,
        }
    }
}

impl<F, S1, S2, S3, S4> Material<F> for Phong<F, S1, S2, S3, S4>
where
    F: Float + Texel,
    S1: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, F>,
{
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let uv = maxel.uv();

        let diff_color = self.kd.sample(uv);
        let spec_color = self.ks.sample(uv);
        let spec_pow = self.pow.sample(uv);

        let ambi_color = self.ambient * rt.ambient();
        let mut res = self.ke.sample(uv) + ambi_color;

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
impl<F, S1, S2, S3, S4> Interactive<F> for Phong<F, S1, S2, S3, S4>
where
    F: Float + Texel,
    S1: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, F>,
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

impl<F, S1, S2, S3, S4> SceneObject<F> for Phong<F, S1, S2, S3, S4>
where
    F: Float + Texel,
    S1: Sampler<F, Color<F>>,
    S2: Sampler<F, Color<F>>,
    S3: Sampler<F, Color<F>>,
    S4: Sampler<F, F>,
{
    sceneobject_impl_body!("Phong");
}
