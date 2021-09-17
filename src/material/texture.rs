use super::mat_util::*;

use std::marker::PhantomData;
use image::GenericImageView;
use image::{Pixel, Rgb};

use num_traits::ToPrimitive;

#[derive(Copy, Clone)]
pub struct Texture<F: Float, I: GenericImageView + Sync>
{
    _f: PhantomData<F>,
    img: I
}

impl<F: Float, I: GenericImageView + Sync> Texture<F, I>
{
    pub fn new(img: I) -> Self
    {
        Self { _f: PhantomData::<F> {}, img }
    }
}

impl<F: Float, I: GenericImageView + Sync> Material for Texture<F, I>
{
    type F = F;

    fn render(&self, hit: &Hit<F>, maxel: &Maxel<F>, lights: &[Light<F>], rt: &dyn RayTracer<F>, lvl: u32) -> Color<F>
    {
        let x: u32 = (maxel.uv.x.abs() * F::from_u32(256)).to_u32().unwrap_or(0) % 256;
        let y: u32 = (maxel.uv.y.abs() * F::from_u32(256)).to_u32().unwrap_or(0) % 256;
        let Rgb([r, g, b]) = self.img.get_pixel(x, y).to_rgb();
        Color::new(
            F::from_f32(r.to_f32().unwrap() / 255.0),
            F::from_f32(g.to_f32().unwrap() / 255.0),
            F::from_f32(b.to_f32().unwrap() / 255.0),
        )
    }
}