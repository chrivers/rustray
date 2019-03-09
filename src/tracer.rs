#![allow(dead_code)]

use crate::traits::Float;
use image::{GenericImage, Pixel, Primitive};

use crate::vector::Vector;
use crate::light::Light;
use crate::color::Color;
use crate::camera::Camera;
use crate::scene::RayTarget;
use crate::ray::Ray;
use crate::point::Point;

//#[derive(Clone)]
pub struct Tracer<F: Float>
{
    camera: Camera<F>,
    objects: Vec<Box<dyn RayTarget<F>>>,
    lights: Vec<Light<F>>,
}

impl<F: Float> Tracer<F>
{
    pub fn new(camera: Camera<F>, objects: Vec<Box<RayTarget<F>>>, lights: Vec<Light<F>>) -> Tracer<F>
    {
        Tracer { camera: camera, objects: objects, lights: lights }
    }

    fn render_pixel(&self, point: Point<F>) -> Option<Color<F>>
    {
        let ray = self.camera.get_ray(point);
        let mut dist = F::max_value();
        let mut hit: Option<Vector<F>> = None;
        let mut obj: Option<&Box<RayTarget<F>>> = None;
        for curobj in &self.objects
        {
            if let Some(curhit) = curobj.ray_hit(&ray)
            {
                let curdist = curhit.length_to(self.camera.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                    obj = Some(curobj);
                }
            }
        }
        if hit.is_none()
        {
            return None;
        }
        let mut res = Color::<F>::black();
        let obj = obj.unwrap();
        let hit = hit.unwrap();

        for light in &self.lights
        {
            let mut isblocked = false;
            if cfg!(feature="self_shadowing")
            {
                let light_length = light.pos.vector_to(hit).length();
                let mut hitray = Ray::new(hit, hit.vector_to(light.pos));
                hitray.pos = hitray.pos + hitray.dir * F::small_value();
                for curobj in &self.objects
                {
                    // if !cfg!(self_shadowing) && curobj == obj
                    // {
                    //     continue
                    // }
                    if let Some(curhit) = curobj.ray_hit(&hitray)
                    {
                        if hit.vector_to(curhit).length() < light_length
                        {
                            isblocked = true;
                            break;
                        }
                    }
                }
            }
            if isblocked
            {
                continue
            }
            res = res + obj.trace(&hit, light);
        }
        Some(res)
    }

    pub fn render_image<I, P, S>(&self, target: &mut I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>,
              S: Primitive
    {
        let e_height = F::from_int(if cfg!(feature="antialias") { target.height() * 4 } else { target.height() });
        let e_width  = F::from_int(if cfg!(feature="antialias") { target.width()  * 4 } else { target.width() });
        for y in 0..target.height()
        {
            for x in 0..target.width()
            {
                let color = if cfg!(feature="antialias")
                {
                    let mut colors = [Color::black(); 16];
                    let mut index = 0;
                    for xa in 0..4
                    {
                        for ya in 0..4
                        {
                            let xp: F = F::from_int(x*4 + xa) / e_width;
                            let yp: F = F::from_int(y*4 + ya) / e_height;
                            if let Some(color) = self.render_pixel(Point::new(xp, yp))
                            {
                                colors[index] = color;
                                index += 1;
                            }
                        }
                    }
                    Some(Color::mixed(&colors))
                } else {
                    let xp: F = F::from_int(x) / e_width;
                    let yp: F = F::from_int(y) / e_height;
                    self.render_pixel(Point::new(xp, yp))
                };
                if let Some(color) = color
                {
                    let chans = color.to_array();
                    let pixel = P::from_slice(&chans);
                    target.put_pixel(x, y, *pixel);
                }
            }
        }
    }
}
