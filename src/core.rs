use crate::{
    component::frame::Frame,
    message::Message,
    utils::{
        input::InputReader,
        out::{Bounds, Out},
    },
};
use crossterm::{
    cursor::{Hide, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use std::io::{self, Write};

pub type Res<T> = anyhow::Result<T>;

pub struct Core {
    frame: Frame,
    out: Out,
}

impl Core {
    pub fn new() -> Res<Self> {
        let (width, height) = terminal::size()?;
        let bounds = Bounds {
            x0: 0,
            y0: 0,
            x1: width,
            y1: height,
        };

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
                frame: Frame::new(bounds)?,
                out,
            })
        }
    }

    pub fn run(mut self) -> Res<()> {
        let input_reader = InputReader::new();
        let mut updated = true;

        'runtime: loop {
            for event in input_reader.read()? {
                if let Ok(input) = event.try_into() {
                    updated = true;

                    let mut quit = false;
                    let mut message = Message::Input(input);

                    while let Some(returned_message) = self.frame.update(&message)? {
                        message = match returned_message {
                            Message::Input(_) => anyhow::bail!("input returned from update"),
                            Message::Quit => {
                                quit = true;
                                Message::Quit
                            }
                            other => other,
                        }
                    }

                    if quit {
                        break 'runtime Ok(());
                    }
                }
            }

            if updated {
                self.out.queue(MoveTo(0, 0))?.queue(Hide)?;
                self.frame.view(&mut self.out)?;
                self.out.flush()?;
            }

            updated = false;
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let _ = self
            .out
            .queue(DisableMouseCapture)
            .and_then(|output| output.queue(LeaveAlternateScreen))
            .and_then(|output| output.flush());
        terminal::disable_raw_mode().unwrap();
    }
}
