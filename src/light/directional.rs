use cgmath::InnerSpace;

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::{Color, Float, Maxel, Vector};

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

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
        egui::CollapsingHeader::new("Directional light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let mut res = false;

                        res |= color_ui(ui, &mut self.color, "Color");
                        ui.end_row();

                        res |= position_ui(ui, &mut self.dir, "Direction");

                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}

impl<F: Float> SceneObject<F> for DirectionalLight<F> {
    fn get_name(&self) -> &str {
        "Directional Light"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float> Light<F> for DirectionalLight<F> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        rt.shadow(
            maxel,
            Lixel {
                // FIXME: precalculate
                dir: -self.dir.normalize(),
                color: self.color,
                len2: F::from_u32(100_000),
            },
        )
    }
}
