use std::{
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

use crate::{
    engine::{RenderEngine, RenderSpan},
    scene::{BoxScene, SceneObject},
    types::{Color, Float, Point},
};
use crate::{point, types::RResult, Vector};

use eframe::egui::Key;
use egui::{CollapsingHeader, Sense, TextureOptions};
use image::{ImageBuffer, Rgba};

pub struct RustRayGui<F: Float> {
    beam: crossbeam_channel::Receiver<RenderSpan<F>>,
    engine: RenderEngine<F>,
    lock: Arc<RwLock<BoxScene<F>>>,
    img: ImageBuffer<Rgba<u8>, Vec<u8>>,
    obj: Option<usize>,
}

pub fn position_ui<F: Float>(ui: &mut egui::Ui, pos: &mut Vector<F>, name: &str) {
    /* let mut rgb: [f32; 3] = (*pos).into(); */
    ui.label(name);
    ui.end_row();

    ui.label("X");
    ui.add(egui::DragValue::new(&mut pos.x).speed(0.1));
    ui.end_row();

    ui.label("Y");
    ui.add(egui::DragValue::new(&mut pos.y).speed(0.1));
    ui.end_row();

    ui.label("Z");
    ui.add(egui::DragValue::new(&mut pos.z).speed(0.1));
    ui.end_row();
}

pub fn color_ui<F: Float>(ui: &mut egui::Ui, color: &mut Color<F>, name: &str) {
    let mut rgb: [f32; 3] = (*color).into();
    ui.label(name);
    ui.color_edit_button_rgb(&mut rgb);
    color.r = F::from_f32(rgb[0]);
    color.g = F::from_f32(rgb[1]);
    color.b = F::from_f32(rgb[2]);
}

impl<F: Float + From<f32>> RustRayGui<F> {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        beam: crossbeam_channel::Receiver<RenderSpan<F>>,
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
            beam,
            engine,
            lock,
            img,
            obj: None,
        }
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
            scene.objects.iter_mut().enumerate().for_each(|(i, obj)| {
                let resp = CollapsingHeader::new(format!("Object {i}: {}", obj.get_name()))
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(interactive) = obj.get_interactive() {
                            interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive object :(");
                        }
                    });

                if self.obj == obj.get_id() {
                    resp.header_response.highlight();
                }
            });

            ui.label("Lights");
            scene.lights.iter_mut().enumerate().for_each(|(i, light)| {
                CollapsingHeader::new(format!("Light {i}"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(interactive) = light.get_interactive() {
                            interactive.ui(ui);
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
                            interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive camera :(");
                        }
                    });
            });

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
        scene: &RwLockWriteGuard<BoxScene<F>>,
    ) {
        let size = [self.img.width() as usize, self.img.height() as usize];

        let img =
            egui::ColorImage::from_rgba_unmultiplied(size, self.img.as_flat_samples().as_slice());

        let tex = ui.ctx().load_texture("foo", img, TextureOptions::LINEAR);

        let response = ui.image(&tex);

        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0)),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        let act = ui.interact(response.rect, response.id, Sense::click());
        if act.clicked() {
            if let Some(pos) = act.interact_pointer_pos {
                let coord = from_screen.transform_pos(pos);
                let ray = scene.cameras[0].get_ray(point!(coord.x, coord.y));
                if let Some(maxel) = scene.intersect(&ray) {
                    self.obj = maxel.obj.get_id();
                    info!("Picked {:?}", &self.obj);
                } else {
                    self.obj = None;
                }
            }
        }

        let progress = self.engine.progress();
        ui_progress(ctx, ui, progress);
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

impl<F: Float + From<f32>> eframe::App for RustRayGui<F> {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(res) = self.beam.try_recv() {
            let y = res.line;
            for (x, pixel) in res.pixels.iter().enumerate() {
                self.img.put_pixel(x as u32, y, Rgba(pixel.to_array4()));
            }
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
                    Rgba(Color::<F>::new(F::ZERO, F::ZERO, F::from_f32(0.75)).to_array4()),
                );
            }

            self.engine.render_lines(0, height);
        }

        if ctx.input(|i| i.key_pressed(Key::F)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| update_top_panel(ctx, ui));

        let lock = self.lock.clone();
        let mut scene = lock.write().unwrap();

        egui::SidePanel::left("Scene controls")
            .resizable(true)
            .show(ctx, |ui| self.update_side_panel(ctx, ui, &mut scene));

        egui::CentralPanel::default().show(ctx, |ui| self.update_center_panel(ctx, ui, &scene));
    }
}

pub fn run<F>(scene: BoxScene<F>, width: u32, height: u32) -> RResult<()>
where
    F: Float + From<f32>,
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
    let engine = RenderEngine::new(lock.clone(), width, height);
    engine.render_lines(0, height);
    let rx = engine.rx.clone();

    Ok(eframe::run_native(
        "Rustray",
        native_options,
        Box::new(move |cc| {
            Box::new(RustRayGui::new(
                cc,
                rx,
                engine,
                lock,
                ImageBuffer::new(width, height),
            ))
        }),
    )?)
}
