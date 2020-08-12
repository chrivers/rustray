use std::fmt::Debug;

use crate::traits::Float;
use crate::light::Light;
use crate::ray::Ray;
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

pub trait RayTarget<F: Float> : Debug + Sync
{
    fn ray_hit(&self, ray: &Ray<F>) -> Option<Vector<F>>;
    fn trace(&self, hit: &Vector<F>, light: &Light<F>) -> Color<F>;
}
