use std::collections::HashMap;

use image::DynamicImage;

use crate::scene::Interactive;
use crate::types::Float;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextureId(pub u32);

pub struct TextureLib {
    pub texs: HashMap<TextureId, DynamicImage>,
    idx: u32,
}

impl TextureLib {
    #[must_use]
    pub fn new() -> Self {
        Self {
            texs: HashMap::new(),
            idx: 0,
        }
    }

    #[must_use]
    pub fn get_name(&self, mat: TextureId) -> String {
        format!("texture-{}", mat.0)
    }

    pub fn insert(&mut self, material: DynamicImage) -> TextureId {
        let next = TextureId(self.idx);
        self.texs.insert(next, material);
        self.idx += 1;
        next
    }
}

impl Default for TextureLib {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Float> Interactive<F> for TextureId {
    #[cfg(feature = "gui")]
    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.label(format!("Texture id {}", self.0));
        ui.end_row();
        false
    }
}
