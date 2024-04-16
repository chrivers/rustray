use egui::emath::RectTransform;
use egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke};

use crate::debug::DebugTracer;
use crate::point;
use crate::scene::BoxScene;
use crate::types::{Camera, Float, Point, Vector, Vectorx};

pub struct VisualTracer<'a, F: Float> {
    to_screen: &'a RectTransform,
    camera: &'a Camera<F>,
    shapes: Vec<Shape>,
}

impl<'a, F: Float> VisualTracer<'a, F> {
    pub const fn new(to_screen: &'a RectTransform, camera: &'a Camera<F>) -> Self {
        Self {
            to_screen,
            camera,
            shapes: vec![],
        }
    }

    #[must_use]
    pub fn color_square(b: Pos2) -> Rect {
        Rect {
            min: Pos2 {
                x: b.x + 20.0,
                y: b.y - 15.0,
            },
            max: Pos2 {
                x: b.x + 50.0,
                y: b.y + 15.0,
            },
        }
    }

    #[must_use]
    pub fn to_screen(&self, a: Pos2, b: Pos2) -> (Pos2, Pos2) {
        (
            self.to_screen.transform_pos(a),
            self.to_screen.transform_pos(b),
        )
    }

    #[must_use]
    pub fn calc_line(&self, a: Vector<F>, b: Vector<F>) -> (Pos2, Pos2) {
        let a: Pos2 = self.camera.world_to_ndc(a).point().into();
        let b: Pos2 = self.camera.world_to_ndc(b).point().into();
        self.to_screen(a, b)
    }

    #[must_use]
    pub fn calc_normal(&self, pos: Vector<F>, dir: Vector<F>) -> (Pos2, Pos2) {
        let end = pos + dir;
        let a: Pos2 = self.camera.world_to_ndc(pos).point().into();
        let b: Pos2 = self.camera.world_to_ndc(end).point().into();
        let n = (b - a).normalized();
        self.to_screen(a, a + n * 0.03)
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<Shape> {
        self.shapes
    }

    pub fn draw_line(&mut self, a: Pos2, b: Pos2, stroke: impl Into<Stroke>) {
        self.shapes.push(Shape::line_segment([a, b], stroke));
    }

    pub fn draw_color_box(&mut self, p: Pos2, color: impl Into<Color32>) {
        let color_sample = Self::color_square(p);
        self.shapes
            .push(Shape::rect_filled(color_sample, Rounding::ZERO, color));
        self.shapes.push(Shape::rect_stroke(
            color_sample,
            Rounding::ZERO,
            Stroke::new(2.0, Color32::WHITE),
        ));
    }

    pub fn dot(&mut self, p: Pos2, size: f32, color: impl Into<Color32>) {
        self.shapes.push(Shape::circle_filled(p, size, color));
    }
}

pub struct VisualTraceWidget {
    pub enabled: bool,
    coord: Option<Pos2>,
    shapes: Vec<Shape>,
}

impl VisualTraceWidget {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            coord: None,
            enabled: false,
            shapes: vec![],
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn clear(&mut self) {
        self.enabled = false;
        self.coord = None;
        self.shapes.clear();
    }

    pub fn set_coord(&mut self, coord: Pos2) {
        self.coord = Some(coord);
    }

    pub fn update<F>(&mut self, scene: &BoxScene<F>, to_screen: &RectTransform)
    where
        F: Float + From<f32>,
    {
        const TRACE_STEPS: u16 = 7;

        let Some(coord) = self.coord else { return };

        let cam = &scene.cameras[0];

        let ray = cam.get_ray(point!(coord.x, coord.y)).with_debug();

        let steps = DebugTracer::trace_single(scene, TRACE_STEPS, &ray);

        let mut vt = VisualTracer::new(to_screen, cam);

        #[allow(clippy::significant_drop_in_scrutinee)]
        for step in &steps {
            let ray = &step.ray;

            let (a, b) = step.maxel.map_or_else(
                || vt.calc_normal(ray.pos, ray.dir),
                |maxel| vt.calc_line(ray.pos, maxel.pos),
            );

            let color = if step.shadow {
                if step.maxel.is_some() {
                    Color32::DARK_RED
                } else {
                    Color32::DARK_GREEN
                }
            } else {
                Color32::from_gray(255_i32.saturating_sub(50 * i32::from(ray.lvl)) as u8)
            };

            if ray.lvl != 0 {
                vt.draw_line(a, b, Stroke::new(2.0, color));
            }

            let Some(mut maxel) = step.maxel else {
                vt.dot(b, 3.0, Color32::GREEN);
                continue;
            };

            vt.dot(
                b,
                5.0,
                if step.shadow {
                    Color32::RED
                } else {
                    Color32::BLUE
                },
            );

            let (a, b) = vt.calc_normal(maxel.pos, maxel.nml());

            vt.draw_line(a, b, Stroke::new(1.0, Color32::BLUE));

            if let Some(color) = step.color {
                vt.draw_color_box(b, Color32::from(color));
            }
        }

        self.shapes = vt.into_inner();
    }

    pub fn draw(&self, painter: &Painter) {
        for shape in &self.shapes {
            painter.add(shape.clone());
        }
    }
}
