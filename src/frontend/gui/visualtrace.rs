use std::sync::RwLockWriteGuard;

use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke};

use crate::debug::tracer::DebugTracer;
use crate::scene::{BoxScene, RayTracer};
use crate::{point, Float, Point, Vectorx};

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

    #[allow(clippy::significant_drop_in_scrutinee)]
    for step in dt.steps.borrow().iter() {
        let ray = &step.ray;

        let a: Pos2 = cam.world_to_ndc(ray.pos).point().into();
        let b: Pos2 = if let Some(maxel) = step.maxel {
            cam.world_to_ndc(maxel.pos).point().into()
        } else {
            let q: Pos2 = cam.world_to_ndc(ray.pos + ray.dir).point().into();
            let n = (q - a).normalized();
            a + n * 0.05
        };

        let a = to_screen.transform_pos(a);
        let b = to_screen.transform_pos(b);
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

        let ax = cam.world_to_ndc(maxel.pos);
        let bx = cam.world_to_ndc(maxel.pos + (maxel.nml() / F::FOUR));
        let a = to_screen.transform_pos(ax.point().into());
        let b = to_screen.transform_pos(bx.point().into());
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
