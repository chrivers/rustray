use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;

use obj::Obj;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use cgmath::{Rad, Matrix, Vector4, Matrix4, InnerSpace, SquareMatrix};
use num_traits::Zero;

use super::sbt::{SbtVersion, face_normals, spherical_uvs};

use crate::geometry::{Cube, Cone, Cylinder, Sphere, Square, Triangle, TriangleMesh, FiniteGeometry};
use crate::lib::{Color, Camera, Float, Point, Vector, vector::Vectorx, float::Lerp};
use crate::lib::light::{PointLight, DirectionalLight};
use crate::lib::result::{RResult, Error};
use crate::material::{Material, DynMaterial, Smart, Triblend, Bumpmap};
use crate::sampler::{Sampler, DynSampler, SamplerExt, Texel, NormalMap, ShineMap};
use crate::scene::{Scene, BoxScene, Light};

#[derive(Parser)]
#[grammar = "format/sbt2.pest"]
pub struct SbtParser2<F: Float> {
    _p: PhantomData<F>
}

#[derive(Debug)]
pub struct SbtProgram<'a, F: Float> {
    version: SbtVersion,
    blocks: Vec<SbtBlock<'a, F>>,
}

#[derive(Debug)]
pub struct SbtBlock<'a, F: Float> {
    name: &'a str,
    value: SbtValue<'a, F>,
}

pub type SbtDict<'a, F> = HashMap<String, SbtValue<'a, F>>;
pub type SbtTuple<'a, F> = Vec<SbtValue<'a, F>>;

trait SDict<F: Float + Texel> {
    fn float(&self, name: &str) -> RResult<F>;
    fn color(&self, name: &str) -> RResult<Color<F>>;
    fn shinemap<'b>(&self, name: &str, resdir: &Path) -> RResult<DynSampler<'b, F, F>>;
    fn sampler3<'a>(&self, name: &str, resdir: &Path) -> RResult<DynSampler<'a, F, Color<F>>>;
    fn string(&self, name: &str) -> RResult<&str>;
    fn vector(&self, name: &str) -> RResult<Vector<F>>;
    fn boolean(&self, name: &str) -> RResult<bool>;
    fn dict(&self, name: &str) -> RResult<&SbtDict<F>>;
    fn tuple(&self, name: &str) -> RResult<&SbtTuple<F>>;

    fn attr<T>(&self, name: &str) -> RResult<T>;
    fn attr2<T>(&self, name: &str, def: T) -> RResult<T>;
}

trait STuple<F: Float> {
    fn string(&self) -> RResult<&str>;
    fn color(&self) -> RResult<Color<F>>;
    fn int3(&self) -> RResult<[usize; 3]>;
    fn int4(&self) -> RResult<[usize; 4]>;
    fn point(&self) -> RResult<Point<F>>;
    fn vector3(&self) -> RResult<Vector<F>>;
    fn vector4(&self) -> RResult<Vector4<F>>;
}

impl<'a, F: Float + Texel + 'static> SDict<F> for &SbtDict<'a, F>
{
    fn float(&self, name: &str) -> RResult<F>
    {
        match self.get(name) {
            Some(val) => val.float(),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn string(&self, name: &str) -> RResult<&str>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Str(string)) => {
                Ok(string)
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn color(&self, name: &str) -> RResult<Color<F>>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Tuple(tuple)) if tuple.len() == 3 => {
                tuple.color()
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn shinemap<'b>(&self, name: &str, resdir: &Path) -> RResult<DynSampler<'b, F, F>>
    {
        let load = |name| {
            info!("{:?}", resdir.join(name));
            Ok(ShineMap::new(image::open(resdir.join(name))?.bilinear(), F::from_u32(128)).dynsampler())
        };

        use SbtValue as S;
        match self.get(name) {
            Some(S::Int(int)) => Ok((F::from_f64(*int as f64)).dynsampler()),
            Some(S::Float(float)) => Ok((*float).dynsampler()),
            Some(S::Str(name)) => load(name),
            Some(S::Block(
                box SbtBlock{
                    name: "map",
                    value
                }
            )) => load(&value.tuple()?.string()?),
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn sampler3<'b>(&self, name: &str, resdir: &Path) -> RResult<DynSampler<'b, F, Color<F>>>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Tuple(tuple)) if tuple.len() == 3 => {
                Ok(tuple.color()?.dynsampler())
            },
            Some(S::Str(name)) => {
                info!("{:?}", resdir.join(name));
                Ok(image::open(resdir.join(name))?.bilinear().dynsampler())
            },
            Some(S::Block(box SbtBlock{name: "map", value})) => {
                let name = value.tuple()?.string()?;
                info!("name: {:#?}", name);
                Ok(image::open(resdir.join(name))?.bilinear().dynsampler())
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn vector(&self, name: &str) -> RResult<Vector<F>>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Tuple(tuple)) if tuple.len() == 3 => {
                tuple.vector3()
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn boolean(&self, name: &str) -> RResult<bool>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Bool(b)) => {
                Ok(*b)
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn dict(&self, name: &str) -> RResult<&SbtDict<F>>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Dict(dict)) => {
                Ok(dict)
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn tuple(&self, name: &str) -> RResult<&SbtTuple<F>>
    {
        use SbtValue as S;
        match self.get(name) {
            Some(S::Tuple(tuple)) => {
                Ok(tuple)
            },
            Some(_) => Err(Error::ParseError("some")),
            None => Err(Error::ParseMissingKey(name.to_string()))
        }
    }

    fn attr<T>(&self, _name: &str) -> RResult<T>
    {
        Err(Error::ParseError("some"))
    }

    fn attr2<T>(&self, _name: &str, _def: T) -> RResult<T>
    {
        Err(Error::ParseError("some"))
    }
}

impl<'a, F: Float> STuple<F> for SbtTuple<'a, F>
{
    fn string(&self) -> RResult<&str>
    {
        match self.as_slice() {
            [SbtValue::Str(s)] => Ok(s),
            _ => Err(Error::ParseError("expected vector3"))
        }
    }

    fn color(&self) -> RResult<Color<F>>
    {
        match self.as_slice() {
            [x, y, z] => Ok(Color::new(
                x.float()?,
                y.float()?,
                z.float()?
            )),
            _ => Err(Error::ParseError("expected color"))
        }
    }

    fn int3(&self) -> RResult<[usize; 3]>
    {
        match self.as_slice() {
            [a, b, c] => Ok([
                a.int()? as usize,
                b.int()? as usize,
                c.int()? as usize
            ]),
            _ => Err(Error::ParseError("expected int3"))
        }
    }

    fn int4(&self) -> RResult<[usize; 4]>
    {
        match self.as_slice() {
            [a, b, c, d] => Ok([
                a.int()? as usize,
                b.int()? as usize,
                c.int()? as usize,
                d.int()? as usize
            ]),
            _ => Err(Error::ParseError("expected int4"))
        }
    }

    fn point(&self) -> RResult<Point<F>>
    {
        match self.as_slice() {
            [x, y] => Ok(Point::new(
                x.float()?,
                y.float()?
            )),
            _ => Err(Error::ParseError("expected point"))
        }
    }

    fn vector3(&self) -> RResult<Vector<F>>
    {
        match self.as_slice() {
            [x, y, z] => Ok(Vector::new(
                x.float()?,
                y.float()?,
                z.float()?
            )),
            _ => Err(Error::ParseError("expected vector3"))
        }
    }

    fn vector4(&self) -> RResult<Vector4<F>>
    {
        match self.as_slice() {
            [x, y, z, w] => Ok(Vector4::new(
                x.float()?,
                y.float()?,
                z.float()?,
                w.float()?
            )),
            _ => Err(Error::ParseError("expected vector4"))
        }
    }
}

#[derive(Debug)]
pub enum SbtValue<'a, F: Float> {
    Int(i64),
    Float(F),
    Str(&'a str),
    Dict(SbtDict<'a, F>),
    Tuple(SbtTuple<'a, F>),
    Block(Box<SbtBlock<'a, F>>),
    Bool(bool),
}

impl<'a, F: Float> SbtValue<'a, F>
{
    pub fn int(&self) -> RResult<i64>
    {
        match self {
            SbtValue::Int(int) => Ok(*int),
            _ => Err(Error::ParseError("number expected")),
        }
    }

    pub fn float(&self) -> RResult<F>
    {
        match self {
            SbtValue::Int(int) => Ok(F::from_f64(*int as f64)),
            SbtValue::Float(float) => Ok(*float),
            _ => Err(Error::ParseError("number expected")),
        }
    }

    pub fn tuple(&self) -> RResult<&'a SbtTuple<F>>
    {
        if let SbtValue::Tuple(ref tuple) = self {
            Ok(tuple)
        } else {
            Err(Error::ParseError("expected tuple"))
        }
    }

    pub fn dict(&self) -> RResult<&'a SbtDict<F>>
    {
        if let SbtValue::Dict(ref dict) = self {
            Ok(dict)
        } else {
            Err(Error::ParseError("expected dictionary"))
        }
    }
}

impl<F: Float> SbtParser2<F>
{
    pub fn dump(pr: Pairs<Rule>) -> RResult<()>
    {
        pub fn _dump(pr: Pair<Rule>, lvl: usize) -> RResult<()>
        {
            print!(
                "{}{:?}",
                "    ".repeat(lvl),
                pr.as_rule(),
            );
            match pr.as_rule() {
                Rule::ident => print!(" '{}'", pr.as_span().as_str()),
                Rule::int |
                Rule::float => print!(" {}", { pr.as_span().as_str() } ),
                _other      => print!(""),
            }
            println!();
            for p in pr.into_inner() {
                _dump(p, lvl + 1)?;
            }
            Ok(())
        }

        for p in pr {
            _dump(p, 0)?;
        }

        Ok(())
    }

    pub fn parse_dict(pr: Pairs<Rule>) -> RResult<SbtValue<F>>
    {
        let mut hash = HashMap::new();
        let mut key = "";
        for p in pr {
            match p.as_rule() {
                Rule::ident => {
                    key = p.as_span().as_str()
                }
                _value => {
                    hash.insert(key.to_string(), Self::parse_value(p)?);
                }
            }
        }
        Ok(SbtValue::Dict(hash))
    }

    pub fn parse_tuple(pr: Pairs<Rule>) -> RResult<SbtValue<F>>
    {
        let mut tuple = vec![];

        /* Manual iteration is significantly faster than map()+collect() */
        for p in pr {
            tuple.push(Self::parse_value(p)?)
        }

        Ok(SbtValue::Tuple(tuple))
    }

    pub fn parse_float(pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        Ok(SbtValue::Float(F::from_f64(pr.as_span().as_str().trim().parse::<f64>()?)))
    }

    pub fn parse_int(pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        Ok(SbtValue::Int(pr.as_span().as_str().trim().parse()?))
    }

    pub fn parse_boolean(pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        match pr.as_span().as_str() {
            "true"  => Ok(SbtValue::Bool(true)),
            "false" => Ok(SbtValue::Bool(false)),
            _       => panic!("internal parser error"),
        }
    }

    pub fn parse_string(pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        let val = pr.as_span().as_str();
        Ok(SbtValue::Str(&val[1..val.len()-1]))
    }

    pub fn parse_value(pr: Pair<Rule>) -> RResult<SbtValue<F>>
    {
        let value = match pr.as_rule() {
            Rule::group    |
            Rule::tuple    |
            Rule::tuple_i3 |
            Rule::tuple_f3 |
            Rule::tuple_f2 => Self::parse_tuple(pr.into_inner())?,
            Rule::dict     => Self::parse_dict(pr.into_inner())?,
            Rule::int      => Self::parse_int(pr)?,
            Rule::float    => Self::parse_float(pr)?,
            Rule::string   => Self::parse_string(pr)?,
            Rule::boolean  => Self::parse_boolean(pr)?,
            Rule::block    => SbtValue::Block(box Self::parse_block(pr.into_inner())?),
            other => return Err(Error::ParseUnsupported(format!("{:?}", other)))
        };
        Ok(value)
    }

    pub fn parse_block(mut pr: Pairs<Rule>) -> RResult<SbtBlock<F>>
    {
        let name = pr.next().unwrap().as_str();
        let value = Self::parse_value(pr.next().unwrap())?;
        Ok(SbtBlock { name, value })
    }

    pub fn ast(pr: Pairs<Rule>) -> RResult<SbtProgram<F>>
    {
        let mut prog = SbtProgram { version: SbtVersion::Sbt1_0, blocks: vec![] };
        let mut name = "";
        for p in pr {
            match p.as_rule() {
                Rule::VERSION => prog.version = SbtVersion::from_str(p.as_str())?,
                Rule::block => prog.blocks.push(Self::parse_block(p.into_inner())?),
                Rule::ident => name = p.as_span().as_str(),
                Rule::dict  => {
                    /* warn!("Workaround for malformed file"); */
                    let value = Self::parse_dict(p.into_inner())?;
                    prog.blocks.push(SbtBlock { name, value })
                }
                Rule::EOI => break,
                other => return Err(Error::ParseUnsupported(format!("{:?}", other)))
            }
        }
        Ok(prog)
    }

}

pub struct SbtBuilder<'a, F: Float>
{
    _p: PhantomData<F>,
    width: u32,
    height: u32,
    resdir: &'a Path,
    version: SbtVersion,
    material: SbtDict<'a, F>,
}

impl<'a, F: Float> SbtBuilder<'a, F>
where
    F: Float + FromStr + Texel + Lerp<Ratio=F> + 'a + 'static,
{
    pub fn new(width: u32, height: u32, resdir: &'a Path) -> Self
    {
        Self { _p: PhantomData{}, width, height, resdir, version: SbtVersion::Sbt1_0, material: SbtDict::new() }
    }

    fn parse_camera(&mut self, dict: impl SDict<F>) -> RResult<Camera<F>>
    {
        let position    = dict.vector("position").unwrap_or_else(|_| Vector::zero());
        let mut viewdir = dict.vector("viewdir").ok();
        let updir       = dict.vector("updir").unwrap_or_else(|_| Vector::unit_y());
        let look_at     = dict.vector("look_at");
        let aspectratio = dict.float("aspectratio").ok();
        let fov         = dict.float("fov").unwrap_or_else(|_| F::from_f32(55.0));

        if viewdir.is_none() && look_at.is_ok() {
            viewdir = Some(look_at? - position);
        }
        if viewdir.is_none() {
            viewdir = Some(-Vector::unit_z())
        }

        info!("Camera:");
        info!("  updir: {:?}", updir);
        info!("  position: {:?}", position);
        info!("  viewdir: {:?}", viewdir);
        info!("  aspectratio: {:?}", aspectratio);
        info!("  updir: {:?}", updir);
        info!("  fov: {:?}", fov);

        Ok(
            Camera::build(
                position,
                viewdir.unwrap(),
                updir,
                fov,
                self.width,
                self.height,
                aspectratio,
            )
        )
    }

    fn parse_point_light(&mut self, dict: impl SDict<F>) -> RResult<PointLight<F>>
    {
        let pos = dict.vector("position")?;
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;
        let a = dict.float("constant_attenuation_coeff").unwrap_or(F::ZERO);
        let b = dict.float("linear_attenuation_coeff").unwrap_or(F::ZERO);
        let c = dict.float("quadratic_attenuation_coeff").unwrap_or(F::ONE);

        let res = PointLight { a, b, c, pos, color };
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_directional_light(&mut self, dict: impl SDict<F>) -> RResult<DirectionalLight<F>>
    {
        let dir = dict.vector("direction")?;
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;

        let res = DirectionalLight { dir, color };
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_material(&mut self, dict: impl SDict<F>) -> RResult<DynMaterial<'a, F>>
    {
        let float = |name| {
            dict.float(name).or_else(|_| (&self.material).float(name))
        };
        let color = |name| {
            dict.color(name).or_else(|_| (&self.material).color(name))
        };

        let shinemap = |name| {
            dict.shinemap(name, self.resdir).or_else(|_| (&self.material).shinemap(name, self.resdir))
        };

        let colormap = |name| {
            dict.sampler3(name, self.resdir).or_else(|_| (&self.material).sampler3(name, self.resdir))
        };

        let idx  = float("index").unwrap_or(F::ZERO);
        let ambi = color("ambient").unwrap_or_else(|_| Color::black());
        let shi  = shinemap("shininess").unwrap_or_else(|_| F::ZERO.dynsampler());
        let emis = colormap("emissive").unwrap_or_else(|_| Color::black().dynsampler());
        let diff = colormap("diffuse").unwrap_or_else(|_| Color::black().dynsampler());
        let spec = colormap("specular").unwrap_or_else(|_| Color::black().dynsampler());
        let tran = colormap("transmissive").unwrap_or_else(|_| Color::black().dynsampler());
        let refl = colormap("reflective").unwrap_or_else(|_| spec.clone());
        let bump = colormap("bump").ok();

        let smart = Smart::new(idx, shi, emis, diff, spec, tran, refl).with_ambient(ambi);
        /* use crate::ColorNormal; let smart = ColorNormal::new(F::ONE).dynamic(); */

        match bump {
            None => Ok(smart.dynamic()),
            Some(b) => Ok(Bumpmap::new(F::from_f32(0.25).dynsampler(), NormalMap::new(b), smart).dynamic())
        }
    }

    fn parse_material_obj(&mut self, dict: impl SDict<F>) -> RResult<DynMaterial<'a, F>>
    {
        self.parse_material(dict.dict("material").unwrap_or(&SbtDict::new()))
    }

    fn parse_polymesh(&mut self, xfrm: Matrix4<F>, dict: impl SDict<F>) -> RResult<Vec<Box<dyn FiniteGeometry<F> + 'a>>>
    {
        let mut tris = vec![];
        let mut points = vec![];
        let mut faces = vec![];
        let mut normals = vec![];
        let mut texture_uvs = vec![];
        let mut materials = vec![];

        if let Ok(nmls) = dict.tuple("normals") {
            for normal in nmls {
                normals.push(normal.tuple()?.vector3()?.xfrm_nml(&xfrm))
            }
        }
        if let Ok(mats) = dict.tuple("materials") {
            for mat in mats {
                materials.push(self.parse_material(&*mat.dict()?)?)
            }
        }
        if let Ok(uvs) = dict.tuple("texture_uv") {
            for uv in uvs {
                texture_uvs.push(uv.tuple()?.point()?)
            }
        }
        if let Ok(path) = dict.string("objfile") {
            info!("Reading {}", path);
            let obj = Obj::load(self.resdir.join(path))?;
            let mesh = TriangleMesh::load_obj(obj, Vector::zero(), F::ONE);
            tris = mesh.tris;
        } else {
            for point in dict.tuple("points")? {
                points.push(point.tuple()?.vector3()?.xfrm_pos(&xfrm))
            }
            for point in dict.tuple("faces")? {
                let tuple = point.tuple()?;
                if tuple.len() == 4 {
                    let f = tuple.int4()?;
                    faces.push([f[0], f[1], f[2]]);
                    faces.push([f[0], f[2], f[3]]);
                } else {
                    faces.push(point.tuple()?.int3()?)
                }
            }
        }
        let mat = self.parse_material_obj(dict)?;

        if normals.is_empty() {
            info!("Generating normals");
            normals = face_normals(&faces, &points);
            /* normals = smooth_normals(&faces, &points); */
        }

        if texture_uvs.is_empty() {
            info!("Generating uv coords");
            texture_uvs = spherical_uvs(&points);
        }

        for face in faces.iter() {
            let m = if !materials.is_empty() {
                Triblend::new(
                    materials[face[0]].clone(),
                    materials[face[1]].clone(),
                    materials[face[2]].clone(),
                ).dynamic()
            } else {
                mat.clone()
            };

            /* let ab = points[face[0]] - points[face[1]]; */
            /* let ac = points[face[0]] - points[face[2]]; */
            /* let n = ab.cross(ac); */

            tris.push(
                Triangle::new(
                    points[face[0]],
                    points[face[1]],
                    points[face[2]],
                    normals[face[0]].normalize(),
                    normals[face[1]].normalize(),
                    normals[face[2]].normalize(),
                    /* n,n,n, */
                    texture_uvs[face[0]],
                    texture_uvs[face[1]],
                    texture_uvs[face[2]],
                    m
                )
            );
        }

        Ok(vec![box TriangleMesh::new(tris)])
    }

    fn build_geometry<'b>(&mut self, blk: &'b SbtValue<F>, xfrm: Matrix4<F>) -> RResult<Vec<Box<dyn FiniteGeometry<F> + 'a>>>
    {
        use SbtValue as S;
        /* info!("block: {:#?}", blk); */
        match blk {
            SbtValue::Block(box SbtBlock { name, value: S::Tuple(tuple) }) => {
                match (*name, tuple.as_slice()) {

                    ("translate", [x, y, z, blk]) => {
                        let (x, y, z) = (x.float()?, y.float()?, z.float()?);
                        info!("translate [{:?}, {:?}, {:?}]", x, y, z);
                        let vec = Vector::new(x, y, z);
                        let x2 = Matrix4::from_translation(vec);
                        self.build_geometry(blk, xfrm * x2)
                    },

                    ("scale", [s, other]) => {
                        let s = s.float()?;
                        info!("scale [{}]", s);
                        let x2 = Matrix4::from_scale(s);
                        self.build_geometry(other, xfrm * x2)
                    },

                    ("scale", [x, y, z, other]) => {
                        let (x, y, z) = (x.float()?, y.float()?, z.float()?);
                        info!("scale [{}, {}, {}]", x, y, z);
                        let x2 = Matrix4::from_nonuniform_scale(x, y, z);
                        self.build_geometry(other, xfrm * x2)
                    },

                    ("rotate", [x, y, z, w, blk]) => {
                        let (x, y, z, w) = (x.float()?, y.float()?, z.float()?, w.float()?);
                        info!("rotate [{}, {}, {}, {}]", x, y, z, w);
                        let x2 = Matrix4::from_axis_angle(Vector::new(x, y, z).normalize(), Rad(w));
                        self.build_geometry(blk, xfrm * x2)
                    },

                    ("transform", [S::Tuple(vx), S::Tuple(vy), S::Tuple(vz), S::Tuple(vw), blk]) => {
                        let x = vx.vector4()?;
                        let y = vy.vector4()?;
                        let z = vz.vector4()?;
                        let w = vw.vector4()?;
                        info!("transform [{:5.2?}, {:5.2?}, {:5.2?}, {:5.2?}]", x, y, z, w);
                        let x2 = Matrix4::from_cols(x, y, z, w);
                        let x2 = match self.version {
                            SbtVersion::Sbt0_9 => x2.transpose(),
                            SbtVersion::Sbt1_0 => x2,
                        };
                        self.build_geometry(blk, xfrm * x2)
                    }

                    other => {
                        info!("unhandled: {:#?}", other);
                        Err(Error::ParseUnsupported(name.to_string()))
                    },
                }
            }

            SbtValue::Block(box SbtBlock { name, value: S::Dict(dict) }) => {
                match (*name, dict) {
                    ("sphere", dict) => {
                        /* info!("Sphere(xfrm={:7.4?})", xfrm); */
                        /* return Ok(box Sphere::new(xfrm, self.parse_material(dict.dict("material").unwrap_or_default())?)) */
                        Ok(vec![box Sphere::new(xfrm, self.parse_material_obj(dict)?)])
                    },

                    ("box", dict) => {
                        /* info!("Cube(xfrm={:7.4?})", xfrm); */
                        Ok(vec![box Cube::new(xfrm, self.parse_material_obj(dict)?)])
                    },

                    ("cone", dict) => {
                        /* info!("Cone(xfrm={:7.4?})", xfrm); */
                        Ok(vec![box Cone::new(
                            dict.float("height").unwrap_or(F::ONE),
                            dict.float("top_radius").unwrap_or(F::ONE),
                            dict.float("bottom_radius").unwrap_or(F::ONE),
                            dict.boolean("capped").unwrap_or(true),
                            xfrm,
                            self.parse_material_obj(dict)?
                        )])
                    },

                    ("square", dict) => {
                        /* info!("Square(xfrm={:7.4?})", xfrm); */
                        Ok(vec![box Square::new(xfrm, self.parse_material_obj(dict)?)])
                    },

                    ("cylinder", dict) => {
                        /* info!("Cube(xfrm={:7.4?})", xfrm); */
                        Ok(vec![box Cylinder::new(xfrm, dict.boolean("capped").unwrap_or(true), self.parse_material_obj(dict)?)])
                    },

                    ("polymesh", dict) => {
                        self.parse_polymesh(xfrm, dict)
                    }

                    _ => {
                        error!("unparsed block: {:?}", blk);
                        Err(Error::ParseUnsupported("foo1".to_string()))
                    },
                }
            }

            SbtValue::Tuple(blks) => {
                let mut res = vec![];
                for blk in blks {
                    res.append(&mut self.build_geometry(blk, xfrm)?)
                }
                Ok(res)
            }

            _ => {
                error!("unparsed block: {:?}", blk);
                Err(Error::ParseUnsupported("foo2".to_string()))
            },
        }
    }

    pub fn build(&mut self, prog: SbtProgram<'a, F>) -> RResult<BoxScene<'a, F>>
    {
        use SbtValue as S;
        let mut cameras = vec![];
        let mut objects: Vec<Box<dyn FiniteGeometry<F> + 'a>> = vec![];
        let mut lights: Vec<Box<dyn Light<F> + 'a>> = vec![];
        let mut ambient = Color::black();
        self.material.clear();
        self.version = prog.version;

        for blk in prog.blocks {
            match (blk.name, blk.value) {
                ("camera",            S::Dict(ref dict)) => cameras.push(self.parse_camera(dict)?),
                ("directional_light", S::Dict(ref dict)) => lights.push(box self.parse_directional_light(dict)?),
                ("point_light",       S::Dict(ref dict)) => lights.push(box self.parse_point_light(dict)?),
                ("ambient_light",     S::Dict(ref dict)) => ambient = dict.color("color").or_else(|_| dict.color("colour"))?,
                ("spot_light",        S::Dict(_)       ) => warn!("spot_light not supported"),
                ("material",          S::Dict(dict)    ) => self.material.extend(dict),

                ("area_light" | "area_light_rect", S::Dict(ref dict)) => {
                    warn!("Simulating {} using point_light", blk.name);
                    lights.push(box self.parse_point_light(dict)?)
                }

                (name, value) => {
                    let block = S::Block(box SbtBlock { name, value });
                    let mut objs = self.build_geometry(&block, Matrix4::identity())?;
                    objects.append(&mut objs)
                }
            }
        }
        Ok(Scene::new(cameras, objects, vec![], lights).with_ambient(ambient))
    }
}