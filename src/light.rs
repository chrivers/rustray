#![allow(dead_code)]

use vector::Vector;
use color::Color;
use scene::*;
use num::Float;

pub struct Light<F: Float>
{
    pub pos: Vector<F>,
    pub color: Color<F>,
}

impl<F: Float> HasPosition<F> for Light<F>
{
    fn get_position(&self) -> Vector<F> { self.pos }
    fn set_position(&mut self, value: Vector<F>) { self.pos = value }
}
