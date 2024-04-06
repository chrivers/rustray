use std::sync::RwLockWriteGuard;

use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};

use crate::debug::tracer::DebugTracer;
use crate::scene::{BoxScene, RayTracer};
use crate::{point, Float, Point};

#[must_use]
pub fn make_shapes<F>(
    scene: &RwLockWriteGuard<BoxScene<F>>,
    coord: Pos2,
    to_screen: &RectTransform,
) -> Vec<Shape>
where
    F: Float + From<f32>,
{
    const TRACE_STEPS: u32 = 20;

    let ray = scene.cameras[0]
        .get_ray(point!(coord.x, coord.y), TRACE_STEPS)
        .with_debug();

    let dt = DebugTracer::new(scene);
    dt.ray_trace(&ray);

    let mut shapes = vec![];
    let cam = &scene.cameras[0];

    #[allow(clippy::significant_drop_in_scrutinee)]
    for step in dt.steps.borrow().iter() {
        let ray = &step.ray;

        let ax = cam.ndc.pos(cam.projection.pos(cam.model.pos(ray.pos)));
        let end_pos = step
            .maxel
            .map(|m| m.pos)
            .unwrap_or_else(|| ray.pos + ray.dir);
        let bx = cam.ndc.pos(cam.projection.pos(cam.model.pos(end_pos)));

        /* ax.y = -ax.y; */
        /* bx.y = -bx.y; */

        /* ax.x += F::ONE; */
        /* ax.y += F::ONE; */
        /* bx.x += F::ONE; */
        /* bx.y += F::ONE; */

        /* ax *= F::HALF; */
        /* bx *= F::HALF; */

        let a = Pos2 {
            x: ax.x.to_f32().unwrap(),
            y: ax.y.to_f32().unwrap(),
        };
        let mut b = Pos2 {
            x: bx.x.to_f32().unwrap(),
            y: bx.y.to_f32().unwrap(),
        };

        if step.maxel.is_none() {
            let n = (b - a).normalized();
            b = a + n * 0.05;
        }

        let a = to_screen.transform_pos(a);
        let b = to_screen.transform_pos(b);
        let color = if step.shadow {
            if step.maxel.is_some() {
                Color32::DARK_RED
            } else {
                Color32::DARK_GREEN
            }
        } else {
            Color32::from_gray(255_i32.saturating_sub(50 * (TRACE_STEPS - ray.lvl) as i32) as u8)
        };

        if ray.lvl != TRACE_STEPS {
            let shape = egui::Shape::line_segment([a, b], Stroke::new(2.0, color));
            shapes.push(shape);
        }
        /* let q = TRACE_STEPS - ray.lvl; */
        /* let shape = egui::Shape::dashed_line(&[a, b], Stroke::new(2.0, color), q as f32, q as f32 + 1.0); */
        /* shapes.extend(shape); */

        if let Some(mut maxel) = step.maxel {
            shapes.push(egui::Shape::circle_filled(
                b,
                5.0,
                if step.shadow {
                    Color32::RED
                } else {
                    Color32::BLUE
                },
            ));

            let ax = cam.ndc.pos(cam.projection.pos(cam.model.pos(maxel.pos)));
            let bx = cam.ndc.pos(cam.projection.pos(cam.model.pos(maxel.pos + maxel.nml() / F::FOUR )));
            let a = Pos2 {
                x: ax.x.to_f32().unwrap(),
                y: ax.y.to_f32().unwrap(),
            };
            let b = Pos2 {
                x: bx.x.to_f32().unwrap(),
                y: bx.y.to_f32().unwrap(),
            };
            let a = to_screen.transform_pos(a);
            let b = to_screen.transform_pos(b);
            let shape = egui::Shape::line_segment([b, a], Stroke::new(1.0, Color32::BLUE));
            shapes.push(shape);

            if let Some(color) = step.color {
                let color_sample = Rect {
                    min: Pos2 {
                        x: b.x + 20.0,
                        y: b.y - 15.0,
                    },
                    max: Pos2 {
                        x: b.x + 50.0,
                        y: b.y + 15.0,
                    },
                };
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
        } else {
            shapes.push(egui::Shape::circle_filled(b, 3.0, Color32::GREEN));
        };
    }

    shapes
}
