use cgmath::{EuclideanSpace, InnerSpace, Matrix, Matrix4, Point3};
use cgmath::{SquareMatrix, Transform as _};

use crate::types::{Float, Vector, Vector4x};

#[derive(Copy, Clone, Debug)]
pub struct Transform<F: Float> {
    xfrm: Matrix4<F>,
    ifrm: Matrix4<F>,
}

pub trait HasTransform<F: Float> {
    fn get_transform(&self) -> &Transform<F>;
    fn set_transform(&mut self, xfrm: &Transform<F>);
}

impl<F: Float> Transform<F> {
    pub fn new(xfrm: Matrix4<F>) -> Self {
        Self {
            xfrm,
            ifrm: xfrm.inverse_transform().unwrap(),
        }
    }

    #[must_use]
    pub fn identity() -> Self {
        Self {
            xfrm: Matrix4::identity(),
            ifrm: Matrix4::identity(),
        }
    }

    pub fn pos(&self, vec: Vector<F>) -> Vector<F> {
        self.xfrm.transform_point(Point3::from_vec(vec)).to_vec()
    }

    pub fn dir(&self, vec: Vector<F>) -> Vector<F> {
        self.xfrm.transform_vector(vec)
    }

    pub fn nml(&self, vec: Vector<F>) -> Vector<F> {
        self.ifrm.transpose().transform_vector(vec).normalize()
    }

    pub fn pos_inv(&self, vec: Vector<F>) -> Vector<F> {
        self.ifrm.transform_point(Point3::from_vec(vec)).to_vec()
    }

    pub fn dir_inv(&self, vec: Vector<F>) -> Vector<F> {
        self.ifrm.transform_vector(vec)
    }

    pub fn nml_inv(&self, vec: Vector<F>) -> Vector<F> {
        self.xfrm.transform_vector(vec).normalize()
    }

    pub fn into_mint(&self) -> mint::ColumnMatrix4<f32> {
        let t = self.xfrm;
        mint::ColumnMatrix4 {
            x: t.x.into_mint(),
            y: t.y.into_mint(),
            z: t.z.into_mint(),
            w: t.w.into_mint(),
        }
    }

    pub fn inv_into_mint(&self) -> mint::ColumnMatrix4<f32> {
        let t = self.ifrm;
        mint::ColumnMatrix4 {
            x: t.x.into_mint(),
            y: t.y.into_mint(),
            z: t.z.into_mint(),
            w: t.w.into_mint(),
        }
    }
}

impl<F: Float> From<mint::ColumnMatrix4<f32>> for Transform<F> {
    fn from(value: mint::ColumnMatrix4<f32>) -> Self {
        let t = value;
        Self::new(Matrix4 {
            x: Vector4x::from_mint(t.x),
            y: Vector4x::from_mint(t.y),
            z: Vector4x::from_mint(t.z),
            w: Vector4x::from_mint(t.w),
        })
    }
}
