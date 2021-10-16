use super::geo_util::*;

pub struct Square<F: Float, M: Material<F=F>>
{
    xfrm: Matrix4<F>,
    mat: M,
    aabb: AABB,
    ni: usize,
}

impl<F: Float, M: Material<F=F>> Bounded for Square<F, M>
{
    fn aabb(&self) -> AABB {
        self.aabb
    }
}

impl<F: Float, M: Material<F=F>> BHShape for Square<F, M>
{
    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }
}

impl<F: Float, M: Material<F=F>> HitTarget<F> for Square<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let normal = hit.nml.unwrap();
        let normalu = Vector::unit_z().cross(normal).normalize();
        let normalv = normalu.cross(normal).normalize();

        let uv = hit.uv.unwrap();

        Maxel::from_uv(uv.x, uv.y, normal, normalu, normalv, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Square<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.inverse_transform(&self.xfrm)?;

        if r.dir.z == F::ZERO {
            return None
        }

        let t = -r.pos.z / r.dir.z;

        if t <= F::BIAS {
            return None
        }

        let mut p = r.extend(t);
        p.x += F::HALF;
        p.y += F::HALF;

        if p.x < F::ZERO || p.x > F::ONE {
            return None
        }

        if p.y < F::ZERO || p.y > F::ONE {
            return None
        }

        let normal = if r.dir.z > F::ZERO {
            vec3!(F::ZERO, F::ZERO, -F::ONE)
        } else {
            vec3!(F::ZERO, F::ZERO,  F::ONE)
        };

        Some(ray.hit_at(t, self)
             .with_normal(self.xfrm.transform_vector(normal).normalize())
             .with_uv(point!(p.x, p.y)))
    }

}

impl<F: Float, M: Material<F=F>> Square<F, M>
{
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Square<F, M>
    {
        /* Transform all corner points, expand aabb with each result */
        let points = [
            vec3!( F::HALF,  F::HALF,  F::ZERO),
            vec3!( F::HALF, -F::HALF,  F::ZERO),
            vec3!(-F::HALF,  F::HALF,  F::ZERO),
            vec3!(-F::HALF, -F::HALF,  F::ZERO),
        ];
        let mut aabb: AABB = AABB::empty();
        for point in &points {
            let p = point.xfrm(&xfrm);
            aabb.grow_mut(&p.into_point3());
        }
        Square { xfrm, mat, aabb, ni: 0 }
    }
}
