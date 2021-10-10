use super::geo_util::*;

pub struct Cylinder<F: Float, M: Material<F=F>>
{
    xfrm: Matrix4<F>,
    capped: bool,
    mat: M,
}

impl<F: Float, M: Material<F=F>> HitTarget<F> for Cylinder<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let normal = hit.nml.unwrap_or_else(Vector::unit_x);
        let normalu = Vector::unit_y();
        let normalv = normalu.cross(normal).normalize();

        let (u, v) = normal.polar_uv();

        Maxel::from_uv(u, v, normal, normalu, normalv, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> RayTarget<F> for Cylinder<F, M>
{
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cylinder.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.inverse_transform(&self.xfrm)?;

        fn isect_body<F: Float>(r: &Ray<F>, capped: bool) -> Option<(F, Vector<F>)>
        {
            let x0 = r.pos.x;
            let y0 = r.pos.y;
            let x1 = r.dir.x;
            let y1 = r.dir.y;

            let a = x1*x1 + y1*y1;
            let b = F::TWO * (x0*x1 + y0*y1);
            let c = x0*x0 + y0*y0 - F::ONE;

            if a == F::ZERO {
                /* This implies that x1 = 0.0 and y1 = 0.0, which further */
                /* implies that the ray is aligned with the body of the cylinder, */
                /* so no intersection. */
                return None
            }

            let discriminant = b * b - F::FOUR * a * c;

            if discriminant < F::ZERO {
                return None
            }

            let discriminant = discriminant.sqrt();

            let t2 = (-b + discriminant) / (F::TWO * a);

            if t2 <= F::BIAS {
                return None
            }

            let t1 = (-b - discriminant) / (F::TWO * a);

            if t1 > F::BIAS {
                /* Two intersections. */
                let p = r.extend(t1);
                let z = p[2];
                if z >= F::ZERO && z <= F::ONE {
                    return Some((t1, vec3!(p[0], p[1], F::ZERO)))
                }
            }

            let p = r.extend(t2);

            let z = p[2];

            if z >= F::ZERO && z <= F::ONE {

                let mut normal = vec3!(p[0], p[1], F::ZERO);
                /* In case we are _inside_ the _uncapped_ cone, we need to flip the normal. */
                /* Essentially, the cone in this case is a double-sided surface */
                /* and has _2_ normals */
                if !capped && r.dir.dot(normal) > F::ZERO {
                    normal = -normal
                }

                return Some((t2, normal))
            }

            None
        }

        fn isect_caps<F: Float>(r: &Ray<F>) -> Option<(F, Vector<F>)>
        {
            let pz = r.pos.z;
            let dz = r.dir.z;

            if dz == F::ZERO {
                 return None
            }

            let t1;
            let t2;

            if dz > F::ZERO {
                 t1 = (-pz)/dz;
                 t2 = (F::ONE-pz)/dz;
            } else {
                 t1 = (F::ONE-pz)/dz;
                 t2 = (-pz)/dz;
            }

            if t2 < F::BIAS {
                return None
            }

            if t1 >= F::BIAS {
                let p = r.extend(t1);
                if (p[0]*p[0] + p[1]*p[1]) <= F::ONE {
                    let n = if dz > F::ZERO {
                        /* Intersection with cap at z = 0. */
                        -Vector::unit_z()
                    } else {
                        Vector::unit_z()
                    };
                    return Some((t1, n))
                }
            } else {
                let p = r.extend(t2);
                if (p[0]*p[0] + p[1]*p[1]) <= F::ONE {
                    let n = if dz > F::ZERO {
                        /* Intersection with interior of cap at z = 1. */
                        Vector::unit_z()
                    } else {
                        -Vector::unit_z()
                    };
                    return Some((t2, n))
                }
            }

            None
        }

        let body = isect_body(&r, self.capped);

        if self.capped {
            if let Some((t1, n1)) = isect_caps(&r) {
                if let Some((t2, n2)) = body {
                    if t2 < t1 {
                        return Some(ray.hit_at(t2, self, Some(self.xfrm.transform_vector(n2).normalize())))
                    }
                }
                return Some(ray.hit_at(t1, self, Some(self.xfrm.transform_vector(n1).normalize())))
            }
        }

        if let Some((t2, n2)) = body {
            Some(ray.hit_at(t2, self, Some(self.xfrm.transform_vector(n2).normalize())))
        } else {
            None
        }
    }

}

impl<F: Float, M: Material<F=F>> Cylinder<F, M>
{
    pub fn new(xfrm: Matrix4<F>, capped: bool, mat: M) -> Cylinder<F, M>
    {
        Cylinder { xfrm, capped, mat }
    }
}
