use crate::{
    component::{line::Line, Bounds, Component},
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{self, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{Hide, MoveDown, MoveToColumn, MoveToRow},
    style::Print,
    QueueableCommand,
};
use std::{
    env,
    fs::{self, File, FileType},
    io::{self, BufRead, BufReader, ErrorKind},
    path::{Path, PathBuf},
};

#[allow(private_interfaces)]
#[derive(Clone, Debug)]
pub enum Content {
    FilePicker(FilePicker),
    Buffer(Buffer),
}

impl Content {
    pub fn new() -> Res<Self> {
        Ok(Self::FilePicker(FilePicker::new()?))
    }
}

impl Component for Content {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Esc) if matches!(self, Self::Buffer(_)) => {
                *self = Self::FilePicker(FilePicker::new()?);
                Ok(None)
            }

            Message::Open(path) => {
                match Buffer::open(path) {
                    Ok(buffer) => {
                        *self = Self::Buffer(buffer);
                    }
                    Err(error) => {
                        let io_error: &io::Error = error.downcast_ref().context("unknown error")?;
                        if io_error.kind() != ErrorKind::InvalidData {
                            return Err(error);
                        }
                    }
                }

                Ok(None)
            }

            _ => match self {
                Content::Buffer(buffer) => buffer.update(message),
                Content::FilePicker(filepicker) => filepicker.update(message),
            },
        }
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        match self {
            Content::Buffer(buffer) => buffer.view(out, bounds, active),
            Content::FilePicker(filepicker) => filepicker.view(out, bounds, active),
        }
    }

    fn finally(&mut self) -> Res<()> {
        match self {
            Content::FilePicker(filepicker) => filepicker.finally(),
            Content::Buffer(buffer) => buffer.finally(),
        }
    }
}

#[derive(Clone, Debug)]
struct Dir {
    path: PathBuf,
    file_type: FileType,
}

#[derive(Clone, Debug)]
struct FilePicker {
    dirs: Vec<Dir>,
    selected: usize,
    history: Vec<PathBuf>,
    refresh: bool,
}

impl FilePicker {
    fn new() -> Res<Self> {
        let mut filepicker = Self {
            dirs: vec![],
            selected: 0,
            history: vec![env::current_dir()?],
            refresh: false,
        };
        filepicker.open()?;

        Ok(filepicker)
    }

    fn open(&mut self) -> Res<()> {
        self.dirs = fs::read_dir(self.history.last().context("history is not empty")?)?
            .map(|res| {
                res.and_then(|dir| {
                    Ok(Dir {
                        path: dir.path(),
                        file_type: dir.file_type()?,
                    })
                })
            })
            .collect::<Result<_, _>>()?;
        self.selected = 0;
        self.refresh = true;

        Ok(())
    }
}

impl Component for FilePicker {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        let update = match message {
            pressed!(Key::Up) => {
                self.selected = if self.selected == 0 {
                    self.dirs.len() - 1
                } else {
                    self.selected - 1
                };
                None
            }

            pressed!(Key::Down) => {
                self.selected = if self.selected == self.dirs.len() - 1 {
                    0
                } else {
                    self.selected + 1
                };
                None
            }

            pressed!(Key::Enter) => {
                let dir = &self.dirs[self.selected];

                if dir.file_type.is_file() {
                    Some(Message::Open(dir.path.clone()))
                } else if dir.file_type.is_dir() {
                    self.history.push(dir.path.clone());
                    self.open()?;
                    None
                } else {
                    None
                }
            }

            pressed!(Key::Esc) => {
                if let Some(prev) = self.history.pop() {
                    if self.history.is_empty() {
                        self.history.push(prev);
                    } else {
                        self.open()?;
                    }
                }
                None
            }

            _ => None,
        };

        Ok(update)
    }

    fn view<'out>(&self, out: &'out mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            out::clear(out, bounds)?;
        }

        out.queue(Hide)?;

        let mut out = out;
        for (i, dir) in self.dirs.iter().enumerate() {
            let queue_line = |out: &'out mut Out| -> Res<&'out mut Out> {
                Ok(out
                    .queue(Print(if dir.file_type.is_dir() { "* " } else { "> " }))?
                    .queue(Print(dir.path.display()))?
                    .queue(MoveDown(1))?
                    .queue(MoveToColumn(bounds.x0))?)
            };

            if active && i == self.selected {
                out = out::with_highlighted(out, queue_line)?;
            } else {
                out = queue_line(out)?;
            }
        }

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        self.refresh = false;

        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
struct Buffer {
    above: Vec<Line>,
    below: Vec<Line>,
    refresh: bool,
}

impl Buffer {
    fn open(path: impl AsRef<Path>) -> Res<Self> {
        let file = BufReader::new(File::open(path)?);
        let mut below = file
            .lines()
            .map(|res| res.map(Into::into))
            .collect::<Result<Vec<_>, _>>()?;

        below.push(Default::default());
        below.reverse();

        Ok(Self {
            above: vec![],
            below,
            refresh: true,
        })
    }

    fn pop_current_line(&mut self) -> Res<Line> {
        self.below.pop().context("below is never empty")
    }

    fn current_line(&self) -> Res<&Line> {
        self.below.last().context("below is never empty")
    }

    fn current_line_mut(&mut self) -> Res<&mut Line> {
        self.below.last_mut().context("below is never empty")
    }
}

impl Component for Buffer {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Up) => {
                if let Some(mut line) = self.above.pop() {
                    line.set_active(self.current_line()?.active());
                    self.below.push(line);
                }

                Ok(None)
            }

            pressed!(Key::Down) => {
                let line = self.pop_current_line()?;
                if self.below.is_empty() {
                    self.below.push(line);
                } else {
                    let prev_active = line.active();
                    self.above.push(line);
                    self.current_line_mut()?.set_active(prev_active);
                }

                Ok(None)
            }

            pressed!(Key::Left) if self.current_line()?.at_front() => {
                if let Some(mut line) = self.above.pop() {
                    line.set_active_back();
                    self.below.push(line);
                }

                Ok(None)
            }

            pressed!(Key::Right) if self.current_line()?.at_back() => {
                let line = self.pop_current_line()?;
                if self.below.is_empty() {
                    self.below.push(line);
                } else {
                    self.above.push(line);
                    self.current_line_mut()?.set_active_front();
                }

                Ok(None)
            }

            pressed!(Key::Enter, shift + ctrl) => {
                self.refresh = true;
                self.below.push(Default::default());

                Ok(None)
            }

            pressed!(Key::Enter, ctrl) => {
                self.refresh = true;
                let line = self.pop_current_line()?;
                self.above.push(line);
                self.below.push(Default::default());

                Ok(None)
            }

            pressed!(Key::Enter) => {
                self.refresh = true;
                let mut line = self.pop_current_line()?;
                self.below.push(line.split());
                self.above.push(line);

                Ok(None)
            }

            pressed!(Key::Backspace) if self.current_line()?.at_front() => {
                if let Some(mut line) = self.above.pop() {
                    self.refresh = true;
                    line.set_active_back();
                    line.append(self.pop_current_line()?);
                    self.below.push(line);
                }

                Ok(None)
            }

            pressed!(Key::Delete) if self.current_line()?.at_back() => {
                let mut line = self.pop_current_line()?;
                if let Ok(next_line) = self.pop_current_line() {
                    self.refresh = true;
                    line.append(next_line);
                }
                self.below.push(line);

                Ok(None)
            }

            _ => self.current_line_mut()?.update(message),
        }
    }

    fn view<'out>(&self, out: &'out mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            out::clear(out, bounds)?;
        }

        let middle = bounds.y0 + ((bounds.y1 - bounds.y0) / 2);
        let above_indices = bounds.y0..middle;
        let below_indices = middle + 1..bounds.y1;

        let line_feed = |out: &'out mut Out| -> Res<&'out mut Out> {
            out.queue(MoveDown(1))?
                .queue(MoveToColumn(bounds.x0))
                .map_err(Into::into)
        };

        let mut out = out;

        for (line, y) in self.above.iter().rev().zip(above_indices.rev()).rev() {
            let line_bounds = Bounds {
                y0: y,
                y1: y + 1,
                ..bounds
            };

            line.view(out, line_bounds, false)?;
            out = line_feed(out)?;
        }
        out = line_feed(out)?;

        for (line, y) in self.below.iter().rev().skip(1).zip(below_indices) {
            let line_bounds = Bounds {
                y0: y,
                y1: y + 1,
                ..bounds
            };
            line.view(out, line_bounds, false)?;
            out = line_feed(out)?;
        }

        out.queue(MoveToRow(bounds.y0 + self.above.len() as u16))?;
        let line_bounds = Bounds {
            y0: middle,
            y1: middle + 1,
            ..bounds
        };
        self.current_line()?.view(out, line_bounds, active)?;
        line_feed(out)?;

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        self.refresh = false;

        Ok(())
    }
}
