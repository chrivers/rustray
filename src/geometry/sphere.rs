use super::geo_util::*;
use num_traits::Zero;

#[derive(Debug)]
pub struct Sphere<F: Float, M: Material<F=F>>
{
    xfrm: Matrix4<F>,
    ifrm: Matrix4<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Sphere<F, M>);

impl<F: Float, M: Material<F=F>> HitTarget<F> for Sphere<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let normal = hit.nml.unwrap();
        let (u, v) = normal.polar_uv();
        Maxel::from_uv(u, v, normal, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Sphere<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.transform(&self.ifrm)?;

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
        let aabb = build_aabb_symmetric(&xfrm, F::ONE, F::ONE, F::ONE);
        let ifrm = xfrm.inverse_transform().unwrap();
        Sphere { xfrm, ifrm, mat, aabb }
    }

    pub fn place(pos: Vector<F>, radius: F, mat: M) -> Sphere<F, M>
    {
        let scale = Matrix4::from_scale(radius);
        let xlate = Matrix4::from_translation(pos);
        Self::new(scale * xlate, mat)
    }
}
