use cgmath::{InnerSpace, Rad};

use num_traits::FloatConst;

use crate::light::{Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::maxel::Maxel;
use crate::types::{Color, Float, Vector};
use crate::Vectorx;

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

use super::Attenuation;

#[derive(Debug)]
pub struct SpotLight<F: Float> {
    pub attn: Attenuation<F>,
    pub umbra: Rad<F>,
    pub penumbra: Rad<F>,
    pub pos: Vector<F>,
    pub dir: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> SceneObject<F> for SpotLight<F> {
    fn get_name(&self) -> &str {
        "Spot Light"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        Some(self)
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<F: Float> Light<F> for SpotLight<F> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        let dir = self.pos.normal_to(maxel.pos);
        let len2 = dir.magnitude2();
        let len = len2.sqrt();
        let color = self.attn.attenuate(self.color, len, len2);

        let angle = self.dir.normalize().dot(dir.normalize()).acos();

        let inner_angle = self.umbra.0;
        let outer_angle = self.penumbra.0;

        if angle > outer_angle {
            return Lixel {
                color: Color::BLACK,
                dir: -dir,
                len2,
            };
        }

        let lixel = if angle > inner_angle {
            let scale = F::ONE - (angle - inner_angle) / (outer_angle - inner_angle).max(F::BIAS);
            Lixel {
                color: color * scale,
                dir: -dir,
                len2,
            }
        } else {
            Lixel {
                color,
                dir: -dir,
                len2,
            }
        };

        rt.shadow(maxel, lixel)
    }
}

impl<F: Float> Interactive<F> for SpotLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::frontend::gui::attenuation_ui;

        egui::CollapsingHeader::new("Spot light")
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

                        ui.label("Umbra");
                        ui.add(
                            egui::Slider::new(&mut self.umbra.0, F::ZERO..=F::PI())
                                .step_by(f64::PI() / 180.0),
                        );
                        ui.end_row();

                        ui.label("Penumbra");
                        ui.add(
                            egui::Slider::new(&mut self.penumbra.0, F::ZERO..=F::PI())
                                .step_by(f64::PI() / 180.0),
                        );
                        ui.end_row();

                        res |= position_ui(ui, &mut self.pos, "Position");
                        res |= position_ui(ui, &mut self.dir, "Direction");

                        res
                    })
                    .inner
            })
            .body_returned
            .unwrap_or(false)
    }
}
