use crossterm::event::{KeyCode, KeyEvent};

use crate::action::Action;
use crate::mode::{Focus, Mode};

/// Resolves a key press into an [`Action`]. Only meaningful in Normal mode —
/// Insert and Command modes forward keystrokes directly to their focused
/// text widget and only intercept a small set of control keys themselves.
pub fn resolve(mode: Mode, focus: Focus, key: KeyEvent) -> Option<Action> {
    if mode != Mode::Normal {
        return None;
    }

    // Keys that behave the same regardless of which pane is focused.
    match key.code {
        KeyCode::Char('q') => return Some(Action::Quit),
        KeyCode::Tab => return Some(Action::ToggleFocus),
        KeyCode::Char(':') => return Some(Action::EnterCommand),
        _ => {}
    }

    match focus {
        Focus::UrlBar => match key.code {
            KeyCode::Char('i') => Some(Action::EnterInsert),
            KeyCode::Enter => Some(Action::SendRequest),
            _ => None,
        },
        Focus::Auth => match key.code {
            KeyCode::Char('a') => Some(Action::AuthCycleKind),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::AuthFieldDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::AuthFieldUp),
            KeyCode::Char('i') => Some(Action::EnterInsert),
            KeyCode::Char('l') | KeyCode::Enter => Some(Action::AuthToggleLocation),
            _ => None,
        },
        Focus::Sidebar => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::TreeDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::TreeUp),
            KeyCode::Char('h') | KeyCode::Left => Some(Action::TreeCollapse),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::TreeExpand),
            KeyCode::Char('G') => Some(Action::JumpBottom),
            KeyCode::Enter => Some(Action::TreeOpen),
            _ => None,
        },
        Focus::Response => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::TreeDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::TreeUp),
            KeyCode::Char('h') | KeyCode::Left => Some(Action::TreeCollapse),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => Some(Action::TreeExpand),
            KeyCode::Char('G') => Some(Action::JumpBottom),
            KeyCode::Char('/') => Some(Action::SearchStart),
            KeyCode::Char('n') => Some(Action::SearchNext),
            KeyCode::Char('N') => Some(Action::SearchPrev),
            _ => None,
        },
    }
}
