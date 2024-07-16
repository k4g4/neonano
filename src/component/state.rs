use crate::{component::Component, message::Message};
use crossterm::{
    event::{EnableMouseCapture, Event},
    terminal::EnterAlternateScreen,
    QueueableCommand,
};

pub struct State {
    size: (u16, u16),
}

impl State {
    pub fn new(size: (u16, u16)) -> Self {
        Self { size }
    }
}

impl Component for State {
    fn update(&mut self, message: Message) -> anyhow::Result<Option<Message>> {
        match message {
            Message::Event(event) => match event {
                Event::FocusGained => todo!(),
                Event::FocusLost => todo!(),
                Event::Key(_) => todo!(),
                Event::Mouse(_) => todo!(),
                Event::Paste(_) => todo!(),
                Event::Resize(_, _) => todo!(),
            },
        }

        Ok(None)
    }

    fn view(&self, output: &mut impl QueueableCommand) -> anyhow::Result<()> {
        output
            .queue(EnableMouseCapture)?
            .queue(EnterAlternateScreen)?;

        Ok(())
    }
}
