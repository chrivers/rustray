use crate::geometry::{FiniteGeometry, Geometry, Group};
use crate::light::{DirectionalLight, Light, Lixel};
use crate::types::{
    Camera, Color, Float, MaterialLib, Maxel, RResult, Ray, TextureLib, Vector, Vectorx,
};
use crate::vec3;

use cgmath::{InnerSpace, Matrix4, MetricSpace, SquareMatrix};

use rtbvh::Primitive;
use std::fmt::Debug;

pub trait SceneObject<F: Float> {
    fn get_name(&self) -> &str;
    fn get_icon(&self) -> &str;
    fn get_interactive(&mut self) -> Option<&mut dyn Interactive<F>>;
    fn get_id(&self) -> Option<usize>;
    fn get_object(&mut self, _id: usize) -> Option<&mut dyn Geometry<F>> {
        None
    }
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
    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&rtbvh::Aabb> {
        None
    }
}

pub trait RayTracer<F: Float> {
    fn ray_shadow(&self, maxel: &mut Maxel<F>, lixel: &Lixel<F>) -> Option<Color<F>>;
    fn ray_trace(&self, ray: &Ray<F>) -> Option<Color<F>>;
    fn scene(&self) -> &BoxScene<F>;
}

pub struct Scene<F: Float, B: FiniteGeometry<F>, G: Geometry<F>, L: Light<F>> {
    pub cameras: Vec<Camera<F>>,
    pub root: Group<F, B>,
    pub geometry: Vec<G>,
    pub textures: TextureLib,
    pub materials: MaterialLib<F>,
    pub lights: Vec<L>,
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
            root: Group::new(objects, Matrix4::identity()),
            geometry,
            textures: TextureLib::new(),
            materials,
            lights,
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
            ambient: Color::BLACK,
        };

        res.root.recompute_bvh()?;

        Ok(res)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self {
            cameras: vec![],
            root: Group::empty(),
            geometry: vec![],
            textures: TextureLib::new(),
            materials: MaterialLib::new(),
            lights: vec![],
            ambient: Color::BLACK,
            background: Color::new(F::ZERO, F::ZERO, F::from_f32(0.2)),
        }
    }

    pub fn clear(&mut self) {
        self.cameras.clear();
        self.root.clear();
        self.geometry.clear();
        self.textures.texs.clear();
        self.materials.mats.clear();
        self.lights.clear();
    }

    pub fn recompute_bvh(&mut self) -> RResult<()> {
        self.root.recompute_bvh()
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

        self.root.nearest_intersection(ray, &mut dist).or(hit)
    }

    pub fn set_ambient(&mut self, ambient: Color<F>) {
        self.ambient = ambient;
    }

    pub const fn get_ambient(&self) -> Color<F> {
        self.ambient
    }

    pub fn set_background(&mut self, background: Color<F>) {
        self.background = background;
    }

    pub const fn get_background(&self) -> Color<F> {
        self.background
    }

    pub fn add_camera(&mut self, camera: Camera<F>) {
        self.cameras.push(camera);
    }
}

impl<F: Float> BoxScene<F> {
    pub fn add_light(&mut self, light: impl Light<F> + 'static) {
        self.lights.push(Box::new(light));
    }

    pub fn add_geometry(&mut self, geometry: impl Geometry<F> + 'static) {
        self.geometry.push(Box::new(geometry));
    }

    pub fn add_object(&mut self, geometry: impl FiniteGeometry<F> + 'static) {
        self.root.add_object(Box::new(geometry));
    }

    pub fn add_camera_if_missing(&mut self) -> RResult<()> {
        if !self.cameras.is_empty() {
            return Ok(());
        }

        self.recompute_bvh()?;

        let bb = self.root.aabb();

        let sz: Vector<F> = Vector::from_vec3(bb.lengths());
        let look: Vector<F> = Vector::from_vec3(bb.center());
        let pos = vec3!(F::ZERO, sz.y / F::TWO, sz.magnitude());

        let cam = Camera::build(pos, look - pos, Vector::UNIT_Y, F::from_f32(60.0), F::ONE);

        info!("Add camera");
        self.cameras.push(cam);

        Ok(())
    }

    pub fn add_light_if_missing(&mut self) -> RResult<()> {
        if !self.lights.is_empty() {
            return Ok(());
        }

        info!("Add light");
        self.add_light(DirectionalLight::new(
            Vector::new(F::ZERO, -F::HALF, -F::ONE),
            Color::WHITE,
        ));

        Ok(())
    }
}
