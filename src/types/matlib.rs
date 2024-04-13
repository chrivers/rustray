use std::collections::HashMap;

use crate::material::BoxMaterial;
use crate::scene::Interactive;
use crate::types::Float;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MaterialId(pub u32);

pub struct MaterialLib<F: Float> {
    pub mats: HashMap<MaterialId, BoxMaterial<F>>,
    idx: u32,
}

impl<F: Float> MaterialLib<F> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mats: HashMap::new(),
            idx: 0,
        }
    }

    #[must_use]
    pub fn get_name(&self, mat: MaterialId) -> String {
        format!("material-{}", mat.0)
    }

    pub fn insert(&mut self, material: BoxMaterial<F>) -> MaterialId {
        let next = MaterialId(self.idx);
        self.mats.insert(next, material);
        self.idx += 1;
        next
    }
}

impl<F: Float> Default for MaterialLib<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Float> Interactive<F> for MaterialId {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(format!("Material id {}", self.0));
        ui.end_row();
        false
    }
}
