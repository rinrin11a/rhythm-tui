mod audio;
mod chart;
mod game;
mod ui;

use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use audio::Audio;
use game::{Game, KEYS};

const BPM: f64 = 120.0;
const BARS: u32 = 32;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    let track = audio::synth_track(BPM, BARS);
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut audio, &track);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut DefaultTerminal,
    audio: &mut Audio,
    track: &[f32],
) -> Result<(), Box<dyn Error>> {
    let mut game = start(audio, track, 0.0);
    loop {
        let now = game.time();
        game.tick(now);
        terminal.draw(|frame| ui::draw(frame, &game, audio.is_available()))?;

        if !event::poll(Duration::from_millis(8))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            break;
        }
        match key.code {
            KeyCode::Esc => break,
            KeyCode::Char(c) => match c.to_ascii_lowercase() {
                'q' => break,
                'r' => game = start(audio, track, game.offset),
                '[' => game.adjust_offset(-0.005),
                ']' => game.adjust_offset(0.005),
                ch => {
                    if let Some(lane) = KEYS.iter().position(|&k| k == ch) {
                        let now = game.time();
                        if game.press(lane, now).is_some() {
                            audio.play_hit();
                        }
                    }
                }
            },
            _ => {}
        }
    }
    Ok(())
}

fn start(audio: &mut Audio, track: &[f32], offset: f64) -> Game {
    let notes = chart::generate(BPM, BARS, seed());
    audio.play_track(track);
    Game::new(notes, offset)
}

fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.as_nanos() as u64)
}
