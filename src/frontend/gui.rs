use cgmath::{Deg, Matrix4, SquareMatrix};
use itertools::Itertools;
use obj::Obj;

use std::{
    ffi::OsStr,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    engine::{RenderEngine, RenderJob},
    format::sbt2::{Rule as SbtRule, SbtBuilder, SbtParser2},
    geometry::{Cone, Cube, Cylinder, FiniteGeometry, Geometry, Sphere, Square},
    gui::{
        controls::{self, Canvas},
        gizmo,
        visualtrace::VisualTraceWidget,
        IconButton,
    },
    light::{AreaLight, Attenuation, DirectionalLight, PointLight, SpotLight},
    point,
    sampler::Texel,
    scene::{BoxScene, Interactive, SceneObject},
    types::{Color, Error, Float, Point, RResult, Vector, Vectorx, RF},
};

use parking_lot::RwLock;

use eframe::{egui::Key, CreationContext};
use egui::{
    vec2, Align, Button, CentralPanel, Context, KeyboardShortcut, Layout, Modifiers, NumExt,
    ProgressBar, RichText, ScrollArea, Sense, SidePanel, TextStyle, TopBottomPanel, Ui,
    ViewportBuilder, ViewportCommand, Visuals, Widget, WidgetText,
};
use egui_file_dialog::FileDialog;
use egui_phosphor::regular as icon;
use pest::Parser;

pub struct RenderModes<F: Float> {
    default: RenderJob<F>,
    preview: RenderJob<F>,
    normals: RenderJob<F>,
}

pub struct RustRayGui<F: Float> {
    engine: RenderEngine<F>,
    paths: Vec<PathBuf>,
    pathindex: usize,
    lock: Arc<RwLock<BoxScene<F>>>,
    obj: Option<usize>,
    obj_last: Option<usize>,
    file_dialog: FileDialog,
    ray_debugger: VisualTraceWidget,
    bounding_box: VisualTraceWidget,
    canvas: Canvas,
    render_modes: RenderModes<F>,
}

impl<F: Float + Texel + From<f32>> RustRayGui<F>
where
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &CreationContext<'_>, engine: RenderEngine<F>, paths: Vec<PathBuf>) -> Self {
        // load fonts
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        // tweak visuals
        let visuals = Visuals {
            slider_trailing_fill: true,
            ..Visuals::default()
        };
        cc.egui_ctx.set_visuals(visuals);

        let lock = Arc::new(RwLock::new(BoxScene::empty()));

        let render_modes = RenderModes {
            default: RenderJob::new(),
            preview: RenderJob::new()
                .with_mult(3)
                .with_ray_flags(RF::Preview.into()),
            normals: RenderJob::new().with_func_debug_normals(),
        };

        // construct gui
        Self {
            engine,
            paths,
            pathindex: 0,
            lock,
            obj: None,
            obj_last: None,
            file_dialog: FileDialog::new().show_devices(false),
            ray_debugger: VisualTraceWidget::new(),
            bounding_box: VisualTraceWidget::new(),
            canvas: Canvas::new("canvas"),
            render_modes,
        }
    }

    fn find_obj<'a>(&self, scene: &'a mut BoxScene<F>) -> Option<&'a mut dyn Geometry<F>> {
        scene
            .root
            .iter_mut()
            .find(|obj| obj.get_id() == self.obj)
            .map(|m| m as &mut _)
    }

    fn change_obj(&self, scene: &mut BoxScene<F>, func: impl Fn(&mut Box<dyn FiniteGeometry<F>>)) {
        scene
            .root
            .iter_mut()
            .find(|obj| obj.get_id() == self.obj)
            .map(func);
    }

    fn delete_current_obj(&mut self, scene: &mut BoxScene<F>) {
        let Some(id) = self.obj else {
            return;
        };
        scene.root.del_object(id);
        self.obj = None;
        self.bounding_box.clear();

        scene.recompute_bvh().unwrap();
        self.engine.submit(&self.render_modes.preview, &self.lock);
    }

    fn update_top_panel(&mut self, ctx: &Context, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open..").clicked() {
                    self.file_dialog.select_file();
                    ui.close_menu();
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
                scene.root.iter_mut().enumerate().for_each(|(i, obj)| {
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
                self.engine.submit(&self.render_modes.preview, &self.lock);
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
                    .icon_button($name::<F>::ICON, stringify!($name))
                    .clicked()
                {
                    scene.root.add_object(Box::new($code));
                    res = true;
                }
            };
        }

        add_geometry_option!(Cube, {
            Cube::new(Matrix4::identity(), scene.materials.default())
        });

        add_geometry_option!(Cone, {
            Cone::new(
                F::ONE,
                F::ZERO,
                F::ONE,
                true,
                Matrix4::identity(),
                scene.materials.default(),
            )
        });

        add_geometry_option!(Cylinder, {
            Cylinder::new(Matrix4::identity(), true, scene.materials.default())
        });

        add_geometry_option!(Sphere, {
            Sphere::place(Vector::ZERO, F::ONE, scene.materials.default())
        });

        add_geometry_option!(Square, {
            Square::new(Matrix4::identity(), scene.materials.default())
        });

        res
    }

    fn context_menu_add_light(ui: &mut egui::Ui, scene: &mut BoxScene<F>) -> bool {
        let mut res = false;

        macro_rules! add_light_option {
            ($name:ident, $code:block) => {
                if ui
                    .icon_button($name::<F>::ICON, stringify!($name))
                    .clicked()
                {
                    scene.add_light($code);
                    res = true;
                }
            };
        }

        let attn = Attenuation {
            a: F::ZERO,
            b: F::ONE,
            c: F::ZERO,
        };

        add_light_option!(PointLight, {
            PointLight::new(Vector::ZERO, attn, Color::WHITE)
        });

        add_light_option!(DirectionalLight, {
            DirectionalLight::new(-Vector::UNIT_Z, Color::WHITE)
        });

        add_light_option!(SpotLight, {
            SpotLight {
                attn,
                umbra: Deg(F::from_u32(45)).into(),
                penumbra: Deg(F::from_u32(60)).into(),
                pos: Vector::ZERO,
                dir: -Vector::UNIT_Z,
                color: Color::WHITE,
            }
        });

        add_light_option!(AreaLight, {
            AreaLight::new(
                attn,
                Vector::ZERO,
                -Vector::UNIT_Z,
                Vector::UNIT_Y,
                Color::WHITE,
                F::from_u32(10),
                F::from_u32(10),
            )
        });

        res
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, scene: &mut BoxScene<F>) {
        if let Some(obj) = self.obj {
            ui.horizontal(|ui| {
                ui.label(format!("Object id: {obj:x}"));
                ui.add_space(50.0);
            });

            if ui.icon_button(icon::TRASH, "Delete").clicked() {
                self.delete_current_obj(scene);
                ui.close_menu();
            }

            ui.icon_menu_button(icon::ARTICLE_MEDIUM, "Set material", |ui| {
                let mut mat_id = None;
                for (id, mat) in &scene.materials.mats {
                    let button = ui.button(format!("{id:?} {}", mat.get_name()));
                    if button.clicked() {
                        mat_id = Some(*id);
                        info!("Select material {mat_id:?}");
                        ui.close_menu();
                    }
                    if button.hovered() {
                        mat_id = Some(*id);
                    }
                }

                if let Some(id) = mat_id {
                    self.change_obj(scene, move |obj| {
                        if let Some(mat) = obj.material() {
                            mat.set_material(id);
                        }
                    });
                    self.engine.submit(&self.render_modes.preview, &self.lock);
                }
            });
            ui.separator();
        }

        ui.icon_menu_button(icon::PLUS_SQUARE, "Add geometry", |ui| {
            if Self::context_menu_add_geometry(ui, scene) {
                scene.recompute_bvh().unwrap();
                self.engine.submit(&self.render_modes.preview, &self.lock);
                ui.close_menu();
            }
        });

        ui.icon_menu_button(icon::PLUS_CIRCLE, "Add light", |ui| {
            if Self::context_menu_add_light(ui, scene) {
                self.engine.submit(&self.render_modes.preview, &self.lock);
                ui.close_menu();
            }
        });

        if ui
            .checkbox(&mut self.ray_debugger.enabled, "Ray trace debugger")
            .changed()
        {
            ui.close_menu();
        }
    }

    fn update_center_panel(&mut self, ctx: &Context, ui: &mut Ui, scene: &mut BoxScene<F>) {
        let img = self.engine.get_epaint_image();

        let cvs = self.canvas.show(ui, img);

        self.ray_debugger.draw(&cvs.inner.painter);
        self.bounding_box.draw(&cvs.inner.painter);

        let to_screen = cvs.inner.to_screen;
        let from_screen = cvs.inner.from_screen;
        let response = cvs.response;

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
                self.bounding_box.clear();
            }
        }
        self.obj_last = self.obj;

        if let Some(pos) = act.hover_pos() {
            if self.ray_debugger.enabled {
                let coord = from_screen.transform_pos(pos);
                self.ray_debugger.set_coord(coord);
            }
        }
        self.ray_debugger.update(scene, &to_screen);
        self.bounding_box.update(scene, &to_screen);

        act.context_menu(|ui| self.context_menu(ui, scene));

        let camera = scene.cameras[0];

        let mut aabb: Option<rtbvh::Aabb> = None;
        if let Some(obj) = self.find_obj(scene) {
            if let Some(int) = obj.get_interactive() {
                aabb = int.ui_bounding_box().copied();
                if int.ui_center(ui, &camera, &response.rect) {
                    scene.recompute_bvh().unwrap();
                    self.engine.submit(&self.render_modes.preview, &self.lock);
                }
            }
        }

        // if the selected object has aabb info, render it
        if let Some(aabb) = aabb {
            self.bounding_box.aabb(scene, &to_screen, &aabb);
        }

        let progress = self.engine.progress();
        ui_progress(ctx, ui, progress);
    }

    fn load_scene_from_file(path: &Path, scene: &mut BoxScene<F>) -> RResult<()> {
        let resdir = path.parent().ok_or(Error::InvalidFilename(path.into()))?;

        let name = path.to_str().ok_or(Error::InvalidFilename(path.into()))?;

        match path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename(path.into()))?
            .to_lowercase()
            .as_str()
        {
            "ray" => {
                let data = std::fs::read_to_string(path)?;
                let p = SbtParser2::parse(SbtRule::program, &data)
                    .map_err(|err| err.with_path(name))?;
                let p = SbtParser2::ast(p)?;
                SbtBuilder::new(resdir, scene).build(p)?;
            }
            "obj" => {
                let obj = Obj::load(path)?;
                crate::format::obj::load(obj, scene)?;
                scene.add_camera_if_missing()?;
                scene.add_light_if_missing()?;
            }
            "ply" => {
                let mut reader = BufReader::new(std::fs::File::open(path)?);
                crate::format::ply::PlyParser::parse_file(&mut reader, scene)?;
                scene.add_camera_if_missing()?;
                scene.add_light_if_missing()?;
            }
            other => Err(Error::UnknownFileExtension(other.into()))?,
        }

        Ok(())
    }

    fn load_file(&mut self, path: &Path) -> RResult<()> {
        info!("Loading file: {path:?}");

        if let Some(parent) = path.parent() {
            self.file_dialog.config_mut().initial_directory = parent.to_path_buf();
        }

        let mut scene = self.lock.write();
        scene.clear();
        if let Err(e) = Self::load_scene_from_file(path, &mut scene) {
            let _ = scene.add_camera_if_missing();
            return Err(e);
        }
        drop(scene);

        self.engine.submit(&self.render_modes.default, &self.lock);

        Ok(())
    }

    fn load_index(&mut self, mut index: usize) -> RResult<()> {
        index %= self.paths.len();
        self.pathindex = index;
        let path = self.paths[index].clone();
        self.load_file(&path)
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
            self.engine.submit(&self.render_modes.default, &self.lock);
        }

        if ctx.input(|i| i.key_pressed(Key::T)) {
            self.engine.submit(&self.render_modes.normals, &self.lock);
        }

        //
        if ctx.input(|i| i.key_pressed(Key::F)) {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
        }

        // file dialog
        if ctx.input(|i| i.key_pressed(Key::O)) {
            self.file_dialog.select_file();
        }

        if ctx.input(|i| i.key_pressed(Key::PageDown)) {
            let _ = self.load_index(self.pathindex + 1);
        }

        if ctx.input(|i| i.key_pressed(Key::PageUp)) {
            let _ = self.load_index(self.pathindex - 1);
        }

        // vtracer controls
        if ctx.input(|i| i.key_pressed(Key::D)) {
            self.ray_debugger.toggle();
        }

        if ctx.input(|i| i.key_pressed(Key::D) && i.modifiers.shift) {
            self.ray_debugger.clear();
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
            self.load_file(&path).unwrap();
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| self.update_top_panel(ctx, ui));

        let lock = Arc::clone(&self.lock);
        let mut scene = lock.write();

        // object control
        if ctx.input(|i| i.key_pressed(Key::Delete)) {
            self.delete_current_obj(&mut scene);
        }

        SidePanel::left("Scene controls")
            .resizable(true)
            .show(ctx, |ui| self.update_side_panel(ctx, ui, &mut scene));

        CentralPanel::default().show(ctx, |ui| self.update_center_panel(ctx, ui, &mut scene));
    }
}

pub fn run<F>(paths: Vec<PathBuf>, width: u32, height: u32) -> RResult<()>
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

                let mut app = RustRayGui::new(cc, engine, paths);
                app.load_index(0).unwrap();

                app
            })
        }),
    )?)
}
