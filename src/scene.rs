use crate::lib::{Float, Vector, Color, Camera, BvhExt};
use crate::lib::ray::{Ray, Maxel};
use crate::geometry::{Geometry, FiniteGeometry};

use cgmath::MetricSpace;

use rtbvh::{Bvh, Builder};
use std::num::NonZeroUsize;

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

pub trait Light<F: Float> : HasPosition<F> + Sync
{
    fn get_color(&self) -> Color<F>;
    fn attenuate(&self, color: Color<F>, d: F) -> Color<F>;
}

pub trait RayTracer<F: Float> : Sync
{
    fn ray_shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
    fn ambient(&self) -> Color<F>;
}

pub struct Scene<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>>
{
    pub cameras: Vec<Camera<F>>,
    pub objects: Vec<B>,
    pub geometry: Vec<G>,
    pub lights: Vec<L>,
    pub bvh: Bvh,
    pub ambient: Color<F>,
}

pub type BoxScene<F> = Scene<F, Box<dyn FiniteGeometry<F>>, Box<dyn Geometry<F>>, Box<dyn Light<F>>>;

impl<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> Scene<F, B, G, L>
{
    pub fn new(cameras: Vec<Camera<F>>, objects: Vec<B>, geometry: Vec<G>, lights: Vec<L>) -> Self {
        if objects.is_empty() {
            panic!("BVH crate fails with empty lists");
        }

        let aabbs = objects
            .iter()
            .map(|t| t.aabb())
            .collect::<Vec<rtbvh::Aabb>>();

        let bvh = Builder {
            aabbs: Some(aabbs.as_slice()),
            primitives: objects.as_slice(),
            primitives_per_leaf: NonZeroUsize::new(16),
        }
        /* .construct_spatial_sah().unwrap(); */
        .construct_binned_sah().unwrap();
        /* .construct_locally_ordered_clustered().unwrap(); */

        Self { cameras, objects, geometry, lights, bvh, ambient: Color::black() }
    }

    pub fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>>
    {
        let mut dist = F::max_value();
        let mut hit: Option<Maxel<F>> = None;

        for g in &self.geometry {
            if let Some(curhit) = g.intersect(ray)
            {
                let curdist = ray.pos.distance2(curhit.pos);
                if curdist > F::BIAS2 && curdist < dist
                {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }

        self.bvh.nearest_intersection(&ray, &self.objects, &mut dist).or(hit)
    }

    pub fn with_ambient(self, ambient: Color<F>) -> Self
    {
        Self { ambient, ..self }
    }
}
