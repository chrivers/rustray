use egui::{DragValue, Slider};

use crate::{
    light::Attenuation,
    types::{Color, Float},
    Vector,
};

#[must_use]
pub fn position<F: Float>(ui: &mut egui::Ui, pos: &mut Vector<F>, name: &str) -> bool {
    let mut res = false;

    ui.label(name);
    ui.end_row();

    ui.label("X");
    res |= ui.add(DragValue::new(&mut pos.x).speed(0.1)).changed();
    ui.end_row();

    ui.label("Y");
    res |= ui.add(DragValue::new(&mut pos.y).speed(0.1)).changed();
    ui.end_row();

    ui.label("Z");
    res |= ui.add(DragValue::new(&mut pos.z).speed(0.1)).changed();
    ui.end_row();

    res
}

#[must_use]
pub fn color<F: Float>(ui: &mut egui::Ui, color: &mut Color<F>, name: &str) -> bool {
    let mut res = false;
    let mut rgb: [f32; 3] = (*color).into();

    ui.label(name);
    res |= ui.color_edit_button_rgb(&mut rgb).changed();
    ui.end_row();

    *color = Color::from(rgb);

    res
}

#[must_use]
pub fn attenuation<F: Float>(ui: &mut egui::Ui, attn: &mut Attenuation<F>) -> bool {
    let mut res = false;

    ui.label("Falloff d^0");
    res |= ui
        .add(Slider::new(&mut attn.a, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    ui.label("Falloff d^1");
    res |= ui
        .add(Slider::new(&mut attn.b, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    ui.label("Falloff d^2");
    res |= ui
        .add(Slider::new(&mut attn.c, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    res
}
