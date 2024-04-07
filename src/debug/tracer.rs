use std::cell::RefCell;
use std::fmt::{self, Debug};
use std::sync::RwLockWriteGuard;

use cgmath::MetricSpace;

use crate::light::{Light, Lixel};
use crate::scene::{BoxScene, RayTracer};
use crate::types::{Color, Float, Maxel, Ray};

#[derive(Debug)]
pub struct Step<'a, F: Float> {
    pub ray: Ray<F>,
    pub maxel: Option<Maxel<'a, F>>,
    pub shadow: bool,
    pub color: Option<Color<F>>,
}

pub struct DebugTracer<'a, F: Float> {
    scene: &'a RwLockWriteGuard<'a, BoxScene<F>>,
    pub steps: RefCell<Vec<Step<'a, F>>>,
    maxlvl: u16,
}

impl<'a, F: Float> DebugTracer<'a, F> {
    #[must_use]
    pub fn new(scene: &'a RwLockWriteGuard<'a, BoxScene<F>>, maxlvl: u16) -> Self {
        Self {
            scene,
            steps: RefCell::new(vec![]),
            maxlvl,
        }
    }
}

impl<'a, F: Float> RayTracer<F> for DebugTracer<'a, F> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>> {
        if maxel.lvl == self.maxlvl {
            return None;
        }

        let pos = maxel.pos + maxel.nml() * F::BIAS2;
        let hitray = maxel.ray(pos, lixel.dir);

        let mut best_length = lixel.len2;
        let mut best_color = None;

        let mut r = hitray.into();

        let mut step = Step {
            ray: hitray,
            maxel: None,
            shadow: true,
            color: None,
        };

        #[allow(clippy::significant_drop_in_scrutinee)]
        for (curobj, _ray) in self.scene.bvh.traverse_iter(&mut r, &self.scene.objects) {
            if let Some(mut curhit) = curobj.intersect(&hitray) {
                let cur_length = maxel.pos.distance2(curhit.pos);
                if cur_length > F::BIAS2 && cur_length < best_length {
                    if let Some(color) = curhit.mat.shadow(&mut curhit, lixel) {
                        best_color = Some(color);
                        best_length = cur_length;

                        step.color = best_color;
                        step.maxel = Some(curhit);
                    }
                }
            }
        }
        self.steps.borrow_mut().push(step);

        best_color
    }

    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>> {
        if ray.lvl == self.maxlvl {
            return None;
        }

        let mut step = Step {
            ray: *ray,
            maxel: None,
            shadow: false,
            color: None,
        };

        step.maxel = self.scene.intersect(ray);
        if let Some(mut maxel) = step.maxel {
            step.color = Some(maxel.mat.render(&mut maxel, self));
        }
        let res = step.color;

        self.steps.borrow_mut().push(step);

        res
    }

    fn ambient(&self) -> Color<F> {
        self.scene.ambient
    }

    fn get_lights(&self) -> &[Box<dyn Light<F>>] {
        &self.scene.lights
    }

    fn background(&self) -> Color<F> {
        self.scene.background
    }
}

impl<'a, F: Float> Debug for DebugTracer<'a, F> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("DebugTracer")
            .field("scene", &"<scene>")
            .field("steps", &self.steps)
            .finish()
    }
}
