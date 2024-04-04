use std::{path::Path, thread::{self, JoinHandle}};

use kira::{manager::AudioManager, sound::{static_sound::{StaticSoundData, StaticSoundHandle}, PlaybackState, Region}};

#[derive(Debug, Clone)]
pub struct MusicMeta {
    pub file_name: String,
    pub name: String,
}

pub struct Music {
    meta: MusicMeta,
    len: f64,
    sound_data: StaticSoundData,
    sound: StaticSoundHandle,
}

impl Music {
    pub fn file_name(&self) -> &str {
        &self.meta.file_name
    }
    
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    pub fn is_playing(&self) -> bool {
        self.sound.state() == PlaybackState::Playing
    }

    pub fn is_stopped(&self) -> bool {
        self.sound.state() == PlaybackState::Stopped
    }

    pub fn play(&mut self, audio_manager: &mut AudioManager) {
        if self.is_stopped() {
            self.sound = audio_manager.play(self.sound_data.clone()).unwrap();
        } else {
            self.sound.resume(Default::default()).unwrap();
        }
    }

    pub fn pause(&mut self) {
        self.sound.pause(Default::default()).unwrap();
    }

    pub fn stop(&mut self) {
        self.sound.stop(Default::default()).unwrap();
    }

    pub fn position(&self) -> f64 {
        self.sound.position()
    }
    
    pub fn scrub(&mut self, amount: f64, audio_manager: &mut AudioManager) {
        if self.is_stopped() {
            self.sound = audio_manager.play(self.sound_data.clone()).unwrap();
        }

        if self.position() + amount <= 0.0 {
            self.sound.seek_to(0.0).unwrap();
        } else {
            self.sound.seek_by(amount).unwrap();
        }
    }

    pub fn len(&self) -> f64 {
        self.len
    }

    pub fn data(&self) -> &StaticSoundData {
        &self.sound_data
    }

    pub fn sample_rate(&self) -> u32 {
        self.sound_data.sample_rate
    }
}

pub struct MusicLoader {
    audio_manager: AudioManager,
    active_channel: Option<LoadingChannel>,
}

impl MusicLoader {
    pub fn new(audio_manager: AudioManager) -> Self {
        Self {
            audio_manager,
            active_channel: None,
        }
    }

    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> MusicMeta {
        let file_name = path.as_ref()
            .file_name()
            .map(|s| s.to_str().unwrap().to_owned())
            .unwrap_or(String::from("<unreadable file name>"));
        let name = path.as_ref()
            .with_extension("")
            .file_name()
            .map(|s| s.to_str().unwrap().to_string())
            .unwrap_or(String::from("Unknown"));

        let path = path.as_ref().to_path_buf();

        let join_handle = thread::spawn(move || -> anyhow::Result<StaticSoundData> {
            let sound_data = StaticSoundData::from_file(&path, Default::default())?;
            log::info!("Loaded");
            Ok(sound_data)
        });

        let music_meta = MusicMeta {
            file_name,
            name,
        };

        self.active_channel = Some(LoadingChannel {
            music_meta: music_meta.clone(),
            join_handle,
        });

        music_meta
    }

    pub fn check_loaded(&mut self) -> Option<Music> {
        if self.active_channel.is_some() {
            let channel = self.active_channel.as_ref().unwrap();

            if channel.join_handle.is_finished() {
                let channel = std::mem::replace(&mut self.active_channel, None).unwrap();

                match channel.join_handle.join().unwrap() {
                    Ok(sound_data) => {
                        let sound = self.audio_manager.play(sound_data.clone()).unwrap();
                        let len = sound_data.frames.len() as f64 / sound_data.sample_rate as f64;
                        
                        return Some(Music {
                            meta: channel.music_meta,
                            len,
                            sound_data,
                            sound,
                        });
                    }
                    Err(e) => {
                        log::error!("There was a problem loading the music: {e:?}");
                    }
                }
            }
        }

        None
    }

    pub fn audio_manager_mut(&mut self) -> &mut AudioManager {
        &mut self.audio_manager
    }
}

struct LoadingChannel {
    pub music_meta: MusicMeta,
    pub join_handle: JoinHandle<anyhow::Result<StaticSoundData>>,
}
