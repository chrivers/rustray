#![allow(dead_code)]

use ray::Ray;
use vector::Vector;
use color::Color;
use num::Float;

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

pub trait RayTarget<F: Float>
{
    fn ray_hit(&self, ray: Ray<F>) -> Option<Vector<F>>;
}
