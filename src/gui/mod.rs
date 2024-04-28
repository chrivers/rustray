use egui::{InnerResponse, Response, Ui};

pub mod controls;
pub mod gizmo;
pub mod visualtrace;

pub trait IconButton {
    fn icon_button(&mut self, icon: &str, text: &str) -> Response;
    fn icon_menu_button<R>(
        &mut self,
        icon: &str,
        text: &str,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<Option<R>>;
}

impl IconButton for Ui {
    fn icon_button(&mut self, icon: &str, text: &str) -> Response {
        self.button(format!("{icon} {text}"))
    }

    fn icon_menu_button<R>(
        &mut self,
        icon: &str,
        text: &str,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<Option<R>> {
        self.menu_button(format!("{icon} {text}"), add_contents)
    }
}
