use super::mat_util::*;

pub trait TextureSampler<F: Float>
where
    Self: Sampler<F, Color<F>> + Sized,
{
    fn texture(self) -> Texture<F, Self> {
        Texture::new(self)
    }
}

impl<F: Float, T: Sampler<F, Color<F>>> TextureSampler<F> for T {}

/// Simple material that maps a sampler to an object based on UV coordinates.
///
/// This material is practically only useful as a part of a material
/// composition, or for extremely simple objects.

#[derive(Copy, Clone, Debug)]
pub struct Texture<F: Float, S: Sampler<F, Color<F>>> {
    _f: PhantomData<F>,
    img: S,
}

impl<F: Float, S: Sampler<F, Color<F>>> Texture<F, S> {
    pub const fn new(img: S) -> Self {
        Self {
            _f: PhantomData {},
            img,
        }
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> Material<F> for Texture<F, S> {
    fn render(&self, maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Color<F> {
        self.img.sample(maxel.uv())
    }

    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Texture")
            .default_open(true)
            .show(ui, |_ui| {});
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> AsRef<Self> for Texture<F, S> {
    fn as_ref(&self) -> &Self {
        self
    }
}
