use crate::input::Input;
use crate::state::State;
use crossterm::{
    event::{EnableMouseCapture, Event},
    terminal, ExecutableCommand,
};
use std::{
    io::{self, StdoutLock},
    thread,
    time::Duration,
};

pub struct Core {
    state: State,
    stdout: StdoutLock<'static>,
    size: (u16, u16),
}

impl Core {
    pub fn new() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout().lock();
        if let Err(error) = stdout.execute(EnableMouseCapture) {
            terminal::disable_raw_mode()?;
            Err(error.into())
        } else {
            Ok(Self {
                state: State::new(),
                stdout,
                size: terminal::size()?,
            })
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let frame_time = Duration::from_secs_f32(1.0 / 60.0);
        let input = Input::new();

        loop {
            for event in input.read()? {
                match event {
                    Event::FocusGained => todo!(),
                    Event::FocusLost => todo!(),
                    Event::Key(_) => todo!(),
                    Event::Mouse(_) => todo!(),
                    Event::Paste(_) => todo!(),
                    Event::Resize(_, _) => todo!(),
                }
            }
            thread::sleep(frame_time);
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
    }
}
