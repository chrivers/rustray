use cgmath;
use num_traits::Zero;

pub use cgmath::{InnerSpace, MetricSpace, EuclideanSpace, Transform, Point3, Matrix4};

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
    ($( $vals:expr ),+ $(,)?) => { Vector::new( $($vals.into()),+ ) }
}

pub type Vector<F> = cgmath::Vector3<F>;

impl<F: Float> Vectorx<F> for Vector<F>
{
    fn identity_x() -> Vector<F>
    {
        Vector { x: F::ONE, y: F::ZERO, z: F::ZERO }
    }

    fn identity_y() -> Vector<F>
    {
        Vector { x: F::ZERO, y: F::ONE, z: F::ZERO }
    }

    fn identity_z() -> Vector<F>
    {
        Vector { x: F::ZERO, y: F::ZERO, z: F::ONE }
    }

    fn into_point3(self) -> bvh::Point3 {
        bvh::Point3::new(
            self.x.to_f32().unwrap_or_default(),
            self.y.to_f32().unwrap_or_default(),
            self.z.to_f32().unwrap_or_default(),
        )
    }

    fn into_vector3(self) -> bvh::Vector3 {
        bvh::Vector3::new(
            self.x.to_f32().unwrap_or_default(),
            self.y.to_f32().unwrap_or_default(),
            self.z.to_f32().unwrap_or_default(),
        )
    }

    fn polar_angles(self) -> (F, F)
    {
        let theta = self.x.atan2(self.z);
        let phi = self.y.acos();
        (phi, theta)
    }

    fn min(&self, other: &Self) -> Self
    {
        vec3!(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    fn max(&self, other: &Self) -> Self
    {
        vec3!(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }

    fn xfrm(&self, xfrm: &Matrix4<F>) -> Self
    {
        xfrm.transform_point(Point3::from_vec(*self)).to_vec()
    }

    fn xfrm_normal(&self, xfrm: &Matrix4<F>) -> Self
    {
        xfrm.transform_vector(*self).normalize()
    }
}


pub trait Vectorx<F: Float> : InnerSpace<Scalar=F> + Zero
where
    Self: Sized + std::ops::Neg<Output=Self>
{
    fn identity_x() -> Self;
    fn identity_y() -> Self;
    fn identity_z() -> Self;
    fn min(&self, other: &Self) -> Self;
    fn max(&self, other: &Self) -> Self;
    fn xfrm(&self, xfrm: &Matrix4<F>) -> Self;
    fn xfrm_normal(&self, xfrm: &Matrix4<F>) -> Self;
    fn into_point3(self) -> bvh::Point3;
    fn into_vector3(self) -> bvh::Vector3;

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
        let mut cosi = self.dot(*normal).clamp(-F::ONE, F::ONE);
        let eta_i;
        let eta_t;
        let n;
        if cosi < F::ZERO {
            eta_i = F::ONE;
            eta_t = ior;
            cosi = -cosi;
            n = *normal;
        } else {
            eta_i = ior;
            eta_t = F::ONE;
            n = -(*normal);
        }

        let eta = eta_i / eta_t;

        let k = F::ONE - eta * eta * (F::ONE - cosi * cosi);

        if k < F::ZERO {
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
        let mut cos_i = self.dot(*normal).clamp(-F::ONE, F::ONE);
        let (eta_i, eta_t);
        if cos_i > F::ZERO {
            eta_i = ior;
            eta_t = F::ONE;
        } else {
            eta_i = F::ONE;
            eta_t = ior;
        }

        /* Compute sin_t using Snell's law */
        let sin_t = eta_i / eta_t * (F::ZERO.max(F::ONE - cos_i * cos_i)).sqrt();

        if sin_t >= F::ONE {
            /* Total internal reflection */
            F::ONE
        } else {
            /* Reflection and refraction */
            let cos_t = (F::ZERO.max(F::ONE - sin_t * sin_t)).sqrt();
            cos_i = cos_i.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            (r_s * r_s  +  r_p * r_p) * F::HALF
        }
    }

}
