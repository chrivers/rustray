use std::fmt::Debug;
use std::str::FromStr;
use std::marker::PhantomData;
use std::path::Path;

use cgmath::{Vector3, Vector4, Matrix4, InnerSpace, Transform};
use num_traits::Zero;

use pest::iterators::Pair;
use pest_derive::Parser;

use crate::geometry::{Sphere, Cylinder, Triangle, TriangleMesh};
use crate::lib::{RResult, Error::ParseError};
use crate::material::{Phong, Smart};
use crate::{Vector, Point, Float, Color, Material, DynMaterial, Sampler, DynSampler, BilinearSampler, RayTarget, Vectorx, point, vec3};

#[derive(Parser)]
#[grammar = "format/sbt.pest"]
pub struct SbtParser<F> {
    _p: PhantomData<F>
}

#[derive(Copy, Clone)]
pub enum SbtVersion {
    Sbt0_9,
    Sbt1_0,
}

impl FromStr for SbtVersion {
    type Err = crate::lib::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.9" => Ok(SbtVersion::Sbt0_9),
            "1.0" => Ok(SbtVersion::Sbt1_0),
            _ => panic!("impossible"),
        }
    }
}

impl<F> SbtParser<F>
where
    F: Float + FromStr,
{
    /* Primitive types */

    pub fn parse_bool(p: Pair<Rule>) -> bool {
        match p.into_inner().next().map(|p| p.as_str()) {
            Some("false") => false,
            Some("true") => true,
            _ => panic!("internal parser error"),
        }
    }

    pub fn parse_num1(m: Pair<Rule>) -> F {
        m.as_str().parse().unwrap_or(F::ZERO)
    }

    pub fn parse_val1(p: Pair<Rule>) -> RResult<F>
    {
        let mut m = p.into_inner();
        m.next().unwrap().as_str().trim().parse().or(Err(ParseError()))
    }

    pub fn parse_val2(p: Pair<Rule>) -> Point<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<_>>();
        point!(v[0], v[1])
    }

    pub fn parse_int3(p: Pair<Rule>) -> [usize; 3] {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(0)
        ).collect::<Vec<usize>>();
        [v[0], v[1], v[2]]
    }

    pub fn parse_val3(p: Pair<Rule>) -> Vector<F> {
        Self::parse_val3b(p.into_inner().next().unwrap())
    }

    pub fn parse_val3b(p: Pair<Rule>) -> Vector<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Vector::new(v[0], v[1], v[2])
    }

    pub fn parse_val4(p: Pair<Rule>) -> Vector4<F> {
        let m = p.into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Vector4::new(v[0], v[1], v[2], v[3])
    }

    pub fn parse_color(p: Pair<Rule>) -> Color<F>
    {
        let m = p.into_inner().next().unwrap().into_inner();
        let v = m.map(
            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
        ).collect::<Vec<F>>();
        Color::new(v[0], v[1], v[2])
    }

    /* Composite types */

    pub fn parse_sampler3<'a>(p: Pair<Rule>, resdir: &Path) -> RResult<DynSampler<'a, F, Color<F>>>
    where
        F: 'a
    {
        let ps = p.into_inner().next().unwrap();
        match ps.as_rule() {
            Rule::sampler3 => {
                let q = ps.into_inner().next().unwrap();
                match q.as_rule() {
                    Rule::map => {
                        let s = q.into_inner().as_str();
                        let name = &s[1..s.len()-1];
                        Ok(image::open(resdir.join(name))?.bilinear().dynsampler())
                    },
                    Rule::val3 => {
                        let m = q.into_inner();
                        let v = m.map(
                            |x| x.as_str().trim().parse().unwrap_or(F::ZERO)
                        ).collect::<Vec<F>>();
                        Ok(Color::new(v[0], v[1], v[2]).dynsampler())
                    }
                    // "unimplemented: {:?}"
                    other => Err(ParseError())
                }
            }
            other => Err(ParseError())
        }
    }

    pub fn parse_material<'a>(p: Pair<Rule>, resdir: &Path) -> RResult<DynMaterial<'a, F>>
    where
         F: 'static
    {
        let mut diff = Color::black().dynsampler();
        let mut spec = Color::black().dynsampler();
        let mut refl = None;
        let mut _ambi = Color::black().dynsampler();
        let mut tran = Color::black();
        let mut emis = Color::black();
        let mut shi = F::ONE;
        let mut idx = F::ZERO;
        let mut _gls = F::ZERO;
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::mat_diffuse      => diff = Self::parse_sampler3(q, resdir)?,
                Rule::mat_specular     => spec = Self::parse_sampler3(q, resdir)?,
                Rule::mat_reflective   => refl = Some(Self::parse_sampler3(q, resdir)?),
                Rule::mat_ambient      => _ambi = Self::parse_sampler3(q, resdir)?,
                Rule::mat_transmissive => tran = Self::parse_color(q),
                Rule::mat_emissive     => emis = Self::parse_color(q),
                Rule::mat_shininess    => shi  = Self::parse_val1(q)?,
                Rule::mat_index        => idx  = Self::parse_val1(q)?,
                Rule::mat_glossiness   => _gls = Self::parse_val1(q)?,
                Rule::name             => {},
                other => {
                    error!("Unknown material prop: {:?}", other);
                    return Err(ParseError())
                }
            }
        }

        Ok(Smart::new(idx, shi, emis, diff, spec.clone(), tran, refl.unwrap_or_else(|| spec.clone())).dynamic())
    }

    /* Geometry types */

    pub fn parse_geo_cyl<'a>(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    where
         F: 'static
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();
        let mut capped = true;

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                Rule::capped        => capped = Self::parse_bool(rule),
                other => error!("unsupported: {:?}", other)
            }
        }
        info!("Cylinder(capped={})", capped);
        Ok(box Cylinder::new(xfrm, capped, mat))
    }

    pub fn parse_geo_sph<'a>(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    where
         F: 'static
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = SbtParser::parse_material(rule, resdir)?,
                Rule::name => {},
                other => error!("unsupported: {:?}", other)
            }
        }
        let edge = Vector3::unit_z().xfrm(&xfrm);
        let pos = Vector3::zero().xfrm(&xfrm);

        info!("Sphere({:.4?}, {:.4?})", pos, (pos - edge).magnitude());
        Ok(box Sphere::new(pos, (pos - edge).magnitude(), mat))
    }

    pub fn parse_geo_box<'a>(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    where
         F: 'static
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = SbtParser::parse_material(rule, resdir)?,
                other => error!("unsupported: {:?}", other)
            }
        }

        let (a, b) = (-F::HALF, F::HALF);
        let p = [
            vec3!(a, a, a),
            vec3!(a, a, b),
            vec3!(a, b, a),
            vec3!(a, b, b),
            vec3!(b, a, a),
            vec3!(b, a, b),
            vec3!(b, b, a),
            vec3!(b, b, b),
        ].map(|p| p.xfrm(&xfrm) );

        let uv = [
            point!(F::ZERO, F::ONE),
            point!(F::ZERO, F::ZERO),
            point!(F::ONE, F::ZERO),
            point!(F::ONE, F::ONE),
        ];

        let t = [
            p[1], p[5], p[7], p[3], // Front face
            p[0], p[2], p[6], p[4], // Back face
            p[2], p[3], p[7], p[6], // Top face
            p[0], p[4], p[5], p[1], // Bottom face
            p[4], p[6], p[7], p[5], // Right face
            p[0], p[1], p[3], p[2], // Left face
        ];

        let n = [
            Vector::unit_z(), -Vector::unit_z(),
            Vector::unit_y(), -Vector::unit_y(),
            Vector::unit_x(), -Vector::unit_x(),
        ].map(|p| xfrm.transform_vector(p).normalize() );

        let mut tris = vec![];

        for x in (0..(t.len()-1)).step_by(4) {
            tris.push(Triangle::new(t[x], t[x+1], t[x+2], n[x/4], n[x/4], n[x/4], uv[0], uv[1], uv[2], mat.clone()));
            tris.push(Triangle::new(t[x], t[x+2], t[x+3], n[x/4], n[x/4], n[x/4], uv[0], uv[2], uv[3], mat.clone()));
        }

        Ok(box TriangleMesh::new(tris))
    }

    pub fn parse_geo_sqr<'a>(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    where
         F: 'static
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = SbtParser::parse_material(rule, resdir)?,
                other => error!("unsupported: {:?}", other)
            }
        }

        // (a, b) | (b, b)
        //        |
        // -------+--------
        //        |
        // (a, a) | (b, a)

        let (a, b) = (-F::HALF, F::HALF);
        let t = [
            vec3!(a, a, F::ZERO),
            vec3!(a, b, F::ZERO),
            vec3!(b, a, F::ZERO),
            vec3!(b, b, F::ZERO),
        ].map(|p| p.xfrm(&xfrm) );

        let uv = [
            point!(F::ZERO, F::ONE),
            point!(F::ZERO, F::ZERO),
            point!(F::ONE, F::ONE),
            point!(F::ONE, F::ZERO),
        ];

        let normal = xfrm.transform_vector(Vector::unit_z()).normalize();

        let tris = vec![
            Triangle::new(t[0], t[1], t[3], normal, normal, normal, uv[0], uv[1], uv[3], mat.clone()),
            Triangle::new(t[0], t[3], t[2], normal, normal, normal, uv[0], uv[3], uv[2], mat.clone()),
        ];

        Ok(box TriangleMesh::new(tris))
    }
}
