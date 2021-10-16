use super::geo_util::*;

pub struct Cone<F: Float, M: Material<F=F>>
{
    height: F,
    top_r: F,
    bot_r: F,
    capped: bool,
    mat: M,
    xfrm: Matrix4<F>,
    aabb: AABB,
    ni: usize,
}

impl<F: Float, M: Material<F=F>> Bounded for Cone<F, M>
{
    fn aabb(&self) -> AABB {
        self.aabb
    }
}

impl<F: Float, M: Material<F=F>> BHShape for Cone<F, M>
{
    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }
}

impl<F: Float, M: Material<F=F>> HitTarget<F> for Cone<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        // let normal = self.pos.normal_to(hit.pos);
        let normal = hit.nml.unwrap_or_else(Vector::unit_x);
        let normalu = Vector::unit_y();
        let normalv = normalu.cross(normal).normalize();

        let (u, v) = normal.polar_uv();

        Maxel::from_uv(u, v, normal, normalu, normalv, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Cone<F, M>
{
    /* Adapted from publicly-available code for University of Washington's course csep557 */
    /* https://courses.cs.washington.edu/courses/csep557/01sp/projects/trace/Cone.cpp */
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.inverse_transform(&self.xfrm)?;

        let bot_r = self.bot_r.abs().max(F::BIAS);
        let top_r = self.top_r.abs().max(F::BIAS);

        let mut beta = (top_r - bot_r) / self.height;

        if beta.abs() < F::BIAS {
            beta = F::BIAS
        }

        let mut gamma;
        gamma = if beta < F::ZERO {
            top_r / beta
        } else {
            bot_r / beta
        };

        if gamma < F::ZERO {
            gamma -= self.height;
        }

        let beta_squared = beta * beta;

        fn good_root<F: Float>(root: &Vector<F>, height: F) -> bool
        {
            !(root.z < F::ZERO || root.z > height)
        }

        let mut normal: Vector<F> = Vector::unit_x();

        let r0: Vector<F> = r.pos;
        let rd: Vector<F> = r.dir;
        let pz = r0.z;
        let dz = rd.z;

        let a = rd.x*rd.x + rd.y*rd.y - beta_squared * rd.z*rd.z;

        if a == F::ZERO {
            /* We're in the x-y plane, no intersection */
            return None
        }

        let b = F::TWO * (r0.x*rd.x + r0.y*rd.y - beta_squared * ((r0.z + gamma) * rd.z));
        let c = -beta_squared*(gamma + r0.z)*(gamma + r0.z) + r0.x * r0.x + r0.y * r0.y;

        let mut discriminant = b * b - F::FOUR * a * c;

        if discriminant <= F::ZERO {
            return None
        }

        discriminant = discriminant.sqrt();

        let mut root = F::BIAS;

        /* We have two roots, so calculate them */
        let near_root = (-b + discriminant) / (F::TWO * a);
        let far_root  = (-b - discriminant) / (F::TWO * a);

        /* This is confusing, but it figures out which */
        /* root is closer and puts into root */
        let near_good = good_root(&r.extend(near_root), self.height);
        if near_good && (near_root > root) {
            root = near_root;
            normal = vec3!((r.extend(root)).x, (r.extend(root)).y, -F::TWO * beta_squared * (r.extend(root).z + gamma));
        }

        let far_good = good_root(&r.extend(far_root), self.height);
        if far_good && ((near_good && (far_root < root)) || (far_root > F::BIAS)) {
            root = far_root;
            normal = vec3!((r.extend(root)).x, (r.extend(root)).y, -F::TWO * beta_squared * (r.extend(root).z + gamma));
        }

        /* In case we are _inside_ the _uncapped_ cone, we need to flip the normal. */
        /* Essentially, the cone in this case is a double-sided surface */
        /* and has _2_ normals */
        if !self.capped && (normal.dot(r.dir)) > F::ZERO {
            normal = -normal;
        }

        /* These are to help with finding caps */
        let t1 = (            - pz) / dz;
        let t2 = (self.height - pz) / dz;

        if self.capped {
            let p = r.extend(t1);

            if p[0]*p[0] + p[1]*p[1] <= self.bot_r*self.bot_r && t1 < root && t1 > F::BIAS {
                root = t1;
                if dz > F::ZERO {
                    /* Intersection with cap at z = 0. */
                    normal = -Vector::unit_z();
                } else {
                    normal =  Vector::unit_z();
                }
            }

            let q = r.extend(t2);

            if q[0]*q[0] + q[1]*q[1] <= self.top_r*self.top_r && t2 < root && t2 > F::BIAS {
                root = t2;
                if dz > F::ZERO {
                    /* Intersection with interior of cap at z = 1. */
                    normal =  Vector::unit_z();
                } else {
                    normal = -Vector::unit_z();
                }
            }
        }

        if root <= F::BIAS {
            return None
        }

        Some(ray.hit_at(root, self).with_normal(self.xfrm.transform_vector(normal).normalize()))
    }

}

impl<F: Float, M: Material<F=F>> Cone<F, M>
{
    pub fn new(height: F, top_r: F, bot_r: F, capped: bool, xfrm: Matrix4<F>, mat: M) -> Cone<F, M>
    {
        let m = bot_r.max(top_r);
        let points = [
            vec3!( m,  m,  F::ZERO),
            vec3!( m, -m,  F::ZERO),
            vec3!(-m,  m,  F::ZERO),
            vec3!(-m, -m,  F::ZERO),
            vec3!( m,  m,  height),
            vec3!( m, -m,  height),
            vec3!(-m,  m,  height),
            vec3!(-m, -m,  height),
        ];
        let mut aabb: AABB = AABB::empty();
        for point in &points {
            let p = point.xfrm(&xfrm);
            aabb.grow_mut(&p.into_point3());
        }
        Cone { height, top_r, bot_r, capped, mat, xfrm, aabb, ni: 0 }
    }
}
