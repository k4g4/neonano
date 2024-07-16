use crate::{
    component::{state::State, Component},
    input::Input,
    message::Message,
};
use crossterm::terminal;
use std::{
    io::{self, StdoutLock, Write},
    thread,
    time::Duration,
};

pub struct Core {
    state: State,
    output: StdoutLock<'static>,
}

impl Core {
    pub fn new() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;

        Ok(Self {
            state: State::new(terminal::size()?),
            output: io::stdout().lock(),
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let frame_time = Duration::from_secs_f32(1.0 / 60.0);
        let input = Input::new();

        loop {
            thread::sleep(frame_time);
            for event in input.read()? {
                self.state.update(Message::Event(event))?;
            }
            self.state.view(&mut self.output)?;
            self.output.flush()?;
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}
