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
    io::{self, BufRead, BufReader, ErrorKind},
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

            out.queue(Print(if dir.file_type.is_dir() { "* " } else { "> " }))?
                .queue(Print(dir.path.display()))?
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

    fn pop_current_row(&mut self) -> Res<Row> {
        self.below.pop_front().context("below is never empty")
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
                let row = self.pop_current_row()?;
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

            pressed!(Key::Left) if self.current_row()?.at_front() => {
                if let Some(mut row) = self.above.pop_back() {
                    row.active = row.chars.len().saturating_sub(1);
                    self.below.push_front(row);
                }

                Ok(None)
            }

            pressed!(Key::Right) if self.current_row()?.at_back() => {
                let row = self.pop_current_row()?;
                if self.below.is_empty() {
                    self.below.push_front(row);
                } else {
                    self.above.push_back(row);
                    self.current_row_mut()?.active = 0;
                }

                Ok(None)
            }

            pressed!(Key::Enter, shift + ctrl) => {
                self.refresh = true;
                self.below.push_front(Default::default());

                Ok(None)
            }

            pressed!(Key::Enter, ctrl) => {
                self.refresh = true;
                let row = self.pop_current_row()?;
                self.above.push_back(row);
                self.below.push_front(Default::default());

                Ok(None)
            }

            pressed!(Key::Enter) => {
                self.refresh = true;
                let mut row = self.pop_current_row()?;
                let new_row = row.split();
                row.chars.push(' ');
                self.above.push_back(row);
                self.below.push_front(new_row);

                Ok(None)
            }

            pressed!(Key::Backspace) if self.current_row()?.at_front() => {
                if let Some(mut row) = self.above.pop_back() {
                    self.refresh = true;
                    row.chars.pop();
                    row.active = row.chars.len();
                    row.chars.extend(self.pop_current_row()?.chars);
                    self.below.push_front(row);
                }

                Ok(None)
            }

            pressed!(Key::Delete) if self.current_row()?.at_back() => {
                let mut row = self.pop_current_row()?;
                if let Ok(mut next_row) = self.pop_current_row() {
                    self.refresh = true;
                    next_row.chars.pop();
                    row.chars.extend(next_row.chars);
                }
                self.below.push_front(row);

                Ok(None)
            }

            _ => self.current_row_mut()?.update(message),
        }
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            clear(out, bounds)?;
        }

        for row in &self.above {
            row.view(out, bounds, false)?;
            out.queue(MoveDown(1))?.queue(MoveToColumn(bounds.x0))?;
        }

        self.current_row()?.view(out, bounds, true)?;
        out.queue(MoveDown(1))?.queue(MoveToColumn(bounds.x0))?;

        for row in self.below.iter().skip(1) {
            row.view(out, bounds, false)?;
            out.queue(MoveDown(1))?.queue(MoveToColumn(bounds.x0))?;
        }

        if active {
            let row = bounds.x0 + <u16>::try_from(self.above.len())?;
            let column = bounds.y0 + <u16>::try_from(self.current_row()?.active)?;

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

impl Row {
    fn split(&mut self) -> Self {
        Self {
            chars: self.chars.split_off(self.active),
            active: 0,
        }
    }

    fn at_front(&self) -> bool {
        self.active == 0
    }

    fn at_back(&self) -> bool {
        self.active == self.chars.len().saturating_sub(1)
    }
}

impl Component for Row {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        let nonalphanum = |&c: &char| !c.is_alphanumeric();

        match message {
            pressed!(Key::Left, ctrl) => {
                self.active = if let Some(backward) =
                    self.chars[..self.active].iter().rev().position(nonalphanum)
                {
                    self.active - backward - 1
                } else {
                    0
                };

                Ok(None)
            }

            pressed!(Key::Left) => {
                self.active = self.active.saturating_sub(1);
                Ok(None)
            }

            pressed!(Key::Right, ctrl) => {
                self.active = if let Some(forward) =
                    self.chars[self.active + 1..].iter().position(nonalphanum)
                {
                    self.active + forward + 1
                } else {
                    self.chars.len().saturating_sub(1)
                };

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

            &pressed!(Key::Char(c)) => {
                self.chars.insert(self.active, c);
                self.active += 1;

                Ok(None)
            }

            pressed!(Key::Backspace, ctrl) => {
                let prev_active = self.active;

                self.active = if let Some(backward) =
                    self.chars[..self.active].iter().rev().position(nonalphanum)
                {
                    self.active - backward - 1
                } else {
                    0
                };

                self.chars.drain(self.active..prev_active);

                Ok(None)
            }

            pressed!(Key::Backspace) => {
                if !self.at_front() {
                    self.chars.remove(self.active - 1);
                    self.active -= 1;
                }

                Ok(None)
            }

            pressed!(Key::Delete, ctrl) => {
                let until =
                    if let Some(forward) = self.chars[self.active..].iter().position(nonalphanum) {
                        self.active + forward + 1
                    } else {
                        self.chars.len().saturating_sub(1)
                    };

                self.chars.drain(self.active..until);

                Ok(None)
            }

            pressed!(Key::Delete) => {
                if !self.at_back() {
                    self.chars.remove(self.active);
                }

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn view(&self, out: &mut Out, Bounds { x0, x1, .. }: Bounds, active: bool) -> Res<()> {
        for &c in self.chars.iter().take((x1 - x0).into()) {
            out.queue(Print(c))?;
        }
        if active {
            let remainder =
                (x1 - x0).saturating_sub(self.chars.len().try_into().unwrap_or(u16::MAX));
            for _ in 0..remainder {
                out.queue(Print(' '))?;
            }
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
            chars: value
                .chars()
                .flat_map(|c| {
                    if c == '\t' {
                        iter::once(' ').cycle().take(3)
                    } else {
                        iter::once(c).cycle().take(1)
                    }
                })
                .chain(iter::once(' '))
                .collect(),
            active: 0,
        }
    }
}
