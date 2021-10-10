use image::{GenericImage, Pixel};

use crate::lib::{Color, Camera, Point, Float};
use crate::lib::ray::{Ray, Hit, Maxel};
use crate::lib::vector::{Vectorx, InnerSpace, MetricSpace};
use crate::scene::{RayTarget, RayTracer, Light};

pub struct Tracer<F: Float>
{
    camera: Camera<F>,
    objects: Vec<&'a dyn RayTarget<F>>,
    lights: Vec<Box<dyn Light<F>>>,
}

impl<'a, F: Float> Tracer<'a, F>
{
    pub fn new(camera: Camera<F>, objects: Vec<&dyn RayTarget<F>>, lights: Vec<Box<dyn Light<F>>>) -> Tracer<F>
    {
        Tracer { camera, objects, lights }
    }

    fn _render_pixel(&self, point: Point<F>) -> Option<Color<F>>
    {
        let ray = self.camera.get_ray(point);

        self.ray_trace(&ray)
    }

    pub fn render_pixel(&self, px: F, py: F) -> Option<Color<F>>
    {
        if cfg!(feature="antialias")
        {
            const SAMPLES_X: u32 = 2;
            const SAMPLES_Y: u32 = 2;
            let mut colors = [Color::black(); (SAMPLES_X * SAMPLES_Y) as usize];
            let mut index = 0;
            for xa in 0..SAMPLES_X
            {
                for ya in 0..SAMPLES_Y
                {
                    let pixelx = px + F::from_u32(xa) / F::from_u32(SAMPLES_X);
                    let pixely = py + F::from_u32(ya) / F::from_u32(SAMPLES_Y);
                    if let Some(color) = self._render_pixel(Point::new(pixelx, pixely))
                    {
                        colors[index] = color.clamped();
                        index += 1;
                    }
                }
            }
            if index > 0 {
                Some(Color::mixed(&colors))
            } else {
                Some(Color::new(F::zero(), F::zero(), F::from_f32(0.2)))
            }
        } else {
            self._render_pixel(Point::new(px, py))
        }
    }

    fn _render_line<I, P>(&self, y: u32, output_line: u32, target: &mut I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>
    {
        let (xres, yres) = self.camera.size();
        let py = F::from_i32(-(y as i32) + yres as i32 / 2);
        for x in 0..target.width()
        {
            let px = F::from_i32(x as i32 - xres as i32 / 2);
            let color = self.render_pixel(px, py);
            if let Some(color) = color
            {
                let chans = color.to_array();
                let pixel = P::from_slice(&chans);
                target.put_pixel(x, output_line, *pixel);
            }
        }
    }

    pub fn render_line<I, P>(&self, y: u32, target: &mut I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>
    {
        self._render_line(y, y, target)
    }

    pub fn render_span<I, P>(&self, y: u32, target: &mut I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>
    {
        self._render_line(y, 0, target)
    }

    pub fn generate_span(&self, y: u32) -> Vec<Color<F>>
    {
        let (xres, yres) = self.camera.size();
        let py = F::from_i32(-(y as i32) + yres as i32 / 2);
        (0..xres).map(
            |x| {
                let px = F::from_i32(x as i32 - xres as i32 / 2);
                self.render_pixel(px, py).unwrap_or_else(Color::black)
            }
        ).collect()
    }

    pub fn render_image<I, P>(&self, target: &mut I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>
    {
        for y in 0..target.height()
        {
            self.render_line(y, target)
        }
    }
}


impl<'a, F: Float> RayTracer<F> for Tracer<'a, F>
{
    fn ray_shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>
    {
        let light_pos = light.get_position();
        let light_length = light_pos.distance2(hit.pos);
        let mut hitray = Ray::new(hit.pos, hit.pos.vector_to(light_pos), 0);
        hitray.pos += hitray.dir * F::BIAS;

        let mut hits: Vec<_> = vec![];
        for curobj in &self.objects
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

        for curobj in &self.objects
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

        // /* converting to box-less lights: */
        // let lights = &self.lights.iter().map(|x| &**x).collect::<Vec<&dyn Light<F>>>();

        Some(maxel.mat.render(&hit, &maxel, &self.lights, self))
    }

}
