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
    geo: Vec<G>,
    bvh: Bvh,
    aabb: Aabb,
}

#[cfg(feature = "gui")]
impl<F: Float, G: FiniteGeometry<F>> Interactive<F> for Group<F, G> {
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        use crate::gui::controls;
        use crate::types::hash;
        use egui::{vec2, Align, Button, Id, Layout, NumExt, Sense, TextStyle, Widget, WidgetText};
        use egui_phosphor::regular as icon;

        let mut res = false;

        let mut self_obj = ui.data(|mem| mem.get_temp("obj".into())).unwrap_or(None);

        let self_obj_last = ui
            .data(|mem| mem.get_temp("obj_last".into()))
            .unwrap_or(None);

        self.iter_mut().enumerate().for_each(|(i, obj)| {
            let name = format!("{} Object {i}: {}", obj.get_icon(), obj.get_name());
            let obj_id = obj.get_id();
            let id = hash(&obj.get_id());
            let proplist = controls::CustomCollapsible::new(Id::new(id));
            let (response, _header_response, _body_response) = proplist.show(
                ui,
                |pl, ui| {
                    let text = WidgetText::from(name);
                    let available = ui.available_rect_before_wrap();
                    let text_pos = available.min;
                    let wrap_width = available.right() - text_pos.x;
                    let galley = text.into_galley(ui, Some(false), wrap_width, TextStyle::Button);
                    let text_max_x = text_pos.x + galley.size().x;
                    let desired_width = text_max_x + available.left();
                    let desired_size =
                        vec2(desired_width, galley.size().y).at_least(ui.spacing().interact_size);
                    let rect = ui.allocate_space(desired_size).1;
                    let header_response = ui.interact(rect, pl.id, Sense::click());
                    if header_response.clicked() {
                        pl.toggle();
                    }
                    let visuals = ui.style().interact_selectable(&header_response, false);
                    ui.painter().galley(text_pos, galley, visuals.text_color());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let button = Button::new(format!("{} Select", icon::CROSSHAIR_SIMPLE))
                            .selected(self_obj == obj_id)
                            .ui(ui);

                        if button.clicked() {
                            if self_obj == obj_id {
                                info!("deselect: {:?} {:?}", self_obj, obj_id);
                                self_obj = None;
                            } else {
                                self_obj = obj_id;
                            }
                        }
                        ui.end_row();
                    });
                },
                |ui| {
                    if let Some(interactive) = obj.get_interactive() {
                        res |= interactive.ui(ui);
                    } else {
                        ui.label("Non-interactive object :(");
                    }
                },
            );

            if self_obj == obj.get_id() && self_obj != self_obj_last {
                response.highlight().scroll_to_me(Some(Align::Center));
            }
        });

        ui.data_mut(|mem| {
            mem.insert_temp("obj".into(), self_obj);
            mem.insert_temp("obj_last".into(), self_obj_last);
        });

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

    fn get_object(&mut self, id: usize) -> Option<&mut dyn Geometry<F>> {
        if SceneObject::get_id(self) == Some(id) {
            return Some(self as &mut dyn Geometry<F>);
        }

        for obj in &mut self.geo {
            if let Some(res) = obj.get_object(id) {
                return Some(res);
            }
        }
        None
    }
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

    #[must_use]
    pub fn empty() -> Self {
        Self {
            geo: vec![],
            xfrm: Transform::identity(),
            bvh: Bvh::default(),
            aabb: Aabb::empty(),
        }
    }

    pub fn clear(&mut self) {
        self.bvh = Bvh::default();
        self.aabb = Aabb::empty();
        self.geo.clear();
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<G> {
        self.geo.iter_mut()
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
        let _ = self.recompute_bvh();
        self.recompute_aabb();
    }

    pub fn del_object(&mut self, id: usize) {
        self.geo.retain(|obj| obj.get_id() != Some(id));
    }
}

impl<'a, F: Float, G: FiniteGeometry<F> + 'a> IntoIterator for &'a mut Group<F, G> {
    type IntoIter = std::slice::IterMut<'a, G>;
    type Item = &'a mut G;
    fn into_iter(self) -> Self::IntoIter {
        self.geo.iter_mut()
    }
}
