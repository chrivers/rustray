use std::fmt::Debug;
use std::sync::Arc;

use crate::light::Lixel;
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, MaterialId, Maxel};

pub trait Material<F: Float>: SceneObject<F> + Interactive<F> + Debug + Send + Sync {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F>;

    fn shadow(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>, _lixel: &Lixel<F>) -> Color<F> {
        Color::BLACK
    }

    fn dynamic(self) -> DynMaterial<F>
    where
        Self: Sized + 'static,
    {
        Arc::new(self)
    }
}

pub type BoxMaterial<F> = Box<dyn Material<F>>;
pub type DynMaterial<F> = Arc<dyn Material<F>>;

impl<F: Float> Material<F> for Color<F> {
    fn render(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Self {
        *self
    }
}

impl<F: Float> SceneObject<F> for BoxMaterial<F> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        (**self).get_interactive()
    }

    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}

impl<F: Float> SceneObject<F> for Color<F> {
    sceneobject_impl_body!("Color");
}

impl<F: Float> SceneObject<F> for MaterialId {
    sceneobject_impl_body!("Material ID");
}

impl<F: Float> Material<F> for MaterialId {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        let mat = &rt.scene().materials.mats[self];
        mat.render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        let mat = &rt.scene().materials.mats[self];
        mat.shadow(maxel, rt, lixel)
    }
}

impl<F: Float> Material<F> for BoxMaterial<F> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        (**self).render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        (**self).shadow(maxel, rt, lixel)
    }
}

impl<F: Float> Interactive<F> for BoxMaterial<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        (**self).ui(ui)
    }
}

impl<F: Float> Material<F> for DynMaterial<F> {
    fn render(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Color<F> {
        (**self).render(maxel, rt)
    }

    fn shadow(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>, lixel: &Lixel<F>) -> Color<F> {
        (**self).shadow(maxel, rt, lixel)
    }
}

impl<F: Float> SceneObject<F> for DynMaterial<F> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Arc::get_mut(self)?.get_interactive()
    }

    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}

impl<F: Float> Interactive<F> for DynMaterial<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        if let Some(mat) = Arc::get_mut(self) {
            mat.ui(ui)
        } else {
            ui.label("nope :(");
            false
        }
    }
}

mod blend;
mod bumpmap;
mod chessboard;
mod debug;
mod fresnel;
mod matte;
mod mirror;
mod phong;
mod scaleuv;
mod smart;
mod texture;
mod triblend;

pub use blend::Blend;
pub use bumpmap::{BumpPower, Bumpmap};
pub use chessboard::{ChessBoard, ChessBoardMode};
pub use debug::ColorDebug;
pub use fresnel::Fresnel;
pub use matte::Matte;
pub use mirror::Mirror;
pub use phong::Phong;
pub use scaleuv::ScaleUV;
pub use smart::Smart;
pub use texture::{Texture, TextureSampler};
pub use triblend::Triblend;
