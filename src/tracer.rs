use std::fmt::{self, Debug};

use crate::engine::RenderSpan;
use crate::light::Lixel;
use crate::material::Material;
use crate::scene::{BoxScene, Interactive, RayTracer, SceneObject};
use crate::sceneobject_impl_body;
use crate::types::{Camera, Color, Float, Maxel, Point, Ray};

pub struct Tracer<'a, F: Float> {
    scene: &'a BoxScene<F>,
    sx: u32,
    sy: u32,
    maxlvl: u16,
}

impl<'a, F: Float> Tracer<'a, F> {
    #[must_use]
    pub fn new(scene: &'a BoxScene<F>) -> Self {
        Self {
            scene,
            sx: 2,
            sy: 2,
            maxlvl: 5,
        }
    }

    pub fn render_pixel_single(&self, camera: &Camera<F>, point: Point<F>) -> Color<F> {
        let ray = camera.get_ray(point);
        self.ray_trace(&ray)
            .map_or_else(|| self.scene.background, Color::clamped)
    }

    pub fn render_pixel(&self, camera: &Camera<F>, point: Point<F>, size: Point<F>) -> Color<F> {
        let fs: Point<F> = (self.sx, self.sy).into();
        let mut colors = Color::BLACK;
        let mut offset = Point::ZERO;
        for x in 0..self.sx {
            offset.x = F::from_u32(x) / (fs.x * size.x);
            for y in 0..self.sy {
                offset.y = F::from_u32(y) / (fs.y * size.y);
                colors += self.render_pixel_single(camera, point + offset);
            }
        }
        colors / F::from_u32(self.sx * self.sy)
    }

    pub fn render_line(&self, camera: &Camera<F>, width: u32, y: u32) -> RenderSpan<F> {
        let pixels = (0..width)
            .map(|x| self.render_pixel_single(camera, (x, y).into()))
            .collect();

        RenderSpan {
            line: y,
            mult_x: 1,
            mult_y: 1,
            pixels,
        }
    }
}

impl<'a, F: Float> RayTracer<F> for Tracer<'a, F> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        if maxel.lvl >= self.maxlvl {
            return None;
        }

        let hitray = maxel.shadow_ray(lixel);

        let mut len2 = lixel.len2;

        self.scene
            .root
            .nearest_intersection(&hitray, &mut len2)
            .map(|mut maxel| {
                let mat = &self.scene.materials.mats[&maxel.mat];
                mat.shadow(&mut maxel, self, lixel)
            })
    }

    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>> {
        if ray.lvl >= self.maxlvl {
            return None;
        }

        let mut maxel = self.scene.intersect(ray)?;

        let mat = &self.scene.materials.mats[&maxel.mat];
        Some(mat.render(&mut maxel, self))
    }

    fn scene(&self) -> &BoxScene<F> {
        self.scene
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

impl<'a, F: Float> Interactive<F> for Tracer<'a, F> {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.add(egui::Slider::new(&mut self.sx, 0..=16).text("X supersampling"));
        ui.add(egui::Slider::new(&mut self.sy, 0..=16).text("Y supersampling"));
        /* color_ui(ui, &mut self.color, "Color"); */
        /* ui.end_row(); */

        /* position_ui(ui, &mut self.dir, "Direction"); */
        false
    }
}

impl<'a, F: Float> SceneObject<F> for Tracer<'a, F> {
    sceneobject_impl_body!("Ray tracer", egui_phosphor::regular::LINE_SEGMENTS);
}
