use super::float::Float;
use crate::types::Vector;
use cgmath::Transform as cgTransform;
use cgmath::{EuclideanSpace, InnerSpace, Matrix, Matrix4, Point3};

#[derive(Copy, Clone, Debug)]
pub struct Transform<F: Float> {
    xfrm: Matrix4<F>,
    ifrm: Matrix4<F>,
}

impl<F: Float> Transform<F> {
    pub fn new(xfrm: Matrix4<F>) -> Self {
        Self {
            xfrm,
            ifrm: xfrm.inverse_transform().unwrap(),
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
}
