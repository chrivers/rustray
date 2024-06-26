use cgmath::InnerSpace;

use crate::light::{Attenuation, Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Vector, Vectorx};

#[derive(Debug)]
pub struct PointLight<F: Float> {
    pub pos: Vector<F>,
    pub attn: Attenuation<F>,
    pub color: Color<F>,
}

impl<F: Float> PointLight<F> {
    pub const ICON: &'static str = egui_phosphor::regular::LIGHTBULB;

    pub const fn new(pos: Vector<F>, attn: Attenuation<F>, color: Color<F>) -> Self {
        Self { pos, attn, color }
    }
}

impl<F: Float> Interactive<F> for PointLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::gui::controls;

        let mut res = false;
        res |= controls::color(ui, &mut self.color, "Color");
        res |= controls::attenuation(ui, &mut self.attn);
        res |= controls::position(ui, &mut self.pos, "Position");
        res
    }
}

impl<F: Float> SceneObject<F> for PointLight<F> {
    sceneobject_impl_body!("Point Light", Self::ICON);
}

impl<F: Float> Light<F> for PointLight<F> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let dir = self.pos.vector_to(maxel.pos);
        let len2 = dir.magnitude2();
        let len = len2.sqrt();
        let color = self.attn.attenuate(self.color, len, len2);

        let mut lixel = Lixel {
            dir: -dir / len,
            color,
            len2,
        };

        if let Some(color) = rt.ray_shadow(maxel, &lixel) {
            lixel.color = color;
        }
        lixel
    }
}
