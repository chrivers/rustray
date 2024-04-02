use cgmath::InnerSpace;

use crate::scene::{Interactive, Light, Lixel, SceneObject};
use crate::types::{Color, Float, Maxel, Vector, Vectorx};

#[cfg(feature = "gui")]
use crate::frontend::gui::{color_ui, position_ui};

#[derive(Debug)]
pub struct PointLight<F: Float> {
    pub a: F,
    pub b: F,
    pub c: F,
    pub pos: Vector<F>,
    pub color: Color<F>,
}

#[derive(Debug)]
pub struct DirectionalLight<F: Float> {
    pub dir: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> Interactive<F> for PointLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Point light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        color_ui(ui, &mut self.color, "Color");
                        ui.end_row();

                        ui.label("Falloff d^0");
                        ui.add(egui::Slider::new(&mut self.a, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        ui.label("Falloff d^1");
                        ui.add(egui::Slider::new(&mut self.b, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        ui.label("Falloff d^2");
                        ui.add(egui::Slider::new(&mut self.c, F::ZERO..=F::FOUR).logarithmic(true));
                        ui.end_row();

                        position_ui(ui, &mut self.pos, "Position");
                    })
            });
    }
}

impl<F: Float> Interactive<F> for DirectionalLight<F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Directional light")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        color_ui(ui, &mut self.color, "Color");
                        ui.end_row();

                        position_ui(ui, &mut self.dir, "Direction");
                    })
            });
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

impl<F: Float> Light<F> for PointLight<F> {
    fn get_color(&self) -> Color<F> {
        self.color
    }

    fn attenuate(&self, color: Color<F>, d: F) -> Color<F> {
        color / (F::ONE + self.a + (self.b * d) + (self.c * d * d))
    }
}

impl<F: Float> Light<F> for DirectionalLight<F> {
    fn get_color(&self) -> Color<F> {
        self.color
    }

    fn attenuate(&self, color: Color<F>, _: F) -> Color<F> {
        color
    }
}

impl<F: Float> HasPosition<F> for DirectionalLight<F> {
    fn get_position(&self) -> Vector<F> {
        self.dir * F::from_f32(-100_000.0)
    }
    fn set_position(&mut self, _: Vector<F>) {}
}

impl<F: Float> HasPosition<F> for PointLight<F> {
    fn get_position(&self) -> Vector<F> {
        self.pos
    }
    fn set_position(&mut self, value: Vector<F>) {
        self.pos = value;
    }
}

impl<'a, F: Float> Light<F> for Box<dyn Light<F> + 'a> {
    fn get_color(&self) -> Color<F> {
        self.as_ref().get_color()
    }

    fn attenuate(&self, color: Color<F>, d: F) -> Color<F> {
        self.as_ref().attenuate(color, d)
    }
}

impl<'a, F: Float> SceneObject<F> for Box<dyn Light<F> + 'a> {
    fn get_name(&self) -> &str {
        self.as_ref().get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        self.as_mut().get_interactive()
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}

impl<'a, F: Float> HasPosition<F> for Box<dyn Light<F> + 'a> {
    fn get_position(&self) -> Vector<F> {
        self.as_ref().get_position()
    }
    fn set_position(&mut self, _value: Vector<F>) {}
}
