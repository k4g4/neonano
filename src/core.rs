use crate::{
    component::frame::Frame,
    message::Message,
    utils::{
        input::InputReader,
        out::{Bounds, Out},
        shared::status,
    },
};
use crossterm::{
    cursor::{Hide, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    queue,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

pub type Res<T> = anyhow::Result<T>;

#[derive(Debug)]
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
        status::reset_all()?;

        let mut out = io::stdout().lock();
        let init_result: Res<_> = (|| {
            queue!(out, EnterAlternateScreen, EnableMouseCapture)?;
            out.flush()?;
            Ok(())
        })();

        if let Err(error) = init_result {
            terminal::disable_raw_mode()?;
            Err(error.into())
        } else {
            Ok(Self {
                frame: Frame::new(bounds)?,
                out,
            })
        }
    }

    pub fn run(mut self) -> Res<Self> {
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
                        break 'runtime Ok(self);
                    }
                }
            }

            if updated {
                queue!(self.out, MoveTo(0, 0), Hide)?;
                self.frame.view(&mut self.out)?;
                self.out.flush()?;
            }

            updated = false;
        }
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        let _ = (|| -> Res<_> {
            queue!(self.out, DisableMouseCapture, LeaveAlternateScreen)?;
            self.out.flush()?;
            Ok(())
        })();
        terminal::disable_raw_mode().unwrap();
    }
}
