use super::geo_util::*;
use num_traits::Zero;

pub struct Sphere<F: Float, M: Material<F=F>>
{
    xfrm: Matrix4<F>,
    mat: M,
    aabb: AABB,
    ni: usize,
}

impl<F: Float, M: Material<F=F>> Bounded for Sphere<F, M>
{
    fn aabb(&self) -> AABB {
        self.aabb
    }
}

impl<F: Float, M: Material<F=F>> BHShape for Sphere<F, M>
{
    fn set_bh_node_index(&mut self, index: usize) {
        self.ni = index;
    }

    fn bh_node_index(&self) -> usize {
        self.ni
    }
}

impl<F: Float, M: Material<F=F>> HitTarget<F> for Sphere<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let up = Vector::unit_z().xfrm(&self.xfrm);
        let normal = hit.nml.unwrap();
        let normalu = up.cross(normal).normalize();
        let normalv = normalu.cross(normal).normalize();

        let (u, v) = normal.polar_uv();

        Maxel::from_uv(u, v, normal, normalu, normalv, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Sphere<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.inverse_transform(&self.xfrm)?;

        let result = r.intersect_sphere(&Vector::zero(), F::ONE)?;
        let normal = r.extend(result);

        Some(
            ray.hit_at(result, self)
                .with_normal(normal.xfrm_normal(&self.xfrm))
        )
    }

}

impl<F: Float, M: Material<F=F>> Sphere<F, M>
{
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Sphere<F, M>
    {
        let points = [
            vec3!( F::ONE,  F::ONE, -F::ONE),
            vec3!( F::ONE,  F::ONE,  F::ONE),
            vec3!( F::ONE, -F::ONE, -F::ONE),
            vec3!( F::ONE, -F::ONE,  F::ONE),
            vec3!(-F::ONE,  F::ONE, -F::ONE),
            vec3!(-F::ONE,  F::ONE,  F::ONE),
            vec3!(-F::ONE, -F::ONE, -F::ONE),
            vec3!(-F::ONE, -F::ONE,  F::ONE),
        ];
        let mut aabb: AABB = AABB::empty();
        for point in &points {
            let p = point.xfrm(&xfrm);
            aabb.grow_mut(&p.into_point3());
        }
        Sphere { xfrm, mat, aabb, ni: 0 }
    }
}
