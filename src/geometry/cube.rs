use super::geo_util::*;

#[derive(Debug)]
pub struct Cube<F: Float, M: Material<F=F>>
{
    xfrm: Matrix4<F>,
    ifrm: Matrix4<F>,
    mat: M,
    aabb: Aabb,
}

aabb_impl_fm!(Cube<F, M>);

impl<F: Float, M: Material<F=F>> HitTarget<F> for Cube<F, M>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let normal = hit.nml.unwrap();
        let uv = hit.uv.unwrap();
        Maxel::from_uv(uv.x, uv.y, normal, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Cube<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.transform(&self.ifrm)?;

        let p = r.pos;
        let d = r.dir;

        let mut best_t = F::max_value();
        let mut best = None;

        for it in 0..6 {
            let mod0 = it % 3;

            if d[mod0] == F::ZERO {
                continue
            }

            let t = (F::from_usize(it / 3) - F::HALF - p[mod0]) / d[mod0];

            if t < F::BIAS2 || t > best_t {
                continue
            }

            let mod1 = (it + 1) % 3;
            let mod2 = (it + 2) % 3;
            let x = p[mod1] + t * d[mod1];
            let y = p[mod2] + t * d[mod2];

            if x <= F::HALF && x >= -F::HALF &&
               y <= F::HALF && y >= -F::HALF {
                if best_t > t {
                    best_t = t;
                    best = Some(it);
                }
            }
        }

        let best = best?;

        let normals = [Vector::unit_x(),  Vector::unit_y(),  Vector::unit_z()];

        let normal = if best < 3 {
            -normals[best % 3]
        } else {
            normals[best % 3]
        };

        let i1 = (best + 1) % 3;
        let i2 = (best + 2) % 3;
        let min = i1.min(i2);
        let max = i1.max(i2);

        let isec = r.extend(best_t);
        let uv = point!(F::HALF - isec[min], F::HALF - isec[max]);

        Some(
            ray.hit_at(best_t, self)
                .with_normal(normal.xfrm_normal(&self.xfrm))
                .with_uv(uv)
        )
    }

}

impl<F: Float, M: Material<F=F>> Cube<F, M>
{
    pub fn new(xfrm: Matrix4<F>, mat: M) -> Cube<F, M>
    {
        let aabb = build_aabb_symmetric(&xfrm, F::HALF, F::HALF, F::HALF);
        let ifrm = xfrm.inverse_transform().unwrap();
        Cube { xfrm, ifrm, mat, aabb }
    }
}
