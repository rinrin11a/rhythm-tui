use std::time::Instant;

use crate::chart::{LANES, Note};

pub const KEYS: [char; LANES] = ['d', 'f', 'j', 'k'];
const GOOD_WINDOW: f64 = 0.120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    Perfect,
    Great,
    Good,
    Miss,
}

impl Judgement {
    pub const ALL: [Judgement; 4] = [Self::Perfect, Self::Great, Self::Good, Self::Miss];

    fn from_error(error: f64) -> Option<Self> {
        match error.abs() {
            e if e <= 0.040 => Some(Self::Perfect),
            e if e <= 0.080 => Some(Self::Great),
            e if e <= GOOD_WINDOW => Some(Self::Good),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Perfect => "PERFECT",
            Self::Great => "GREAT",
            Self::Good => "GOOD",
            Self::Miss => "MISS",
        }
    }

    fn points(self) -> u32 {
        match self {
            Self::Perfect => 300,
            Self::Great => 200,
            Self::Good => 100,
            Self::Miss => 0,
        }
    }

    fn weight(self) -> f64 {
        match self {
            Self::Perfect => 1.0,
            Self::Great => 0.7,
            Self::Good => 0.4,
            Self::Miss => 0.0,
        }
    }
}

pub struct Feedback {
    pub judgement: Judgement,
    pub error: f64,
    pub at: f64,
}

pub struct Game {
    pub notes: Vec<Note>,
    pub judged: Vec<Option<Judgement>>,
    pub counts: [u32; 4],
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub feedback: Option<Feedback>,
    pub lane_pressed_at: [f64; LANES],
    pub offset: f64,
    start: Instant,
}

impl Game {
    pub fn new(notes: Vec<Note>, offset: f64) -> Self {
        Self {
            judged: vec![None; notes.len()],
            notes,
            counts: [0; 4],
            score: 0,
            combo: 0,
            max_combo: 0,
            feedback: None,
            lane_pressed_at: [f64::NEG_INFINITY; LANES],
            offset,
            start: Instant::now(),
        }
    }

    pub fn time(&self) -> f64 {
        self.start.elapsed().as_secs_f64() - self.offset
    }

    pub fn adjust_offset(&mut self, delta: f64) {
        self.offset = (self.offset + delta).clamp(-0.3, 0.3);
    }

    pub fn press(&mut self, lane: usize, now: f64) -> Option<Judgement> {
        self.lane_pressed_at[lane] = now;
        let (idx, error) = self
            .notes
            .iter()
            .enumerate()
            .filter(|(i, note)| note.lane == lane && self.judged[*i].is_none())
            .map(|(i, note)| (i, now - note.time))
            .filter(|(_, error)| error.abs() <= GOOD_WINDOW)
            .min_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))?;
        let judgement = Judgement::from_error(error)?;
        self.resolve(idx, judgement, error, now);
        Some(judgement)
    }

    pub fn tick(&mut self, now: f64) {
        let missed: Vec<usize> = self
            .notes
            .iter()
            .enumerate()
            .filter(|(i, note)| self.judged[*i].is_none() && now - note.time > GOOD_WINDOW)
            .map(|(i, _)| i)
            .collect();
        for idx in missed {
            let error = now - self.notes[idx].time;
            self.resolve(idx, Judgement::Miss, error, now);
        }
    }

    pub fn finished(&self) -> bool {
        self.judged.iter().all(Option::is_some)
    }

    pub fn accuracy(&self) -> f64 {
        if self.notes.is_empty() {
            return 0.0;
        }
        let weighted: f64 = Judgement::ALL
            .iter()
            .map(|&j| self.counts[j as usize] as f64 * j.weight())
            .sum();
        weighted / self.notes.len() as f64 * 100.0
    }

    pub fn rank(&self) -> char {
        match self.accuracy() {
            a if a >= 95.0 => 'S',
            a if a >= 85.0 => 'A',
            a if a >= 70.0 => 'B',
            a if a >= 50.0 => 'C',
            _ => 'D',
        }
    }

    fn resolve(&mut self, idx: usize, judgement: Judgement, error: f64, now: f64) {
        self.judged[idx] = Some(judgement);
        self.counts[judgement as usize] += 1;
        self.feedback = Some(Feedback {
            judgement,
            error,
            at: now,
        });
        if judgement == Judgement::Miss {
            self.combo = 0;
        } else {
            self.combo += 1;
            self.max_combo = self.max_combo.max(self.combo);
            let multiplier = (1 + self.combo / 10).min(4);
            self.score += judgement.points() * multiplier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(notes: &[(usize, f64)]) -> Game {
        let notes = notes
            .iter()
            .map(|&(lane, time)| Note { lane, time })
            .collect();
        Game::new(notes, 0.0)
    }

    #[test]
    fn judgement_windows() {
        assert_eq!(Judgement::from_error(0.03), Some(Judgement::Perfect));
        assert_eq!(Judgement::from_error(-0.07), Some(Judgement::Great));
        assert_eq!(Judgement::from_error(0.10), Some(Judgement::Good));
        assert_eq!(Judgement::from_error(0.20), None);
    }

    #[test]
    fn press_hits_nearest_note_in_same_lane() {
        let mut g = game(&[(0, 1.0), (0, 1.1), (1, 1.05)]);
        assert_eq!(g.press(0, 1.09), Some(Judgement::Perfect));
        assert_eq!(g.judged[1], Some(Judgement::Perfect));
        assert_eq!(g.judged[0], None);
        assert_eq!(g.press(1, 1.0), Some(Judgement::Great));
        assert_eq!(g.press(2, 1.0), None);
    }

    #[test]
    fn unhit_notes_become_misses_and_break_combo() {
        let mut g = game(&[(0, 1.0), (0, 2.0)]);
        g.press(0, 1.0);
        g.tick(2.5);
        assert_eq!(g.judged[1], Some(Judgement::Miss));
        assert_eq!(g.combo, 0);
        assert_eq!(g.max_combo, 1);
        assert!(g.finished());
    }
}
