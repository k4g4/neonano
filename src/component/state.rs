use crate::{component::Component, message::Message, view::Viewer};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

pub struct State {
    #[allow(unused)]
    size: (u16, u16),
}

impl State {
    pub fn new(size: (u16, u16)) -> Self {
        Self { size }
    }
}

impl Component for State {
    fn update(&mut self, message: Message) -> anyhow::Result<Option<Message>> {
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
                    modifiers: _,
                    state: _,
                }) => match (code, kind) {
                    (KeyCode::Char('q'), KeyEventKind::Press) => Some(Message::Quit),
                    _ => None,
                },
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

    fn view<'a>(&self, viewer: Viewer<'a>) -> anyhow::Result<Viewer<'a>> {
        viewer.write_line("foobar")?.write_line("bazqux")
    }
}
