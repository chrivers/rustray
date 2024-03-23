use crate::geometry::{FiniteGeometry, Geometry};
use crate::point;
use crate::scene::{Light, RayTracer, Scene};
use crate::types::ray::{Maxel, Ray};
use crate::types::vector::Vectorx;
use crate::types::{Camera, Color, Float, Point};
use cgmath::MetricSpace;

pub struct Tracer<'a, F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> {
    scene: &'a Scene<F, B, G, L>,
    lights: Vec<&'a dyn Light<F>>,
    sx: u32,
    sy: u32,
    background: Color<F>,
    maxlvl: u32,
}

impl<'a, F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> Tracer<'a, F, B, G, L> {
    pub fn new(scene: &'a Scene<F, B, G, L>) -> Self {
        let lights = scene.lights.iter().map(|x| x as &dyn Light<F>).collect();
        Self {
            scene,
            lights,
            sx: 2,
            sy: 2,
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
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
                    colors += color.clamped()
                } else {
                    colors += self.background
                }
            }
        }
        colors / (fsx * fsy)
    }

    pub fn generate_span(&self, camera: &Camera<F>, y: u32) -> Vec<Color<F>> {
        let (xres, yres) = camera.size();
        let fx = F::from_u32(xres);
        let fy = F::from_u32(yres);
        let py = F::from_u32(y);
        (0..xres)
            .map(|x| {
                let px = F::from_u32(x);
                self.render_pixel(camera, px / fx, py / fy, fx, fy)
            })
            .collect()
    }

    pub fn scene(&self) -> &Scene<F, B, G, L> {
        self.scene
    }
}

impl<'a, F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> RayTracer<F>
    for Tracer<'a, F, B, G, L>
{
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

        Some(maxel.mat.render(&mut maxel, &self.lights, self))
    }

    fn ambient(&self) -> Color<F> {
        self.scene.ambient
    }

    fn background(&self) -> Color<F> {
        self.background
    }
}
