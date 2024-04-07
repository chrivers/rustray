use cgmath::InnerSpace;

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::{Color, Float, Maxel, Vector, Vectorx};

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

use super::Attenuation;

#[derive(Debug)]
pub struct PointLight<F: Float> {
    pub attn: Attenuation<F>,
    pub pos: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> Interactive<F> for PointLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::frontend::gui::attenuation_ui;

        egui::CollapsingHeader::new("Point light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let mut res = false;
                        res |= color_ui(ui, &mut self.color, "Color");
                        res |= attenuation_ui(ui, &mut self.attn);
                        res |= position_ui(ui, &mut self.pos, "Position");
                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}

impl<F: Float> SceneObject<F> for PointLight<F> {
    fn get_name(&self) -> &str {
        "Point Light"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float> Light<F> for PointLight<F> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let dir = self.pos.vector_to(maxel.pos);
        let len2 = dir.magnitude2();
        let len = len2.sqrt();
        let color = self.attn.attenuate(self.color, len, len2);

        rt.shadow(
            maxel,
            Lixel {
                dir: -dir / len,
                color,
                len2,
            },
        )
    }
}
