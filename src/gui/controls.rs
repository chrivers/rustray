use egui::{
    CollapsingHeader, CollapsingResponse, DragValue, Grid,
    InnerResponse, Response, RichText, Slider, Ui,
};
use egui::collapsing_header::CollapsingState;

use crate::light::Attenuation;
use crate::types::{Color, Float, Vector};

pub fn collapsing_group(name: &str, icon: &str) -> CollapsingHeader {
    CollapsingHeader::new(RichText::new(format!("{icon} {name}")).heading().strong())
        .default_open(true)
}

pub fn property_list<R>(
    name: &str,
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> CollapsingResponse<InnerResponse<R>> {
    CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            Grid::new(name)
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, add_contents)
        })
}

pub trait EguiId {
    fn egui_id(&self) -> egui::Id;
}

impl EguiId for &str {
    fn egui_id(&self) -> egui::Id {
        egui::Id::new(self)
    }
}

pub struct CustomCollapsible {
    pub id: egui::Id,
    toggle: bool,
}

impl CustomCollapsible {
    pub fn new(id: impl Into<egui::Id>) -> Self {
        Self {
            id: id.into(),
            toggle: false,
        }
    }

    pub fn toggle(&mut self) {
        self.toggle = true;
    }

    pub fn show<R1, R2>(
        mut self,
        ui: &mut Ui,
        header: impl FnOnce(&mut Self, &mut Ui) -> R1,
        body: impl FnOnce(&mut Ui) -> R2,
    ) -> (Response, InnerResponse<R1>, Option<InnerResponse<R2>>) {
        let mut ctrl = CollapsingState::load_with_default_open(ui.ctx(), self.id, false);

        if ui.data(|mem| mem.get_temp(self.id).unwrap_or(false)) {
            ctrl.toggle(ui);
        }

        let res = ctrl.show_header(ui, |ui| header(&mut self, ui)).body(body);

        ui.data_mut(|mem| mem.insert_temp(self.id, self.toggle));

        res
    }
}

#[must_use]
pub fn position<F: Float>(ui: &mut Ui, pos: &mut Vector<F>, name: &str) -> bool {
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
pub fn color<F: Float>(ui: &mut Ui, color: &mut Color<F>, name: &str) -> bool {
    let mut res = false;
    let mut rgb: [f32; 3] = (*color).into();

    ui.label(name);
    res |= ui.color_edit_button_rgb(&mut rgb).changed();
    ui.end_row();

    *color = Color::from(rgb);

    res
}

fn _plot_attenuation<F: Float>(ui: &mut Ui, attn: &Attenuation<F>) -> egui::Response {
    use egui_plot::{Line, PlotPoints};
    let n = 128;
    let line_points: PlotPoints = (0..=n)
        .map(|i| {
            let x = F::from_u32(i);
            [
                f64::from(i),
                attn.attenuate(Color::WHITE, x, x * x).r.to_f64(),
            ]
        })
        .collect();
    let line = Line::new(line_points);
    egui_plot::Plot::new("attenuation")
        .allow_drag(false)
        .allow_boxed_zoom(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .show(ui, |plot_ui| plot_ui.line(line))
        .response
}

#[must_use]
pub fn attenuation<F: Float>(ui: &mut Ui, attn: &mut Attenuation<F>) -> bool {
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

    /* ui.label("Falloff"); */
    /* plot_attenuation(ui, attn); */
    /* ui.end_row(); */

    res
}
