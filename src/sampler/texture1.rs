use std::marker::PhantomData;
use image::GenericImageView;
use image::{Pixel, Rgb};
use num_traits::ToPrimitive;

use super::{Float, Point, Color, Sampler};

#[derive(Copy, Clone)]
/**
`Texture1` implements [`Sampler<F, F>`] by looking up values in a texture
*/
pub struct Texture1<I: GenericImageView + Sync>
{
    /** Backing image */
    img: I,
}

impl<I: GenericImageView + Sync> Texture1<I>
{
    pub fn new(img: I) -> Self
    {
        Self { img }
    }
}

impl<F: Float, I: GenericImageView + Sync> Sampler<F, F> for Texture1<I>
{
    fn sample(&self, uv: Point<F>) -> F
    {
        let (w, h) = self.img.dimensions();
        let x: u32 = (uv.x * F::from_u32(w)).to_u32().unwrap_or(0) % w;
        let y: u32 = (uv.y * F::from_u32(h)).to_u32().unwrap_or(0) % h;
        let r = self.img.get_pixel(x, y).channels()[0];
        F::from_f32(r.to_f32().unwrap() / 255.0)
    }

    fn dimensions(&self) -> (usize, usize) {
        let wh = self.img.dimensions();
        (wh.0 as usize, wh.1 as usize)
    }
}
