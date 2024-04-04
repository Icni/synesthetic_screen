use kira::manager::{backend::DefaultBackend, AudioManager};

use crate::{control_panel::{MusicControl, MusicControlPanel}, music::{Music, MusicLoader, MusicMeta}, settings_window::{Settings, SettingsWindow}, style::load_style, synesthetizer::Synesthetizer};

pub enum MusicState {
    Silence,
    Loading(MusicMeta),
    Loaded(Music),
}

impl MusicState {
    pub fn play(&mut self, audio_manager: &mut AudioManager) {
        if let Self::Loaded(music) = self {
            music.play(audio_manager);
        }
    }

    pub fn pause(&mut self) {
        if let Self::Loaded(music) = self {
            music.pause();
        }
    }
}

pub struct App {
    texture: egui::TextureHandle,
    synesthetizer: Synesthetizer,
    music_state: MusicState,
    music_loader: MusicLoader,
    control_panel: MusicControlPanel,
    settings_window: SettingsWindow,
    settings: Settings,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        log::trace!("Starting app...");
    
        egui_extras::install_image_loaders(&cc.egui_ctx);

        load_style(&cc.egui_ctx);

        let image = egui::ColorImage::new([1600, 900], egui::Color32::BLACK);
        let texture = cc.egui_ctx.load_texture("screen", image.clone(), egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
        });

        let audio_manager = AudioManager::<DefaultBackend>::new(Default::default()).unwrap();

        Self {
            texture,
            synesthetizer: Synesthetizer::new(),
            music_state: MusicState::Silence,
            music_loader: MusicLoader::new(audio_manager),
            control_panel: MusicControlPanel::new(),
            settings_window: SettingsWindow::new(),
            settings: Settings::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(music) = self.music_loader.check_loaded() {
            self.synesthetizer.load_music(&music);
            self.music_state = MusicState::Loaded(music);
        }

        match self.control_panel.show(&mut self.music_state, self.music_loader.audio_manager_mut(), ctx) {
            MusicControl::Settings => {
                self.settings_window.toggle_open();
            }
            MusicControl::LoadMusic => {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let MusicState::Loaded(music) = &mut self.music_state {
                        music.stop();
                    }

                    self.music_state = MusicState::Loading(self.music_loader.load_from_file(path));
                }
            }
            MusicControl::Snapshot => {
                self.music_state.pause();
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    self.synesthetizer.request_snapshot(path);
                }
                self.music_state.play(self.music_loader.audio_manager_mut());
            }
            MusicControl::TogglePause => {
                if let MusicState::Loaded(music) = &mut self.music_state {
                    if music.is_playing() {
                        music.pause();
                    } else {
                        music.play(self.music_loader.audio_manager_mut());
                    }
                }
            }
            MusicControl::Nothing => {}
        }

        self.texture.set(
            self.synesthetizer.new_frame(&self.music_state, &self.settings).clone(),
            egui::TextureOptions::default()
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Image::new(&self.texture).fit_to_exact_size(ui.available_size()));
        });

        self.settings_window.show(ctx, &mut self.settings);

        // Repaint every frame
        ctx.request_repaint();
    }
}
