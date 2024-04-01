use crate::engine::RenderSpan;
use crate::point;
use crate::scene::{BoxScene, Interactive, Light, RayTracer, SceneObject};
use crate::types::ray::{Maxel, Ray};
use crate::types::vector::Vectorx;
use crate::types::{Camera, Color, Float, Point};
use cgmath::MetricSpace;
use std::sync::RwLockReadGuard;

use std::fmt::{self, Debug};

pub struct Tracer<'a, F: Float> {
    scene: RwLockReadGuard<'a, BoxScene<F>>,
    sx: u32,
    sy: u32,
    maxlvl: u32,
}

impl<'a, F: Float> Tracer<'a, F> {
    #[must_use]
    pub fn new(scene: RwLockReadGuard<'a, BoxScene<F>>) -> Self {
        Self {
            scene,
            sx: 2,
            sy: 2,
            maxlvl: 5,
        }
    }

    pub fn render_pixel(&self, camera: &Camera<F>, px: F, py: F, fx: F, fy: F) -> Color<F> {
        let mut colors = Color::black();
        let fsx = F::from_u32(self.sx);
        let fsy = F::from_u32(self.sy);
        for xa in 0..self.sx {
            let pixelx = px + F::from_u32(xa) / (fsx * fx);
            for ya in 0..self.sy {
                let pixely = py + F::from_u32(ya) / (fsy * fy);
                let ray = camera.get_ray(point!(pixelx, pixely));
                if let Some(color) = self.ray_trace(&ray) {
                    colors += color.clamped();
                } else {
                    colors += self.scene.background;
                }
            }
        }
        colors / (fsx * fsy)
    }

    pub fn generate_span(&self, camera: &Camera<F>, y: u32) -> RenderSpan<F> {
        let (xres, yres) = camera.size();
        let fx = F::from_u32(xres);
        let fy = F::from_u32(yres);
        let py = F::from_u32(y);
        let pixels = (0..xres)
            .map(|x| {
                let px = F::from_u32(x);
                self.render_pixel(camera, px / fx, py / fy, fx, fy)
            })
            .collect();

        RenderSpan { line: y, pixels }
    }

    #[must_use]
    pub fn scene(&self) -> &BoxScene<F> {
        &self.scene
    }
}

impl<'a, F: Float> RayTracer<F> for Tracer<'a, F> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>> {
        if maxel.lvl == 0 {
            return None;
        }
        let light_pos = light.get_position();
        let hitray = Ray::new(maxel.pos, maxel.pos.normal_to(light_pos), maxel.lvl - 1);

        let mut best_length = light_pos.distance2(maxel.pos);
        let mut best_color = None;

        let mut r = hitray.into();

        #[allow(clippy::significant_drop_in_scrutinee)]
        for (curobj, _ray) in self.scene.bvh.traverse_iter(&mut r, &self.scene.objects) {
            if let Some(curhit) = curobj.intersect(&hitray) {
                let cur_length = maxel.pos.distance2(curhit.pos);
                if cur_length > F::BIAS2 && cur_length < best_length {
                    if let Some(color) = maxel.mat.shadow(maxel, light) {
                        best_color = Some(color);
                        best_length = cur_length;
                    }
                }
            }
        }
        best_color
    }

    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>> {
        let mut maxel = self.scene.intersect(ray)?;

        Some(maxel.mat.render(&mut maxel, self))
    }

    fn ambient(&self) -> Color<F> {
        self.scene.ambient
    }

    fn get_lights(&self) -> &[Box<dyn Light<F>>] {
        &self.scene.lights
    }

    fn background(&self) -> Color<F> {
        self.scene.background
    }
}

impl<'a, F: Float> Debug for Tracer<'a, F> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Tracer")
            .field("scene", &"<scene>")
            .field("sx", &self.sx)
            .field("sy", &self.sy)
            .field("maxlvl", &self.maxlvl)
            .finish()
    }
}

#[cfg(feature = "gui")]
impl<'a, F: Float> Interactive for Tracer<'a, F> {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Ray tracer")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.add(egui::Slider::new(&mut self.sx, 0..=16).text("X supersampling"));
                        ui.add(egui::Slider::new(&mut self.sy, 0..=16).text("Y supersampling"));
                        /* color_ui(ui, &mut self.color, "Color"); */
                        /* ui.end_row(); */

                        /* position_ui(ui, &mut self.dir, "Direction"); */
                    })
            });
    }
}

#[cfg(feature = "gui")]
impl<'a, F: Float> SceneObject for Tracer<'a, F> {
    fn get_name(&self) -> &str {
        "Ray tracer"
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive> {
        Some(self)
    }

    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}
