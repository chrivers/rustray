use std::ops::{Add, AddAssign, Sub, Mul, Div, Neg};
use std::fmt::{self, Display};

use super::Float;

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

impl<F: Float> AddAssign for Vector<F>
{
    fn add_assign(&mut self, other: Vector<F>)
    {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
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

    pub fn identity_x() -> Vector<F>
    {
        Vector { x: F::one(), y: F::zero(), z: F::zero() }
    }

    pub fn identity_y() -> Vector<F>
    {
        Vector { x: F::zero(), y: F::one(), z: F::zero() }
    }

    pub fn identity_z() -> Vector<F>
    {
        Vector { x: F::zero(), y: F::zero(), z: F::one() }
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

    pub fn cross(self, other: Vector<F>) -> Vector<F>
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

    pub fn polar_angles(self) -> (F, F)
    {
        let theta = self.x.atan2(self.z);
        let phi = self.y.acos();
        (phi, theta)
    }

    pub fn polar_uv(self) -> (F, F)
    {
        let (phi, theta) = self.polar_angles();

        let raw_u = theta / (F::TWO * F::PI());

        let u = raw_u + F::HALF;
        let v = phi / (F::TWO * F::PI());

        (u, v)
    }

    /* Fast computation of len^2. Useful optimization for e.g. comparing lengths */
    pub fn len_sqr(self) -> F
    {
        self.dot(self)
    }

    /* Reflect vector (self) around normal */
    pub fn reflect(self, normal: &Vector<F>) -> Vector<F>
    {
        self - *normal * (F::TWO * self.dot(*normal))
    }

    /* Refract vector (self) relative to surface normal, according to ior.
       (Index of Refraction) */
    pub fn refract(self, normal: &Vector<F>, ior: F) -> Vector<F>
    {
        let mut cosi = self.dot(*normal).clamp(-F::one(), F::one());
        let eta_i;
        let eta_t;
        let n;
        if cosi < F::zero() {
            eta_i = F::one();
            eta_t = ior;
            cosi = -cosi;
            n = *normal;
        } else {
            eta_i = ior;
            eta_t = F::one();
            n = -(*normal);
        }

        let eta = eta_i / eta_t;

        let k = F::one() - eta * eta * (F::one() - cosi * cosi);

        if k < F::zero() {
            Vector::zero()
        } else {
            self * eta + n * (eta * cosi - k.sqrt())
        }
    }

    /* Compute the Fresnel coefficient for the relative magnitude of reflection
     * vs refraction between <self> and <normal>, using specified Index of Refraction.
     *
     * https://en.wikipedia.org/wiki/Fresnel_equations
     */
    pub fn fresnel(self, normal: &Vector<F>, ior: F) -> F
    {
        let mut cos_i = self.dot(*normal).clamp(-F::one(), F::one());
        let (eta_i, eta_t);
        if cos_i > F::zero() {
            eta_i = ior;
            eta_t = F::one();
        } else {
            eta_i = F::one();
            eta_t = ior;
        }

        /* Compute sin_t using Snell's law */
        let sin_t = eta_i / eta_t * (F::zero().max(F::one() - cos_i * cos_i)).sqrt();

        if sin_t >= F::one() {
            /* Total internal reflection */
            F::one()
        } else {
            /* Reflection and refraction */
            let cos_t = (F::zero().max(F::one() - sin_t * sin_t)).sqrt();
            cos_i = cos_i.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            (r_s * r_s  +  r_p * r_p) * F::HALF
        }
    }

}

impl<F: Float> std::fmt::Display for Vector<F>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        Display::fmt(&self.x, f)?;
        write!(f, ", ")?;
        Display::fmt(&self.y, f)?;
        write!(f, ", ")?;
        Display::fmt(&self.z, f)?;
        f.write_str("]")
    }
}
