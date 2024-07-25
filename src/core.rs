use crate::{
    component::{state::State, Component},
    input::Input,
    message::Message,
    view::{self, Output, Point, Viewer},
};
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use std::io::Write;

pub struct Core {
    state: State,
    size: Point,
    output: Output,
}

impl Core {
    pub fn new() -> anyhow::Result<Self> {
        let (width, height) = terminal::size()?;

        terminal::enable_raw_mode()?;

        let mut output = view::get_output();
        if let Err(error) = output
            .queue(EnterAlternateScreen)
            .and_then(|output| output.queue(EnableMouseCapture))
            .and_then(|output| output.flush())
        {
            terminal::disable_raw_mode()?;
            Err(error.into())
        } else {
            Ok(Self {
                state: State::new(),
                size: Point {
                    x: width,
                    y: height,
                },
                output,
            })
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let input = Input::new();
        let mut updated = true;

        loop {
            for event in input.read()? {
                updated = true;
                let mut message = Message::Event(event);
                while let Some(new_message) = self.state.update(&message)? {
                    match new_message {
                        Message::Quit => {
                            return Ok(());
                        }
                        new_message => {
                            message = new_message;
                        }
                    }
                }
            }
            if updated {
                self.output
                    .queue(Clear(ClearType::All))?
                    .queue(MoveTo(0, 0))?;
                self.state
                    .view(Viewer::new(&mut self.output, Default::default(), self.size))?;
                self.output.flush()?;
            }
            updated = false;
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let _ = self
            .output
            .queue(DisableMouseCapture)
            .and_then(|output| output.queue(LeaveAlternateScreen))
            .and_then(|output| output.flush());
        terminal::disable_raw_mode().unwrap();
    }
}
