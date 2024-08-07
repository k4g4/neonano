use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Message {
    Input(Input),
    Open(PathBuf),
    Quit,
}

#[derive(Copy, Clone, Debug)]
pub enum Input {
    FocusGained,
    FocusLost,
    KeyCombo(KeyCombo),
}

#[derive(Copy, Clone, Debug)]
pub struct KeyCombo {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Key {
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Delete,
    Insert,
    Esc,
    CapsLock,
}

impl TryFrom<Event> for Input {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(match event {
            Event::FocusGained => Self::FocusGained,
            Event::FocusLost => Self::FocusLost,
            Event::Key(KeyEvent {
                code,
                kind,
                modifiers,
                ..
            }) => {
                if kind == KeyEventKind::Press || kind == KeyEventKind::Repeat {
                    Self::KeyCombo(KeyCombo {
                        key: match code {
                            KeyCode::Char(c) => Key::Char(c),
                            KeyCode::Backspace => Key::Backspace,
                            KeyCode::Enter => Key::Enter,
                            KeyCode::Left => Key::Left,
                            KeyCode::Right => Key::Right,
                            KeyCode::Up => Key::Up,
                            KeyCode::Down => Key::Down,
                            KeyCode::Home => Key::Home,
                            KeyCode::End => Key::End,
                            KeyCode::PageUp => Key::PageUp,
                            KeyCode::PageDown => Key::PageDown,
                            KeyCode::Tab => Key::Tab,
                            KeyCode::Delete => Key::Delete,
                            KeyCode::Insert => Key::Insert,
                            KeyCode::Esc => Key::Esc,
                            KeyCode::CapsLock => Key::CapsLock,
                            _ => return Err(()),
                        },
                        shift: modifiers.contains(KeyModifiers::SHIFT),
                        ctrl: modifiers.contains(KeyModifiers::CONTROL),
                    })
                } else {
                    return Err(());
                }
            }
            Event::Mouse(_) => return Err(()),
            Event::Paste(_) => return Err(()),
            Event::Resize(_, _) => return Err(()),
        })
    }
}

#[macro_export]
macro_rules! pressed {
    ($key:pat) => {
        Message::Input(Input::KeyCombo(KeyCombo { key: $key, .. }))
    };

    ($key:pat, shift + ctrl) => {
        Message::Input(Input::KeyCombo(KeyCombo {
            key: $key,
            shift: true,
            ctrl: true,
            ..
        }))
    };

    ($key:pat, ctrl) => {
        Message::Input(Input::KeyCombo(KeyCombo {
            key: $key,
            ctrl: true,
            ..
        }))
    };

    ($key:pat, shift) => {
        Message::Input(Input::KeyCombo(KeyCombo {
            key: $key,
            shift: true,
            ..
        }))
    };
}
