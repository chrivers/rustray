use cgmath::{InnerSpace, Matrix4};
use num_traits::Zero;

use crate::sampler::Texel;
use crate::types::{transform::Transform, Float, Point};

/**
Convenience macro to construct a [`Vector<F>`] from input values.

Shortens
```
# use rustray::types::Vector;
# let x = 1.0f32;
# let y = 2.0f32;
# let z = 3.0f32;
let v: Vector<f32> = Vector::new(x, y, z);
```

 to

```
# use rustray::types::Vector;
# use rustray::vec3;
# let x = 1.0f32;
# let y = 2.0f32;
# let z = 3.0f32;
let v: Vector<f32> = vec3!(x, y, z);
```
*/
#[macro_export]
macro_rules! vec3 {
    ($( $vals:expr ),+ $(,)?) => { Vector::new( $($vals.into()),+ ) }
}

pub type Vector<F> = cgmath::Vector3<F>;

impl<F: Float> Vectorx<F> for Vector<F> {
    const ZERO: Self = Self {
        x: F::ZERO,
        y: F::ZERO,
        z: F::ZERO,
    };

    const UNIT_X: Self = Self {
        x: F::ONE,
        y: F::ZERO,
        z: F::ZERO,
    };

    const UNIT_Y: Self = Self {
        x: F::ZERO,
        y: F::ONE,
        z: F::ZERO,
    };

    const UNIT_Z: Self = Self {
        x: F::ZERO,
        y: F::ZERO,
        z: F::ONE,
    };

    #[cfg(debug_assertions)]
    fn assert_normalized(self) {
        if self != Self::ZERO {
            assert!(self.magnitude() < F::from_f32(1.01));
            assert!(self.magnitude() > F::from_f32(0.99));
        }
    }

    #[must_use]
    fn surface_tangents(&self) -> (Self, Self) {
        let u = if self.x.abs() <= self.y.abs() && self.x.abs() <= self.z.abs() {
            /* x smallest: tangent in yz plane */
            vec3![F::ZERO, self.z, -self.y]
        } else if self.y.abs() <= self.z.abs() {
            /* y smallest: tangent in xz plane */
            vec3![-self.z, F::ZERO, self.x]
        } else {
            /* z smallest: tangent in xy plane */
            vec3![self.y, -self.x, F::ZERO]
        }
        .normalize();

        (u, self.cross(u))
    }

    fn point(self) -> Point<F> {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    fn from_f32(val: Vector<f32>) -> Self {
        Self {
            x: F::from_f32(val[0]),
            y: F::from_f32(val[1]),
            z: F::from_f32(val[2]),
        }
    }

    #[must_use]
    fn from_f32s(val: [f32; 3]) -> Self {
        Self {
            x: F::from_f32(val[0]),
            y: F::from_f32(val[1]),
            z: F::from_f32(val[2]),
        }
    }

    #[must_use]
    fn into_f32(self) -> Vector<f32> {
        Vector::new(
            self.x.to_f32().unwrap_or_default(),
            self.y.to_f32().unwrap_or_default(),
            self.z.to_f32().unwrap_or_default(),
        )
    }

    fn into_vec3(self) -> glam::Vec3 {
        glam::Vec3::new(
            self.x.to_f32().unwrap_or_default(),
            self.y.to_f32().unwrap_or_default(),
            self.z.to_f32().unwrap_or_default(),
        )
    }

    #[must_use]
    fn from_vec3(val: glam::Vec3) -> Self {
        Self {
            x: F::from_f32(val[0]),
            y: F::from_f32(val[1]),
            z: F::from_f32(val[2]),
        }
    }

    fn polar_angles(self) -> (F, F) {
        let theta = self.x.atan2(self.z);
        let phi = self.y.asin();
        (phi, theta)
    }

    #[must_use]
    fn min(&self, other: &Self) -> Self {
        vec3!(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z)
        )
    }

    fn max(&self, other: &Self) -> Self {
        vec3!(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z)
        )
    }

    #[must_use]
    fn xfrm_pos(&self, xfrm: &Matrix4<F>) -> Self {
        Transform::new(*xfrm).pos(*self)
    }

    #[must_use]
    fn xfrm_nml(&self, xfrm: &Matrix4<F>) -> Self {
        Transform::new(*xfrm).nml(*self)
    }
}

pub trait Vectorx<F: Float>: InnerSpace<Scalar = F> + Zero
where
    Self: Sized + std::ops::Neg<Output = Self>,
{
    const ZERO: Self;
    const UNIT_X: Self;
    const UNIT_Y: Self;
    const UNIT_Z: Self;

    #[cfg(debug_assertions)]
    fn assert_normalized(self);

    #[must_use]
    fn min(&self, other: &Self) -> Self;
    #[must_use]
    fn max(&self, other: &Self) -> Self;
    #[must_use]
    fn xfrm_nml(&self, xfrm: &Matrix4<F>) -> Self;
    #[must_use]
    fn xfrm_pos(&self, xfrm: &Matrix4<F>) -> Self;

    fn surface_tangents(&self) -> (Self, Self);

    fn into_vec3(self) -> glam::Vec3;
    fn from_vec3(val: glam::Vec3) -> Self;

    fn from_f32s(val: [f32; 3]) -> Self;
    fn from_f32(value: Vector<f32>) -> Self;
    fn into_f32(self) -> Vector<f32>;

    fn point(self) -> Point<F>;

    #[inline]
    #[must_use]
    fn vector_to(self, other: Self) -> Self {
        other - self
    }

    #[inline]
    #[must_use]
    fn normal_to(self, other: Self) -> Self {
        self.vector_to(other).normalize()
    }

    fn polar_angles(self) -> (F, F);

    fn polar_uv(self) -> (F, F) {
        let (phi, theta) = self.polar_angles();

        let raw_u = theta / (F::TWO * F::PI());

        let u = raw_u + F::HALF;
        let v = phi / F::PI();

        (u, v)
    }

    /* Reflect vector (self) around normal */
    #[must_use]
    fn reflect(self, normal: &Self) -> Self {
        self - *normal * (F::TWO * self.dot(*normal))
    }

    /* Refract vector (self) relative to surface normal, according to ior.
    (Index of Refraction) */
    #[must_use]
    fn refract(self, normal: &Self, ior: F) -> Self {
        let mut cosi = self.dot(*normal).clamp(-F::ONE, F::ONE);
        let eta_i;
        let eta_t;
        let n;
        if cosi.is_negative() {
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

        if k.is_negative() {
            Self::ZERO
        } else {
            (self * eta + n * (eta * cosi - k.sqrt())).normalize()
        }
    }

    /* Compute the Fresnel coefficient for the relative magnitude of reflection
     * vs refraction between <self> and <normal>, using specified Index of Refraction.
     *
     * https://en.wikipedia.org/wiki/Fresnel_equations
     */
    #[must_use]
    fn fresnel(self, normal: &Self, ior: F) -> F {
        let mut cos_i = self.dot(*normal).clamp(-F::ONE, F::ONE);
        let (eta_i, eta_t);
        if cos_i.is_positive() {
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
            (r_s * r_s + r_p * r_p) * F::HALF
        }
    }
}

impl<F: Float> Texel for Vector<F> {}

pub trait Vector4x<F: Float> {
    fn from_mint(val: mint::Vector4<f32>) -> Self;
    fn into_mint(self) -> mint::Vector4<f32>;
}

impl<F: Float> Vector4x<F> for cgmath::Vector4<F> {
    fn from_mint(val: mint::Vector4<f32>) -> Self {
        Self {
            x: F::from_f32(val.x),
            y: F::from_f32(val.y),
            z: F::from_f32(val.z),
            w: F::from_f32(val.w),
        }
    }

    fn into_mint(self) -> mint::Vector4<f32> {
        mint::Vector4::<f32> {
            x: self.x.to_f32().unwrap_or_default(),
            y: self.y.to_f32().unwrap_or_default(),
            z: self.z.to_f32().unwrap_or_default(),
            w: self.w.to_f32().unwrap_or_default(),
        }
    }
}
