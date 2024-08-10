use crate::{
    component::line::Line,
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{self, Bounds, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{Hide, MoveDown, MoveToColumn, MoveToRow},
    style::Print,
    QueueableCommand,
};
use std::{
    collections::VecDeque,
    env,
    fs::{self, File, FileType},
    io::{self, BufRead, BufReader, ErrorKind},
    path::{Path, PathBuf},
};

const SCROLL_DIST: usize = 3;

#[allow(private_interfaces)]
#[derive(Clone, Debug)]
pub enum Content {
    FilePicker(FilePicker),
    Buffer(Buffer),
}

impl Content {
    pub fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self::FilePicker(FilePicker::new(bounds)?))
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Esc) => {
                if let Content::Buffer(buffer) = self {
                    *self = Self::FilePicker(FilePicker::new(buffer.bounds)?);
                }
                Ok(None)
            }

            Message::Open(path) => {
                if let Content::FilePicker(filepicker) = self {
                    match Buffer::open(path, filepicker.bounds) {
                        Ok(buffer) => {
                            *self = Self::Buffer(buffer);
                        }
                        Err(error) => {
                            let io_error: &io::Error =
                                error.downcast_ref().context("unknown error")?;
                            if io_error.kind() != ErrorKind::InvalidData {
                                return Err(error);
                            }
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

    pub fn view(&self, out: &mut Out, active: bool) -> Res<()> {
        match self {
            Content::Buffer(buffer) => buffer.view(out, active),
            Content::FilePicker(filepicker) => filepicker.view(out, active),
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
    bounds: Bounds,
}

impl FilePicker {
    fn new(bounds: Bounds) -> Res<Self> {
        let mut filepicker = Self {
            dirs: vec![],
            selected: 0,
            history: vec![env::current_dir()?],
            bounds,
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

        Ok(())
    }

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

    fn view<'out>(&self, out: &'out mut Out, active: bool) -> Res<()> {
        out.queue(Hide)?;

        let mut out = out;
        for (i, dir) in self.dirs.iter().enumerate() {
            let queue_line = |out: &'out mut Out| -> Res<&'out mut Out> {
                Ok(out
                    .queue(Print(if dir.file_type.is_dir() { "* " } else { "> " }))?
                    .queue(Print(dir.path.display()))?
                    .queue(MoveDown(1))?
                    .queue(MoveToColumn(self.bounds.x0))?)
            };

            if active && i == self.selected {
                out = out::with_highlighted(out, queue_line)?;
            } else {
                out = queue_line(out)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
struct Buffer {
    lines: VecDeque<Line>,
    above: Vec<Line>,
    below: Vec<Line>,
    active: usize,
    bounds: Bounds,
}

impl Buffer {
    fn open(path: impl AsRef<Path>, bounds: Bounds) -> Res<Self> {
        let file = BufReader::new(File::open(path)?);
        let mut lines = file
            .lines()
            .map(|res| res.map(Into::into))
            .collect::<Result<Vec<_>, _>>()?;
        let height = bounds.height().into();
        let below = if height < lines.len() {
            lines.split_off(height)
        } else {
            vec![]
        };

        Ok(Self {
            lines: lines.into(),
            above: vec![],
            below,
            active: 0,
            bounds,
        })
    }

    fn current_line(&self) -> Res<&Line> {
        self.lines.get(self.active).context("active is valid")
    }

    fn current_line_mut(&mut self) -> Res<&mut Line> {
        self.lines.get_mut(self.active).context("active is valid")
    }

    fn scroll_down(&mut self) -> Res<bool> {
        if let Some(line_from_below) = self.below.pop() {
            self.lines.push_back(line_from_below);
            let line_to_above = self.lines.pop_front().context("at least one line")?;
            self.above.push(line_to_above);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scroll_up(&mut self) -> Res<bool> {
        if let Some(line_from_above) = self.above.pop() {
            self.lines.push_back(line_from_above);
            let line_to_below = self.lines.pop_back().context("at least one line")?;
            self.below.push(line_to_below);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn cursor_down(&mut self) -> Res<()> {
        let prev_line_active = self.current_line()?.active();

        if self.active < self.lines.len() - SCROLL_DIST {
            self.active += 1;
            self.current_line_mut()?.set_active(prev_line_active);

            Ok(())
        } else if self.scroll_down()? {
            self.current_line_mut()?.set_active(prev_line_active);

            Ok(())
        } else {
            self.active = (self.active + 1).clamp(0, self.lines.len() - 1);

            Ok(())
        }
    }

    fn cursor_up(&mut self) -> Res<()> {
        let prev_line_active = self.current_line()?.active();

        if self.active > SCROLL_DIST {
            self.active -= 1;
            self.current_line_mut()?.set_active(prev_line_active);

            Ok(())
        } else if self.scroll_up()? {
            self.current_line_mut()?.set_active(prev_line_active);

            Ok(())
        } else {
            self.active = self.active.saturating_sub(1);

            Ok(())
        }
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Up) => {
                self.cursor_up()?;

                Ok(None)
            }

            pressed!(Key::Down) => {
                self.cursor_down()?;

                Ok(None)
            }

            pressed!(Key::Left) if self.current_line()?.at_front() => {
                self.cursor_up()?;
                self.current_line_mut()?.set_active_back();

                Ok(None)
            }

            pressed!(Key::Right) if self.current_line()?.at_back() => {
                self.cursor_down()?;
                self.current_line_mut()?.set_active_front();

                Ok(None)
            }

            pressed!(Key::Enter, shift + ctrl) => {
                self.lines.insert(self.active, Default::default());
                self.below
                    .push(self.lines.pop_back().context("at least one line")?);

                Ok(None)
            }

            pressed!(Key::Enter, ctrl) => {
                self.cursor_down()?;
                self.lines.insert(self.active, Default::default());
                self.below
                    .push(self.lines.pop_back().context("at least one line")?);

                Ok(None)
            }

            pressed!(Key::Enter) => {
                let new_line = self.current_line_mut()?.split();
                self.cursor_down()?;
                self.lines.insert(self.active, new_line);
                self.below
                    .push(self.lines.pop_back().context("at least one line")?);

                Ok(None)
            }

            pressed!(Key::Backspace) if self.current_line()?.at_front() => {
                let line = self.lines.remove(self.active).context("active is valid")?;
                self.scroll_up()?;
                self.current_line_mut()?.append(line);
                if let Some(line_from_below) = self.below.pop() {
                    self.lines.push_back(line_from_below);
                }

                Ok(None)
            }

            pressed!(Key::Delete) if self.current_line()?.at_back() => {
                let line = self.lines.remove(self.active).context("active is valid")?;
                self.current_line_mut()?.append(line);
                if let Some(line_from_below) = self.below.pop() {
                    self.lines.push_back(line_from_below);
                }

                Ok(None)
            }

            _ => {
                self.current_line_mut()?.update(message)?;

                Ok(None)
            }
        }
    }

    fn view(&self, out: &mut Out, active: bool) -> Res<()> {
        for (i, line) in self.lines.iter().enumerate() {
            line.view(out, self.bounds.width(), active && i == self.active)?;
            out.queue(MoveDown(1))?
                .queue(MoveToColumn(self.bounds.x0))?;
        }

        Ok(())
    }
}
