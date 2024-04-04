use crate::types::{transform::HasTransform, Camera, Float};

use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};

pub fn switch_orientation(mem: &mut egui::util::IdTypeMap) {
    let val: &mut GizmoOrientation = mem.get_temp_mut_or("mode".into(), GizmoOrientation::Global);
    *val = match val {
        GizmoOrientation::Global => GizmoOrientation::Local,
        GizmoOrientation::Local => GizmoOrientation::Global,
    }
}

pub fn switch_mode(mem: &mut egui::util::IdTypeMap) {
    let val: &mut GizmoMode = mem.get_temp_mut_or("mode".into(), GizmoMode::Rotate);
    *val = match val {
        GizmoMode::Rotate => GizmoMode::Scale,
        GizmoMode::Scale => GizmoMode::Translate,
        GizmoMode::Translate => GizmoMode::Rotate,
    }
}

pub fn gizmo_ui<F: Float>(
    ui: &mut egui::Ui,
    camera: &Camera<F>,
    obj: &mut impl HasTransform<F>,
    rect: &egui::Rect,
) -> bool {
    ui.horizontal(|ui| {
        let mut mode: GizmoMode = ui
            .data(|mem| mem.get_temp("mode".into()))
            .unwrap_or(GizmoMode::Rotate);
        ui.selectable_value(&mut mode, GizmoMode::Rotate, "Rotate");
        ui.selectable_value(&mut mode, GizmoMode::Scale, "Scale");
        ui.selectable_value(&mut mode, GizmoMode::Translate, "Translate");
        ui.data_mut(|mem| mem.insert_temp("mode".into(), mode));
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        let mut mode: GizmoOrientation = ui
            .data(|mem| mem.get_temp("mode".into()))
            .unwrap_or(GizmoOrientation::Global);
        ui.selectable_value(&mut mode, GizmoOrientation::Global, "Global");
        ui.selectable_value(&mut mode, GizmoOrientation::Local, "Local");
        ui.data_mut(|mem| mem.insert_temp("mode".into(), mode));
    });

    let mode: GizmoMode = ui
        .data(|mem| mem.get_temp("mode".into()))
        .unwrap_or(GizmoMode::Rotate);
    let orient: GizmoOrientation = ui
        .data(|mem| mem.get_temp("mode".into()))
        .unwrap_or(GizmoOrientation::Global);

    let gizmo = Gizmo::new("Gizmo")
        .viewport(*rect)
        .projection_matrix(camera.projection.into_mint())
        .view_matrix(camera.model.into_mint())
        .model_matrix(obj.get_transform().into_mint())
        .mode(mode)
        .orientation(orient);

    if let Some(response) = gizmo.interact(ui) {
        let f = response.transform();
        obj.set_transform(&f.into());
        true
    } else {
        false
    }
}
