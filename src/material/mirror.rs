use super::mat_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Mirror<F, T>
where
    F: Float,
    T: Sampler<F, Color<F>>,
{
    refl: T,
    _p: PhantomData<F>,
}

impl<F: Float, T: Sampler<F, Color<F>>> Mirror<F, T> {
    pub const fn new(refl: T) -> Self {
        Self {
            refl,
            _p: PhantomData {},
        }
    }
}

impl<F: Float, T: Sampler<F, Color<F>>> Material<F> for Mirror<F, T> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let refl_color = self.refl.sample(maxel.uv());

        if !refl_color.is_zero() {
            rt.ray_trace(&maxel.reflected_ray())
                .map(|c| c * refl_color)
                .unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
        }
    }
}

#[cfg(feature = "gui")]
impl<F: Float, T: Sampler<F, Color<F>>> Interactive<F> for Mirror<F, T> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        self.refl.ui(ui, "Reflectance")
    }
}

#[cfg(feature = "gui")]
impl<F: Float, T: Sampler<F, Color<F>>> SceneObject<F> for Mirror<F, T> {
    sceneobject_impl_body!("Mirror");
}
