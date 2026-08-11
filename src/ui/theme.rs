use ratatui::style::{Color, Modifier, Style};

pub const FOCUS_BORDER: Color = Color::Green;
pub const SELECTED_FG: Color = Color::Black;
pub const SELECTED_BG: Color = Color::Green;
pub const MODE_INDICATOR_FG: Color = Color::Black;
pub const MODE_INDICATOR_BG: Color = Color::Cyan;
pub const ACCENT: Color = Color::Cyan;
pub const MUTED: Color = Color::DarkGray;

pub const SYNTAX_KEY: Color = Color::Cyan;
pub const SYNTAX_STRING: Color = Color::Green;
pub const SYNTAX_NUMBER: Color = Color::Yellow;
pub const SYNTAX_BOOL: Color = Color::Magenta;
pub const SYNTAX_NULL: Color = Color::DarkGray;

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(FOCUS_BORDER)
    } else {
        Style::default()
    }
}

pub fn selection_style() -> Style {
    Style::default().fg(SELECTED_FG).bg(SELECTED_BG).add_modifier(Modifier::BOLD)
}

pub fn mode_indicator_style() -> Style {
    Style::default().fg(MODE_INDICATOR_FG).bg(MODE_INDICATOR_BG)
}

pub fn accent_style() -> Style {
    Style::default().fg(ACCENT)
}
