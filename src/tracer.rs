use crate::point;
use crate::lib::{Color, Camera, Point, Float};
use crate::lib::ray::{Ray, Hit, Maxel};
use crate::lib::vector::{Vectorx, InnerSpace, MetricSpace};
use crate::scene::{RayTarget, RayTracer, Light, Scene};

pub struct Tracer<'a, F: Float, T: RayTarget<F>, L: Light<F>>
{
    scene: &'a Scene<F, T, L>,
    camera: &'a Camera<F>,
    lights: Vec<&'a dyn Light<F>>,
    sx: u32,
    sy: u32,
    background: Color<F>,
}

impl<'a, F: Float, T: RayTarget<F>, L: Light<F>> Tracer<'a, F, T, L>
{
    pub fn new(scene: &'a Scene<F, T, L>, camera: &'a Camera<F>) -> Self
    {
        let lights = scene.lights.iter().map(|x| (x as &dyn Light<F>)).collect();
        Self { scene, camera, lights, sx: 2, sy: 2, background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)) }
    }

    pub fn render_pixel(&self, px: F, py: F, fx: F, fy: F) -> Color<F>
    {
        let mut colors = Color::black();
        let fsx = F::from_u32(self.sx);
        let fsy = F::from_u32(self.sy);
        for xa in 0..self.sx
        {
            let pixelx = px + F::from_u32(xa) / (fsx * fx);
            for ya in 0..self.sy
            {
                let pixely = py + F::from_u32(ya) / (fsy * fy);
                let ray = self.camera.get_ray(point!(pixelx, pixely));
                if let Some(color) = self.ray_trace(&ray) {
                    colors += color.clamped()
                } else {
                    colors += self.background
                }
            }
        }
        colors / (fsx * fsy)
    }

    pub fn generate_span(&self, y: u32) -> Vec<Color<F>>
    {
        let (xres, yres) = self.camera.size();
        let fx = F::from_u32(xres);
        let fy = F::from_u32(yres);
        let py = F::from_u32(y);
        (0..xres).map(
            |x| {
                let px = F::from_u32(x);
                self.render_pixel(px/fx, py/fy, fx, fy)
            }
        ).collect()
    }
}


impl<'a, F: Float, T: RayTarget<F>, L: Light<F>> RayTracer<F> for Tracer<'a, F, T, L>
{
    fn ray_shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>
    {
        let light_pos = light.get_position();
        let light_length = light_pos.distance2(hit.pos);
        let mut hitray = Ray::new(hit.pos, hit.pos.vector_to(light_pos), 0);
        hitray.pos += hitray.dir * F::BIAS;

        let mut hits: Vec<_> = vec![];
        for curobj in &self.scene.objects
        {
            if let Some(curhit) = curobj.intersect(&hitray)
            {
                if hit.pos.distance2(curhit.pos) < light_length {
                    hits.push(curhit)
                }
            }
        }

        hits.sort_by(|a, b| (a.pos - light_pos).magnitude2().partial_cmp(&(b.pos - light_pos).magnitude2()).unwrap_or(std::cmp::Ordering::Equal) );

        for hit in hits {
            if let Some(col) = maxel.mat.shadow(&hit, maxel, light, self) {
                return Some(col)
            }
        }
        None
    }

    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>
    {
        if ray.lvl > 5 {
            return None;
        }

        let mut dist = F::max_value();
        let mut hit: Option<Hit<F>> = None;

        for curobj in &self.scene.objects
        {
            if let Some(curhit) = curobj.intersect(ray)
            {
                let curdist = self.camera.distance2(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }

        let hit = hit?;

        let maxel = hit.obj.resolve(&hit);

        Some(maxel.mat.render(&hit, &maxel, &self.lights, self))
    }

}
