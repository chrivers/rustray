use std::fmt::Debug;
use std::str::FromStr;
use std::marker::PhantomData;
use std::path::Path;

use cgmath::{Vector3, Vector4, Matrix4, InnerSpace, Transform, Rad, Matrix, SquareMatrix};
use num_traits::Zero;

use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

use crate::geometry::{Sphere, Cylinder, Cone, Triangle, TriangleMesh};
use crate::lib::{Camera};
use crate::lib::{RResult, Error::ParseError};
use crate::lib::{PointLight, DirectionalLight};
use crate::scene::{Scene, BoxScene};
use crate::material::{Phong, Smart, Triblend};
use crate::{Vector, Point, Float, Color, Material, DynMaterial, Sampler, DynSampler, BilinearSampler, RayTarget, Vectorx, Light, point, vec3};

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
    F: Float + FromStr + 'static,
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
                    _ => Err(ParseError())
                }
            }
            _ => Err(ParseError())
        }
    }

    pub fn parse_material<'a>(p: Pair<Rule>, resdir: &Path) -> RResult<DynMaterial<'a, F>>
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

    pub fn parse_camera(p: Pair<Rule>, width: usize, height: usize) -> RResult<Camera<F>>
    {
        let mut position: Vector<F> = Vector::zero();
        let mut viewdir: Option<Vector<F>> = None;
        let mut updir: Vector<F> = Vector::unit_y();
        let mut look_at: RResult<Vector<F>> = Err(ParseError());
        let mut aspectratio: Option<F> = None;
        let mut fov: F = F::from_u32(55);
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::position    => position = Self::parse_val3(q),
                Rule::viewdir     => viewdir = Some(Self::parse_val3(q)),
                Rule::aspectratio => aspectratio = Some(Self::parse_val1(q)?),
                Rule::updir       => updir = Self::parse_val3(q),
                Rule::fov         => fov = Self::parse_val1(q)?,
                Rule::look_at     => look_at = Ok(Self::parse_val3(q)),
                _ => { error!("{}", q) },
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

        Ok(
            Camera::build(
                position,
                viewdir.unwrap(),
                updir,
                fov,
                width,
                height,
                aspectratio,
            )
        )
    }

    /* Geometry types */

    pub fn parse_geo_cyl(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
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

    pub fn parse_geo_sph(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
                Rule::name => {},
                other => error!("unsupported: {:?}", other)
            }
        }
        let edge = Vector3::unit_z().xfrm(&xfrm);
        let pos = Vector3::zero().xfrm(&xfrm);

        info!("Sphere({:.4?}, {:.4?})", pos, (pos - edge).magnitude());
        Ok(box Sphere::new(pos, (pos - edge).magnitude(), mat))
    }

    pub fn parse_geo_box(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
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

    pub fn parse_geo_sqr(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat = Self::parse_material(rule, resdir)?,
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

    pub fn parse_geo_plm(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
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
                Rule::material_spec => {
                    mat = Self::parse_material(rule, resdir)?
                },

                Rule::points => for f in rule.into_inner() {
                    // info!("point: {:?}", f);
                    points.push(Self::parse_val3b(f).xfrm(&xfrm))
                    /* points.push(parse_val3(f.into_inner().next().ok_or(ParseError())?).xfrm(&xfrm)) */
                }
                Rule::faces => for f in rule.into_inner() {
                    // info!("face: {:?}", f);
                    faces.push(Self::parse_int3(f))
                }
                Rule::normals => for f in rule.into_inner() {
                    info!("norm: {:?}", f);
                    // normals.push(parse_val3b(f));
                    normals.push(xfrm.transform_vector(Self::parse_val3b(f)));
                }
                Rule::materials => for f in rule.into_inner() {
                    // info!("material: {:#?}", f);
                    materials.push(Self::parse_material(f, resdir)?);
                }
                Rule::texture_uv => for f in rule.into_inner() {
                    // info!("face: {:?}", f);
                    texture_uvs.push(Self::parse_val2(f));
                }
                other => error!("unsupported: {:?}", other)
            }
        }

        info!("building..");

        if normals.is_empty() {
            normals = points.iter().map(|_| Vector::zero()).collect();
            for face in &faces {
                let ab = points[face[0]] - points[face[1]];
                let ac = points[face[0]] - points[face[2]];
                let n = ab.cross(ac);
                normals[face[0]] += n;
                normals[face[1]] += n;
                normals[face[2]] += n;
            }
        }

        info!("building2..");

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

            let (uv1, uv2, uv3) = if !texture_uvs.is_empty() {
                (
                    texture_uvs[face[0]],
                    texture_uvs[face[1]],
                    texture_uvs[face[2]],
                )
            } else {
                let z = Point::zero();
                (z, z, z)
            };
            tris.push(
                Triangle::new(
                    points[face[0]],
                    points[face[1]],
                    points[face[2]],
                    normals[face[0]].normalize(),
                    normals[face[1]].normalize(),
                    normals[face[2]].normalize(),
                    uv1, uv2, uv3,
                    m
                )
            );
        }

        Ok(box TriangleMesh::new(tris))
    }

    pub fn parse_geo_con(p: Pair<Rule>, xfrm: Matrix4<F>, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let body = p.into_inner();
        let mut mat = Phong::white().dynamic();
        let mut height = F::BIAS;
        let mut top_r = F::BIAS;
        let mut bot_r = F::BIAS;
        let mut capped = false;

        for rule in body {
            match rule.as_rule() {
                Rule::material_spec => mat    = Self::parse_material(rule, resdir)?,
                Rule::height        => height = Self::parse_val1(rule)?,
                Rule::top_radius    => top_r  = Self::parse_val1(rule)?,
                Rule::bottom_radius => bot_r  = Self::parse_val1(rule)?,
                Rule::capped        => capped = Self::parse_bool(rule),
                Rule::material_ref => {},
                other => error!("unsupported: {:?}", other)
            }
        }

        info!("Cone(h={}, t={}, b={}, xfrm={:.4?})", height, top_r, bot_r, xfrm);
        Ok(box Cone::new(height, top_r, bot_r, capped, xfrm, mat))
    }

    /* Light types */

    pub fn parse_point_light(p: Pair<Rule>) -> RResult<PointLight<F>>
    {
        let mut pos: RResult<Vector<F>> = Err(ParseError());
        let mut color: RResult<Vector<F>> = Err(ParseError());
        let mut a = F::ZERO;
        let mut b = F::ZERO;
        let mut c = F::ONE;
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::position => pos   = Ok(Self::parse_val3(q)),
                Rule::color    => color = Ok(Self::parse_val3(q)),
                Rule::coeff0   => a     = Self::parse_val1(q)?,
                Rule::coeff1   => b     = Self::parse_val1(q)?,
                Rule::coeff2   => c     = Self::parse_val1(q)?,
                _ => { error!("{}", q) },
            }
        }
        let pos = pos?;
        let color = color?;
        let color = Color::new(color.x, color.y, color.z);
        let res = PointLight { a, b, c, pos, color };
        info!("{:?}", res);
        Ok(res)
    }

    pub fn parse_directional_light(p: Pair<Rule>) -> RResult<DirectionalLight<F>>
    {
        let mut direction: RResult<Vector<F>> = Err(ParseError());
        let mut color: RResult<Vector<F>> = Err(ParseError());
        for q in p.into_inner() {
            match q.as_rule() {
                Rule::direction => direction = Ok(Self::parse_val3(q)),
                Rule::color     => color     = Ok(Self::parse_val3(q)),
                _ => { error!("{}", q) },
            }
        }
        let dir = direction?;
        let color = color?;
        let color = Color::new(color.x, color.y, color.z);
        let res = DirectionalLight { dir, color };
        info!("{:?}", res);
        Ok(res)
    }

    pub fn parse_translate(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let mut body = p.into_inner();
        let a = Self::parse_num1(body.next().unwrap());
        let b = Self::parse_num1(body.next().unwrap());
        let c = Self::parse_num1(body.next().unwrap());
        let x2 = Matrix4::from_translation(Vector3::new(a, b, c));
        Self::parse_statement(body.next().unwrap(), xfrm * x2, version, resdir)
    }

    pub fn parse_rotate(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        let mut body = p.into_inner();
        let a = Self::parse_num1(body.next().unwrap());
        let b = Self::parse_num1(body.next().unwrap());
        let c = Self::parse_num1(body.next().unwrap());
        let d = Self::parse_num1(body.next().unwrap());
        let x2 = Matrix4::from_axis_angle(Vector3::new(a, b, c).normalize(), Rad(d));
        Self::parse_statement(body.next().unwrap(), xfrm * x2, version, resdir)
    }

    pub fn parse_transform(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
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

    pub fn parse_scale(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
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
            _ => return Err(ParseError()),
        };
        Self::parse_statement(it.remove(0), xfrm * x2, version, resdir)
    }

    pub fn parse_statement(p: Pair<Rule>, xfrm: Matrix4<F>, version: SbtVersion, resdir: &Path) -> RResult<Box<dyn RayTarget<F>>>
    {
        /* info!("-- statement: {:?} {:.4?}", p.as_rule(), xfrm); */
        match p.as_rule() {
            Rule::translate => Self::parse_translate(p, xfrm, version, resdir),
            Rule::rotate    => Self::parse_rotate(p, xfrm, version, resdir),
            Rule::transform => Self::parse_transform(p, xfrm, version, resdir),
            Rule::scale     => Self::parse_scale(p, xfrm, version, resdir),
            Rule::geo_cyl => Self::parse_geo_cyl(p, xfrm, resdir),
            Rule::geo_sph => Self::parse_geo_sph(p, xfrm, resdir),
            Rule::geo_box => Self::parse_geo_box(p, xfrm, resdir),
            Rule::geo_sqr => Self::parse_geo_sqr(p, xfrm, resdir),
            Rule::geo_plm => Self::parse_geo_plm(p, xfrm, resdir),
            Rule::geo_con => Self::parse_geo_con(p, xfrm, resdir),

            _ => { error!("unimplemented: {:?}", p.as_rule()); Err(ParseError()) },
        }
    }

    pub fn parse_file(p: Pairs<Rule>, resdir: &Path, width: usize, height: usize) -> RResult<BoxScene<F>>
    {
        let mut cameras = vec![];
        let mut objects: Vec<Box<dyn RayTarget<F>>> = vec![];
        let mut lights: Vec<Box<dyn Light<F>>> = vec![];
        let mut version: SbtVersion = SbtVersion::Sbt1_0;

        for r in p {
            match r.as_rule() {
                Rule::VERSION => version = SbtVersion::from_str(r.as_str())?,
                Rule::EOI     => break,

                Rule::camera            => cameras.push(Self::parse_camera(r, width, height)?),
                Rule::directional_light => lights.push(Box::new(Self::parse_directional_light(r)?)),
                Rule::point_light       => lights.push(Box::new(Self::parse_point_light(r)?)),

                Rule::area_light => {
                    warn!("Simulating area_light using point_light");
                    lights.push(Box::new(Self::parse_point_light(r)?))
                }

                Rule::spot_light        => warn!("unimplemented: spot_light"),
                Rule::ambient_light     => warn!("unimplemented: ambient_light"),
                Rule::material_obj      => warn!("unimplemented: material_obj"),

                _ => objects.push(Self::parse_statement(r, Matrix4::identity(), version, resdir)?),
            }
        }
        Ok(Scene { cameras, objects, lights })
    }

}
