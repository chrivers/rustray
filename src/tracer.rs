use image::{GenericImage, Pixel};

use crate::light::Light;
use crate::lib::{Color, Camera, Point, Float};
use crate::lib::ray::{Ray, Hit};
use crate::scene::{RayTarget, RayTracer};

pub struct Tracer<'a, F: Float>
{
    camera: Camera<F>,
    objects: Vec<&'a dyn RayTarget<F>>,
    lights: Vec<Light<F>>,
}

impl<'a, F: Float> Tracer<'a, F>
{
    pub fn new(camera: Camera<F>, objects: Vec<&dyn RayTarget<F>>, lights: Vec<Light<F>>) -> Tracer<F>
    {
        Tracer { camera, objects, lights }
    }

    fn _render_pixel(&self, point: Point<F>) -> Option<Color<F>>
    {
        let ray = self.camera.get_ray(point);

        self.ray_trace(&ray, 0)
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
    fn ray_shadow(&self, hit: &Hit<F>, light: &Light<F>) -> bool
    {
        if !cfg!(feature="self_shadowing") {
            return true
        }

        let light_length = light.pos.vector_to(hit.pos).len_sqr();
        let mut hitray = Ray::new(hit.pos, hit.pos.vector_to(light.pos));
        hitray.pos += hitray.dir * F::BIAS;
        for curobj in &self.objects
        {
            if let Some(curhit) = curobj.intersect(&hitray)
            {
                if hit.pos.vector_to(curhit.pos).len_sqr() < light_length
                {
                    return true
                }
            }
        }

        false
    }

    fn ray_trace(&self, ray: &Ray<F>, lvl: u32) -> Option<Color<F>>
    {
        if lvl > 5 {
            return None;
        }

        let mut dist = F::max_value();
        let mut hit: Option<Hit<F>> = None;

        for curobj in &self.objects
        {
            if let Some(curhit) = curobj.intersect(ray)
            {
                let curdist = self.camera.length_to(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }

        let mut res = Color::black();
        let hit = hit?;

        let maxel = hit.obj.resolve(&hit);
        res += maxel.mat.render(&hit, &maxel, &self.lights, self, lvl);

        Some(res)
    }

}
