use super::samp_util::*;

#[derive(Copy, Clone, Debug)]
pub struct Adjust<F: Float, T: Texel, S: Sampler<F, T>> {
    scale: F,
    offset: F,
    samp: S,
    _p1: PhantomData<T>,
}

impl<F: Float, T: Texel, S: Sampler<F, T>> Adjust<F, T, S> {
    pub fn new(scale: F, offset: F, samp: S) -> Self {
        Self {
            scale,
            offset,
            samp,
            _p1: PhantomData {},
        }
    }
}

impl<F, T, S> Sampler<F, T> for Adjust<F, T, S>
where
    F: Float,
    T: Texel,
    T: std::ops::Mul<F, Output = T>,
    T: std::ops::Add<F, Output = T>,
    T: Lerp<Ratio = F>,
    S: Sampler<F, T>,
{
    fn sample(&self, uv: Point<F>) -> T {
        self.samp.sample(uv) * self.scale + self.offset
    }

    fn dimensions(&self) -> (u32, u32) {
        self.samp.dimensions()
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.add(Slider::new(&mut self.scale, F::ZERO..=F::from_u32(100)).text("Scaling"));
                ui.add(Slider::new(&mut self.offset, F::ZERO..=F::from_u32(100)).text("Offset"));
                self.samp.ui(ui, name);
            });
    }
}
