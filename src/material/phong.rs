use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Phong<F: Float + Texel, S: Sampler<F, F>, M: Material<F>> {
    pow: S,
    mat: M,
    _p: PhantomData<F>,
}

impl<F: Float + Texel, S: Sampler<F, F>, M: Material<F>> Phong<F, S, M> {
    #[must_use]
    pub const fn new(pow: S, mat: M) -> Self {
        Self {
            pow,
            mat,
            _p: PhantomData,
        }
    }
}

impl<F: Float + Texel> Phong<F, F, Color<F>> {
    #[must_use]
    pub fn white() -> Self {
        Self::new(F::from_u32(8), Color::WHITE)
    }

    #[must_use]
    pub fn black() -> Self {
        Self::new(F::from_u32(8), Color::BLACK)
    }
}

impl<F: Float + Texel, S: Sampler<F, F>, M: Material<F>> Material<F> for Phong<F, S, M> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let mut res = Color::BLACK;

        let self_color = self.mat.render(maxel, rt);
        let spec_adjust = self.pow.sample(maxel.uv()) / F::TWO;

        for light in rt.get_lights() {
            let lixel = light.contribution(maxel, rt);

            let lambert = maxel.nml().dot(lixel.dir);

            if lambert.is_positive() {
                let light_color = lixel.color * self_color;
                res += light_color * lambert;

                let refl_dir = lixel.dir.reflect(&maxel.nml());
                let spec_angle = refl_dir.dot(maxel.dir).clamp(F::ZERO, F::ONE);
                let specular = spec_angle.pow(spec_adjust);
                res += light_color * specular;
            }
        }
        res
    }

    fn shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        self.mat.shadow(maxel, lixel)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Phong")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        self.pow.ui(ui, "Power");
                        ui.end_row();

                        self.mat.ui(ui);
                        ui.end_row();
                    });
            });
    }
}
