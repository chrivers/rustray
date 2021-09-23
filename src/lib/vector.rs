use cgmath;
use num_traits::Zero;

pub use cgmath::{InnerSpace, MetricSpace};

use super::Float;

/**
Convenience macro to construct a [`Vector<F>`] from input values.

Shortens
```
Vector::new(x, y, z)
```

 to

```
vec3!(x, y, z)
```
*/
#[macro_export]
macro_rules! vec3 {
    ($( $vals:expr ),+) => { Vector::new( $($vals),+ ) }
}

pub type Vector<F> = cgmath::Vector3<F>;

impl<F: Float> Vectorx<F> for Vector<F>
{
    fn identity_x() -> Vector<F>
    {
        Vector { x: F::one(), y: F::zero(), z: F::zero() }
    }

    fn identity_y() -> Vector<F>
    {
        Vector { x: F::zero(), y: F::one(), z: F::zero() }
    }

    fn identity_z() -> Vector<F>
    {
        Vector { x: F::zero(), y: F::zero(), z: F::one() }
    }

    fn polar_angles(self) -> (F, F)
    {
        let theta = self.x.atan2(self.z);
        let phi = self.y.acos();
        (phi, theta)
    }
}


pub trait Vectorx<F: Float> : InnerSpace<Scalar=F> + Zero
where
    Self: Sized + std::ops::Neg<Output=Self>
{
    fn identity_x() -> Self;
    fn identity_y() -> Self;
    fn identity_z() -> Self;

    #[inline]
    fn vector_to(self, other: Self) -> Self
    {
        other - self
    }

    #[inline]
    fn normal_to(self, other: Self) -> Self
    {
        self.vector_to(other).normalize()
    }

    fn polar_angles(self) -> (F, F);

    fn polar_uv(self) -> (F, F)
    {
        let (phi, theta) = self.polar_angles();

        let raw_u = theta / (F::TWO * F::PI());

        let u = raw_u + F::HALF;
        let v = phi / (F::TWO * F::PI());

        (u, v)
    }

    /* Reflect vector (self) around normal */
    fn reflect(self, normal: &Self) -> Self
    {
        self - *normal * (F::TWO * self.dot(*normal))
    }

    /* Refract vector (self) relative to surface normal, according to ior.
       (Index of Refraction) */
    fn refract(self, normal: &Self, ior: F) -> Self
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
            Self::zero()
        } else {
            self * eta + n * (eta * cosi - k.sqrt())
        }
    }

    /* Compute the Fresnel coefficient for the relative magnitude of reflection
     * vs refraction between <self> and <normal>, using specified Index of Refraction.
     *
     * https://en.wikipedia.org/wiki/Fresnel_equations
     */
    fn fresnel(self, normal: &Self, ior: F) -> F
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
