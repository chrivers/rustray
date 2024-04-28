use crate::geometry::{FiniteGeometry, Geometry};
use crate::light::{Light, Lixel};
use crate::types::{BvhExt, Camera, Color, Float, Maxel, RResult, Ray};

use cgmath::MetricSpace;

use rtbvh::{Builder, Bvh};
use std::fmt::Debug;
use std::num::NonZeroUsize;

pub trait SceneObject<F: Float> {
    fn get_name(&self) -> &str {
        "Unknown object"
    }
    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
        None
    }
    fn get_id(&self) -> Option<usize>;
}

pub trait Interactive<F: Float>: Debug {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool;
    #[cfg(feature = "gui")]
    fn ui_center(&mut self, _ui: &mut egui::Ui, _camera: &Camera<F>, _rect: &egui::Rect) -> bool {
        false
    }
}

pub trait RayTracer<F: Float> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
    fn ambient(&self) -> Color<F>;
    fn background(&self) -> Color<F>;
    fn get_lights(&self) -> &[Box<dyn Light<F>>];
    fn shadow(&self, maxel: &mut Maxel<F>, mut lixel: Lixel<F>) -> Lixel<F> {
        if let Some(color) = self.ray_shadow(maxel, &lixel) {
            lixel.color = color;
        }
        lixel
    }
}

pub struct Scene<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> {
    pub cameras: Vec<Camera<F>>,
    pub objects: Vec<B>,
    pub geometry: Vec<G>,
    pub lights: Vec<L>,
    pub bvh: Bvh,
    pub ambient: Color<F>,
    pub background: Color<F>,
}

pub type BoxScene<F> = Scene<
    F,
    Box<dyn FiniteGeometry<F> + 'static>,
    Box<dyn Geometry<F> + 'static>,
    Box<dyn Light<F> + 'static>,
>;

impl<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> Scene<F, B, G, L> {
    pub fn new(
        cameras: Vec<Camera<F>>,
        objects: Vec<B>,
        geometry: Vec<G>,
        lights: Vec<L>,
    ) -> RResult<Self> {
        let mut res = Self {
            cameras,
            objects,
            geometry,
            lights,
            bvh: Bvh::default(),
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
            ambient: Color::BLACK,
        };

        res.recompute_bvh()?;

        Ok(res)
    }

    pub fn recompute_bvh(&mut self) -> RResult<()> {
        let aabbs = self
            .objects
            .iter()
            .map(rtbvh::Primitive::aabb)
            .collect::<Vec<rtbvh::Aabb>>();

        let builder = Builder {
            aabbs: Some(aabbs.as_slice()),
            primitives: self.objects.as_slice(),
            primitives_per_leaf: NonZeroUsize::new(16),
        };

        self.bvh = builder.construct_binned_sah()?;

        Ok(())
    }

    pub fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        let mut dist = F::max_value();
        let mut hit: Option<Maxel<F>> = None;

        for g in &self.geometry {
            if let Some(curhit) = g.intersect(ray) {
                let curdist = ray.pos.distance2(curhit.pos);
                if curdist > F::BIAS2 && curdist < dist {
                    dist = curdist;
                    hit = Some(curhit);
                }
            }
        }

        self.bvh
            .nearest_intersection(ray, &self.objects, &mut dist)
            .or(hit)
    }

    #[must_use]
    pub fn with_ambient(self, ambient: Color<F>) -> Self {
        Self { ambient, ..self }
    }
}
