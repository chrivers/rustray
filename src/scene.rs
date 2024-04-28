use crate::geometry::{FiniteGeometry, Geometry};
use crate::light::{Light, Lixel};
use crate::types::{BvhExt, Camera, Color, Float, MaterialLib, Maxel, RResult, Ray, TextureLib};

use cgmath::MetricSpace;

use rtbvh::{Builder, Bvh};
use std::fmt::Debug;
use std::num::NonZeroUsize;

pub trait SceneObject<F: Float> {
    fn get_name(&self) -> &str;
    fn get_icon(&self) -> &str;
    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>>;
    fn get_id(&self) -> Option<usize>;
}

#[macro_export]
macro_rules! sceneobject_impl_body {
    ( $name:expr, $icon:expr ) => {
        fn get_name(&self) -> &str {
            $name
        }

        fn get_icon(&self) -> &str {
            $icon
        }

        #[cfg(feature = "gui")]
        fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
            Some(self)
        }

        #[cfg(not(feature = "gui"))]
        fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>> {
            None
        }

        fn get_id(&self) -> Option<usize> {
            Some(std::ptr::addr_of!(*self) as usize)
        }
    };
}

pub trait Interactive<F: Float>: Debug {
    #[cfg(feature = "gui")]
    fn ui(&mut self, _ui: &mut egui::Ui) -> bool {
        false
    }
    #[cfg(feature = "gui")]
    fn ui_center(&mut self, _ui: &mut egui::Ui, _camera: &Camera<F>, _rect: &egui::Rect) -> bool {
        false
    }
}

pub trait RayTracer<F: Float> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
    fn scene(&self) -> &BoxScene<F>;
}

pub struct Scene<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> {
    pub cameras: Vec<Camera<F>>,
    pub objects: Vec<B>,
    pub geometry: Vec<G>,
    pub textures: TextureLib,
    pub materials: MaterialLib<F>,
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
        materials: MaterialLib<F>,
        lights: Vec<L>,
    ) -> RResult<Self> {
        let mut res = Self {
            cameras,
            objects,
            geometry,
            textures: TextureLib::new(),
            materials,
            lights,
            bvh: Bvh::default(),
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
            ambient: Color::BLACK,
        };

        res.recompute_bvh()?;

        Ok(res)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self {
            cameras: vec![],
            objects: vec![],
            geometry: vec![],
            textures: TextureLib::new(),
            materials: MaterialLib::new(),
            lights: vec![],
            bvh: Bvh::default(),
            ambient: Color::BLACK,
            background: Color::BLACK,
        }
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
