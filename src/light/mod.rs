mod arealight;
mod directional;
mod pointlight;
mod spotlight;

pub use arealight::AreaLight;
pub use directional::DirectionalLight;
pub use pointlight::PointLight;
pub use spotlight::SpotLight;

use crate::scene::{Interactive, RayTracer, SceneObject};
use crate::types::{Color, Float, Maxel, Vector, Vectorx};

pub trait Light<F: Float>: SceneObject<F> + Sync + Send {
    fn contribution(&self, _maxel: &mut Maxel<F>, _rt: &dyn RayTracer<F>) -> Lixel<F> {
        Lixel {
            dir: Vector::UNIT_Z,
            color: Color::BLACK,
            len2: F::from_u32(100_000),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Attenuation<F: Float> {
    pub a: F,
    pub b: F,
    pub c: F,
}

impl<F: Float> Attenuation<F> {
    pub fn attenuate(&self, color: Color<F>, len: F, len2: F) -> Color<F> {
        color / (F::ONE + self.a + (self.b * len) + (self.c * len2))
    }
}

pub struct Lixel<F: Float> {
    pub dir: Vector<F>,
    pub color: Color<F>,
    pub len2: F,
}

impl<F: Float> Light<F> for Box<dyn Light<F> + 'static> {
    fn contribution(&self, maxel: &mut Maxel<F>, rt: &dyn RayTracer<F>) -> Lixel<F> {
        (**self).contribution(maxel, rt)
    }
}

impl<F: Float> SceneObject<F> for Box<dyn Light<F> + 'static> {
    fn get_name(&self) -> &str {
        (**self).get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        (**self).get_interactive()
    }
    fn get_id(&self) -> Option<usize> {
        (**self).get_id()
    }
}
