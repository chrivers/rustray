use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use camino::Utf8Path;
use itertools::Itertools;
use obj::Obj;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use cgmath::{Deg, InnerSpace, Matrix, Matrix4, Rad, SquareMatrix, Vector4};

use crate::geometry::{
    Cone, Cube, Cylinder, FiniteGeometry, Sphere, Square, Triangle, TriangleMesh,
};
use crate::light::{AreaLight, Attenuation, DirectionalLight, PointLight, SpotLight};
use crate::material::{BoxMaterial, BumpPower, Bumpmap, Smart, Triblend};
use crate::sampler::{DynSampler, NormalMap, Sampler, SamplerExt, ShineMap, Texel};
use crate::scene::BoxScene;
use crate::types::{Camera, Color, Error, Float, MaterialId, Point, RResult, Vector, Vectorx};

#[derive(Copy, Clone, Debug)]
pub enum SbtVersion {
    Sbt0_9,
    Sbt1_0,
}

impl FromStr for SbtVersion {
    type Err = crate::types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.9" => Ok(Self::Sbt0_9),
            "1.0" => Ok(Self::Sbt1_0),
            _ => Err(Error::ParseError("internal parser error".into())),
        }
    }
}

pub fn face_normals<F: Float>(faces: &[[usize; 3]], points: &[Vector<F>]) -> Vec<Vector<F>> {
    let mut normals = vec![Vector::ZERO; points.len()];
    /* Single-face normals */
    for face in faces {
        let ab = points[face[0]] - points[face[1]];
        let ac = points[face[0]] - points[face[2]];
        let n = ab.cross(ac);
        normals[face[0]] += n;
        normals[face[1]] += n;
        normals[face[2]] += n;
    }
    normals
}

pub fn smooth_normals<F: Float>(faces: &[[usize; 3]], points: &[Vector<F>]) -> Vec<Vector<F>> {
    /* Vertex-smoothed normals */
    let mut norms: HashMap<u64, Vector<F>> = HashMap::new();
    let mut normals = vec![Vector::ZERO; points.len()];

    for face in faces {
        let ab = points[face[0]] - points[face[1]];
        let ac = points[face[0]] - points[face[2]];
        let n = ab.cross(ac);
        normals[face[0]] = n;
        normals[face[1]] = n;
        normals[face[2]] = n;
        *norms.entry(points[face[0]].hash()).or_insert(Vector::ZERO) += n;
        *norms.entry(points[face[1]].hash()).or_insert(Vector::ZERO) += n;
        *norms.entry(points[face[2]].hash()).or_insert(Vector::ZERO) += n;
    }
    for face in faces {
        normals[face[0]] = norms[&points[face[0]].hash()];
        normals[face[1]] = norms[&points[face[1]].hash()];
        normals[face[2]] = norms[&points[face[2]].hash()];
    }
    normals
}

pub fn spherical_uvs<F: Float>(points: &[Vector<F>]) -> Vec<Point<F>> {
    let mut center = Vector::ZERO;
    for point in points {
        center += *point;
    }
    center /= F::from_usize(points.len());

    let mut uvs = vec![];
    for point in points {
        uvs.push((point - center).normalize().polar_uv().into());
    }
    uvs
}

#[derive(Parser)]
#[grammar = "format/sbt2.pest"]
pub struct SbtParser2 {}

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

pub type SbtDict<'a, F> = HashMap<&'a str, SbtValue<'a, F>>;
pub type SbtTuple<'a, F> = Vec<SbtValue<'a, F>>;

trait SDict<F: Float + Texel> {
    fn get_result(&self, name: &str) -> RResult<&SbtValue<'_, F>>;
    fn float(&self, name: &str) -> RResult<F>;
    fn color(&self, name: &str) -> RResult<Color<F>>;
    fn shinemap(&self, name: &str, resdir: &Utf8Path) -> RResult<DynSampler<F, F>>;
    fn sampler3(&self, name: &str, resdir: &Utf8Path) -> RResult<DynSampler<F, Color<F>>>;
    fn string(&self, name: &str) -> RResult<&str>;
    fn vector(&self, name: &str) -> RResult<Vector<F>>;
    fn boolean(&self, name: &str) -> RResult<bool>;
    fn dict(&self, name: &str) -> RResult<&SbtDict<F>>;
    fn tuple(&self, name: &str) -> RResult<&SbtTuple<F>>;
    fn attenuation(&self) -> RResult<Attenuation<F>>;
    fn hash_item(&self, hasher: &mut impl Hasher);
    fn hash(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash_item(&mut hasher);
        hasher.finish()
    }
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

impl<'a, F: Float + Texel> SDict<F> for SbtDict<'a, F> {
    fn get_result(&self, name: &str) -> RResult<&SbtValue<'a, F>> {
        self.get(name)
            .ok_or_else(|| Error::ParseMissingKey(name.into()))
    }

    fn float(&self, name: &str) -> RResult<F> {
        self.get_result(name)?.float()
    }

    fn string(&self, name: &str) -> RResult<&str> {
        self.get_result(name)?.string()
    }

    fn color(&self, name: &str) -> RResult<Color<F>> {
        self.get_result(name)?.tuple()?.color()
    }

    fn vector(&self, name: &str) -> RResult<Vector<F>> {
        self.get_result(name)?.tuple()?.vector3()
    }

    fn boolean(&self, name: &str) -> RResult<bool> {
        self.get_result(name)?.boolean()
    }

    #[allow(clippy::use_self)] // false positive
    fn dict(&self, name: &str) -> RResult<&SbtDict<F>> {
        self.get_result(name)?.dict()
    }

    fn tuple(&self, name: &str) -> RResult<&SbtTuple<F>> {
        self.get_result(name)?.tuple()
    }

    fn shinemap(&self, name: &str, resdir: &Utf8Path) -> RResult<DynSampler<F, F>> {
        let load = |name| {
            info!("{:?}", resdir.join(name));
            Ok(
                ShineMap::new(image::open(resdir.join(name))?.bilinear(), F::from_u32(128))
                    .dynsampler(),
            )
        };

        match self.get_result(name)? {
            SbtValue::Int(int) => Ok((F::from_f64(*int as f64)).dynsampler()),
            SbtValue::Float(float) => Ok((*float).dynsampler()),
            SbtValue::Str(name) => load(name),
            SbtValue::Block(box SbtBlock { name: "map", value }) => load(&value.tuple()?.string()?),
            _ => Err(Error::ParseError(format!(
                "Could not parse sampler, found {self:?}"
            ))),
        }
    }

    fn sampler3(&self, name: &str, resdir: &Utf8Path) -> RResult<DynSampler<F, Color<F>>> {
        let load = |filename| {
            let file = resdir.join(filename);
            info!("name: {file:?}");
            Ok(image::open(file)?.bilinear().dynsampler())
        };

        match self.get_result(name)? {
            SbtValue::Tuple(tuple) => Ok(tuple.color()?.dynsampler()),
            SbtValue::Str(name) => load(*name),
            SbtValue::Block(box SbtBlock { name: "map", value }) => {
                let name = value.tuple()?.string()?;
                load(name)
            }
            _ => Err(Error::ParseError(format!(
                "Could not parse sampler, found {self:?}"
            ))),
        }
    }

    fn attenuation(&self) -> RResult<Attenuation<F>> {
        let a = self.float("constant_attenuation_coeff").unwrap_or(F::ZERO);
        let b = self.float("linear_attenuation_coeff").unwrap_or(F::ZERO);
        let c = self.float("quadratic_attenuation_coeff").unwrap_or(F::ONE);
        Ok(Attenuation { a, b, c })
    }

    fn hash_item(&self, hasher: &mut impl Hasher) {
        for (k, v) in self.iter().sorted_by_key(|kv| kv.0) {
            hasher.write(k.as_bytes());
            v.hash_item(hasher);
        }
    }
}

impl<'a, F: Float + Texel> STuple<F> for SbtTuple<'a, F> {
    fn string(&self) -> RResult<&str> {
        match self.as_slice() {
            [SbtValue::Str(s)] => Ok(s),
            _ => Err(Error::ParseError(format!(
                "expected vector3, found {self:?}"
            ))),
        }
    }

    fn color(&self) -> RResult<Color<F>> {
        match self.as_slice() {
            [x, y, z] => Ok(Color::new(x.float()?, y.float()?, z.float()?)),
            _ => Err(Error::ParseError(format!("expected color, found {self:?}"))),
        }
    }

    fn int3(&self) -> RResult<[usize; 3]> {
        match self.as_slice() {
            [a, b, c] => Ok([a.int()? as usize, b.int()? as usize, c.int()? as usize]),
            _ => Err(Error::ParseError(format!("expected int3, found {self:?}"))),
        }
    }

    fn int4(&self) -> RResult<[usize; 4]> {
        match self.as_slice() {
            [a, b, c, d] => Ok([
                a.int()? as usize,
                b.int()? as usize,
                c.int()? as usize,
                d.int()? as usize,
            ]),
            _ => Err(Error::ParseError(format!("expected int4, found {self:?}"))),
        }
    }

    fn point(&self) -> RResult<Point<F>> {
        match self.as_slice() {
            [x, y] => Ok(Point::new(x.float()?, y.float()?)),
            _ => Err(Error::ParseError(format!("expected point, found {self:?}"))),
        }
    }

    fn vector3(&self) -> RResult<Vector<F>> {
        match self.as_slice() {
            [x, y, z] => Ok(Vector::new(x.float()?, y.float()?, z.float()?)),
            _ => Err(Error::ParseError(format!(
                "expected vector3, found {self:?}"
            ))),
        }
    }

    fn vector4(&self) -> RResult<Vector4<F>> {
        match self.as_slice() {
            [x, y, z, w] => Ok(Vector4::new(x.float()?, y.float()?, z.float()?, w.float()?)),
            _ => Err(Error::ParseError(format!(
                "expected vector4, found {self:?}"
            ))),
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

impl<'a, F: Float + Texel> SbtValue<'a, F> {
    pub fn int(&self) -> RResult<i64> {
        if let SbtValue::Int(int) = self {
            Ok(*int)
        } else {
            Err(Error::ParseError(format!(
                "number expected, found {self:?}"
            )))
        }
    }

    pub fn boolean(&self) -> RResult<bool> {
        if let SbtValue::Bool(b) = self {
            Ok(*b)
        } else {
            Err(Error::ParseError(format!(
                "expected boolean, found {self:?}"
            )))
        }
    }

    pub fn string(&self) -> RResult<&'a str> {
        if let SbtValue::Str(s) = self {
            Ok(s)
        } else {
            Err(Error::ParseError(format!(
                "expected boolean, found {self:?}"
            )))
        }
    }

    pub fn float(&self) -> RResult<F> {
        match self {
            SbtValue::Int(int) => Ok(F::from_f64(*int as f64)),
            SbtValue::Float(float) => Ok(*float),
            _ => Err(Error::ParseError(format!(
                "number expected, found {self:?}"
            ))),
        }
    }

    pub fn tuple(&self) -> RResult<&'a SbtTuple<F>> {
        if let SbtValue::Tuple(ref tuple) = self {
            Ok(tuple)
        } else {
            Err(Error::ParseError(format!("expected tuple, found {self:?}")))
        }
    }

    pub fn dict(&self) -> RResult<&'a SbtDict<F>> {
        if let SbtValue::Dict(ref dict) = self {
            Ok(dict)
        } else {
            Err(Error::ParseError(format!(
                "expected dictionary, found {self:?}"
            )))
        }
    }

    pub fn hash_item(&self, hasher: &mut impl Hasher) {
        match self {
            SbtValue::Int(i) => hasher.write_i64(*i),
            SbtValue::Float(f) => hasher.write_u64(f.to_f64().expect("wat").to_bits()),
            SbtValue::Str(s) => hasher.write(s.as_bytes()),
            SbtValue::Dict(dict) => dict.hash_item(hasher),
            SbtValue::Tuple(t) => t.iter().for_each(|val| val.hash_item(hasher)),
            SbtValue::Block(b) => {
                hasher.write(b.name.as_bytes());
                b.value.hash_item(hasher);
            }
            SbtValue::Bool(b) => hasher.write_u8(u8::from(*b)),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash_item(&mut hasher);
        hasher.finish()
    }
}

impl SbtParser2 {
    pub fn dump(pr: Pairs<Rule>) -> RResult<()> {
        pub fn _dump(pr: Pair<Rule>, lvl: usize) -> RResult<()> {
            print!("{}{:?}", "    ".repeat(lvl), pr.as_rule(),);
            match pr.as_rule() {
                Rule::ident => print!(" '{pr}'"),
                Rule::int | Rule::float => print!(" {pr}"),
                _other => print!(""),
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

    pub fn parse_dict<F: Float>(pr: Pair<Rule>) -> RResult<SbtValue<F>> {
        let mut hash = HashMap::new();
        for [key, value] in pr.into_inner().array_chunks() {
            hash.insert(key.as_str(), Self::parse_value(value)?);
        }
        Ok(SbtValue::Dict(hash))
    }

    fn parse_tuple<F: Float>(p: Pair<Rule>) -> RResult<SbtValue<F>> {
        let tuple: RResult<_> = p.into_inner().map(Self::parse_value).collect();

        Ok(SbtValue::Tuple(tuple?))
    }

    pub fn parse_float<'a, F: Float>(pr: &Pair<Rule>) -> RResult<SbtValue<'a, F>> {
        Ok(SbtValue::Float(F::from_f64(pr.as_str().trim().parse()?)))
    }

    pub fn parse_int<'a, F: Float>(pr: &Pair<Rule>) -> RResult<SbtValue<'a, F>> {
        Ok(SbtValue::Int(pr.as_str().trim().parse()?))
    }

    pub fn parse_boolean<'a, F: Float>(pr: &Pair<Rule>) -> RResult<SbtValue<'a, F>> {
        match pr.as_str() {
            "true" => Ok(SbtValue::Bool(true)),
            "false" => Ok(SbtValue::Bool(false)),
            _ => Err(Error::ParseError("internal parser error".into())),
        }
    }

    pub fn parse_string<'a, F: Float>(pr: &Pair<'a, Rule>) -> RResult<SbtValue<'a, F>> {
        let val = pr.as_str();
        Ok(SbtValue::Str(&val[1..val.len() - 1]))
    }

    pub fn parse_value<F: Float>(pr: Pair<Rule>) -> RResult<SbtValue<F>> {
        let value = match pr.as_rule() {
            Rule::group | Rule::tuple | Rule::tuple_i3 | Rule::tuple_f3 | Rule::tuple_f2 => {
                Self::parse_tuple(pr)?
            }
            Rule::dict => Self::parse_dict(pr)?,
            Rule::int => Self::parse_int(&pr)?,
            Rule::float => Self::parse_float(&pr)?,
            Rule::string => Self::parse_string(&pr)?,
            Rule::boolean => Self::parse_boolean(&pr)?,
            Rule::block => SbtValue::Block(Box::new(Self::parse_block(pr)?)),
            other => return Err(Error::ParseUnsupported(format!("{other:?}"))),
        };
        Ok(value)
    }

    pub fn parse_block<F: Float>(pr: Pair<Rule>) -> RResult<SbtBlock<F>> {
        let mut pr = pr.into_inner();
        let name = pr.next().unwrap().as_str();
        let value = Self::parse_value(pr.next().unwrap())?;
        Ok(SbtBlock { name, value })
    }

    pub fn ast<F: Float>(pr: Pairs<Rule>) -> RResult<SbtProgram<F>> {
        let mut prog = SbtProgram {
            version: SbtVersion::Sbt1_0,
            blocks: vec![],
        };
        let mut name = "";
        for p in pr {
            match p.as_rule() {
                Rule::VERSION => prog.version = SbtVersion::from_str(p.as_str())?,
                Rule::block => prog.blocks.push(Self::parse_block(p)?),
                Rule::ident => name = p.as_str(),
                Rule::dict => {
                    /* warn!("Workaround for malformed file"); */
                    let value = Self::parse_dict(p)?;
                    prog.blocks.push(SbtBlock { name, value });
                }
                Rule::EOI => break,
                other => return Err(Error::ParseUnsupported(format!("{other:?}"))),
            }
        }
        Ok(prog)
    }
}

pub struct SbtBuilder<'a, F: Float> {
    resdir: &'a Utf8Path,
    version: SbtVersion,
    material: SbtDict<'a, F>,
    scene: &'a mut BoxScene<F>,
    hashmat: HashMap<u64, MaterialId>,
}

impl<'a, F> SbtBuilder<'a, F>
where
    F: Float + Texel,
    rand::distributions::Standard: rand::distributions::Distribution<F>,
{
    #[must_use]
    pub fn new(resdir: &'a Utf8Path, scene: &'a mut BoxScene<F>) -> Self {
        Self {
            resdir,
            version: SbtVersion::Sbt1_0,
            material: SbtDict::new(),
            scene,
            hashmat: HashMap::new(),
        }
    }

    fn parse_camera(dict: &impl SDict<F>) -> RResult<Camera<F>> {
        let position = dict.vector("position").unwrap_or(Vector::ZERO);
        let mut viewdir = dict.vector("viewdir").ok();
        let updir = dict.vector("updir").unwrap_or(Vector::UNIT_Y);
        let look_at = dict.vector("look_at");
        let aspectratio = dict.float("aspectratio").unwrap_or(F::ONE);
        let fov = dict.float("fov").unwrap_or_else(|_| F::from_f32(55.0));

        if viewdir.is_none() && look_at.is_ok() {
            viewdir = Some(look_at? - position);
        }
        if viewdir.is_none() {
            viewdir = Some(-Vector::UNIT_Z);
        }

        info!("Camera:");
        info!("  updir: {:?}", updir);
        info!("  position: {:?}", position);
        info!("  viewdir: {:?}", viewdir);
        info!("  aspectratio: {:?}", aspectratio);
        info!("  updir: {:?}", updir);
        info!("  fov: {:?}", fov);

        Ok(Camera::build(
            position,
            viewdir.unwrap(),
            updir,
            fov,
            aspectratio,
        ))
    }

    fn parse_point_light(dict: &impl SDict<F>) -> RResult<PointLight<F>> {
        let pos = dict.vector("position")?;
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;
        let attn = dict.attenuation()?;

        let res = PointLight { pos, attn, color };
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_spot_light(dict: &impl SDict<F>) -> RResult<SpotLight<F>> {
        let pos = dict.vector("position")?;
        let dir = dict.vector("direction")?.normalize();
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;
        let attn = dict.attenuation()?;
        let umbra = Deg(dict.float("umbra").unwrap_or_else(|_| F::from_f32(45.0))).into();
        let penumbra = Deg(dict.float("penumbra").unwrap_or_else(|_| F::from_f32(45.0))).into();

        let res = SpotLight {
            attn,
            umbra,
            penumbra,
            pos,
            dir,
            color,
        };
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_area_light(dict: &impl SDict<F>) -> RResult<AreaLight<F>> {
        let pos = dict.vector("position")?;
        let dir = dict.vector("direction")?.normalize();
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;
        let attn = dict.attenuation()?;
        let upd = dict.vector("updir")?.normalize();
        let width = dict.float("width").unwrap_or(F::ONE);
        let height = dict.float("height").unwrap_or(F::ONE);
        let res = AreaLight::new(attn, pos, dir, upd, color, width, height);
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_directional_light(dict: &impl SDict<F>) -> RResult<DirectionalLight<F>> {
        let dir = dict.vector("direction")?;
        let color = dict.color("color").or_else(|_| dict.color("colour"))?;

        let res = DirectionalLight::new(dir, color);
        info!("{:7.3?}", res);
        Ok(res)
    }

    fn parse_material_props(&mut self, dict: &impl SDict<F>) -> MaterialId {
        let black = |_| Color::BLACK.dynsampler();
        let float = |name| dict.float(name).or_else(|_| self.material.float(name));
        let color = |name| dict.color(name).or_else(|_| self.material.color(name));

        let shinemap = |name| {
            dict.shinemap(name, self.resdir)
                .or_else(|_| self.material.shinemap(name, self.resdir))
        };

        let colormap = |name| {
            dict.sampler3(name, self.resdir)
                .or_else(|_| self.material.sampler3(name, self.resdir))
        };

        let idx = float("index").unwrap_or(F::ZERO);
        let ambi = color("ambient").unwrap_or(Color::BLACK);
        let shi = shinemap("shininess").unwrap_or_else(|_| F::ZERO.dynsampler());
        let emis = colormap("emissive").unwrap_or_else(black);
        let diff = colormap("diffuse").unwrap_or_else(black);
        let spec = colormap("specular").unwrap_or_else(black);
        let tran = colormap("transmissive").unwrap_or_else(black);
        let refl = colormap("reflective")
            .or_else(|_| colormap("specular"))
            .unwrap_or_else(black);

        let smart = Smart::new(idx, shi, emis, diff, spec, tran, refl).with_ambient(ambi);

        let res: BoxMaterial<F> = match colormap("bump").ok() {
            None => Box::new(smart),
            Some(b) => Box::new(Bumpmap::new(
                BumpPower(F::from_f32(0.25)),
                NormalMap::new(b),
                smart,
            )),
        };

        self.scene.materials.insert(res)
    }

    fn parse_material(&mut self, dict: &impl SDict<F>) -> MaterialId {
        let hash = dict.hash();

        if let Some(id) = self.hashmat.get(&hash) {
            *id
        } else {
            let id = self.parse_material_props(dict);
            self.hashmat.insert(hash, id);
            id
        }
    }

    fn parse_material_obj(&mut self, dict: &impl SDict<F>) -> MaterialId {
        self.parse_material(dict.dict("material").unwrap_or(&SbtDict::new()))
    }

    fn parse_polymesh(
        &mut self,
        xfrm: Matrix4<F>,
        dict: &impl SDict<F>,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let mut tris = vec![];
        let mut points = vec![];
        let mut faces = vec![];
        let mut normals = vec![];
        let mut texture_uvs = vec![];
        let mut materials = vec![];

        for normal in dict.tuple("normals").into_iter().flatten() {
            normals.push(normal.tuple()?.vector3()?);
        }
        for mat in dict.tuple("materials").into_iter().flatten() {
            materials.push(self.parse_material(mat.dict()?));
        }
        for uv in dict.tuple("texture_uv").into_iter().flatten() {
            texture_uvs.push(uv.tuple()?.point()?);
        }
        if let Ok(path) = dict.string("objfile") {
            info!("Reading {}", path);
            let obj = Obj::load(self.resdir.join(path))?;
            crate::format::obj::load(obj, self.scene)?;
            return Ok(vec![]);
        }

        for point in dict.tuple("points")? {
            points.push(point.tuple()?.vector3()?);
        }
        let center = points.iter().sum::<Vector<F>>() / F::from_usize(points.len());
        let pos_xfrm = Matrix4::from_translation(center);
        points.iter_mut().for_each(|p| *p -= center);
        for point in dict.tuple("faces")? {
            let tuple = point.tuple()?;
            if tuple.len() == 4 {
                let f = tuple.int4()?;
                faces.push([f[0], f[1], f[2]]);
                faces.push([f[0], f[2], f[3]]);
            } else {
                faces.push(point.tuple()?.int3()?);
            }
        }

        let mat = self.parse_material_obj(dict);

        if normals.is_empty() {
            info!("Generating normals");
            normals = face_normals(&faces, &points);
            /* normals = smooth_normals(&faces, &points); */
        }

        if texture_uvs.is_empty() {
            info!("Generating uv coords");
            texture_uvs = spherical_uvs(&points);
        }

        let mut mats: HashMap<u64, MaterialId> = HashMap::new();

        for face in &faces {
            let m: MaterialId = if !materials.is_empty() {
                let (m0, m1, m2) = (materials[face[0]], materials[face[1]], materials[face[2]]);
                let mut hasher = std::hash::DefaultHasher::new();
                m0.hash(&mut hasher);
                m1.hash(&mut hasher);
                m2.hash(&mut hasher);
                let hash = hasher.finish();
                *mats.entry(hash).or_insert_with(|| {
                    if m0 == m1 && m1 == m2 {
                        m0
                    } else {
                        let mat = Box::new(Triblend::new(m0, m1, m2));
                        self.scene.materials.insert(mat)
                    }
                })
            } else {
                mat
            };

            /* let ab = points[face[0]] - points[face[1]]; */
            /* let ac = points[face[0]] - points[face[2]]; */
            /* let n = ab.cross(ac); */

            tris.push(Triangle::new(
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
                m,
            ));
        }

        Ok(vec![Box::new(TriangleMesh::new(tris, xfrm * pos_xfrm))])
    }

    #[allow(clippy::too_many_lines)]
    fn build_geometry(
        &mut self,
        blk: &SbtValue<F>,
        xfrm: Matrix4<F>,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        /* info!("block: {:#?}", blk); */
        match blk {
            SbtValue::Block(box SbtBlock {
                name,
                value: SbtValue::Tuple(tuple),
            }) => match (*name, tuple.as_slice()) {
                ("translate", [x, y, z, blk]) => {
                    let (x, y, z) = (x.float()?, y.float()?, z.float()?);
                    info!("translate [{:?}, {:?}, {:?}]", x, y, z);
                    let vec = Vector::new(x, y, z);
                    let x2 = Matrix4::from_translation(vec);
                    self.build_geometry(blk, xfrm * x2)
                }

                ("scale", [s, other]) => {
                    let s = s.float()?;
                    info!("scale [{}]", s);
                    let x2 = Matrix4::from_scale(s);
                    self.build_geometry(other, xfrm * x2)
                }

                ("scale", [x, y, z, other]) => {
                    let (x, y, z) = (x.float()?, y.float()?, z.float()?);
                    info!("scale [{}, {}, {}]", x, y, z);
                    let x2 = Matrix4::from_nonuniform_scale(x, y, z);
                    self.build_geometry(other, xfrm * x2)
                }

                ("rotate", [x, y, z, w, blk]) => {
                    let (x, y, z, w) = (x.float()?, y.float()?, z.float()?, w.float()?);
                    info!("rotate [{}, {}, {}, {}]", x, y, z, w);
                    let x2 = Matrix4::from_axis_angle(Vector::new(x, y, z).normalize(), Rad(w));
                    self.build_geometry(blk, xfrm * x2)
                }

                (
                    "transform",
                    [SbtValue::Tuple(vx), SbtValue::Tuple(vy), SbtValue::Tuple(vz), SbtValue::Tuple(vw), blk],
                ) => {
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

                other => Err(Error::ParseUnsupported(format!("unhandled: {other:#?}"))),
            },

            SbtValue::Block(box SbtBlock {
                name,
                value: SbtValue::Dict(dict),
            }) => {
                match (*name, dict) {
                    ("sphere", dict) => {
                        /* info!("Sphere(xfrm={:7.4?})", xfrm); */
                        /* return Ok(box Sphere::new(xfrm, self.parse_material(dict.dict("material").unwrap_or_default())?)) */
                        Ok(vec![Box::new(Sphere::new(
                            xfrm,
                            self.parse_material_obj(dict),
                        ))])
                    }

                    ("box", dict) => {
                        /* info!("Cube(xfrm={:7.4?})", xfrm); */
                        Ok(vec![Box::new(Cube::new(
                            xfrm,
                            self.parse_material_obj(dict),
                        ))])
                    }

                    ("cone", dict) => {
                        /* info!("Cone(xfrm={:7.4?})", xfrm); */
                        Ok(vec![Box::new(Cone::new(
                            dict.float("height").unwrap_or(F::ONE),
                            dict.float("top_radius").unwrap_or(F::ONE),
                            dict.float("bottom_radius").unwrap_or(F::ONE),
                            dict.boolean("capped").unwrap_or(true),
                            xfrm,
                            self.parse_material_obj(dict),
                        ))])
                    }

                    ("square", dict) => {
                        /* info!("Square(xfrm={:7.4?})", xfrm); */
                        Ok(vec![Box::new(Square::new(
                            xfrm,
                            self.parse_material_obj(dict),
                        ))])
                    }

                    ("cylinder", dict) => {
                        /* info!("Cube(xfrm={:7.4?})", xfrm); */
                        Ok(vec![Box::new(Cylinder::new(
                            xfrm,
                            dict.boolean("capped").unwrap_or(true),
                            self.parse_material_obj(dict),
                        ))])
                    }

                    ("polymesh", dict) => self.parse_polymesh(xfrm, dict),

                    _ => Err(Error::ParseUnsupported(format!("unparsed block: {blk:?}"))),
                }
            }

            SbtValue::Tuple(blks) => {
                let mut res = vec![];
                for blk in blks {
                    res.append(&mut self.build_geometry(blk, xfrm)?);
                }
                Ok(res)
            }

            _ => Err(Error::ParseUnsupported(format!("unparsed block: {blk:?}"))),
        }
    }

    pub fn build(mut self, prog: SbtProgram<'a, F>) -> RResult<()> {
        self.material.clear();
        self.version = prog.version;

        for blk in prog.blocks {
            #[allow(clippy::mut_mut)]
            let scene = &mut self.scene;
            let lights = &mut scene.lights;
            match (blk.name, blk.value) {
                ("camera", SbtValue::Dict(ref dict)) => {
                    scene.cameras.push(Self::parse_camera(dict)?);
                }
                ("directional_light", SbtValue::Dict(ref dict)) => {
                    lights.push(Box::new(Self::parse_directional_light(dict)?));
                }
                ("point_light", SbtValue::Dict(ref dict)) => {
                    lights.push(Box::new(Self::parse_point_light(dict)?));
                }
                ("ambient_light", SbtValue::Dict(ref dict)) => {
                    scene.ambient = dict.color("color").or_else(|_| dict.color("colour"))?;
                }
                ("spot_light", SbtValue::Dict(ref dict)) => {
                    lights.push(Box::new(Self::parse_spot_light(dict)?));
                }
                ("material", SbtValue::Dict(dict)) => self.material.extend(dict),

                ("area_light" | "area_light_rect", SbtValue::Dict(ref dict)) => {
                    lights.push(Box::new(Self::parse_area_light(dict)?));
                }

                (name, value) => {
                    let block = SbtValue::Block(Box::new(SbtBlock { name, value }));
                    for obj in self.build_geometry(&block, Matrix4::identity())? {
                        self.scene.add_object(obj);
                    }
                }
            }
        }

        self.scene.recompute_bvh()
    }
}
