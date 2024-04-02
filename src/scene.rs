use crate::geometry::{FiniteGeometry, Geometry};
use crate::types::{BvhExt, Camera, Color, Float, Maxel, RResult, Ray, Vector};

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
    fn ui(&mut self, ui: &mut egui::Ui);
    #[cfg(feature = "gui")]
    fn ui_center(&mut self, _ui: &mut egui::Ui, _camera: &Camera<F>, _rect: &egui::Rect) -> bool {
        false
    }
}

pub trait HasPosition<F: Float> {
    fn get_position(&self) -> Vector<F>;
    fn set_position(&mut self, value: Vector<F>);
}

pub trait HasDirection<F: Float> {
    fn get_direction(&self) -> Vector<F>;
    fn set_direction(&mut self, value: Vector<F>);
}

pub trait HasColor<F: Float> {
    fn get_color(&self) -> Color<F>;
    fn set_color(&mut self, value: Color<F>);
}

pub trait Light<F: Float>: HasPosition<F> + SceneObject<F> + Sync + Send {
    fn get_color(&self) -> Color<F>;
    fn attenuate(&self, color: Color<F>, d: F) -> Color<F>;
}

pub trait RayTracer<F: Float>: Sync {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, light: &dyn Light<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
    fn ambient(&self) -> Color<F>;
    fn background(&self) -> Color<F>;
    fn get_lights(&self) -> &[Box<dyn Light<F>>];
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
        let bvh = if objects.is_empty() {
            Bvh::default()
        } else {
            let aabbs = objects
                .iter()
                .map(rtbvh::Primitive::aabb)
                .collect::<Vec<rtbvh::Aabb>>();

            Builder {
                aabbs: Some(aabbs.as_slice()),
                primitives: objects.as_slice(),
                primitives_per_leaf: NonZeroUsize::new(16),
            }
            /* .construct_spatial_sah()? */
            /* .construct_locally_ordered_clustered()?; */
            .construct_binned_sah()?
        };

        Ok(Self {
            cameras,
            objects,
            geometry,
            lights,
            bvh,
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
            ambient: Color::black(),
        })
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
