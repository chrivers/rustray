use crate::point;
use crate::scene::{BoxScene, Light, RayTracer};
use crate::types::ray::{Maxel, Ray};
use crate::types::vector::Vectorx;
use crate::types::{Camera, Color, Float, Point};
use cgmath::MetricSpace;
use std::sync::RwLockReadGuard;

pub struct Tracer<'a, F: Float> {
    scene: RwLockReadGuard<'a, BoxScene<F>>,
    sx: u32,
    sy: u32,
    maxlvl: u32,
}

pub struct RenderSpan<F: Float> {
    pub line: u32,
    pub pixels: Vec<Color<F>>,
}

impl<'a, F: Float> Tracer<'a, F> {
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

    pub fn scene(&self) -> &BoxScene<F> {
        &self.scene
    }
}

impl<'a, F: Float> RayTracer<F> for Tracer<'a, F> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>> {
        let light_pos = light.get_position();
        let hitray = Ray::new(maxel.pos, maxel.pos.normal_to(light_pos), maxel.lvl + 1);

        let mut best_length = light_pos.distance2(maxel.pos);
        let mut best_color = None;

        let mut r = hitray.into();

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
        if ray.lvl > self.maxlvl {
            return None;
        }

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
