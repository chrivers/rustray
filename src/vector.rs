use crate::traits::Float;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[macro_export]
macro_rules! vec3 {
    ($( $vals:expr ),+) => { Vector::new( $($vals),+ ) }
}

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

impl<F: Float> Neg for Vector<F>
{
    type Output = Vector<F>;

    fn neg(self) -> Vector<F>
    {
        Self::Output { x: -self.x, y: -self.y, z: -self.z, }
    }
}

impl<F: Float> Vector<F>
{
    pub fn new(x: F, y: F, z: F) -> Vector<F>
    {
        Vector { x, y, z }
    }

    pub fn zero() -> Vector<F>
    {
        Vector { x: F::zero(), y: F::zero(), z: F::zero() }
    }

    pub fn length(&self) -> F
    {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(self) -> Vector<F>
    {
        let l = self.length();

        if l.non_zero() {
            self / l
        } else {
            Self::zero()
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

    pub fn normal_to(self, other: Vector<F>) -> Vector<F>
    {
        self.vector_to(other).normalized()
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

    pub fn len_squared(self) -> F
    {
        self.dot(self)
    }
}
