use crate::{component::Component, message::Message, view::Viewer};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub struct State {
    contents: String,
}

impl State {
    pub fn new() -> Self {
        Self {
            contents: Default::default(),
        }
    }
}

impl Component for State {
    fn update(&mut self, message: &Message) -> anyhow::Result<Option<Message>> {
        Ok(match message {
            Message::Event(event) => match event {
                Event::FocusGained => {
                    //
                    None
                }
                Event::FocusLost => {
                    //
                    None
                }
                Event::Key(KeyEvent {
                    code,
                    kind,
                    modifiers,
                    ..
                }) => {
                    if *kind == KeyEventKind::Press {
                        if modifiers.is_empty() {
                            match code {
                                KeyCode::Char(c) => {
                                    self.contents.push(*c);
                                    None
                                }
                                KeyCode::Backspace => {
                                    self.contents.pop();
                                    None
                                }
                                KeyCode::Tab => {
                                    self.contents += "    ";
                                    None
                                }
                                KeyCode::Enter => {
                                    self.contents.push('\n');
                                    None
                                }
                                _ => None,
                            }
                        } else if modifiers.contains(KeyModifiers::SHIFT) {
                            match code {
                                KeyCode::Char(c) => {
                                    self.contents.push(c.to_ascii_uppercase());
                                    None
                                }
                                _ => None,
                            }
                        } else if modifiers.contains(KeyModifiers::CONTROL) {
                            match code {
                                KeyCode::Char('c' | 'x') => Some(Message::Quit),
                                _ => None,
                            }
                        } else {
                            match code {
                                _ => None,
                            }
                        }
                    } else {
                        None
                    }
                }
                Event::Mouse(_) => {
                    //
                    None
                }
                Event::Paste(_) => {
                    //
                    None
                }
                Event::Resize(_, _) => {
                    //
                    None
                }
            },
            _ => None,
        })
    }

    fn view<'core>(&self, viewer: Viewer<'core>) -> anyhow::Result<Viewer<'core>> {
        let mut lines = self.contents.split('\n');
        let last_line = lines.next_back();
        let viewer = lines.try_fold(viewer, |viewer, line| viewer.write(line))?;
        if let Some(last_line) = last_line {
            viewer.write(last_line)
        } else {
            Ok(viewer)
        }
    }
}
