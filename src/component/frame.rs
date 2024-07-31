use crate::{
    component::{
        statusbars::{BottomBar, TopBar},
        window::Window,
        Component, Update,
    },
    core::{Out, Res},
    message::Message,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Default, Debug)]
pub struct Frame {
    top: TopBar,
    window: Window,
    bottom: BottomBar,
}

impl Frame {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Component for Frame {
    fn update(&mut self, message: &Message) -> Res<Update> {
        let mut contents = String::new();
        match message {
            Message::Event(event) => match event {
                Event::FocusGained => {
                    //
                }
                Event::FocusLost => {
                    //
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
                                    contents.push(*c);
                                }
                                KeyCode::Backspace => {
                                    contents.pop();
                                }
                                KeyCode::Tab => {
                                    contents += "    ";
                                }
                                KeyCode::Enter => {
                                    contents.push('\n');
                                }
                                _ => {}
                            }
                        } else if modifiers.contains(KeyModifiers::SHIFT) {
                            match code {
                                KeyCode::Char(c) => {
                                    contents.push(c.to_ascii_uppercase());
                                }
                                _ => {}
                            }
                        } else if modifiers.contains(KeyModifiers::CONTROL) {
                            match code {
                                KeyCode::Char('c' | 'x') => {
                                    Some(Message::Quit);
                                }
                                _ => {}
                            }
                        } else {
                            match code {
                                _ => {}
                            }
                        }
                    } else {
                        //
                    }
                }
                Event::Mouse(_) => {
                    //
                }
                Event::Paste(_) => {
                    //
                }
                Event::Resize(_, _) => {
                    //
                }
            },
            _ => {}
        }

        self.window.update(message)
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        let out = self.window.view(out, width, height)?;
        let out = self.top.view(out, width, height)?;
        self.bottom.view(out, width, height)
    }
}
