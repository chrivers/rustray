#![allow(dead_code)]

use std::ops::{Add, Sub};
use num::Float;

#[derive(Clone, Copy)]
struct Vector<F: Float>
{
    x: F,
    y: F,
    z: F,
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

impl<F: Float> Vector<F>
{
    fn length(&self) -> F
    {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(self) -> Vector<F>
    {
        let l = self.length();
        if l != F::zero()
        {
            Vector
            {
                x: self.x / l,
                y: self.y / l,
                z: self.z / l,
            }
        } else
        {
            Vector { x: F::zero(), y: F::zero(), z: F::zero() }
        }
    }

    fn dot(self, other: Vector<F>) -> F
    {
        other.x * self.x + other.y * self.y + other.z * self.z
    }

    fn cross(self, other: Vector<F>) -> Vector<F>
    {
        Vector
        {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    fn vector_to(self, other: Vector<F>) -> Vector<F>
    {
        Vector
        {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z
        }
    }

    fn length_to(self, other: Vector<F>) -> F
    {
        self.vector_to(other).length()
    }

    fn cos_angle(self, other: Vector<F>) -> F
    {
        self.normalized().dot(other.normalized())
    }

    fn scaled(self, scale: F) -> Vector<F>
    {
        Vector
        {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
        }
    }
}
