use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::theme;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const HEIGHT: u16 = 5;

const LOGO: [&str; HEIGHT as usize] = [
    "   ______      ____    ",
    "  / ____/___ _/ / /____",
    " / /   / __ `/ / / ___/",
    "/ /___/ /_/ / / / /    ",
    "\\____/\\__,_/_/_/_/     ",
];

const TAGLINE: &str = "HTTP REST Client";

pub fn render(frame: &mut Frame, area: Rect) {
    let mid = LOGO.len() / 2;
    let lines: Vec<Line> = LOGO
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut spans =
                vec![Span::styled(*row, Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))];
            if i == mid {
                spans.push(Span::styled(format!("  v{VERSION}"), Style::default().fg(theme::MUTED)));
            } else if i == mid + 1 {
                spans.push(Span::styled(
                    format!("  {TAGLINE}"),
                    Style::default().fg(theme::MUTED).add_modifier(Modifier::ITALIC),
                ));
            }
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}
