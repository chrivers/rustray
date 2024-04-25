use std::fmt::Debug;
use std::io::BufRead;
use std::marker::PhantomData;

use cgmath::{Matrix4, SquareMatrix};
use num_traits::Zero;

use crate::geometry::{Triangle, TriangleMesh};
use crate::sampler::Texel;
use crate::scene::BoxScene;
use crate::types::{Error, Float, Point, RResult, Vector, Vectorx};

use ply_rs::{parser, ply};

pub struct PlyParser<F: Float> {
    _p: PhantomData<F>,
}

#[derive(Copy, Clone, Debug)]
struct Vertex<F: Float>(Vector<F>, Vector<F>);

#[derive(Debug)]
struct Face<F: Float> {
    idx: Vec<usize>,
    uv: Vec<F>,
}

impl<F: Float> ply::PropertyAccess for Vertex<F> {
    fn new() -> Self {
        Self(Vector::ZERO, Vector::ZERO)
    }

    fn set_property(&mut self, key: String, property: ply::Property) {
        match property {
            ply::Property::Float(v) => match key.as_ref() {
                "x" => self.0.x = F::from_f32(v),
                "y" => self.0.y = F::from_f32(v),
                "z" => self.0.z = F::from_f32(v),
                "nx" => self.1.x = F::from_f32(v),
                "ny" => self.1.y = F::from_f32(v),
                "nz" => self.1.z = F::from_f32(v),
                "s" | "t" | "tx" | "ty" | "tz" | "bx" | "by" | "bz" => {}
                k => error!("Vertex: Unexpected key/value combination: key: {k}"),
            },
            ply::Property::UChar(_v) => match key.as_ref() {
                "red" | "green" | "blue" | "alpha" => {}
                k => error!("Vertex: Unexpected key/value combination: key: {k}"),
            },
            t => error!("Vertex: Unexpected type: {t:?}"),
        }
    }
}

impl<F: Float> ply::PropertyAccess for Face<F> {
    fn new() -> Self {
        Self {
            idx: vec![],
            uv: vec![],
        }
    }
    fn set_property(&mut self, key: String, property: ply::Property) {
        match (key.as_ref(), property) {
            ("vertex_indices", ply::Property::ListInt(vec)) => {
                self.idx = vec.into_iter().map(|x| x as usize).collect();
            }
            ("vertex_indices", ply::Property::ListUInt(vec)) => {
                self.idx = vec.into_iter().map(|x| x as usize).collect();
            }
            ("vertex_index", ply::Property::ListUInt(vec)) => {
                self.idx = vec.into_iter().map(|x| x as usize).collect();
            }
            ("texcoord", ply::Property::ListFloat(vec)) => {
                self.uv = vec.into_iter().map(F::from_f32).collect();
            }
            ("red" | "green" | "blue" | "alpha" | "flags" | "texnumber", _) => {}
            (k, t) => error!("Face: Unexpected key/value combination: key: {k} (type {t:?})"),
        }
    }
}

impl<F: Float + Texel> PlyParser<F> {
    pub fn parse_file(file: &mut impl BufRead, scene: &mut BoxScene<F>) -> RResult<()> {
        let vertex_parser = parser::Parser::<Vertex<F>>::new();
        let face_parser = parser::Parser::<Face<F>>::new();

        let header = vertex_parser.read_header(file).unwrap();

        let mut vertex_list = Vec::new();
        let mut face_list = Vec::new();
        for (_, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertex_list = vertex_parser.read_payload_for_element(file, element, &header)?;
                }
                "face" => {
                    face_list = face_parser.read_payload_for_element(file, element, &header)?;
                }
                other => return Err(Error::ParseUnsupported(other.to_owned())),
            }
        }
        info!("vl: {:#?}", vertex_list.len());
        info!("fl: {:#?}", face_list.len());

        let mat = scene.materials.default();

        let mut tris = vec![];
        for face in &face_list {
            for n in 1..(face.idx.len() - 1) {
                let mut a = vertex_list[face.idx[0]];
                let mut b = vertex_list[face.idx[n]];
                let mut c = vertex_list[face.idx[n + 1]];
                let n = (a.0 - b.0).cross(a.0 - c.0);
                if a.1.is_zero() {
                    a.1 = n;
                }
                if b.1.is_zero() {
                    b.1 = n;
                }
                if c.1.is_zero() {
                    c.1 = n;
                }
                let tri = Triangle::new(
                    a.0,
                    b.0,
                    c.0,
                    a.1,
                    b.1,
                    c.1,
                    Point::ZERO,
                    Point::ZERO,
                    Point::ZERO,
                    mat,
                );
                tris.push(tri);
            }
        }

        let mesh = TriangleMesh::new(tris, Matrix4::identity());
        scene.add_object(mesh);
        scene.recompute_bvh()
    }
}
