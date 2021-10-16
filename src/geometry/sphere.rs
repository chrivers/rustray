use super::geo_util::*;

pub struct Sphere<F: Float, M: Material<F=F>>
{
    pos: Vector<F>,
    dir1: Vector<F>,
    radius2: F,
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
        let normal = self.pos.normal_to(hit.pos);
        let normalu = self.dir1.cross(normal).normalize();
        let normalv = normalu.cross(normal).normalize();

        let (u, v) = normal.polar_uv();

        Maxel::from_uv(u, v, normal, normalu, normalv, &self.mat)
    }
}

impl<F: Float, M: Material<F=F>> Geometry<F> for Sphere<F, M>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let t = ray.intersect_sphere(&self.pos, self.radius2)?;

        Some(ray.hit_at(t, self))
    }

}

impl<F: Float, M: Material<F=F>> Sphere<F, M>
{
    pub fn new(pos: Vector<F>, radius: F, mat: M) -> Sphere<F, M>
    {
        let mut aabb: AABB = AABB::empty();
        let rad = vec3!(radius, radius, radius);
        aabb.grow_mut(&(pos - rad).into_point3());
        aabb.grow_mut(&(pos + rad).into_point3());
        Sphere { pos, radius2: radius * radius, mat, dir1: Vector::identity_y(), aabb, ni: 0 }
    }
}

impl<F: Float, M: Material<F=F>> HasPosition<F> for Sphere<F, M>
{
    fn get_position(&self) -> Vector<F> { self.pos }
    fn set_position(&mut self, value: Vector<F>) { self.pos = value }
}
