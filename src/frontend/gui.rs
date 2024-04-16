use cgmath::{Matrix4, SquareMatrix};
use itertools::Itertools;

use std::{
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    engine::RenderEngine,
    format::sbt2::{Rule as SbtRule, SbtBuilder, SbtParser2},
    geometry::{Cone, Cube, Cylinder, Geometry, Sphere, Square},
    gui::{controls, gizmo, visualtrace::VisualTraceWidget},
    material::Phong,
    point,
    sampler::Texel,
    scene::{BoxScene, SceneObject},
    types::{Color, Error, Float, Point, RResult, Vector, Vectorx, RF},
};

use eframe::egui::Key;
use egui::{
    emath::RectTransform, pos2, vec2, Align, Button, CentralPanel, Color32, Context,
    KeyboardShortcut, Layout, Modifiers, NumExt, Pos2, ProgressBar, Rect, RichText, ScrollArea,
    Sense, SidePanel, TextStyle, TextureOptions, TopBottomPanel, Ui, ViewportBuilder,
    ViewportCommand, Visuals, Widget, WidgetText,
};
use egui_file_dialog::FileDialog;
use egui_phosphor::regular as icon;
use pest::Parser;

pub struct RustRayGui<F: Float> {
    engine: RenderEngine<F>,
    lock: Arc<RwLock<BoxScene<F>>>,
    obj: Option<usize>,
    obj_last: Option<usize>,
    file_dialog: FileDialog,
    vtracer: VisualTraceWidget,
}

impl<F: Float + Texel + From<f32>> RustRayGui<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        engine: RenderEngine<F>,
        scene: BoxScene<F>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

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
            lock: Arc::new(RwLock::new(scene)),
            obj: None,
            obj_last: None,
            file_dialog: FileDialog::new().show_devices(false),
            vtracer: VisualTraceWidget::new(),
        }
    }

    fn find_obj<'a>(&self, scene: &'a mut BoxScene<F>) -> Option<&'a mut dyn Geometry<F>> {
        scene
            .objects
            .iter_mut()
            .find(|obj| obj.get_id() == self.obj)
            .map(|m| m as &mut _)
    }

    fn render_preview(&mut self) {
        self.engine.render_all_by_step(&self.lock, 4, 4);
    }

    fn update_top_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open..").clicked() {
                    self.file_dialog.select_file();
                }
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

    fn update_side_panel(&mut self, _ctx: &Context, ui: &mut Ui, scene: &mut BoxScene<F>) {
        ui.label(RichText::new("RustRay").heading().strong());

        ScrollArea::vertical().show(ui, |ui| {
            let mut changed = false;

            controls::collapsing_group("Materials", icon::ARTICLE_MEDIUM).show(ui, |ui| {
                let mat_keys = scene.materials.mats.keys().copied().sorted();

                for (idx, id) in mat_keys.into_iter().enumerate() {
                    let mat = scene.materials.mats.get_mut(&id).unwrap();
                    let name = format!("{} Material {idx}: {}", mat.get_icon(), mat.get_name());
                    controls::property_list(&name, ui, |ui| {
                        changed |= mat.ui(ui);
                    });
                }
            });

            controls::collapsing_group("Objects", icon::SHAPES).show(ui, |ui| {
                scene.objects.iter_mut().enumerate().for_each(|(i, obj)| {
                    let name = format!("{} Object {i}: {}", obj.get_icon(), obj.get_name());
                    let obj_id = obj.get_id();
                    let proplist = controls::CustomCollapsible::new(name.clone());
                    let (response, _header_response, _body_response) = proplist.show(
                        ui,
                        |pl, ui| {
                            let text = WidgetText::from(name);
                            let available = ui.available_rect_before_wrap();
                            let text_pos = available.min;
                            let wrap_width = available.right() - text_pos.x;
                            let galley =
                                text.into_galley(ui, Some(false), wrap_width, TextStyle::Button);
                            let text_max_x = text_pos.x + galley.size().x;
                            let desired_width = text_max_x + available.left();
                            let desired_size = vec2(desired_width, galley.size().y)
                                .at_least(ui.spacing().interact_size);
                            let rect = ui.allocate_space(desired_size).1;
                            let header_response = ui.interact(rect, pl.id, Sense::click());
                            if header_response.clicked() {
                                pl.toggle();
                            }
                            let visuals = ui.style().interact_selectable(&header_response, false);
                            ui.painter().galley(text_pos, galley, visuals.text_color());

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let button =
                                    Button::new(format!("{} Select", icon::CROSSHAIR_SIMPLE))
                                        .selected(self.obj == obj_id)
                                        .ui(ui);

                                if button.clicked() {
                                    if self.obj == obj_id {
                                        info!("deselect: {:?} {:?}", self.obj, obj_id);
                                        self.obj = None;
                                    } else {
                                        self.obj = obj_id;
                                    }
                                }
                                ui.end_row();
                            });
                        },
                        |ui| {
                            if let Some(interactive) = obj.get_interactive() {
                                changed |= interactive.ui(ui);
                            } else {
                                ui.label("Non-interactive object :(");
                            }
                        },
                    );

                    if self.obj == obj.get_id() && self.obj != self.obj_last {
                        response.highlight().scroll_to_me(Some(Align::Center));
                    }
                });
            });

            controls::collapsing_group("Lights", icon::LIGHTBULB).show(ui, |ui| {
                scene.lights.iter_mut().enumerate().for_each(|(i, light)| {
                    let name = format!("{} Light {i}: {}", light.get_icon(), light.get_name());
                    controls::property_list(&name, ui, |ui| {
                        if let Some(interactive) = light.get_interactive() {
                            changed |= interactive.ui(ui);
                        } else {
                            ui.label("Non-interactive light :(");
                        }
                    });
                });
            });

            controls::collapsing_group("Cameras", icon::VIDEO_CAMERA).show(ui, |ui| {
                scene.cameras.iter_mut().enumerate().for_each(|(i, cam)| {
                    let name = format!("{} Camera {i}", cam.get_icon());
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

    fn context_menu_add_geometry(ui: &mut egui::Ui, scene: &mut BoxScene<F>) -> bool {
        let mut res = false;

        macro_rules! add_geometry_option {
            ($name:ident, $code:block) => {
                if ui
                    .button(format!(
                        "{} {}",
                        $name::<_, Color<F>>::ICON,
                        stringify!($name)
                    ))
                    .clicked()
                {
                    scene.objects.push(Box::new($code));
                    res = true;
                }
            };
        }

        add_geometry_option!(Cube, { Cube::new(Matrix4::identity(), Phong::white()) });

        add_geometry_option!(Cone, {
            Cone::new(
                F::ONE,
                F::ZERO,
                F::ONE,
                true,
                Matrix4::identity(),
                Phong::white(),
            )
        });

        add_geometry_option!(Cylinder, {
            Cylinder::new(Matrix4::identity(), true, Phong::white())
        });

        add_geometry_option!(Sphere, {
            Sphere::place(Vector::ZERO, F::ONE, Phong::white())
        });

        add_geometry_option!(Square, { Square::new(Matrix4::identity(), Phong::white()) });

        res
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, scene: &mut BoxScene<F>) {
        ui.menu_button("Add geometry", |ui| {
            if Self::context_menu_add_geometry(ui, scene) {
                scene.recompute_bvh().unwrap();
                self.render_preview();
                ui.close_menu();
            }
        });
        if ui
            .checkbox(&mut self.vtracer.enabled, "Ray trace debugger")
            .changed()
        {
            ui.close_menu();
        }
    }

    fn update_center_panel(&mut self, ctx: &Context, ui: &mut Ui, scene: &mut BoxScene<F>) {
        let img = self.engine.get_epaint_image();

        let tex = ui.ctx().load_texture("foo", img, TextureOptions::LINEAR);

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::drag());
        painter.image(
            (&tex).into(),
            painter.clip_rect(),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        self.vtracer.draw(&painter);

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
            if self.vtracer.enabled {
                let coord = from_screen.transform_pos(pos);
                self.vtracer.set_coord(coord);
            }
        }
        self.vtracer.update(scene, &to_screen);

        act.context_menu(|ui| self.context_menu(ui, scene));

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

        let resdir = path
            .parent()
            .ok_or(Error::ParseError("Invalid filename".into()))?;

        let name = path
            .to_str()
            .ok_or(Error::ParseError("Invalid UTF-8 filename".into()))?;
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

        // render (hq, normals)
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.engine.render_all(&self.lock);
        }

        if ctx.input(|i| i.key_pressed(Key::T)) {
            self.engine.render_normals(&self.lock);
        }

        //
        if ctx.input(|i| i.key_pressed(Key::F)) {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        }

        // file dialog
        if ctx.input(|i| i.key_pressed(Key::O)) {
            self.file_dialog.select_file();
        }

        // vtracer controls
        if ctx.input(|i| i.key_pressed(Key::D)) {
            self.vtracer.toggle();
        }

        if ctx.input(|i| i.key_pressed(Key::D) && i.modifiers.shift) {
            self.vtracer.clear();
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

        TopBottomPanel::top("top_panel").show(ctx, |ui| self.update_top_panel(ctx, ui));

        let lock = Arc::clone(&self.lock);
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

    Ok(eframe::run_native(
        "Rustray",
        native_options,
        Box::new(move |cc| {
            Box::new({
                let engine = RenderEngine::new(width, height);

                let mut app = RustRayGui::new(cc, engine, scene);
                app.engine.render_lines(&app.lock, 0, height);

                app
            })
        }),
    )?)
}
