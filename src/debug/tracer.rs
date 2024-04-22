use std::cell::RefCell;
use std::fmt::{self, Debug};

use cgmath::MetricSpace;

use crate::light::Lixel;
use crate::material::Material;
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
    scene: &'a BoxScene<F>,
    pub steps: RefCell<Vec<Step<'a, F>>>,
    maxlvl: u16,
}

impl<'a, F: Float> DebugTracer<'a, F> {
    #[must_use]
    pub fn new(scene: &'a BoxScene<F>, maxlvl: u16) -> Self {
        Self {
            scene,
            steps: RefCell::new(vec![]),
            maxlvl,
        }
    }

    pub fn into_inner(self) -> Vec<Step<'a, F>> {
        self.steps.into_inner()
    }

    pub fn trace_single(scene: &'a BoxScene<F>, maxlvl: u16, ray: &Ray<F>) -> Vec<Step<'a, F>> {
        let dt = Self::new(scene, maxlvl);
        dt.ray_trace(ray);
        dt.into_inner()
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
                    let mat = &self.scene.materials.mats[&maxel.mat];
                    let color = mat.shadow(&mut curhit, self, lixel);
                    best_color = Some(color);
                    best_length = cur_length;

                    step.color = best_color;
                    step.maxel = Some(curhit);
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
            let mat = &self.scene.materials.mats[&maxel.mat];
            step.color = Some(mat.render(&mut maxel, self));
        }
        let res = step.color;

        self.steps.borrow_mut().push(step);

        res
    }

    fn scene(&self) -> &BoxScene<F> {
        self.scene
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
