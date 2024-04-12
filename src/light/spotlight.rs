use cgmath::{InnerSpace, Rad};

use crate::light::{Attenuation, Light, Lixel};
use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Color, Float, Maxel, Vector, Vectorx};

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
    sceneobject_impl_body!("Spot Light");
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
        use crate::frontend::gui::controls;
        use num_traits::FloatConst;

        let mut res = false;

        res |= controls::color(ui, &mut self.color, "Color");
        res |= controls::attenuation(ui, &mut self.attn);

        ui.label("Umbra");
        res |= ui
            .add(egui::Slider::new(&mut self.umbra.0, F::ZERO..=F::PI()).step_by(f64::PI() / 180.0))
            .changed();
        ui.end_row();

        ui.label("Penumbra");
        res |= ui
            .add(
                egui::Slider::new(&mut self.penumbra.0, F::ZERO..=F::PI())
                    .step_by(f64::PI() / 180.0),
            )
            .changed();
        ui.end_row();

        res |= controls::position(ui, &mut self.pos, "Position");
        res |= controls::position(ui, &mut self.dir, "Direction");

        res
    }
}
