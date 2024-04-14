use itertools::Itertools;

use std::{
    path::Path,
    sync::{Arc, RwLock, RwLockWriteGuard},
    time::Duration,
};

use crate::{
    engine::RenderEngine,
    format::sbt2::{Rule as SbtRule, SbtBuilder, SbtParser2},
    geometry::Geometry,
    gui::{controls, gizmo, visualtrace},
    point,
    sampler::Texel,
    scene::{BoxScene, SceneObject},
    types::{Error, Float, Point, RResult, RF},
};

use eframe::egui::Key;
use egui::{
    emath::RectTransform, pos2, vec2, Align, CentralPanel, CollapsingHeader, Color32, Context,
    KeyboardShortcut, Modifiers, PointerButton, Pos2, ProgressBar, Rect, RichText, ScrollArea,
    Sense, Shape, SidePanel, TextureOptions, TopBottomPanel, Ui, ViewportBuilder, ViewportCommand,
    Visuals,
};
use egui_file_dialog::FileDialog;
use pest::Parser;

pub struct RustRayGui<F: Float> {
    engine: RenderEngine<F>,
    lock: Arc<RwLock<BoxScene<F>>>,
    obj: Option<usize>,
    obj_last: Option<usize>,
    file_dialog: FileDialog,
    shapes: Vec<Shape>,
    coord: Option<Pos2>,
    trace: bool,
}

impl<F: Float + Texel + From<f32>> RustRayGui<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        engine: RenderEngine<F>,
        lock: Arc<RwLock<BoxScene<F>>>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        /* let mut fonts = egui::FontDefinitions::default(); */
        /* egui_nerdfonts::add_to_fonts(&mut fonts, egui_nerdfonts::Variant::Regular); */
        /* cc.egui_ctx.set_fonts(fonts); */

        let visuals = Visuals {
            slider_trailing_fill: true,
            ..Visuals::default()
        };

        cc.egui_ctx.set_visuals(visuals);

        Self {
            engine,
            lock,
            obj: None,
            obj_last: None,
            file_dialog: FileDialog::new().show_devices(false),
            shapes: vec![],
            coord: None,
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

    fn render_preview(&mut self) {
        self.engine.render_all_by_step(&self.lock, 4, 4);
    }

    fn update_side_panel(
        &mut self,
        _ctx: &Context,
        ui: &mut Ui,
        scene: &mut RwLockWriteGuard<BoxScene<F>>,
    ) {
        ui.label(RichText::new("RustRay").heading().strong());

        ScrollArea::vertical().show(ui, |ui| {
            let mut changed = false;

            CollapsingHeader::new(RichText::new("Materials").heading().strong())
                .default_open(true)
                .show(ui, |ui| {
                    let mat_keys = scene.materials.mats.keys().copied().sorted();

                    for (idx, id) in mat_keys.into_iter().enumerate() {
                        let mat = scene.materials.mats.get_mut(&id).unwrap();
                        /* let name = format!("material-{}", mat.get_id().unwrap_or(0)); */
                        /* egui::collapsing_header::CollapsingState::load_with_default_open( */
                        /*     ui.ctx(), */
                        /*     name.into(), */
                        /*     true, */
                        /* ) */
                        /* .show_header(ui, |ui| { */
                        /*     ui.label(format!("Material: {idx}")); */
                        /* }) */
                        /* .body(|ui| { */
                        let name = format!("Material {idx}: {}", mat.get_name());
                        controls::property_list(&name, ui, |ui| {
                            changed |= mat.ui(ui);
                        });
                    }
                });

            CollapsingHeader::new(RichText::new("Objects").heading().strong())
                .default_open(true)
                .show(ui, |ui| {
                    scene.objects.iter_mut().enumerate().for_each(|(i, obj)| {
                        let name = format!("Object {i}: {}", obj.get_name());
                        let resp = controls::property_list(&name, ui, |ui| {
                            ui.selectable_value(&mut self.obj, obj.get_id(), "Select");
                            ui.end_row();

                            if let Some(interactive) = obj.get_interactive() {
                                changed |= interactive.ui(ui);
                            } else {
                                ui.label("Non-interactive object :(");
                            }
                        });

                        if self.obj == obj.get_id() && self.obj != self.obj_last {
                            resp.header_response
                                .highlight()
                                .scroll_to_me(Some(Align::Center));
                        }
                    });
                });

            CollapsingHeader::new(RichText::new("Lights").heading().strong())
                .default_open(true)
                .show(ui, |ui| {
                    scene.lights.iter_mut().enumerate().for_each(|(i, light)| {
                        let name = format!("Light {i}: {}", light.get_name());
                        controls::property_list(&name, ui, |ui| {
                            if let Some(interactive) = light.get_interactive() {
                                changed |= interactive.ui(ui);
                            } else {
                                ui.label("Non-interactive light :(");
                            }
                        });
                    });
                });

            CollapsingHeader::new(RichText::new("Cameras").heading().strong())
                .default_open(true)
                .show(ui, |ui| {
                    scene.cameras.iter_mut().enumerate().for_each(|(i, cam)| {
                        let name = format!("Camera {i}");
                        controls::property_list(&name, ui, |ui| {
                            if let Some(interactive) = cam.get_interactive() {
                                changed |= interactive.ui(ui);
                            } else {
                                ui.label("Non-interactive camera :(");
                            }
                        });
                    });
                });

            if changed {
                scene.recompute_bvh().unwrap();
                self.render_preview();
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
        ctx: &Context,
        ui: &mut Ui,
        scene: &mut RwLockWriteGuard<BoxScene<F>>,
    ) {
        let img = self.engine.get_epaint_image();

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

        let to_screen = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, vec2(1.0, 1.0)),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        let act = response.interact(Sense::click_and_drag());

        if act.clicked() {
            if let Some(pos) = act.interact_pointer_pos {
                let coord = from_screen.transform_pos(pos);
                let mut ray = scene.cameras[0].get_ray(point!(coord.x, coord.y));
                ray.flags |= RF::StopAtGroup;
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
        self.obj_last = self.obj;

        if let Some(pos) = act.hover_pos() {
            let coord = from_screen.transform_pos(pos);
            if act.double_clicked_by(PointerButton::Secondary) {
                self.coord = None;
                self.shapes.clear();
                self.trace = false;
            } else if act.clicked_by(PointerButton::Secondary) {
                self.trace = !self.trace;
            }
            if self.trace {
                self.coord = Some(coord);
            }
        }

        if let Some(coord) = self.coord {
            self.shapes = visualtrace::make_shapes(scene, coord, &to_screen);
        }

        let camera = scene.cameras[0];
        if let Some(obj) = self.find_obj(scene) {
            if let Some(int) = obj.get_interactive() {
                if int.ui_center(ui, &camera, &response.rect) {
                    scene.recompute_bvh().unwrap();
                    self.render_preview();
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
        let p = SbtParser2::parse(SbtRule::program, &data).map_err(|err| err.with_path(name))?;
        let p = SbtParser2::ast(p)?;
        let scene =
            SbtBuilder::new(self.engine.img.width(), self.engine.img.height(), resdir).build(p)?;
        Ok(scene)
    }
}

fn ui_progress(ctx: &Context, ui: &mut Ui, (queued, max): (usize, usize)) {
    if queued > 0 {
        ctx.request_repaint_after(Duration::from_millis(20));

        let progress_bar = ProgressBar::new(1.0 - (queued as f32 / max as f32)).show_percentage();

        ui.add(progress_bar);
    }
}

fn update_top_panel(ctx: &Context, ui: &mut Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        });

        ui.add_space(16.0);

        if ui.button("Full screen").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        }

        if ui.button("Window").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
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
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let recv = self.engine.update();
        if recv {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else {
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        if ctx.input(|i| i.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.engine.render_all(&self.lock);
        }

        if ctx.input(|i| i.key_pressed(Key::T)) {
            self.engine.render_normals(&self.lock);
        }

        if ctx.input(|i| i.key_pressed(Key::F)) {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
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
                    .render_lines(&self.lock, 0, self.engine.img.height());
            }
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| update_top_panel(ctx, ui));

        let lock = self.lock.clone();
        let mut scene = lock.write().unwrap();

        SidePanel::left("Scene controls")
            .resizable(true)
            .show(ctx, |ui| self.update_side_panel(ctx, ui, &mut scene));

        CentralPanel::default().show(ctx, |ui| self.update_center_panel(ctx, ui, &mut scene));
    }
}

pub fn run<F>(scene: BoxScene<F>, width: u32, height: u32) -> RResult<()>
where
    F: Float + Texel + From<f32>,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
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
    engine.render_lines(&lock, 0, height);

    Ok(eframe::run_native(
        "Rustray",
        native_options,
        Box::new(move |cc| Box::new(RustRayGui::new(cc, engine, lock))),
    )?)
}
