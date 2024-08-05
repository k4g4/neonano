use crate::{
    component::{Bounds, Component},
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{clear, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{EnableBlinking, Hide, MoveDown, MoveToColumn, MoveToRow, Show},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use std::{
    collections::VecDeque,
    env,
    fs::{self, File, FileType},
    io::{BufRead, BufReader},
    iter,
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
                *self = Self::Buffer(Buffer::open(path)?);
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

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            clear(out, bounds)?;
        }

        out.queue(Hide)?;

        for (i, dir) in self.dirs.iter().enumerate() {
            let highlighted = active && i == self.selected;

            if highlighted {
                out.queue(SetBackgroundColor(Color::White))?
                    .queue(SetForegroundColor(Color::Black))?;
            }

            out.queue(Print(dir.path.display()))?
                .queue(MoveDown(1))?
                .queue(MoveToColumn(bounds.x0))?;

            if highlighted {
                out.queue(ResetColor)?;
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
    above: VecDeque<Row>,
    below: VecDeque<Row>,
    refresh: bool,
}

impl Buffer {
    fn open(path: impl AsRef<Path>) -> Res<Self> {
        let file = BufReader::new(File::open(path)?);
        let below = file
            .lines()
            .map(|res| res.map(|s| s.as_str().into()))
            .chain(iter::once(Ok(Default::default())))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            above: VecDeque::new(),
            below,
            refresh: true,
        })
    }

    fn current_row(&self) -> Res<&Row> {
        self.below.front().context("below is never empty")
    }

    fn current_row_mut(&mut self) -> Res<&mut Row> {
        self.below.front_mut().context("below is never empty")
    }
}

impl Component for Buffer {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Up) => {
                if let Some(mut row) = self.above.pop_back() {
                    let prev_active = self.current_row()?.active;
                    row.active = prev_active.clamp(0, row.chars.len().saturating_sub(1));
                    self.below.push_front(row);
                }
                Ok(None)
            }

            pressed!(Key::Down) => {
                let row = self.below.pop_front().context("below is never empty")?;
                if self.below.is_empty() {
                    self.below.push_front(row);
                } else {
                    let prev_active = row.active;
                    self.above.push_back(row);
                    self.current_row_mut()?.active =
                        prev_active.clamp(0, self.current_row()?.chars.len().saturating_sub(1));
                }
                Ok(None)
            }

            _ => self.current_row_mut()?.update(message),
        }
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            clear(out, bounds)?;
        }

        let rows = self.above.iter().chain(&self.below);

        for row in rows.take((bounds.y1 - bounds.y0).into()) {
            row.view(out, bounds, false)?;
            out.queue(MoveDown(1))?.queue(MoveToColumn(bounds.x0))?;
        }

        if active {
            let row = bounds.x0 + <u16>::try_from(self.above.len())?;
            let column = self.current_row()?.active.try_into()?;

            out.queue(MoveToRow(row))?
                .queue(MoveToColumn(column))?
                .queue(Show)?
                .queue(EnableBlinking)?;
        }

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        self.refresh = false;

        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
struct Row {
    chars: Vec<char>,
    active: usize,
}

impl Component for Row {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Left) => {
                self.active = self.active.saturating_sub(1);
                Ok(None)
            }

            pressed!(Key::Right) => {
                self.active = (self.chars.len().saturating_sub(1)).min(self.active + 1);
                Ok(None)
            }

            pressed!(Key::Home) => {
                self.active = 0;
                Ok(None)
            }

            pressed!(Key::End) => {
                self.active = self.chars.len().saturating_sub(1);
                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn view(&self, out: &mut Out, _bounds: Bounds, _active: bool) -> Res<()> {
        for &c in &self.chars {
            out.queue(Print(c))?;
        }

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        Ok(())
    }
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        Self {
            chars: value.chars().chain(iter::once(' ')).collect(),
            active: 0,
        }
    }
}
