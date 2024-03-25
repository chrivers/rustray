use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;

use obj::Obj;

use cgmath::{InnerSpace, Matrix, Matrix4, Rad, SquareMatrix, Vector3, Vector4};
use num_traits::Zero;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use crate::geometry::{
    Cone, Cube, Cylinder, FiniteGeometry, Sphere, Square, Triangle, TriangleMesh,
};
use crate::material::{Bumpmap, Phong, Smart, Triblend};
use crate::sampler::{NormalMap, ShineMap, Texel};
use crate::scene::{BoxScene, Scene};
use crate::types::float::Lerp;
use crate::types::Camera;
use crate::types::{DirectionalLight, PointLight};
use crate::types::{Error::ParseError, RResult};
use crate::{
    point, Color, DynMaterial, DynSampler, Float, Light, Material, Point, Sampler, SamplerExt,
    Vector, Vectorx,
};

#[derive(Parser)]
#[grammar = "format/sbt.pest"]
pub struct SbtParser<F> {
    _p: PhantomData<F>,
}

#[derive(Copy, Clone, Debug)]
pub enum SbtVersion {
    Sbt0_9,
    Sbt1_0,
}

impl FromStr for SbtVersion {
    type Err = crate::types::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.9" => Ok(SbtVersion::Sbt0_9),
            "1.0" => Ok(SbtVersion::Sbt1_0),
            _ => panic!("impossible"),
        }
    }
}

fn hash<F: Float>(p: Vector<F>) -> (u64, u64, u64) {
    (
        p.x.to_f64().unwrap().to_bits(),
        p.y.to_f64().unwrap().to_bits(),
        p.z.to_f64().unwrap().to_bits(),
    )
}

pub fn face_normals<F: Float>(faces: &[[usize; 3]], points: &[Vector<F>]) -> Vec<Vector<F>> {
    let mut normals = vec![Vector::zero(); points.len()];
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
    let mut norms: HashMap<(u64, u64, u64), Vector<F>> = HashMap::new();
    let mut normals = vec![Vector::zero(); points.len()];

    for face in faces {
        let ab = points[face[0]] - points[face[1]];
        let ac = points[face[0]] - points[face[2]];
        let n = ab.cross(ac);
        normals[face[0]] = n;
        normals[face[1]] = n;
        normals[face[2]] = n;
        *norms
            .entry(hash(points[face[0]]))
            .or_insert_with(Vector::zero) += n;
        *norms
            .entry(hash(points[face[1]]))
            .or_insert_with(Vector::zero) += n;
        *norms
            .entry(hash(points[face[2]]))
            .or_insert_with(Vector::zero) += n;
    }
    for face in faces {
        normals[face[0]] = norms[&hash(points[face[0]])];
        normals[face[1]] = norms[&hash(points[face[1]])];
        normals[face[2]] = norms[&hash(points[face[2]])];
    }
    normals
}

pub fn spherical_uvs<F: Float>(points: &[Vector<F>]) -> Vec<Point<F>> {
    let mut center = Vector::zero();
    for point in points {
        center += *point
    }
    center /= F::from_usize(points.len());

    let mut uvs = vec![];
    for point in points {
        uvs.push((point - center).normalize().polar_uv().into())
    }
    uvs
}

impl<F> SbtParser<F>
where
    F: Float + FromStr + Texel + Lerp<Ratio = F> + 'static,
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

    pub fn parse_val1(p: Pair<Rule>) -> RResult<F> {
        let mut m = p.into_inner();
        m.next()
            .unwrap()
            .as_str()
            .trim()
            .parse()
            .or(Err(ParseError("val1")))
    }

    pub fn parse_val2(p: Pair<Rule>) -> Point<F> {
        let m = p.into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(F::ZERO))
            .collect::<Vec<_>>();
        point!(v[0], v[1])
    }

    pub fn parse_int3(p: Pair<Rule>) -> [usize; 3] {
        let m = p.into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(0))
            .collect::<Vec<usize>>();
        [v[0], v[1], v[2]]
    }

    pub fn parse_int4(p: Pair<Rule>) -> [usize; 4] {
        let m = p.into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(0))
            .collect::<Vec<usize>>();
        [v[0], v[1], v[2], v[3]]
    }

    pub fn parse_val3(p: Pair<Rule>) -> Vector<F> {
        Self::parse_val3b(p.into_inner().next().unwrap())
    }

    pub fn parse_val3b(p: Pair<Rule>) -> Vector<F> {
        let m = p.into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(F::ZERO))
            .collect::<Vec<F>>();
        Vector::new(v[0], v[1], v[2])
    }

    pub fn parse_val4(p: Pair<Rule>) -> Vector4<F> {
        let m = p.into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(F::ZERO))
            .collect::<Vec<F>>();
        Vector4::new(v[0], v[1], v[2], v[3])
    }

    pub fn parse_color(p: Pair<Rule>) -> Color<F> {
        let m = p.into_inner().next().unwrap().into_inner();
        let v = m
            .map(|x| x.as_str().trim().parse().unwrap_or(F::ZERO))
            .collect::<Vec<F>>();
        if v.len() != 3 {
            return Color::black();
        }
        Color::new(v[0], v[1], v[2])
    }

    /* Composite types */

    pub fn parse_sampler3<'a>(
        p: Pair<Rule>,
        resdir: &Path,
    ) -> RResult<DynSampler<'a, F, Color<F>>> {
        let ps = p.into_inner().next().unwrap();
        match ps.as_rule() {
            Rule::sampler3 => {
                let q = ps.into_inner().next().unwrap();
                match q.as_rule() {
                    Rule::map => {
                        let s = q.into_inner().as_str();
                        let name = &s[1..s.len() - 1];
                        Ok(image::open(resdir.join(name))?.bilinear().dynsampler())
                    }
                    Rule::val3 => {
                        let m = q.into_inner();
                        let v = m
                            .map(|x| x.as_str().trim().parse().unwrap_or(F::ZERO))
                            .collect::<Vec<F>>();
                        Ok(Color::new(v[0], v[1], v[2]).dynsampler())
                    }
                    // "unimplemented: {:?}"
                    _ => Err(ParseError("sampler3")),
                }
            }
            _ => Err(ParseError("sampler3")),
        }
    }

    pub fn parse_sampler1<'a>(p: Pair<Rule>, resdir: &Path) -> RResult<DynSampler<'a, F, F>> {
        let ps = p.into_inner().next().unwrap();
        match ps.as_rule() {
            Rule::sampler1 => {
                let q = ps.into_inner().next().unwrap();
                match q.as_rule() {
                    Rule::map => {
                        let s = q.into_inner().as_str();
                        let name = &s[1..s.len() - 1];
                        let img = image::open(resdir.join(name))?;
                        Ok(ShineMap::new(img.bilinear(), F::from_u32(255)).dynsampler())
                    }
                    Rule::val1 => {
                        let m = q.into_inner().as_str();
                        let v = m.trim().parse().unwrap_or(F::ZERO);
                        Ok(v.dynsampler())
                    }
                    other => {
                        error!("Unknown sampler prop: {:?}", other);
                        Err(ParseError("sampler1"))
                    }
                }
            }

            other => {
                error!("Unknown sampler prop: {:?}", other);
                Err(ParseError("sampler1"))
            }
        }
    }

    pub fn parse_material<'a>(p: Pair<Rule>, resdir: &Path) -> RResult<DynMaterial<'a, F>> {
        let mut diff = Color::black().dynsampler();
        let mut spec = Color::black().dynsampler();
        let mut refl = None;
        let mut ambi = Color::black();
        let mut bump = None;
        let mut tran = Color::black().dynsampler();
        let mut emis = Color::black().dynsampler();
        let mut shi = F::ZERO.dynsampler();
        let mut idx = F::ZERO;
        let mut _gls = F::ZERO;
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::mat_diffuse => diff = Self::parse_sampler3(q, resdir)?,
                Rule::mat_specular => spec = Self::parse_sampler3(q, resdir)?,
                Rule::mat_reflective => refl = Some(Self::parse_sampler3(q, resdir)?),
                Rule::mat_ambient => ambi = Self::parse_color(q.into_inner().next().unwrap()),
                Rule::mat_transmissive => tran = Self::parse_sampler3(q, resdir)?,
                Rule::mat_emissive => emis = Self::parse_sampler3(q, resdir)?,
                Rule::mat_shininess => shi = Self::parse_sampler1(q, resdir)?,
                Rule::mat_index => idx = Self::parse_val1(q)?,
                Rule::mat_glossiness => _gls = Self::parse_val1(q)?,
                Rule::mat_bump => bump = Some(Self::parse_sampler3(q, resdir)?),
                Rule::name => {}
                other => {
                    error!("Unknown material prop: {:?}", other);
                    return Err(ParseError("material"));
                }
            }
        }

        let smart = Smart::new(
            idx,
            shi,
            emis,
            diff,
            spec.clone(),
            tran,
            refl.unwrap_or_else(|| spec.clone()),
        )
        .with_ambient(ambi);

        match bump {
            None => Ok(smart.dynamic()),
            Some(b) => Ok(
                Bumpmap::new(F::from_f32(0.25).dynsampler(), NormalMap::new(b), smart).dynamic(),
            ),
        }
    }

    pub fn parse_camera(p: Pair<Rule>, width: u32, height: u32) -> RResult<Camera<F>> {
        let mut position: Vector<F> = Vector::zero();
        let mut viewdir: Option<Vector<F>> = None;
        let mut updir: Vector<F> = Vector::unit_y();
        let mut look_at: RResult<Vector<F>> = Err(ParseError("look_at"));
        let mut aspectratio: Option<F> = None;
        let mut fov: F = F::from_u32(55);
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::position => position = Self::parse_val3(q),
                Rule::viewdir => viewdir = Some(Self::parse_val3(q)),
                Rule::aspectratio => aspectratio = Some(Self::parse_val1(q)?),
                Rule::updir => updir = Self::parse_val3(q),
                Rule::fov => fov = Self::parse_val1(q)?,
                Rule::look_at => look_at = Ok(Self::parse_val3(q)),
                _ => {
                    error!("{}", q)
                }
            }
        }

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

        Ok(Camera::build(
            position,
            viewdir.unwrap(),
            updir,
            fov,
            width,
            height,
            aspectratio,
        ))
    }

    /* Geometry types */

    pub fn parse_geo_cyl(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();
        let mut capped = true;

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                Rule::capped => capped = Self::parse_bool(rule),
                other => error!("unsupported: {:?}", other),
            }
        }
        info!("Cylinder(xfrm={:7.4?}, capped={})", xfrm, capped);
        Ok(vec![Box::new(Cylinder::new(xfrm, capped, mat))])
    }

    pub fn parse_geo_sph(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                Rule::name => {}
                other => error!("unsupported: {:?}", other),
            }
        }

        info!("Sphere(xfrm={:7.4?})", xfrm);
        Ok(vec![Box::new(Sphere::new(xfrm, mat))])
    }

    pub fn parse_geo_box(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                other => error!("unsupported: {:?}", other),
            }
        }

        info!("Cube(xfrm={:7.4?})", xfrm);
        Ok(vec![Box::new(Cube::new(xfrm, mat))])
    }

    pub fn parse_geo_sqr(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                other => error!("unsupported: {:?}", other),
            }
        }

        info!("Square(xfrm={:7.4?})", xfrm);
        Ok(vec![Box::new(Square::new(xfrm, mat))])
    }

    pub fn parse_geo_plm(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();
        let mut tris = vec![];
        let mut points = vec![];
        let mut faces = vec![];
        let mut normals = vec![];
        let mut texture_uvs = vec![];
        let mut materials: Vec<DynMaterial<F>> = vec![];
        // let mut material_names: HashMap<&str, &DynMaterial<F>> = HashMap::new();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,

                Rule::points => {
                    for f in rule.into_inner() {
                        // info!("point: {:?}", f);
                        points.push(Self::parse_val3b(f).xfrm_pos(&xfrm))
                        /* points.push(parse_val3(f.into_inner().next().ok_or(ParseError())?).xfrm(&xfrm)) */
                    }
                }
                Rule::faces3 => {
                    for f in rule.into_inner() {
                        // info!("face: {:?}", f);
                        faces.push(Self::parse_int3(f))
                    }
                }
                Rule::faces4 => {
                    for f in rule.into_inner() {
                        // info!("face: {:?}", f);
                        let f = Self::parse_int4(f);
                        faces.push([f[0], f[1], f[2]]);
                        faces.push([f[0], f[2], f[3]]);
                    }
                }
                Rule::normals => {
                    for f in rule.into_inner() {
                        /* info!("norm: {:?}", f); */
                        normals.push(Self::parse_val3b(f).xfrm_nml(&xfrm));
                    }
                }
                Rule::materials => {
                    for f in rule.into_inner() {
                        // info!("material: {:#?}", f);
                        materials.push(Self::parse_material(f, resdir)?);
                    }
                }
                Rule::texture_uv => {
                    for f in rule.into_inner() {
                        // info!("face: {:?}", f);
                        texture_uvs.push(Self::parse_val2(f));
                    }
                }
                Rule::objfile => {
                    let path = rule.into_inner().next().unwrap().as_str();
                    let path = &path[1..path.len() - 1];
                    info!("Reading {}", path);
                    let obj = Obj::load(resdir.join(path))?;
                    tris = crate::format::obj::load(obj, Vector::zero(), F::ONE)?;
                }
                other => error!("unsupported: {:?}", other),
            }
        }

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
                )
                .dynamic()
            } else {
                mat.clone()
            };

            tris.push(Triangle::new(
                points[face[0]],
                points[face[1]],
                points[face[2]],
                normals[face[0]].normalize(),
                normals[face[1]].normalize(),
                normals[face[2]].normalize(),
                texture_uvs[face[0]],
                texture_uvs[face[1]],
                texture_uvs[face[2]],
                m,
            ));
        }

        info!("TriangleMesh(tris={})", tris.len());
        Ok(vec![Box::new(TriangleMesh::new(tris))])
    }

    pub fn parse_geo_con(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();
        let mut height = F::BIAS;
        let mut top_r = F::BIAS;
        let mut bot_r = F::BIAS;
        let mut capped = true;

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                Rule::height => height = Self::parse_val1(rule)?,
                Rule::top_radius => top_r = Self::parse_val1(rule)?,
                Rule::bottom_radius => bot_r = Self::parse_val1(rule)?,
                Rule::capped => capped = Self::parse_bool(rule),
                Rule::material_ref => {}
                other => error!("unsupported: {:?}", other),
            }
        }

        info!(
            "Cone(h={:.3}, t={:.3}, b={:.3}, xfrm={:.4?})",
            height, top_r, bot_r, xfrm
        );
        Ok(vec![Box::new(Cone::new(
            height, top_r, bot_r, capped, xfrm, mat,
        ))])
    }

    /* Light types */

    pub fn parse_point_light(p: Pair<Rule>) -> RResult<PointLight<F>> {
        let mut pos: RResult<Vector<F>> = Err(ParseError("missing position"));
        let mut color: RResult<Vector<F>> = Err(ParseError("missing color"));
        let mut a = F::ZERO;
        let mut b = F::ZERO;
        let mut c = F::ONE;
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::position => pos = Ok(Self::parse_val3(q)),
                Rule::color => color = Ok(Self::parse_val3(q)),
                Rule::coeff0 => a = Self::parse_val1(q)?,
                Rule::coeff1 => b = Self::parse_val1(q)?,
                Rule::coeff2 => c = Self::parse_val1(q)?,
                _ => {
                    error!("{}", q)
                }
            }
        }
        let pos = pos?;
        let color = color?;
        let color = Color::new(color.x, color.y, color.z);
        let res = PointLight {
            a,
            b,
            c,
            pos,
            color,
        };
        info!("{:7.3?}", res);
        Ok(res)
    }

    pub fn parse_directional_light(p: Pair<Rule>) -> RResult<DirectionalLight<F>> {
        let mut direction: RResult<Vector<F>> = Err(ParseError("missing direction"));
        let mut color: RResult<Vector<F>> = Err(ParseError("missing color"));
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::direction => direction = Ok(Self::parse_val3(q)),
                Rule::color => color = Ok(Self::parse_val3(q)),
                _ => {
                    error!("{}", q)
                }
            }
        }
        let dir = direction?;
        let color = color?;
        let color = Color::new(color.x, color.y, color.z);
        let res = DirectionalLight { dir, color };
        info!("{:7.3?}", res);
        Ok(res)
    }

    pub fn parse_translate(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let mut body = p.into_inner();
        let a = Self::parse_num1(body.next().unwrap());
        let b = Self::parse_num1(body.next().unwrap());
        let c = Self::parse_num1(body.next().unwrap());
        let x2 = Matrix4::from_translation(Vector3::new(a, b, c));
        Self::parse_statement(body.next().unwrap(), xfrm * x2, version, resdir)
    }

    pub fn parse_rotate(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let mut body = p.into_inner();
        let a = Self::parse_num1(body.next().unwrap());
        let b = Self::parse_num1(body.next().unwrap());
        let c = Self::parse_num1(body.next().unwrap());
        let d = Self::parse_num1(body.next().unwrap());
        let x2 = Matrix4::from_axis_angle(Vector3::new(a, b, c).normalize(), Rad(d));
        Self::parse_statement(body.next().unwrap(), xfrm * x2, version, resdir)
    }

    pub fn parse_transform(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let mut body = p.into_inner();
        let a = Self::parse_val4(body.next().unwrap());
        let b = Self::parse_val4(body.next().unwrap());
        let c = Self::parse_val4(body.next().unwrap());
        let d = Self::parse_val4(body.next().unwrap());
        let x2 = Matrix4::from_cols(a, b, c, d);
        let x2 = match version {
            SbtVersion::Sbt0_9 => x2.transpose(),
            SbtVersion::Sbt1_0 => x2,
        };
        Self::parse_statement(body.next().unwrap(), xfrm * x2, version, resdir)
    }

    pub fn parse_scale(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let body = p.into_inner();
        let mut it: Vec<Pair<Rule>> = body.collect();

        let x2 = match it.len() {
            2 => {
                let a = Self::parse_num1(it.remove(0));
                Matrix4::from_scale(a)
            }
            4 => {
                let a = Self::parse_num1(it.remove(0));
                let b = Self::parse_num1(it.remove(0));
                let c = Self::parse_num1(it.remove(0));
                Matrix4::from_nonuniform_scale(a, b, c)
            }
            _ => return Err(ParseError("invalid scale")),
        };
        Self::parse_statement(it.remove(0), xfrm * x2, version, resdir)
    }

    pub fn parse_group(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        let mut res = vec![];
        for r in p.into_inner() {
            let mut stmt = Self::parse_statement(r, xfrm, version, resdir)?;
            res.append(&mut stmt);
        }
        Ok(res)
    }

    pub fn parse_statement(
        p: Pair<Rule>,
        xfrm: Matrix4<F>,
        version: SbtVersion,
        resdir: &Path,
    ) -> RResult<Vec<Box<dyn FiniteGeometry<F>>>> {
        /* info!("-- statement: {:?} {:.4?}", p.as_rule(), xfrm); */
        match p.as_rule() {
            Rule::translate => Self::parse_translate(p, xfrm, version, resdir),
            Rule::rotate => Self::parse_rotate(p, xfrm, version, resdir),
            Rule::transform => Self::parse_transform(p, xfrm, version, resdir),
            Rule::scale => Self::parse_scale(p, xfrm, version, resdir),
            Rule::geo_cyl => Self::parse_geo_cyl(p, xfrm, resdir),
            Rule::geo_sph => Self::parse_geo_sph(p, xfrm, resdir),
            Rule::geo_box => Self::parse_geo_box(p, xfrm, resdir),
            Rule::geo_sqr => Self::parse_geo_sqr(p, xfrm, resdir),
            Rule::geo_plm => Self::parse_geo_plm(p, xfrm, resdir),
            Rule::geo_con => Self::parse_geo_con(p, xfrm, resdir),
            Rule::group => Self::parse_group(p, xfrm, version, resdir),

            _ => {
                error!("unimplemented: {:?}", p.as_rule());
                Err(ParseError("statement"))
            }
        }
    }

    pub fn parse_file<'a>(
        p: Pairs<Rule>,
        resdir: &Path,
        width: u32,
        height: u32,
    ) -> RResult<BoxScene<'a, F>> {
        let mut cameras = vec![];
        let mut objects: Vec<Box<dyn FiniteGeometry<F>>> = vec![];
        let mut lights: Vec<Box<dyn Light<F>>> = vec![];
        let mut version: SbtVersion = SbtVersion::Sbt1_0;
        let mut ambient = Color::black();

        for r in p {
            match r.as_rule() {
                Rule::VERSION => version = SbtVersion::from_str(r.as_str())?,
                Rule::EOI => break,

                Rule::camera => cameras.push(Self::parse_camera(r, width, height)?),
                Rule::directional_light => lights.push(Box::new(Self::parse_directional_light(r)?)),
                Rule::point_light => lights.push(Box::new(Self::parse_point_light(r)?)),

                Rule::area_light => {
                    warn!("Simulating area_light using point_light");
                    lights.push(Box::new(Self::parse_point_light(r)?))
                }

                Rule::spot_light => warn!("unimplemented: spot_light"),
                Rule::ambient_light => ambient = Self::parse_color(r.into_inner().next().unwrap()),
                Rule::material_obj => warn!("unimplemented: material_obj"),

                _ => objects.append(&mut Self::parse_statement(
                    r,
                    Matrix4::identity(),
                    version,
                    resdir,
                )?),
            }
        }
        Ok(Scene::new(cameras, objects, vec![], lights)?.with_ambient(ambient))
    }
}
