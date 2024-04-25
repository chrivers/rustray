use std::num::NonZeroUsize;

use cgmath::Matrix4;
use glam::Vec3;
use rtbvh::{Aabb, Bounds, Builder, Bvh, Primitive};

#[cfg(feature = "gui")]
use crate::types::Camera;

use crate::geometry::{build_aabb_ranged, FiniteGeometry, Geometry};
use crate::material::HasMaterial;
use crate::scene::{Interactive, SceneObject};
use crate::types::{
    BvhExt, Float, HasTransform, Maxel, RResult, Ray, Transform, Vector, Vectorx, RF,
};

#[derive(Debug)]
pub struct Group<F: Float, G: FiniteGeometry<F>> {
    xfrm: Transform<F>,
    pub geo: Vec<G>,
    pub bvh: Bvh,
    aabb: Aabb,
}

#[cfg(feature = "gui")]
impl<F: Float, G: FiniteGeometry<F>> Interactive<F> for Group<F, G> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut res = false;
        for g in &mut self.geo {
            ui.label(g.get_name());
            res |= g.get_interactive().is_some_and(|i| i.ui(ui));
        }
        res
    }

    fn ui_center(&mut self, ui: &mut egui::Ui, camera: &Camera<F>, rect: &egui::Rect) -> bool {
        crate::gui::gizmo::gizmo_ui(ui, camera, self, rect)
    }

    #[cfg(feature = "gui")]
    fn ui_bounding_box(&mut self) -> Option<&Aabb> {
        Some(&self.aabb)
    }
}

impl<F: Float, G: FiniteGeometry<F>> SceneObject<F> for Group<F, G> {
    crate::sceneobject_impl_body!("Group", Self::ICON);
}

impl<F: Float, G: FiniteGeometry<F>> HasTransform<F> for Group<F, G> {
    fn get_transform(&self) -> &Transform<F> {
        &self.xfrm
    }

    fn set_transform(&mut self, xfrm: &Transform<F>) {
        self.xfrm = *xfrm;
        self.recompute_aabb();
    }
}

impl<F: Float, G: FiniteGeometry<F>> rtbvh::Primitive for Group<F, G> {
    fn center(&self) -> Vec3 {
        self.aabb.center()
    }

    fn aabb(&self) -> Aabb {
        self.aabb
    }
}

impl<F: Float, G: FiniteGeometry<F>> FiniteGeometry<F> for Group<F, G> {
    fn recompute_aabb(&mut self) {
        let bounds = self.bvh.bounds();

        let min = Vector::from_vec3(bounds.min);
        let max = Vector::from_vec3(bounds.max);

        self.aabb = build_aabb_ranged(&self.xfrm, [min.x, max.x], [min.y, max.y], [min.z, max.z]);
    }
}

impl<F: Float, G: FiniteGeometry<F>> Geometry<F> for Group<F, G> {
    fn intersect(&self, ray: &Ray<F>) -> Option<Maxel<F>> {
        if ray.flags.contains(RF::StopAtGroup) {
            let center = self.xfrm.pos_inv(Vector::from_vec3(self.center()));
            return Some(ray.synthetic_hit(center, self));
        }

        let ray = ray.xfrm_inv(&self.xfrm);

        let mut dist = F::max_value();

        self.bvh
            .nearest_intersection(&ray, &self.geo, &mut dist)
            .map(|maxel| maxel.xfrm(&self.xfrm))
    }

    fn material(&mut self) -> Option<&mut dyn HasMaterial> {
        None
    }
}

impl<F: Float, G: FiniteGeometry<F>> Group<F, G> {
    const ICON: &'static str = egui_phosphor::regular::POLYGON;

    pub fn new(geo: Vec<G>, xfrm: Matrix4<F>) -> Self {
        debug!("building bvh for {} geometries..", geo.len());

        let mut res = Self {
            xfrm: Transform::new(xfrm),
            geo,
            bvh: Bvh::default(),
            aabb: Aabb::empty(),
        };
        res.recompute_bvh().unwrap();
        res.recompute_aabb();
        res
    }

    pub fn clear(&mut self) {
        self.bvh = Bvh::default();
        self.aabb = Aabb::empty();
        self.geo.clear();
    }

    pub fn nearest_intersection(&self, ray: &Ray<F>, dist: &mut F) -> Option<Maxel<F>> {
        self.bvh.nearest_intersection(ray, &self.geo, dist)
    }

    pub fn recompute_bvh(&mut self) -> RResult<()> {
        let aabbs = self
            .geo
            .iter()
            .map(rtbvh::Primitive::aabb)
            .collect::<Vec<rtbvh::Aabb>>();

        if aabbs.is_empty() {
            self.bvh = Bvh::default();
        } else {
            let builder = Builder {
                aabbs: Some(aabbs.as_slice()),
                primitives: self.geo.as_slice(),
                primitives_per_leaf: NonZeroUsize::new(16),
            };

            self.bvh = builder.construct_binned_sah()?;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.geo.len()
    }

    pub fn is_empty(&self) -> bool {
        self.geo.is_empty()
    }

    pub fn add_object(&mut self, geometry: G) {
        self.geo.push(geometry);
    }
}
