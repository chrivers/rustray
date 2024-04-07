pub mod visualtrace;

mod gizmo;

use std::{
    path::Path,
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

use crate::{
    engine::RenderEngine,
    format::sbt2::{Rule as SbtRule, SbtBuilder, SbtParser2},
    geometry::Geometry,
    light::Attenuation,
    point,
    sampler::Texel,
    scene::{BoxScene, SceneObject},
    types::{Color, Error, Float, Point, RResult},
    Vector,
};

use eframe::egui::Key;
use egui::{
    pos2, CollapsingHeader, Color32, KeyboardShortcut, Modifiers, Rect, Sense, TextureOptions,
};
use egui_file_dialog::FileDialog;
use image::{ImageBuffer, Rgba};
use pest::Parser;

pub use gizmo::gizmo_ui;

pub struct RustRayGui<F: Float> {
    engine: RenderEngine<F>,
    lock: Arc<RwLock<BoxScene<F>>>,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    obj: Option<usize>,
    file_dialog: FileDialog,
    shapes: Vec<egui::Shape>,
    trace: bool,
}

#[must_use]
pub fn position_ui<F: Float>(ui: &mut egui::Ui, pos: &mut Vector<F>, name: &str) -> bool {
    let mut res = false;

    ui.label(name);
    ui.end_row();

    ui.label("X");
    res |= ui
        .add(egui::DragValue::new(&mut pos.x).speed(0.1))
        .changed();
    ui.end_row();

    ui.label("Y");
    res |= ui
        .add(egui::DragValue::new(&mut pos.y).speed(0.1))
        .changed();
    ui.end_row();

    ui.label("Z");
    res |= ui
        .add(egui::DragValue::new(&mut pos.z).speed(0.1))
        .changed();
    ui.end_row();

    res
}

pub fn color_ui<F: Float>(ui: &mut egui::Ui, color: &mut Color<F>, name: &str) -> bool {
    let mut res = false;
    let mut rgb: [f32; 3] = (*color).into();

    ui.label(name);
    res |= ui.color_edit_button_rgb(&mut rgb).changed();
    ui.end_row();

    *color = Color::from(rgb);

    res
}

pub fn attenuation_ui<F: Float>(ui: &mut egui::Ui, attn: &mut Attenuation<F>) -> bool {
    let mut res = false;

    ui.label("Falloff d^0");
    res |= ui
        .add(egui::Slider::new(&mut attn.a, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    ui.label("Falloff d^1");
    res |= ui
        .add(egui::Slider::new(&mut attn.b, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    ui.label("Falloff d^2");
    res |= ui
        .add(egui::Slider::new(&mut attn.c, F::ZERO..=F::TWO).logarithmic(true))
        .changed();
    ui.end_row();

    res
}

impl<F: Float + Texel + From<f32>> RustRayGui<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        engine: RenderEngine<F>,
        lock: Arc<RwLock<BoxScene<F>>>,
        img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        /* let mut fonts = egui::FontDefinitions::default(); */
        /* egui_nerdfonts::add_to_fonts(&mut fonts, egui_nerdfonts::Variant::Regular); */
        /* cc.egui_ctx.set_fonts(fonts); */

        Self {
            engine,
            lock,
            img,
            obj: None,
            file_dialog: FileDialog::new().show_devices(false),
            shapes: vec![],
            trace: false,
        }
    }

    fn find_obj<'a>(
        &self,
        scene: &'a mut RwLockWriteGuard<BoxScene<F>>,
    ) -> Option<&'a mut dyn Geometry<F>> {
        scene
            .objects
            .iter_mut()
            .find(|obj| obj.get_id() == self.obj)
            .map(|m| m as &mut _)
    }

    fn render_preview(&mut self, scene: &mut RwLockWriteGuard<BoxScene<F>>) {
        self.engine
            .render_lines_by_step(self.lock.clone(), 0, self.img.height(), 4, 4);
        scene.recompute_bvh().unwrap();
    }

    fn update_side_panel(
        &mut self,
        _ctx: &egui::Context,
        ui: &mut egui::Ui,
        scene: &mut RwLockWriteGuard<BoxScene<F>>,
    ) {
        ui.heading("Rustray");
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Objects");
            let mut changed = false;

            scene.objects.iter_mut().enumerate().for_each(|(i, obj)| {
                let resp = CollapsingHeader::new(format!("Object {i}: {}", obj.get_name()))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.selectable_value(&mut self.obj, obj.get_id(), "Select");
                        if let Some(interactive) = obj.get_interactive() {
                            changed |= interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive object :(");
                        }
                    });

                if self.obj == obj.get_id() {
                    resp.header_response
                        .highlight()
                        .scroll_to_me(Some(egui::Align::Center));
                }
            });

            ui.label("Lights");
            scene.lights.iter_mut().enumerate().for_each(|(i, light)| {
                CollapsingHeader::new(format!("Light {i}"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(interactive) = light.get_interactive() {
                            changed |= interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive light :(");
                        }
                    });
            });

            ui.label("Cameras");
            scene.cameras.iter_mut().enumerate().for_each(|(i, cam)| {
                CollapsingHeader::new(format!("Camera {i}"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(interactive) = cam.get_interactive() {
                            changed |= interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive camera :(");
                        }
                    });
            });

            if changed {
                self.render_preview(scene);
            }

            /* CollapsingHeader::new(format!("Raytracer")) */
            /*     .default_open(true) */
            /*     .show(ui, |ui| { */
            /*         if let Some(interactive) = self.engine.tracer.get_interactive() { */
            /*             interactive.ui(ui); */
            /*         } else { */
            /*             ui.label("Non-interactive camera :("); */
            /*         } */
            /*     }); */
        });
    }

    fn update_center_panel(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        scene: &mut RwLockWriteGuard<BoxScene<F>>,
    ) {
        let size = [self.img.width() as usize, self.img.height() as usize];

        let img =
            egui::ColorImage::from_rgba_unmultiplied(size, self.img.as_flat_samples().as_slice());

        let tex = ui.ctx().load_texture("foo", img, TextureOptions::LINEAR);

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());
        painter.image(
            (&tex).into(),
            painter.clip_rect(),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        for shape in &self.shapes {
            painter.add(shape.clone());
        }

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0)),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        let act = response.interact(Sense::click_and_drag());

        if act.clicked() {
            if let Some(pos) = act.interact_pointer_pos {
                let coord = from_screen.transform_pos(pos);
                let mut ray = scene.cameras[0].get_ray(point!(coord.x, coord.y));
                ray.grp = 0;
                if let Some(maxel) = scene.intersect(&ray) {
                    let id = maxel.obj.get_id();
                    if self.obj == id {
                        info!("Deselect {:?}", maxel.obj.get_name());
                        self.obj = None;
                    } else {
                        info!("Select {:?}", maxel.obj.get_name());
                        self.obj = id;
                    }
                } else {
                    self.obj = None;
                }
            }
        }

        if act.clicked_by(egui::PointerButton::Secondary) {
            self.trace = !self.trace;
        }

        if self.trace {
            if let Some(pos) = act.hover_pos() {
                let coord = from_screen.transform_pos(pos);
                self.shapes =
                    crate::frontend::gui::visualtrace::make_shapes(scene, coord, &to_screen);
            }
        }

        let camera = scene.cameras[0];
        if let Some(obj) = self.find_obj(scene) {
            if let Some(int) = obj.get_interactive() {
                if int.ui_center(ui, &camera, &response.rect) {
                    self.render_preview(scene);
                }
            }
        }

        let progress = self.engine.progress();
        ui_progress(ctx, ui, progress);
    }

    fn load_file(&self, path: &Path) -> RResult<BoxScene<F>> {
        let data = std::fs::read_to_string(path)?;

        let resdir = path.parent().ok_or(Error::ParseError("Invalid filename"))?;

        let name = path
            .to_str()
            .ok_or(Error::ParseError("Invalid UTF-8 filename"))?;
        let p =
            SbtParser2::<F>::parse(SbtRule::program, &data).map_err(|err| err.with_path(name))?;
        let p = SbtParser2::<F>::ast(p)?;
        let scene = SbtBuilder::new(self.img.width(), self.img.height(), resdir).build(p)?;
        Ok(scene)
    }
}

fn ui_progress(ctx: &egui::Context, ui: &mut egui::Ui, progress: f32) {
    if progress != f32::ONE {
        ctx.request_repaint_after(Duration::from_millis(20));

        let progress_bar = egui::ProgressBar::new(progress).show_percentage();

        ui.add(progress_bar);
    }
}

fn update_top_panel(ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.add_space(16.0);

        if ui.button("Full screen").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }

        if ui.button("Window").clicked() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }

        ui.add_space(16.0);

        egui::widgets::global_dark_light_mode_buttons(ui);
    });
}

impl<F> eframe::App for RustRayGui<F>
where
    F: Float + Texel + From<f32>,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut recv = false;
        for res in self.engine.iter() {
            for (base_x, pixel) in res.pixels.iter().enumerate() {
                let rgba = Rgba(pixel.to_array4());
                for y in 0..res.mult_y {
                    for x in 0..res.mult_x {
                        self.img
                            .put_pixel((base_x as u32) * res.mult_x + x, res.line + y, rgba);
                    }
                }
            }
            recv = true;
        }
        if recv {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        if ctx.input(|i| i.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if ctx.input(|i| i.key_pressed(Key::R)) {
            let height = self.img.height();
            for y in 0..height {
                self.img.put_pixel(
                    0,
                    y,
                    Rgba(Color::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4()),
                );
            }

            self.engine.render_lines(self.lock.clone(), 0, height);
        }

        if ctx.input(|i| i.key_pressed(Key::T)) {
            let height = self.img.height();
            for y in 0..height {
                self.img.put_pixel(
                    0,
                    y,
                    Rgba(Color::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4()),
                );
            }

            self.engine.render_normals(self.lock.clone(), 0, height);
        }

        if ctx.input(|i| i.key_pressed(Key::F)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }

        if ctx.input(|i| i.key_pressed(Key::O)) {
            self.file_dialog.select_file();
        }

        let kbd_space = KeyboardShortcut::new(Modifiers::NONE, Key::Space);
        let kbd_shift_space = KeyboardShortcut::new(Modifiers::SHIFT, Key::Space);

        if ctx.input_mut(|i| i.consume_shortcut(&kbd_shift_space)) {
            ctx.data_mut(gizmo::switch_orientation);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&kbd_space)) {
            ctx.data_mut(gizmo::switch_mode);
        }

        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_selected() {
            info!("New file: {path:?}");
            self.file_dialog.config_mut().initial_directory = path.parent().unwrap().to_path_buf();
            if let Ok(scene) = self.load_file(&path) {
                self.lock = Arc::new(RwLock::new(scene));
                self.engine
                    .render_lines(self.lock.clone(), 0, self.img.height());
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| update_top_panel(ctx, ui));

        let lock = self.lock.clone();
        let mut scene = lock.write().unwrap();

        egui::SidePanel::left("Scene controls")
            .resizable(true)
            .show(ctx, |ui| self.update_side_panel(ctx, ui, &mut scene));

        egui::CentralPanel::default().show(ctx, |ui| self.update_center_panel(ctx, ui, &mut scene));
    }
}

pub fn run<F>(scene: BoxScene<F>, width: u32, height: u32) -> RResult<()>
where
    F: Float + Texel + From<f32>,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            /* .with_fullscreen(true) */
            /* .with_min_inner_size([300.0, 220.0]), */
            /* .with_icon( */
            /*     // NOTE: Adding an icon is optional */
            /*     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..]) */
            /*         .unwrap(), */
            /* ), */
            .with_inner_size([width as f32 + 360.0, height as f32 + 60.0]),
        ..Default::default()
    };

    let lock = Arc::new(RwLock::new(scene));
    let mut engine = RenderEngine::new(width, height);
    engine.render_lines(lock.clone(), 0, height);

    Ok(eframe::run_native(
        "Rustray",
        native_options,
        Box::new(move |cc| {
            Box::new(RustRayGui::new(
                cc,
                engine,
                lock,
                ImageBuffer::new(width, height),
            ))
        }),
    )?)
}
