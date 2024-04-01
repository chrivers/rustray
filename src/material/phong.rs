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
        let spec_adjust = self.pow.sample(maxel.uv()) / F::from_u32(2);

        for light in rt.get_lights() {
            let light_color = rt
                .ray_shadow(maxel, light)
                .unwrap_or_else(|| light.get_color());

            let light_vec = maxel.pos.vector_to(light.get_position());
            let light_dir = light_vec.normalize();

            let lambert = maxel.nml().dot(light_dir);

            if lambert.is_positive() {
                let light_color = light.attenuate(light_color * self_color, light_vec.magnitude());
                res += light_color * lambert;

                let refl_dir = light_dir.reflect(&maxel.nml());
                let spec_angle = refl_dir.dot(maxel.dir).clamp(F::ZERO, F::ONE);
                let specular = spec_angle.pow(spec_adjust);
                res += light_color * specular;
            }
        }
        res
    }

    fn shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>> {
        self.mat.shadow(maxel, light)
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Phong")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}
