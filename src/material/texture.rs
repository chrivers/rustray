use std::marker::PhantomData;

use crate::material::Material;
use crate::sampler::Sampler;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel};

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
}

impl<F: Float, S: Sampler<F, Color<F>>> Interactive<F> for Texture<F, S> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.strong("Texture");
        ui.end_row();
        false
    }
}

impl<F: Float, S: Sampler<F, Color<F>>> SceneObject<F> for Texture<F, S> {
    sceneobject_impl_body!("Texture", egui_phosphor::regular::IMAGE_SQUARE);
}

impl<F: Float, S: Sampler<F, Color<F>>> AsRef<Self> for Texture<F, S> {
    fn as_ref(&self) -> &Self {
        self
    }
}
