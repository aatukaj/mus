use macroquad::audio::{self, Sound};

const AUDIO_FILES: &[&str] = &[
    "Slam_Layer_Clap06.wav",
    "Slam_MPC_Snare04.wav",
    "Slam_RX8_Conga01.wav",
    "Slam_Layer_Kick05.wav",
    "Slam_MPC_Tom03.wav",
    "Slam_RX8_Tambourine02.wav",
    "Slam_Layer_OHat03.wav",
    "Slam_R5_CHat04.wav",
    "Slam_Layer_Snare01.wav",
    "Slam_R5_Crash02.wav",
];
pub struct Audio {
    samples: Vec<Sound>,
}
pub async fn load_samples() -> Vec<Sound> {
    let mut samples = vec![];
    for s in AUDIO_FILES {
        samples.push(audio::load_sound(s).await.unwrap());
    }
    samples
}
impl Audio {
    pub fn new(samples: Vec<Sound>) -> Self {
        Self { samples }
    }
    pub fn play(&self, idx: usize) {
        audio::play_sound_once(&self.samples[idx]);
    }
}
