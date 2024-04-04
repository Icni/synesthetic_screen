use kira::manager::AudioManager;

use crate::app::MusicState;

pub enum MusicControl {
    Settings,
    LoadMusic,
    Snapshot,
    TogglePause,
    Nothing,
}

pub struct MusicControlPanel {
    music_position: f64,
    music_len: f64,
    scrub_bar_rect: egui::Rect,
    is_playing: bool,
}

impl MusicControlPanel {
    pub fn new() -> Self {
        Self {
            music_position: 0.0,
            music_len: 0.0,
            scrub_bar_rect: egui::Rect::ZERO,
            is_playing: false
        }
    }

    pub fn show(
        &mut self,
        music_state: &mut MusicState,
        audio_manager: &mut AudioManager,
        ctx: &egui::Context
    ) -> MusicControl {
        egui::TopBottomPanel::bottom("control_panel")
            .exact_height(ctx.available_rect().height() - ctx.available_rect().width() * 9./16. - 10.)
            .resizable(false)
            .show(ctx, |ui| {
                let mut control = MusicControl::Nothing;

                ui.add_space(ui.available_height() * 0.05);

                match music_state {
                    MusicState::Loaded(music) => {
                        self.is_playing = music.is_playing();
                        self.music_position = music.position();
                        self.music_len = music.len();

                        ui.horizontal(|ui| {
                            ui.label(music.name());
                            ui.add_space(10.0);
                            if ui.button("Open another file...").clicked() {
                                control = MusicControl::LoadMusic;
                            }
                            if ui.button("Settings...").clicked() {
                                control = MusicControl::Settings;
                            }
                            if ui.button("Take snapshot").clicked() {
                                control = MusicControl::Snapshot;
                            }
                        });

                        ui.horizontal(|ui| {
                            if self.pause_toggle(ui).changed() {
                                control = MusicControl::TogglePause;
                            }
                            ui.add_space(10.0);
                            let scrub_response = self.scrub_bar(ui);
                            if scrub_response.dragged() {
                                let amount_percent = scrub_response.drag_delta().x / self.scrub_bar_rect.width();
                                let amount = self.music_len * amount_percent as f64;
                                music.scrub(amount, audio_manager);
                            }
                        });
                    }
                    MusicState::Loading(meta) => {
                        ui.horizontal(|ui| {
                            ui.label(format!("Loading {}... (this may take several seconds)", meta.name));
                        });
                    }
                    MusicState::Silence => {
                        if ui.horizontal(|ui| {
                            ui.label("No audio input selected.");
                            ui.add_space(10.0);
                            ui.button("Open file...")
                        }).inner.clicked() {
                            control = MusicControl::LoadMusic;
                        }
                    }
                }

                control
            }).inner
    }

    fn pause_toggle(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = (ui.spacing().interact_size.x, ui.spacing().interact_size.x);
        let (rect, mut response) = ui.allocate_exact_size(size.into(), egui::Sense::click());

        if response.clicked() {
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let rect = rect.expand(visuals.expansion);
            let radius = rect.width() * 0.5;
            ui.painter().circle(rect.center(), radius, visuals.bg_fill, visuals.bg_stroke);

            if self.is_playing {
                egui::Image::new(egui::include_image!("../assets/images/pause.png"))
                    .tint(egui::Color32::LIGHT_RED)
                    .paint_at(&ui, rect.shrink(10.0));
            } else {
                egui::Image::new(egui::include_image!("../assets/images/play.png"))
                    .tint(egui::Color32::LIGHT_GREEN)
                    .paint_at(&ui, rect.shrink(10.0));
            }
        }

        response
    }

    fn scrub_bar(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = (
            ui.available_width() - 10.0,
            ui.spacing().interact_size.y,
        );
        let (rect, response) = ui.allocate_exact_size(size.into(), egui::Sense::drag());

        if ui.is_rect_visible(rect) {
            let slider_visuals = ui.style().noninteractive();
            let rect = rect.expand(slider_visuals.expansion);
            self.scrub_bar_rect = rect;

            let radius = rect.height() * 0.5;
            ui.painter().rect(rect, radius, slider_visuals.weak_bg_fill, slider_visuals.bg_stroke);

            let cursor_visuals = ui.style().interact(&response);
            let mut cursor_rect = rect;
            cursor_rect.set_width(rect.height());
            cursor_rect = cursor_rect.expand(2.0);
            let position = rect.min.x + (rect.width() * (self.music_position / self.music_len) as f32);
            cursor_rect.set_center((position, rect.center().y).into());
            let radius = cursor_rect.width() * 0.5;
            ui.painter().circle(cursor_rect.center(), radius, cursor_visuals.bg_fill, cursor_visuals.bg_stroke);

            ui.painter().text(
                (rect.min.x, rect.max.y + 15.0).into(),
                egui::Align2::LEFT_BOTTOM,
                format!("{} / {}", format_min_sec(self.music_position), format_min_sec(self.music_len)),
                egui::FontId::monospace(12.0),
                ui.style().visuals.text_color(),
            );
        }

        response
    }
}

fn format_min_sec(seconds: f64) -> String {
    let minutes = seconds as u32 / 60;
    let seconds = (seconds % 60.0) as u32;
    format!("{minutes:02}:{seconds:02}")
}
