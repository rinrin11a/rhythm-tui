use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    chart::LANES,
    game::{Game, Judgement, KEYS},
};

const LANE_WIDTH: usize = 7;
const VIEW_SECONDS: f64 = 2.0;
const LANE_COLORS: [Color; LANES] = [Color::Cyan, Color::Magenta, Color::Yellow, Color::Green];

pub fn draw(frame: &mut Frame, game: &Game, audio_on: bool) {
    let [status, play, help] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let audio = if audio_on {
        "音频: 开"
    } else {
        "音频: 无设备(静音)"
    };
    let offset_ms = (game.offset * 1000.0).round() as i64;
    let stats = format!(
        " 分数 {}   连击 {}   最高连击 {}   延迟校准 {:+}ms   {}",
        game.score, game.combo, game.max_combo, offset_ms, audio
    );
    frame.render_widget(
        Paragraph::new(stats).block(Block::bordered().title(" RHYTHM-TUI ")),
        status,
    );
    draw_lanes(frame, play, game);
    frame.render_widget(
        Paragraph::new("D F J K 打击 · [ ] 校准延迟 · R 重新开始 · Q 退出").centered(),
        help,
    );
    if game.finished() {
        draw_result(frame, game);
    }
}

fn draw_lanes(frame: &mut Frame, area: Rect, game: &Game) {
    let width = (LANES * LANE_WIDTH + LANES - 1 + 2) as u16;
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    let block = Block::bordered().title(" LANES ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = inner.height as usize;
    if rows < 3 {
        return;
    }
    let judge_row = rows - 2;
    let per_second = judge_row as f64 / VIEW_SECONDS;
    let now = game.time();

    let mut hot = vec![[false; LANES]; rows];
    for (note, judged) in game.notes.iter().zip(&game.judged) {
        let y = judge_row as f64 - (note.time - now) * per_second;
        if judged.is_none() && y >= 0.0 {
            hot[(y as usize).min(judge_row)][note.lane] = true;
        }
    }

    let lines: Vec<Line> = (0..rows)
        .map(|row| {
            if row == judge_row {
                judge_line(&hot[row], game, now)
            } else if row == judge_row + 1 {
                key_line()
            } else if row + 2 == judge_row {
                feedback_line(game, now)
            } else {
                note_line(&hot[row])
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

fn lanes_line(cell: impl Fn(usize) -> Span<'static>) -> Line<'static> {
    let mut spans = Vec::with_capacity(LANES * 2 - 1);
    for lane in 0..LANES {
        if lane > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(cell(lane));
    }
    Line::from(spans)
}

fn note_line(hot: &[bool; LANES]) -> Line<'static> {
    lanes_line(|lane| {
        if hot[lane] {
            Span::styled("█".repeat(LANE_WIDTH), Style::new().fg(LANE_COLORS[lane]))
        } else {
            Span::raw(" ".repeat(LANE_WIDTH))
        }
    })
}

fn judge_line(hot: &[bool; LANES], game: &Game, now: f64) -> Line<'static> {
    lanes_line(|lane| {
        let pressed = now - game.lane_pressed_at[lane] < 0.1;
        let style = if pressed {
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(LANE_COLORS[lane])
        };
        let glyph = if hot[lane] { "█" } else { "━" };
        Span::styled(glyph.repeat(LANE_WIDTH), style)
    })
}

fn key_line() -> Line<'static> {
    lanes_line(|lane| {
        let label = format!(
            "{:^width$}",
            KEYS[lane].to_ascii_uppercase(),
            width = LANE_WIDTH
        );
        Span::styled(
            label,
            Style::new()
                .fg(LANE_COLORS[lane])
                .add_modifier(Modifier::BOLD),
        )
    })
}

fn feedback_line(game: &Game, now: f64) -> Line<'static> {
    let Some(feedback) = &game.feedback else {
        return Line::default();
    };
    if now - feedback.at > 0.6 {
        return Line::default();
    }
    let judgement = feedback.judgement;
    let text = if judgement == Judgement::Miss {
        judgement.label().to_string()
    } else {
        let ms = (feedback.error * 1000.0).round() as i64;
        format!("{} {:+}ms", judgement.label(), ms)
    };
    Line::from(Span::styled(
        text,
        Style::new()
            .fg(judgement_color(judgement))
            .add_modifier(Modifier::BOLD),
    ))
    .centered()
}

fn judgement_color(judgement: Judgement) -> Color {
    match judgement {
        Judgement::Perfect => Color::LightYellow,
        Judgement::Great => Color::LightGreen,
        Judgement::Good => Color::LightBlue,
        Judgement::Miss => Color::Red,
    }
}

fn draw_result(frame: &mut Frame, game: &Game) {
    let area = centered(frame.area(), 44, 11);
    let count = |j: Judgement| game.counts[j as usize];
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("评级  {}", game.rank()),
            Style::new()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("分数  {}", game.score)),
        Line::from(format!("最高连击  {}", game.max_combo)),
        Line::from(format!("准确率  {:.2}%", game.accuracy())),
        Line::from(format!(
            "PERFECT {}  GREAT {}  GOOD {}  MISS {}",
            count(Judgement::Perfect),
            count(Judgement::Great),
            count(Judgement::Good),
            count(Judgement::Miss)
        )),
        Line::from(""),
        Line::from("R 再来一局    Q 退出"),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .centered()
            .block(Block::bordered().title(" 结算 ")),
        area,
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}
