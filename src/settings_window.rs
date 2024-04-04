#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub is_overlay: bool,
}

pub struct SettingsWindow {
    is_open: bool,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            is_open: false,
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn show(&mut self, ctx: &egui::Context, settings: &mut Settings) {
        egui::Window::new("Settings")
            .open(&mut self.is_open)
            .show(ctx, |ui| {
                ui.checkbox(&mut settings.is_overlay, "Overlay");
            });
    }
}
