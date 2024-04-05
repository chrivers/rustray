mod directional;
mod pointlight;
mod spotlight;

pub use directional::DirectionalLight;
pub use pointlight::PointLight;
pub use spotlight::SpotLight;

use crate::scene::{Interactive, SceneObject};
use crate::types::{Color, Float, Maxel, Vector};

pub trait Light<F: Float>: SceneObject<F> + Sync + Send {
    fn contribution(&self, _maxel: &Maxel<F>) -> Lixel<F> {
        Lixel {
            dir: Vector::unit_z(),
            color: Color::BLACK,
            len2: F::from_u32(100_000),
        }
    }
}

pub struct Lixel<F: Float> {
    pub dir: Vector<F>,
    pub color: Color<F>,
    pub len2: F,
}

impl<'a, F: Float> Light<F> for Box<dyn Light<F> + 'a> {
    fn contribution(&self, maxel: &Maxel<F>) -> Lixel<F> {
        (**self).contribution(maxel)
    }
}

impl<'a, F: Float> SceneObject<F> for Box<dyn Light<F> + 'a> {
    fn get_name(&self) -> &str {
        self.as_ref().get_name()
    }

    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        self.as_mut().get_interactive()
    }
    fn get_id(&self) -> Option<usize> {
        Some(std::ptr::addr_of!(*self) as usize)
    }
}
