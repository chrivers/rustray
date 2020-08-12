#![allow(dead_code)]

use crate::traits::Float;
use std::ops::{Add, Sub, Mul, Div};

#[derive(Clone, Copy, Debug)]
pub struct Vector<F: Float>
{
    pub x: F,
    pub y: F,
    pub z: F,
}

impl<F: Float> Add for Vector<F>
{
    type Output = Vector<F>;

    fn add(self, other: Vector<F>) -> Vector<F>
    {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<F: Float> Sub for Vector<F>
{
    type Output = Vector<F>;

    fn sub(self, other: Vector<F>) -> Vector<F>
    {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<F: Float> Mul<F> for Vector<F>
{
    type Output = Vector<F>;

    fn mul(self, other: F) -> Vector<F>
    {
        Vector
        {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl<F: Float> Div<F> for Vector<F>
{
    type Output = Vector<F>;

    fn div(self, other: F) -> Vector<F> {
        Vector {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}


impl<F: Float> Mul<Vector<F>> for Vector<F>
{
    type Output = Vector<F>;

    fn mul(self, other: Vector<F>) -> Vector<F>
    {
        Vector
        {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<F: Float> Vector<F>
{
    pub fn new(x: F, y: F, z: F) -> Vector<F>
    {
        Vector { x, y, z }
    }

    pub fn length(&self) -> F
    {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(self) -> Vector<F>
    {
        let l = self.length();
        if l != F::zero() {
            self / l
        } else {
            Vector { x: F::zero(), y: F::zero(), z: F::zero() }
        }
    }

    pub fn dot(self, other: Vector<F>) -> F
    {
        other.x * self.x + other.y * self.y + other.z * self.z
    }

    pub fn crossed(self, other: Vector<F>) -> Vector<F>
    {
        Vector
        {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    pub fn vector_to(self, other: Vector<F>) -> Vector<F>
    {
        Vector
        {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z
        }
    }

    pub fn length_to(self, other: Vector<F>) -> F
    {
        self.vector_to(other).length()
    }

    pub fn cos_angle(self, other: Vector<F>) -> F
    {
        self.normalized().dot(other.normalized())
    }

    pub fn scaled(self, scale: F) -> Vector<F>
    {
        Vector
        {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
        }
    }
}
