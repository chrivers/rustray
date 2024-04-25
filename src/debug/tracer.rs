use std::cell::RefCell;
use std::fmt::{self, Debug};

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

        let hitray = maxel.shadow_ray(lixel);

        let mut step = Step {
            ray: hitray,
            maxel: None,
            shadow: true,
            color: None,
        };

        let mut len2 = lixel.len2;

        if let Some(mut maxel) = self.scene.root.nearest_intersection(&hitray, &mut len2) {
            let mat = &self.scene.materials.mats[&maxel.mat];
            step.color = Some(mat.shadow(&mut maxel, self, lixel));
            step.maxel = Some(maxel);
        };

        let res = step.color;

        self.steps.borrow_mut().push(step);

        res
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
