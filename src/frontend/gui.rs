use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use obj::Obj;

use std::{io::BufReader, sync::Arc, time::Duration};

use crate::{
    engine::{RenderEngine, RenderJob},
    format::sbt2::{Rule as SbtRule, SbtBuilder, SbtParser2},
    geometry::{FiniteGeometry, Geometry},
    gui::{
        context_menu,
        controls::{self, Canvas},
        gizmo,
        visualtrace::VisualTraceWidget,
        IconButton,
    },
    point,
    sampler::Texel,
    scene::{BoxScene, Interactive, SceneObject},
    types::{Error, Float, Point, RResult, RF},
};

use parking_lot::RwLock;

use eframe::{egui::Key, CreationContext};
use egui::{
    CentralPanel, Context, KeyboardShortcut, Modifiers, ProgressBar, RichText, ScrollArea, Sense,
    SidePanel, TopBottomPanel, Ui, ViewportBuilder, ViewportCommand, Visuals,
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
    paths: Vec<Utf8PathBuf>,
    pathindex: usize,
    lock: Arc<RwLock<BoxScene<F>>>,
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
    pub fn new(cc: &CreationContext<'_>, engine: RenderEngine<F>, paths: Vec<Utf8PathBuf>) -> Self {
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
            file_dialog: FileDialog::new().show_devices(false),
            ray_debugger: VisualTraceWidget::new(),
            bounding_box: VisualTraceWidget::new(),
            canvas: Canvas::new("canvas"),
            render_modes,
        }
    }

    fn find_obj<'a>(ui: &Ui, scene: &'a mut BoxScene<F>) -> Option<&'a mut dyn Geometry<F>> {
        let self_obj = ui.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);
        scene
            .root
            .iter_mut()
            .find(|obj| obj.get_id() == self_obj)
            .map(|m| m as &mut _)
    }

    fn change_obj(
        ui: &Ui,
        scene: &mut BoxScene<F>,
        func: impl Fn(&mut Box<dyn FiniteGeometry<F>>),
    ) {
        let self_obj = ui.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);
        scene
            .root
            .iter_mut()
            .find(|obj| obj.get_id() == self_obj)
            .map(func);
    }

    fn delete_current_obj(&mut self, ctx: &Context, scene: &mut BoxScene<F>) {
        let mut self_obj = ctx.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);
        let Some(id) = self_obj else {
            return;
        };
        scene.root.del_object(id);
        self_obj = None;
        self.bounding_box.clear();

        scene.recompute_bvh().unwrap();
        self.engine.submit(&self.render_modes.preview, &self.lock);

        ctx.data_mut(|mem| {
            mem.insert_temp("obj".into(), self_obj);
        });
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

            controls::collapsing_group("Objects", icon::SHAPES).show(ui, |ui| scene.root.ui(ui));

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
                    let name = format!("{} Camera {i}: {}", cam.get_icon(), cam.get_name());
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

    fn context_menu(&mut self, ui: &mut egui::Ui, scene: &mut BoxScene<F>) {
        let self_obj: Option<usize> = ui.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);

        if let Some(obj) = self_obj {
            ui.horizontal(|ui| {
                ui.label(format!("Object id: {obj:x}"));
                ui.add_space(50.0);
            });

            if ui.icon_button(icon::TRASH, "Delete").clicked() {
                self.delete_current_obj(ui.ctx(), scene);
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
                    Self::change_obj(ui, scene, move |obj| {
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
            if context_menu::add_geometry(ui, scene) {
                scene.recompute_bvh().unwrap();
                self.engine.submit(&self.render_modes.preview, &self.lock);
                ui.close_menu();
            }
        });

        ui.icon_menu_button(icon::PLUS_CIRCLE, "Add light", |ui| {
            if context_menu::add_light(ui, scene) {
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

        let mut self_obj = ui.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);

        if act.clicked() {
            if let Some(pos) = act.interact_pointer_pos {
                let coord = from_screen.transform_pos(pos);
                let mut ray = scene.cameras[0].get_ray(point!(coord.x, coord.y));
                ray.flags |= RF::StopAtGroup;
                if let Some(maxel) = scene.intersect(&ray) {
                    let id = maxel.obj.get_id();
                    if self_obj == id {
                        info!("Deselect {:?}", maxel.obj.get_name());
                        self_obj = None;
                    } else {
                        info!("Select {:?}", maxel.obj.get_name());
                        self_obj = id;
                    }
                } else {
                    self_obj = None;
                }
                self.bounding_box.clear();
            }
        }

        ui.data_mut(|mem| {
            mem.insert_temp("obj".into(), self_obj);
            mem.insert_temp("obj_last".into(), self_obj);
        });

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
        if let Some(obj) = Self::find_obj(ui, scene) {
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

    fn load_scene_from_file(path: &Utf8Path, scene: &mut BoxScene<F>) -> RResult<()> {
        let resdir = path.parent().ok_or(Error::InvalidFilename(path.into()))?;

        match path
            .extension()
            .ok_or(Error::InvalidFilename(path.into()))?
            .to_lowercase()
            .as_str()
        {
            "ray" => {
                let data = std::fs::read_to_string(path)?;
                let p = SbtParser2::parse(SbtRule::program, &data)
                    .map_err(|err| err.with_path(path.as_str()))?;
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

    fn load_file(&mut self, path: &Utf8Path) -> RResult<()> {
        info!("Loading file: {path:?}");

        if let Some(parent) = path.parent() {
            self.file_dialog.config_mut().initial_directory = parent.to_path_buf().into();
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
        self.engine.update();

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
            self.load_file(Utf8Path::from_path(&path).unwrap()).unwrap();
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| self.update_top_panel(ctx, ui));

        let lock = Arc::clone(&self.lock);
        let mut scene = lock.write();

        // object control
        if ctx.input(|i| i.key_pressed(Key::Delete)) {
            self.delete_current_obj(ctx, &mut scene);
        }

        SidePanel::left("Scene controls")
            .resizable(true)
            .show(ctx, |ui| self.update_side_panel(ctx, ui, &mut scene));

        CentralPanel::default().show(ctx, |ui| self.update_center_panel(ctx, ui, &mut scene));
    }
}

pub fn run<F>(paths: Vec<Utf8PathBuf>, width: u32, height: u32) -> RResult<()>
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
