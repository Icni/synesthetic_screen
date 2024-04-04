use std::ops::Range;

use image::{imageops, Rgba, RgbaImage};
use imageproc::{drawing, pixelops, point::Point};

use crate::synesthetizer::{ColorPalette, FRAME_HEIGHT_F32, FRAME_WIDTH_F32};

#[derive(Debug, Clone, Copy)]
pub struct Pitch {
    frequency: f32,
    midi: f32,
}

impl Pitch {
    pub fn from_frequency(fr: f32) -> Self {
        // source: <https://newt.phys.unsw.edu.au/jw/notes.html>
        Self {
            frequency: fr,
            midi: 12. * (fr / 440.).log2() + 69.
        }
    }

    pub fn from_midi(midi: f32) -> Self {
        // source: <https://newt.phys.unsw.edu.au/jw/notes.html>
        Self {
            frequency: 2f32.powf((midi - 69.) / 12.) * 440.,
            midi,
        }
    }

    pub fn midi(&self) -> f32 {
        self.midi
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }
}

impl PartialEq for Pitch {
    fn eq(&self, other: &Self) -> bool {
        self.frequency() == other.frequency()
    }
}

pub struct SoundInclusionError;

#[derive(Debug, Clone)]
pub struct Note {
    pub peak_pitch: Pitch,
    pub peak_amplitude: f32,
    pub amp_range: Range<f32>,
    pub midi_range: Range<f32>,
}

impl Note {
    const MAX_MIDI_RANGE: f32 = 1.0;
    const MAX_AMPLITUDE_RANGE: f32 = 0.25;

    pub fn new(pitch: Pitch, amplitude: f32) -> Self {
        Self {
            peak_pitch: pitch,
            peak_amplitude: amplitude,
            amp_range: amplitude..amplitude,
            midi_range: pitch.midi..pitch.midi,
        }
    }

    pub fn midi(&self) -> f32 {
        self.peak_pitch.midi
    }

    pub fn frequency(&self) -> f32 {
        self.peak_pitch.frequency
    }

    pub fn amplitude(&self) -> f32 {
        self.peak_amplitude
    }

    pub fn distance_from_midi(&self, midi: f32) -> f32 {
        dist_from_range_bounds(midi, &self.midi_range)
    }

    pub fn try_include(&mut self, pitch: Pitch, amplitude: f32) -> Result<(), SoundInclusionError> {
        if {
            dist_from_range_bounds(pitch.midi, &self.midi_range) + range_len(&self.midi_range)
                > Self::MAX_MIDI_RANGE
                ||
            dist_from_range_bounds(amplitude, &self.amp_range) + range_len(&self.amp_range)
                > Self::MAX_AMPLITUDE_RANGE
        } {
            Err(SoundInclusionError)
        } else {
            include_in_range(amplitude, &mut self.amp_range);
            include_in_range(pitch.midi, &mut self.midi_range);

            if amplitude > self.peak_amplitude {
                self.peak_amplitude = amplitude;
                self.peak_pitch = pitch;
            }

            Ok(())
        }
    }

    pub fn paint(&self, image: &mut RgbaImage, color_palette: &ColorPalette) {
        let width = self.width() as i32;
        let height = self.height() as i32;

        if width == 0 || height == 0 {
            return;
        }

        let polygon = [
            Point::new(0, height / 2),
            Point::new(width / 2, 0),
            Point::new(width, height / 2),
            Point::new(width / 2, height),
        ];
        
        let x = self.x() - (width / 2);
        let y = self.y() - (height / 2);

        let mut star = RgbaImage::new(self.width(), self.height());
        drawing::draw_polygon_mut(
            &mut star,
            polygon.as_slice(),
            self.color(color_palette)
        );

        imageops::overlay(image, &star, x as i64, y as i64);
    }

    pub fn width(&self) -> u32 {
        (2500 / self.height()) * 2
    }

    pub fn height(&self) -> u32 {
        (self.amplitude() * 100.).ceil() as u32 + 3
    }

    pub fn x(&self) -> i32 {
        (FRAME_WIDTH_F32 * (self.midi()/127.)).round() as i32
    }

    pub fn y(&self) -> i32 {
        (FRAME_HEIGHT_F32 / 2.).round() as i32
    }

    pub fn color(&self, color_palette: &ColorPalette) -> Rgba<u8> {
        let midi = self.midi();
        let diatonic_note = midi % 12.;

        let ceil = color_palette.0[{
            let ceil = diatonic_note.ceil() as usize;
            if ceil == 12 {
                0
            } else {
                ceil
            }
        }];
        let floor = color_palette.0[diatonic_note.floor() as usize];
        
        let fractional = diatonic_note % 1.;

        let mut color = pixelops::interpolate(ceil, floor, fractional);
        color = pixelops::interpolate(color, Rgba([0, 0, 0, 0]), self.amplitude().sqrt() * 0.5);

        color
    }
}

fn dist_from_range_bounds(v: f32, r: &Range<f32>) -> f32 {
    if r.contains(&v) {
        0.
    } else if v > r.start {
        v - r.start
    } else {
        r.end - v
    }
}

fn include_in_range(v: f32, r: &mut Range<f32>) {
    if !r.contains(&v) {
        if v > r.start {
            r.start = v;
        } else if v < r.end {
            r.end = v;
        }
    }
}

fn range_len(r: &Range<f32>) -> f32 {
    r.end - r.start
}
