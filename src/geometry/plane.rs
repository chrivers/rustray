use super::geo_util::*;

pub struct Plane<'a, F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
    u: Vector<F>,
    v: Vector<F>,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> HitTarget<F> for Plane<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let u = self.u.dot(hit.pos);
        let v = self.v.dot(hit.pos);
        Maxel::from_uv(u, v, self.normal, self.dir1, self.dir2, self.mat)
    }
}

impl<'a, F: Float> RayTarget<F> for Plane<'a, F>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, self))
    }

}

impl<'a, F: Float> Plane<'a, F>
{
    pub fn new(pos: Vector<F>, d1: Vector<F>, d2: Vector<F>, mat: &'a dyn Material<F=F>) -> Plane<'a, F>
    {
        let dir1 = d1.normalized();
        let dir2 = d2.normalized();
        let normal = dir1.cross(dir2);

        Plane { pos, dir1, dir2, normal, u: d1, v: d2, mat }
    }
}
