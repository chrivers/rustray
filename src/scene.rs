use crate::traits::Float;
use crate::light::Light;
use crate::ray::{Ray, Hit, Maxel};
use crate::vector::Vector;
use crate::color::Color;

pub trait HasPosition<F: Float>
{
    fn get_position(&self) -> Vector<F>;
    fn set_position(&mut self, value: Vector<F>);
}

pub trait HasDirection<F: Float>
{
    fn get_direction(&self) -> Vector<F>;
    fn set_direction(&mut self, value: Vector<F>);
}

pub trait HasColor<F: Float>
{
    fn get_color(&self) -> Color<F>;
    fn set_color(&mut self, value: Color<F>);
}

pub trait RayTarget<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>;
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>;
}

pub trait RayTracer<F: Float> : Sync
{
    fn ray_shadow(&self, hit: &Hit<F>, light: &Light<F>) -> bool;
    fn ray_trace(&self, ray: &Ray<F>, lvl: u32) -> Option<Color<F>>;
}
