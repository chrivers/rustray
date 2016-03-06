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

    fn normalize(self) -> Vector<F>
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
}
