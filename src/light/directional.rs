use cgmath::InnerSpace;

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Vector};

#[derive(Debug)]
pub struct DirectionalLight<F: Float> {
    dir: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> DirectionalLight<F> {
    pub fn new(dir: Vector<F>, color: Color<F>) -> Self {
        Self {
            dir: dir.normalize(),
            color,
        }
    }
}

impl<F: Float> Interactive<F> for DirectionalLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::gui::controls;

        let mut res = false;
        res |= controls::color(ui, &mut self.color, "Color");
        res |= controls::position(ui, &mut self.dir, "Direction");
        res
    }
}

impl<F: Float> SceneObject<F> for DirectionalLight<F> {
    sceneobject_impl_body!("Directional Light");
}

impl<F: Float> Light<F> for DirectionalLight<F> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let mut lixel = Lixel {
            // FIXME: precalculate
            dir: -self.dir.normalize(),
            color: self.color,
            len2: F::from_u32(100_000),
        };
        if let Some(color) = rt.ray_shadow(maxel, &lixel) {
            lixel.color = color;
        }
        lixel
    }
}
