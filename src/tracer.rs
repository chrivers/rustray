#![allow(dead_code)]
#![allow(unused_imports)]

use std::cell::Cell;
use num::Float;
use image;
use image::{GenericImage, Pixel, Primitive};

use vector::Vector;
use light::Light;
//use ray::Ray;
use scene;
use color::Color;
use camera::Camera;
use scene::RayTarget;

//#[derive(Clone)]
pub struct Tracer<F: Float>
{
    camera: Camera<F>,
    objects: Vec<Box<RayTarget<F>>>,
    lights: Vec<Box<Light<F>>>,
}

impl<F: Float> Tracer<F>
{
    fn render_pixel(&self, x: F, y: F) -> Option<Color<F>>
    {
        let ray = self.camera.get_ray(x, y);
        let mut dist = F::max_value();
        let mut hit: Option<Vector<F>> = None;
        let mut obj: Option<&Box<RayTarget<F>>> = None;
        for curobj in &self.objects
        {
            if let Some(curhit) = curobj.ray_hit(ray)
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

    }

    fn render_image<I, P, S>(&self, mut target: I)
        where I: GenericImage<Pixel=P>,
              P: Pixel<Subpixel=u8>,
              S: Primitive
    {
        for y in 0..target.height()
        {
            for x in 0..target.width()
            {
                let xp: F = F::from(x).unwrap() / F::from(target.width()).unwrap();
                let yp: F = F::from(x).unwrap() / F::from(target.height()).unwrap();
                let color = self.render_pixel(xp, yp);
                let pixel = P::from_channels(0, 0, 0, 0);
                target.put_pixel(x, y, pixel)
            }
        }
    }
}
