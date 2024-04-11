use std::sync::RwLockWriteGuard;

use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};

use crate::debug::tracer::DebugTracer;
use crate::scene::{BoxScene, RayTracer};
use crate::types::Camera;
use crate::{point, Float, Point, Vector, Vectorx};

fn color_square(b: Pos2) -> Rect {
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
    pub fn to_screen(&self, a: Pos2, b: Pos2) -> (Pos2, Pos2) {
        (
            self.to_screen.transform_pos(a),
            self.to_screen.transform_pos(b),
        )
    }

    #[must_use]
    pub fn line(&self, a: Vector<F>, b: Vector<F>) -> (Pos2, Pos2) {
        let a: Pos2 = self.camera.world_to_ndc(a).point().into();
        let b: Pos2 = self.camera.world_to_ndc(b).point().into();
        self.to_screen(a, b)
    }

    #[must_use]
    pub fn normal(&self, pos: Vector<F>, dir: Vector<F>) -> (Pos2, Pos2) {
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
}

#[must_use]
pub fn make_shapes<F>(
    scene: &RwLockWriteGuard<BoxScene<F>>,
    coord: Pos2,
    to_screen: &RectTransform,
) -> Vec<Shape>
where
    F: Float + From<f32>,
{
    const TRACE_STEPS: u16 = 7;

    let ray = scene.cameras[0]
        .get_ray(point!(coord.x, coord.y))
        .with_debug();

    let dt = DebugTracer::new(scene, TRACE_STEPS);
    dt.ray_trace(&ray);

    let mut shapes = vec![];
    let cam = &scene.cameras[0];

    let vt = VisualTracer::new(to_screen, cam);

    #[allow(clippy::significant_drop_in_scrutinee)]
    for step in dt.steps.borrow().iter() {
        let ray = &step.ray;

        let (a, b) = match step.maxel {
            Some(maxel) => vt.line(ray.pos, maxel.pos),
            None => vt.normal(ray.pos, ray.dir),
        };

        let color = if step.shadow {
            if step.maxel.is_some() {
                Color32::DARK_RED
            } else {
                Color32::DARK_GREEN
            }
        } else {
            Color32::from_gray(255_i32.saturating_sub(50 * ray.lvl as i32) as u8)
        };

        if ray.lvl != 0 {
            let shape = egui::Shape::line_segment([a, b], Stroke::new(2.0, color));
            shapes.push(shape);
        }

        let Some(mut maxel) = step.maxel else {
            shapes.push(egui::Shape::circle_filled(b, 3.0, Color32::GREEN));
            continue;
        };

        shapes.push(egui::Shape::circle_filled(
            b,
            5.0,
            if step.shadow {
                Color32::RED
            } else {
                Color32::BLUE
            },
        ));

        let (a, b) = vt.normal(maxel.pos, maxel.nml());

        let shape = egui::Shape::line_segment([b, a], Stroke::new(1.0, Color32::BLUE));
        shapes.push(shape);

        if let Some(color) = step.color {
            let color_sample = color_square(b);
            shapes.push(egui::Shape::rect_filled(
                color_sample,
                Rounding::ZERO,
                Color32::from(color),
            ));
            shapes.push(egui::Shape::rect_stroke(
                color_sample,
                Rounding::ZERO,
                Stroke::new(2.0, Color32::WHITE),
            ));
        }
    }

    shapes
}
