use std::f32::consts::TAU;
use std::num::NonZero;

use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::chart::Rng;

const SAMPLE_RATE: u32 = 44_100;
const SR: f32 = SAMPLE_RATE as f32;

struct Chord {
    bass: f32,
    tones: [f32; 3],
}

const CHORDS: [Chord; 4] = [
    Chord {
        bass: 110.00,
        tones: [220.00, 261.63, 329.63],
    },
    Chord {
        bass: 87.31,
        tones: [174.61, 220.00, 261.63],
    },
    Chord {
        bass: 65.41,
        tones: [261.63, 329.63, 392.00],
    },
    Chord {
        bass: 98.00,
        tones: [196.00, 246.94, 293.66],
    },
];

pub struct Audio {
    device: Option<MixerDeviceSink>,
    player: Option<Player>,
    hit: Vec<f32>,
}

impl Audio {
    pub fn new() -> Self {
        let device = DeviceSinkBuilder::open_default_sink()
            .ok()
            .map(|mut device| {
                device.log_on_drop(false);
                device
            });
        Self {
            device,
            player: None,
            hit: synth_hit(),
        }
    }

    pub fn is_available(&self) -> bool {
        self.device.is_some()
    }

    pub fn play_track(&mut self, samples: &[f32]) {
        if let Some(device) = &self.device {
            let player = Player::connect_new(device.mixer());
            player.append(buffer(samples.to_vec()));
            self.player = Some(player);
        }
    }

    pub fn play_hit(&self) {
        if let Some(device) = &self.device {
            device.mixer().add(buffer(self.hit.clone()));
        }
    }
}

fn buffer(samples: Vec<f32>) -> SamplesBuffer {
    SamplesBuffer::new(
        NonZero::new(1).unwrap(),
        NonZero::new(SAMPLE_RATE).unwrap(),
        samples,
    )
}

pub fn synth_track(bpm: f64, bars: u32) -> Vec<f32> {
    let beat = (60.0 / bpm) as f32;
    let len = ((bars as f32 * 4.0 * beat + 1.0) * SR) as usize;
    let mut buf = vec![0.0; len];
    let mut rng = Rng::new(7);

    for i in 0..bars * 8 {
        let start = i as f32 * beat / 2.0;
        if i % 2 == 0 {
            kick(&mut buf, start);
        } else {
            hat(&mut buf, start, &mut rng);
        }
        let chord = &CHORDS[(i / 8) as usize % CHORDS.len()];
        bass(&mut buf, start, chord.bass);
    }
    for bar in 0..bars {
        let start = bar as f32 * 4.0 * beat;
        let chord = &CHORDS[bar as usize % CHORDS.len()];
        pad(&mut buf, start, 4.0 * beat, &chord.tones);
    }
    for s in &mut buf {
        *s = s.tanh();
    }
    buf
}

fn add(buf: &mut [f32], index: usize, value: f32) {
    if let Some(sample) = buf.get_mut(index) {
        *sample += value;
    }
}

fn kick(buf: &mut [f32], start: f32) {
    let first = (start * SR) as usize;
    let mut phase = 0.0;
    for n in 0..(0.35 * SR) as usize {
        let t = n as f32 / SR;
        phase += TAU * (45.0 + 120.0 * (-30.0 * t).exp()) / SR;
        add(buf, first + n, 0.9 * (-10.0 * t).exp() * phase.sin());
    }
}

fn hat(buf: &mut [f32], start: f32, rng: &mut Rng) {
    let first = (start * SR) as usize;
    let mut prev = 0.0;
    for n in 0..(0.08 * SR) as usize {
        let t = n as f32 / SR;
        let noise = rng.next_f32() * 2.0 - 1.0;
        add(buf, first + n, 0.2 * (-60.0 * t).exp() * (noise - prev));
        prev = noise;
    }
}

fn bass(buf: &mut [f32], start: f32, freq: f32) {
    let first = (start * SR) as usize;
    for n in 0..(0.25 * SR) as usize {
        let t = n as f32 / SR;
        let phase = TAU * freq * t;
        let env = (t * 200.0).min(1.0) * (-9.0 * t).exp();
        add(
            buf,
            first + n,
            0.2 * env * (phase.sin() + 0.4 * (2.0 * phase).sin()),
        );
    }
}

fn pad(buf: &mut [f32], start: f32, dur: f32, tones: &[f32; 3]) {
    let first = (start * SR) as usize;
    for n in 0..(dur * SR) as usize {
        let t = n as f32 / SR;
        let env = (t * 2.0).min(1.0) * ((dur - t) * 2.0).min(1.0);
        let sum: f32 = tones.iter().map(|f| (TAU * f * t).sin()).sum();
        add(buf, first + n, 0.04 * env * sum);
    }
}

fn synth_hit() -> Vec<f32> {
    (0..(0.08 * SR) as usize)
        .map(|n| {
            let t = n as f32 / SR;
            0.3 * (-60.0 * t).exp() * (TAU * 1800.0 * t).sin()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_length_and_range() {
        let track = synth_track(120.0, 4);
        assert_eq!(track.len(), (9.0 * SR) as usize);
        assert!(track.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        assert!(track.iter().any(|s| s.abs() > 0.1));
    }
}
