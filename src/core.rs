use crate::{
    component::{frame::Frame, Component},
    input::Input,
    message::Message,
};
use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use std::io::{self, StdoutLock, Write};

pub type Res<T> = anyhow::Result<T>;
pub type Out = StdoutLock<'static>;

pub struct Core {
    frame: Frame,
    size: (u16, u16),
    out: Out,
}

impl Core {
    pub fn new() -> Res<Self> {
        let (width, height) = terminal::size()?;

        terminal::enable_raw_mode()?;

        let mut out = io::stdout().lock();
        if let Err(error) = out
            .queue(EnterAlternateScreen)
            .and_then(|out| out.queue(EnableMouseCapture))
            .and_then(|out| out.flush())
        {
            terminal::disable_raw_mode()?;
            Err(error.into())
        } else {
            Ok(Self {
                frame: Frame::new(),
                size: Point {
                    x: width,
                    y: height,
                },
                out,
            })
        }
    }

    pub fn run(mut self) -> Res<()> {
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
