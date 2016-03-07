#![allow(dead_code)]
#![allow(unused_imports)]

use num::Float;
use image;
use image::{GenericImage, Pixel, Primitive};

use color::Color;
use camera::Camera;

#[derive(Clone, Copy)]
struct Tracer<F: Float>
{
    camera: Camera<F>
}

impl<F: Float> Tracer<F>
{
    fn render_pixel(&self, x: F, y: F) -> Color<F>
    {
        let ray = self.camera.get_ray(x, y);
        Color::<F>::black()
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
