use std::fmt::{self, Debug};

use cgmath::MetricSpace;

use crate::engine::RenderSpan;
use crate::light::Lixel;
use crate::material::{ColorDebug, Material};
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

    fn generate_span_map(
        camera: &Camera<F>,
        width: u32,
        height: u32,
        y: u32,
        mult_x: u32,
        span: impl Fn(Ray<F>) -> Color<F>,
    ) -> RenderSpan<F> {
        let size: Point<F> = (width, height).into();
        let mut point: Point<F> = (0, y).into();
        let half_offset = F::from_u32(mult_x) / F::TWO;
        let pixels = (0..width)
            .step_by(mult_x as usize)
            .map(|x| {
                point.x = F::from_u32(x) + half_offset;
                let ray = camera.get_ray(point / size);
                span(ray)
            })
            .collect();

        RenderSpan {
            line: y,
            mult_x,
            mult_y: 1,
            pixels,
        }
    }

    pub fn generate_span_coarse(
        &self,
        camera: &Camera<F>,
        width: u32,
        height: u32,
        y: u32,
        mult_x: u32,
    ) -> RenderSpan<F> {
        Self::generate_span_map(camera, width, height, y, mult_x, |ray| {
            self.ray_trace(&ray)
                .map_or_else(|| self.scene.background, Color::clamped)
        })
    }

    pub fn generate_span(
        &self,
        camera: &Camera<F>,
        width: u32,
        height: u32,
        y: u32,
    ) -> RenderSpan<F> {
        Self::generate_span_map(camera, width, height, y, 1, |ray| {
            self.ray_trace(&ray)
                .map_or_else(|| self.scene.background, Color::clamped)
        })
    }

    pub fn generate_normal_span(
        &self,
        camera: &Camera<F>,
        width: u32,
        height: u32,
        y: u32,
    ) -> RenderSpan<F> {
        Self::generate_span_map(camera, width, height, y, 1, |ray| {
            self.ray_trace_normal(&ray).unwrap_or(Color::BLACK)
        })
    }

    fn ray_trace_normal(&self, ray: &Ray<F>) -> Option<Color<F>> {
        let mut maxel = self.scene.intersect(ray)?;

        Some(ColorDebug::normal().render(&mut maxel, self))
    }
}

impl<'a, F: Float> RayTracer<F> for Tracer<'a, F> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        if maxel.lvl >= self.maxlvl {
            return None;
        }

        let pos = maxel.pos + maxel.nml() * F::BIAS2;
        let hitray = maxel.ray(pos, lixel.dir);

        let mut best_length = lixel.len2;
        let mut best_color = None;

        let mut r = hitray.into();

        #[allow(clippy::significant_drop_in_scrutinee)]
        for (curobj, _ray) in self.scene.bvh.traverse_iter(&mut r, &self.scene.objects) {
            if let Some(mut curhit) = curobj.intersect(&hitray) {
                let cur_length = maxel.pos.distance2(curhit.pos);
                if cur_length > F::BIAS2 && cur_length < best_length {
                    let color = curhit.mat.shadow(&mut curhit, self, lixel);
                    best_color = Some(color);
                    best_length = cur_length;
                }
            }
        }
        best_color
    }

    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>> {
        if ray.lvl >= self.maxlvl {
            return None;
        }

        let mut maxel = self.scene.intersect(ray)?;

        Some(maxel.mat.render(&mut maxel, self))
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
