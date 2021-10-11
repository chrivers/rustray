use crate::lib::{Float, Vector, Color, Camera};
use crate::lib::ray::{Ray, Hit, Maxel};

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

pub trait RayTarget<F: Float> : Sync
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>;
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

impl<F: Float> RayTarget<F> for Box<dyn RayTarget<F>>
{
    fn intersect(&self, ray: &Ray<F>) -> Option<Hit<F>>
    {
        self.as_ref().intersect(ray)
    }
}

pub struct Scene<F: Float, T: RayTarget<F>, L: Light<F>>
{
    pub cameras: Vec<Camera<F>>,
    pub objects: Vec<T>,
    pub lights: Vec<L>,
}

pub type BoxScene<F> = Scene<F, Box<dyn RayTarget<F>>, Box<dyn Light<F>>>;
