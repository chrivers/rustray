use super::geo_util::*;

#[derive(Clone, Copy, Debug)]
pub struct Plane<F: Float, M: Material<F=F>>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    dir2: Vector<F>,
    normal: Vector<F>,
    u: Vector<F>,
    v: Vector<F>,
    mat: M,
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Plane<F, M>
{
    fn uv(&self, maxel: &mut Maxel<F>) -> Point<F>
    {
        let u = self.u.dot(maxel.pos);
        let v = self.v.dot(maxel.pos);
        point!(u, v)
    }

    fn normal(&self, hit: &mut Maxel<F>) -> Vector<F>
    {
        self.normal
    }

    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        let t = ray.intersect_plane(&self.pos, &self.dir1, &self.dir2)?;
        Some(ray.hit_at(t, self, &self.mat))
    }
}

impl<F: Float, M: Material<F=F>> Plane<F, M>
{
    pub fn new(pos: Vector<F>, d1: Vector<F>, d2: Vector<F>, mat: M) -> Plane<F, M>
    {
        let dir1 = d1.normalize();
        let dir2 = d2.normalize();
        let normal = dir1.cross(dir2);

        Plane { pos, dir1, dir2, normal, u: d1, v: d2, mat }
    }
}
