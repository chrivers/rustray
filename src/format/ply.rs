use std::fmt::Debug;
use std::str::FromStr;
use std::marker::PhantomData;
use std::path::Path;
use std::io::{Read, BufRead};

use cgmath::InnerSpace;
use num_traits::Zero;

use crate::vec3;
use crate::geometry::{FiniteGeometry, Triangle, TriangleMesh};
use crate::lib::{Camera};
use crate::lib::{RResult, Error};
use crate::lib::PointLight;
use crate::scene::{Scene, BoxScene};
use crate::material::Phong;
use crate::sampler::Texel;
use crate::{Vector, Point, Float, Color, Material, Vectorx, Light};

use ply_rs::{parser, ply};
use rtbvh::Primitive;

pub struct PlyParser<F: Float> {
    _p: PhantomData<F>,
}

#[derive(Copy, Clone, Debug)]
struct Vertex<F: Float> (Vector<F>, Vector<F>);

#[derive(Debug)]
struct Face<F: Float> {
    idx: Vec<usize>,
    uv: Vec<F>,
}

impl<F: Float> ply::PropertyAccess for Vertex<F> {
    fn new() -> Self {
        Vertex(Vector::<F>::zero(), Vector::<F>::zero())
    }
    fn set_property(&mut self, key: String, property: ply::Property) {
        match property {
            ply::Property::Float(v) => {
                match key.as_ref() {
                    "x"  => self.0.x = F::from_f32(v),
                    "y"  => self.0.y = F::from_f32(v),
                    "z"  => self.0.z = F::from_f32(v),
                    "nx" => self.1.x = F::from_f32(v),
                    "ny" => self.1.y = F::from_f32(v),
                    "nz" => self.1.z = F::from_f32(v),
                    "s" | "t" => {},
                    "tx"|"ty"|"tz" => {},
                    "bx"|"by"|"bz" => {},
                    k    => panic!("Vertex: Unexpected key/value combination: key: {}", k),
                }
            }
            ply::Property::UChar(_v) => {
                match key.as_ref() {
                    "red" | "green" | "blue" | "alpha" => {},
                    k    => panic!("Vertex: Unexpected key/value combination: key: {}", k),
                }
            }
            t => panic!("Vertex: Unexpected type: {:?}", t),
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
            ("vertex_indices", ply::Property::ListInt(vec)) => self.idx = vec.iter().map(|x| *x as usize).collect(),
            ("vertex_indices", ply::Property::ListUInt(vec)) => self.idx = vec.iter().map(|x| *x as usize).collect(),
            ("vertex_index", ply::Property::ListUInt(vec)) => self.idx = vec.iter().map(|x| *x as usize).collect(),
            ("texcoord", ply::Property::ListFloat(vec)) => self.uv = vec.iter().map(|x| F::from_f32(*x)).collect(),
            ("red" | "green" | "blue" | "alpha", _) => {},
            ("flags", _) => {},
            ("texnumber", _) => {},
            (k, t) => panic!("Face: Unexpected key/value combination: key: {} (type {:?})", k, t),
        }
    }
}

impl<F> PlyParser<F>
where
    F: Float + FromStr + Texel + 'static,
{

    pub fn parse_file(file: &mut (impl Read + BufRead), _resdir: &Path, width: u32, height: u32) -> RResult<BoxScene<F>>
    {
        let mut cameras = vec![];
        let mut objects: Vec<Box<dyn FiniteGeometry<F>>> = vec![];
        let mut lights: Vec<Box<dyn Light<F>>> = vec![];

        let vertex_parser = parser::Parser::<Vertex<F>>::new();
        let face_parser = parser::Parser::<Face<F>>::new();

        let header = vertex_parser.read_header(file).unwrap();

        let mut vertex_list = Vec::new();
        let mut face_list = Vec::new();
        for (_, element) in &header.elements {
            match element.name.as_ref() {
                "vertex" => {
                    vertex_list = vertex_parser.read_payload_for_element(file, element, &header)?
                },
                "face" => {
                    face_list = face_parser.read_payload_for_element(file, element, &header)?
                },
                other => return Err(Error::ParseUnsupported(other.to_owned())),
            }
        }
        info!("vl: {:#?}", vertex_list.len());
        info!("fl: {:#?}", face_list.len());

        let mat = Phong::white().dynamic();

        let mut tris = vec![];
        for face in &face_list {
            for n in 1..(face.idx.len() - 1) {
                let mut a = vertex_list[face.idx[0]];
                let mut b = vertex_list[face.idx[n]];
                let mut c = vertex_list[face.idx[n+1]];
                let n = (a.0 - b.0).cross(a.0 - c.0);
                if a.1.is_zero() { a.1 = n }
                if b.1.is_zero() { b.1 = n }
                if c.1.is_zero() { c.1 = n }
                let tri = Triangle::new(
                    a.0, b.0, c.0,
                    a.1, b.1, c.1,
                    Point::zero(),
                    Point::zero(),
                    Point::zero(),
                    mat.clone(),
                );
                tris.push(tri);
            }
        }

        let mesh = TriangleMesh::new(tris);

        let bb = mesh.aabb();
        info!("aabb {:?}", bb);

        let sz: Vector<F> = Vectorx::from_vector3(bb.lengths());
        let look: Vector<F> = Vectorx::from_vector3(bb.center());
        let pos = vec3!(F::ZERO, sz.y/F::TWO, sz.magnitude());

        let cam = Camera::build(
            pos,
            look - pos,
            Vector::unit_y(),
            F::from_f32(120.0),
            width,
            height,
            None,
        );

        cameras.push(cam);

        objects.push(box mesh);

        let lgt = PointLight {
            a: F::from_f32(0.1),
            b: F::from_f32(0.0001),
            c: F::from_f32(0.0),
            pos,
            color: Color::white(),
        };

        lights.push(box lgt);

        Ok(Scene::new(cameras, objects, vec![], lights))
    }

}
