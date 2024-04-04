use std::path::PathBuf;

use image::{Rgba, RgbaImage};
use spectrum_analyzer::{samples_fft_to_spectrum, scaling, windows::hann_window, FrequencyLimit, FrequencySpectrum};

use crate::{app::MusicState, music::Music, note::{Note, Pitch}, settings_window::Settings};

pub const C0_FREQ: f32 = 16.35;
pub const A8_FREQ: f32 = 7902.13;

pub const FRAME_WIDTH: u32 = 1600;
pub const FRAME_HEIGHT: u32 = 900;
pub const FRAME_WIDTH_F32: f32 = FRAME_WIDTH as f32;
pub const FRAME_HEIGHT_F32: f32 = FRAME_HEIGHT as f32;

#[derive(serde::Deserialize)]
pub struct ColorPalette(#[serde(deserialize_with = "from_hex")] pub [Rgba<u8>; 12]);

fn from_hex<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<[Rgba<u8>; 12], D::Error> {
    let list: Vec<&str> = serde::Deserialize::deserialize(deserializer)?;
    assert_eq!(list.len(), 12);

    let mut colors = [Rgba([255, 255, 255, 255]); 12];

    for (idx, str) in list.iter().enumerate() {
        assert_eq!(str.len(), 7);
        let r = u8::from_str_radix(&str[1..3], 16).unwrap();
        let g = u8::from_str_radix(&str[3..5], 16).unwrap();
        let b = u8::from_str_radix(&str[5..7], 16).unwrap();
        colors[idx] = Rgba([r, g, b, 255]);
    }

    Ok(colors)
}

pub struct Synesthetizer {
    samples_per_frame: usize,
    current_frame: Vec<f32>,
    current_notes: Vec<Note>,
    palette: ColorPalette,
    previous_image: RgbaImage,
    is_overlay: bool,
    snapshot_request: Option<PathBuf>,
}

impl Synesthetizer {
    pub fn new() -> Self {
        let palette = serde_yaml::from_slice(include_bytes!("colors.yaml")).unwrap();

        Self {
            samples_per_frame: 0,
            current_frame: Vec::new(),
            current_notes: Vec::with_capacity(64), // Allocate a lot to avoid reallocating
            palette,
            previous_image: RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT),
            is_overlay: false,
            snapshot_request: None,
        }
    }

    pub fn clear_overlay(&mut self) {
        self.previous_image = RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT);
    }

    pub fn load_music(&mut self, music: &Music) {
        let target_fps = 12.;

        self.samples_per_frame = 2;

        // The number of samples needs to be a power of two for the spectrum analyzer.
        loop {
            let current_diff = (music.sample_rate() as f64 / self.samples_per_frame as f64 - target_fps).abs();
            let times_2_diff = (music.sample_rate() as f64 / (self.samples_per_frame * 2) as f64 - target_fps).abs();

            if times_2_diff < current_diff {
                self.samples_per_frame *= 2;
            } else {
                break;
            }
        }

        self.current_frame.clear();
        self.current_frame.reserve(self.samples_per_frame);
    }

    pub fn request_snapshot(&mut self, path: PathBuf) {
        self.snapshot_request = Some(path);
        log::info!("Snapshot requested.");
    }

    pub fn new_frame(&mut self, music_state: &MusicState, settings: &Settings) -> egui::ColorImage {
        if settings.is_overlay != self.is_overlay {
            self.is_overlay = settings.is_overlay;

            if !self.is_overlay {
                self.clear_overlay();
                log::info!("Overlay cleared.");
            }
        }

        let mut image = if self.is_overlay {
            self.previous_image.clone()
        } else {
            RgbaImage::new(FRAME_WIDTH, FRAME_HEIGHT)
        };

        match music_state {
            MusicState::Loaded(music) if !music.is_stopped() => {
                self.update_samples(music);
                let spectrum = samples_fft_to_spectrum(
                    &self.current_frame,
                    music.sample_rate(),
                    FrequencyLimit::Range(C0_FREQ, A8_FREQ),
                    Some(&scaling::divide_by_N_sqrt),
                ).unwrap();
                self.find_tones(&spectrum);

                for note in &self.current_notes {
                    note.paint(&mut image, &self.palette);
                }
            }
            _ => {}
        }

        if self.is_overlay {
            self.previous_image = image.clone();
        }

        if let Some(path) = &self.snapshot_request {
            image::save_buffer(
                path,
                &image,
                FRAME_WIDTH,
                FRAME_HEIGHT,
                image::ColorType::Rgba8
            ).unwrap();
            self.snapshot_request = None;
            log::info!("Snapshot saved!");
        }

        egui::ColorImage::from_rgba_premultiplied(
            [FRAME_WIDTH as usize, FRAME_HEIGHT as usize],
            &image,
        )
    }

    /// Call before `samples_fft_to_spectrum`
    fn update_samples(&mut self, music: &Music) {
        self.current_frame.clear();

        let start_sample = (music.position() * music.sample_rate() as f64) as usize;
        // Don't go past the end of the song!
        let end_sample = (start_sample + self.samples_per_frame).min(music.data().frames.len());

        if end_sample > start_sample {
            for frame in &music.data().frames[start_sample..end_sample] {
                self.current_frame.push(frame.as_mono().left);
            }
        }

        self.current_frame = hann_window(&self.current_frame);
        self.current_frame.resize(self.samples_per_frame, 0.0)
    }

    /// Call after `samples_fft_to_spectrum`
    fn find_tones(&mut self, spectrum: &FrequencySpectrum) {
        self.current_notes.clear();

        for (fr, amp) in spectrum.data() {
            let pitch = Pitch::from_frequency(fr.val());
            let amplitude = amp.val();
            if let Some(closest) = self.current_notes.iter_mut().min_by(|a, b| {
                // Closest in frequency
                a.distance_from_midi(pitch.midi()).total_cmp(&b.distance_from_midi(pitch.midi()))
            }) {
                if let Err(_) = closest.try_include(pitch, amplitude) {
                    self.current_notes.push(Note::new(pitch, amplitude))
                }
            } else {
                self.current_notes.push(Note::new(pitch, amplitude));
            }
        }

        self.current_notes.sort_by(|a, b| a.peak_amplitude.total_cmp(&b.peak_amplitude));
    }
}
