pub const LANES: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub lane: usize,
    pub time: f64,
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_f64() as f32
    }
}

/// 在八分音符网格上随机生成谱面，第一小节留作前奏
pub fn generate(bpm: f64, bars: u32, seed: u64) -> Vec<Note> {
    let mut rng = Rng::new(seed);
    let step = 60.0 / bpm / 2.0;
    let mut notes = Vec::new();
    let mut prev: Option<usize> = None;

    for i in 8..bars * 8 {
        let chance = if i % 2 == 0 { 0.9 } else { 0.55 };
        if rng.next_f64() >= chance {
            continue;
        }
        let time = i as f64 * step;
        let lane = loop {
            let lane = rng.next_u64() as usize % LANES;
            if Some(lane) != prev {
                break lane;
            }
        };
        notes.push(Note { lane, time });
        prev = Some(lane);

        if i % 8 == 0 && rng.next_f64() < 0.3 {
            let other = (lane + 1 + rng.next_u64() as usize % (LANES - 1)) % LANES;
            notes.push(Note { lane: other, time });
        }
    }
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_are_sorted_and_in_range() {
        let notes = generate(120.0, 32, 42);
        assert!(!notes.is_empty());
        assert!(notes.windows(2).all(|w| w[0].time <= w[1].time));
        assert!(notes.iter().all(|n| n.lane < LANES && n.time >= 2.0));
    }
}
