use crate::lib::{Float, Vector, Color, Camera};
use crate::lib::ray::{Ray, Hit, Maxel};
use crate::geometry::{Geometry, FiniteGeometry};

use cgmath::MetricSpace;
use bvh::bvh::BVH;

pub trait HasPosition<F: Float>
{
    fn get_position(&self) -> Vector<F>;
    fn set_position(&mut self, value: Vector<F>);
}

pub trait HasDirection<F: Float>
{
    fn get_direction(&self) -> Vector<F>;
    fn set_direction(&mut self, value: Vector<F>);
}

pub trait HasColor<F: Float>
{
    fn get_color(&self) -> Color<F>;
    fn set_color(&mut self, value: Color<F>);
}

pub trait HitTarget<F: Float> : Sync
{
    fn resolve(&self, hit: &Hit<F>) -> Maxel<F>;
}

pub trait Light<F: Float> : HasPosition<F> + Sync
{
    fn get_color(&self) -> Color<F>;
    fn attenuate(&self, color: Color<F>, d: F) -> Color<F>;
}

pub trait RayTracer<F: Float> : Sync
{
    fn ray_shadow(&self, hit: &Hit<F>, maxel: &Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
}

pub struct Scene<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>>
{
    pub cameras: Vec<Camera<F>>,
    pub objects: Vec<B>,
    pub geometry: Vec<G>,
    pub lights: Vec<L>,
    pub bvh: BVH,
}

pub type BoxScene<F> = Scene<F, Box<dyn FiniteGeometry<F>>, Box<dyn Geometry<F>>, Box<dyn Light<F>>>;

impl<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> Scene<F, B, G, L>
{
    pub fn new(cameras: Vec<Camera<F>>, mut objects: Vec<B>, geometry: Vec<G>, lights: Vec<L>) -> Self {
        let bvh = BVH::build(&mut objects);
        Self { cameras, objects, geometry, lights, bvh }
    }

    pub fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        let r = ray.into();
        let aabbs = self.bvh.traverse(&r, &self.objects);

        let mut dist = F::max_value();
        let mut hit: Option<Hit<F>> = None;

        for g in &self.geometry {
            if let Some(curhit) = g.intersect(ray)
            {
                let curdist = ray.pos.distance2(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }

        for t in &aabbs {
            if let Some(curhit) = t.intersect(ray)
            {
                let curdist = ray.pos.distance2(curhit.pos);
                if curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }
        hit
    }
}
