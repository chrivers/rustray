use std::collections::HashMap;

use cgmath::InnerSpace;

use crate::{
    geometry::Triangle,
    types::{Float, Vector, Vectorx},
};

pub fn face_normals<F: Float>(tris: &mut [Triangle<F>]) {
    /* Single-face normals */
    for tri in tris {
        let n = tri.edge1.cross(tri.edge2).normalize();
        tri.na = n;
        tri.nb = n;
        tri.nc = n;
    }
}

pub fn smooth_normals<F: Float>(tris: &mut [Triangle<F>]) {
    /* Vertex-smoothed normals */
    let mut norms: HashMap<u64, Vector<F>> = HashMap::new();

    for tri in tris.iter() {
        let n = tri.edge1.cross(tri.edge2).normalize();
        *norms.entry(tri.a.hash()).or_insert(Vector::ZERO) += n;
        *norms.entry(tri.b.hash()).or_insert(Vector::ZERO) += n;
        *norms.entry(tri.c.hash()).or_insert(Vector::ZERO) += n;
    }

    for tri in tris.iter_mut() {
        tri.na = norms[&tri.a.hash()].normalize();
        tri.nb = norms[&tri.b.hash()].normalize();
        tri.nc = norms[&tri.c.hash()].normalize();
    }
}
