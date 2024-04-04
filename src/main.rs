use app::App;
use egui::ViewportBuilder;

mod app;
mod control_panel;
mod music;
mod note;
mod settings_window;
mod style;
mod synesthetizer;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size((1000., 650.))
            .with_min_inner_size((500., 500.))
            .with_title("Synesthetic Screen"),
        ..Default::default()
    };

    eframe::run_native(
        "Synesthetic Screen",
        native_options,
        Box::new(|cc| Box::new(App::new(cc)))
    )
}
