use super::geo_util::*;

pub struct Sphere<'a, F: Float>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    radius2: F,
    mat: &'a dyn Material<F=F>
}

impl<'a, F: Float> HitTarget<F> for Sphere<'a, F>
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>
    {
        let normal = self.pos.normal_to(hit.pos);
        let normalu = self.dir1.cross(normal).normalize();
        let normalv = normalu.cross(normal).normalize();

        let (u, v) = normal.polar_uv();

        Maxel::from_uv(u, v, normal, normalu, normalv, self.mat)
    }
}

impl<'a, F: Float> RayTarget<F> for Sphere<'a, F>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_sphere(&self.pos, self.radius2)?;

        Some(ray.hit_at(t, self, None))
    }

}

impl<'a, F: Float> Sphere<'a, F>
{
    pub fn new(pos: Vector<F>, radius: F, mat: &'a dyn Material<F=F>) -> Sphere<'a, F>
    {
        Sphere { pos, radius2: radius * radius, mat, dir1: Vector::identity_y() }
    }
}

impl<'a, F: Float> HasPosition<F> for Sphere<'a, F>
{
    fn get_position(&self) -> Vector<F> { self.pos }
    fn set_position(&mut self, value: Vector<F>) { self.pos = value }
}
