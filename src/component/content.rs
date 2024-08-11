use crate::{
    component::line::{Line, RawIndex},
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::{
        out::{self, Bounds, Out},
        shared::status::{self, Pos},
    },
};
use anyhow::Context;
use crossterm::{
    cursor::{Hide, MoveDown, MoveToColumn, MoveToRow},
    style::Print,
    QueueableCommand,
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    env,
    fmt::Write,
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
        let filepicker = FilePicker::new(bounds)?;
        filepicker.update_status()?;

        Ok(Self::FilePicker(filepicker))
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Esc) => {
                if let Content::Buffer(buffer) = self {
                    let filepicker = FilePicker::new(buffer.bounds)?;

                    status::reset_all()?;
                    filepicker.update_status()?;

                    *self = Self::FilePicker(filepicker);
                }

                Ok(None)
            }

            Message::Open(path) => {
                if let Content::FilePicker(filepicker) = self {
                    match Buffer::open(path, filepicker.bounds) {
                        Ok(buffer) => {
                            status::reset_all()?;
                            buffer.update_status()?;

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
                Content::Buffer(buffer) => {
                    let update = buffer.update(message)?;
                    buffer.update_status()?;

                    Ok(update)
                }
                Content::FilePicker(filepicker) => {
                    let update = filepicker.update(message)?;
                    filepicker.update_status()?;

                    Ok(update)
                }
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
struct FilePickerEntry {
    path: PathBuf,
    file_type: FileType,
}

#[derive(Clone, Debug)]
struct FilePicker {
    entries: Vec<FilePickerEntry>,
    selected: usize,
    history: Vec<PathBuf>,
    bounds: Bounds,
}

impl FilePicker {
    fn new(bounds: Bounds) -> Res<Self> {
        let mut filepicker = Self {
            entries: vec![],
            selected: 0,
            history: vec![env::current_dir()?],
            bounds,
        };
        filepicker.open()?;

        Ok(filepicker)
    }

    fn open(&mut self) -> Res<()> {
        self.entries = fs::read_dir(self.history.last().context("history is not empty")?)?
            .map(|res| {
                res.and_then(|entry| {
                    Ok(FilePickerEntry {
                        path: entry.path(),
                        file_type: entry.file_type()?,
                    })
                })
            })
            .collect::<Result<_, _>>()?;
        self.selected = 0;

        Ok(())
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Up) => {
                self.selected = if self.selected == 0 {
                    self.entries.len() - 1
                } else {
                    self.selected - 1
                };

                Ok(None)
            }

            pressed!(Key::Down) => {
                self.selected = if self.selected == self.entries.len() - 1 {
                    0
                } else {
                    self.selected + 1
                };

                Ok(None)
            }

            pressed!(Key::Enter) => {
                let dir = &self.entries[self.selected];

                if dir.file_type.is_file() {
                    Ok(Some(Message::Open(dir.path.clone())))
                } else if dir.file_type.is_dir() {
                    self.history.push(dir.path.clone());
                    self.open()?;

                    Ok(None)
                } else {
                    Ok(None)
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

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn update_status(&self) -> Res<()> {
        status::set(Pos::Bottom, |status| -> Res<_> {
            Ok(write!(
                status,
                "{}",
                self.history
                    .last()
                    .context("history is not empty")?
                    .display()
            )?)
        })??;

        Ok(())
    }

    fn view<'out>(&self, out: &'out mut Out, active: bool) -> Res<()> {
        out::anchor(out.queue(Hide)?, self.bounds)?;

        let width = self.bounds.width().into();
        let mut out = out;
        for (i, dir) in self.entries.iter().enumerate() {
            let queue_line = |out: &'out mut Out| -> Res<&'out mut Out> {
                let line = format!(
                    "{} {:<width$}",
                    if dir.file_type.is_dir() { '*' } else { '>' },
                    dir.path.display()
                );

                Ok(out
                    .queue(Print(&line[..width]))?
                    .queue(MoveDown(1))?
                    .queue(MoveToColumn(self.bounds.x0))?)
            };

            if active && i == self.selected {
                out = out::with_highlighted(out, queue_line)?;
            } else {
                out = queue_line(out)?;
            }
        }

        if self.entries.len() < self.bounds.height().into() {
            out::clear(
                out,
                Bounds {
                    y0: self.bounds.y0 + u16::try_from(self.entries.len())?,
                    ..self.bounds
                },
            )?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Buffer {
    lines: VecDeque<Line>,
    above: String,
    below: String,
    active: usize,
    index: RawIndex,
    bounds: Bounds,
    recycle: Vec<Line>,
}

impl Buffer {
    fn open(path: impl AsRef<Path>, bounds: Bounds) -> Res<Self> {
        let height = bounds.height().into();
        let file = BufReader::new(File::open(path)?);
        let mut lines = file.lines().collect::<Result<Vec<_>, _>>()?;
        let below = (height < lines.len())
            .then(|| {
                lines
                    .split_off(height)
                    .iter()
                    .rev()
                    .flat_map(|line| ["\n", line])
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            lines: lines.into_iter().map(Into::into).collect(),
            above: String::new(),
            below,
            active: 0,
            index: RawIndex::index_front(),
            bounds,
            recycle: vec![],
        })
    }

    fn current_line(&self) -> Res<&Line> {
        self.lines.get(self.active).context("active is valid")
    }

    fn current_line_mut(&mut self) -> Res<&mut Line> {
        self.lines.get_mut(self.active).context("active is valid")
    }

    fn at_top(&self) -> bool {
        self.active == 0
    }

    fn at_bottom(&self) -> bool {
        self.active == self.lines.len() - 1
    }

    fn insert_below(&mut self, line: Line) {
        self.below.push('\n');
        self.below.push_str(line.as_ref());
        self.recycle.push(line);
    }

    fn insert_above(&mut self, line: Line) {
        self.above.push('\n');
        self.above.push_str(line.as_ref());
        self.recycle.push(line);
    }

    fn take_from_below(&mut self) -> Res<Option<Line>> {
        if self.below.is_empty() {
            Ok(None)
        } else {
            let mut new_line = self.recycle.pop().unwrap_or_default();
            let pos = self.below.rfind('\n').context("newline before each line")?;

            new_line.clear();
            new_line.append(self.below[pos..].trim_start_matches('\n'));
            self.below.truncate(pos);

            Ok(Some(new_line))
        }
    }

    fn take_from_above(&mut self) -> Res<Option<Line>> {
        if self.above.is_empty() {
            Ok(None)
        } else {
            let mut new_line = self.recycle.pop().unwrap_or_default();
            let pos = self.above.rfind('\n').context("newline before each line")?;

            new_line.clear();
            new_line.append(self.above[pos..].trim_start_matches('\n'));
            self.above.truncate(pos);

            Ok(Some(new_line))
        }
    }

    fn scroll_down(&mut self) -> Res<bool> {
        if let Some(line_from_below) = self.take_from_below()? {
            self.lines.push_back(line_from_below);
            let line_to_above = self.lines.pop_front().context("at least one line")?;
            self.insert_above(line_to_above);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scroll_up(&mut self) -> Res<bool> {
        if let Some(line_from_above) = self.take_from_above()? {
            self.lines.push_front(line_from_above);
            self.fix_lines()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn cursor_down(&mut self) -> Res<bool> {
        if self.active < self.lines.len() - SCROLL_DIST {
            self.active += 1;
            self.index.invalidate();

            Ok(true)
        } else if self.scroll_down()? {
            self.index.invalidate();

            Ok(true)
        } else if self.active < self.lines.len() - 1 {
            self.active += 1;
            self.index.invalidate();

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn cursor_up(&mut self) -> Res<bool> {
        if self.active > SCROLL_DIST {
            self.active -= 1;
            self.index.invalidate();

            Ok(true)
        } else if self.scroll_up()? {
            self.index.invalidate();

            Ok(true)
        } else if self.active > 0 {
            self.active -= 1;
            self.index.invalidate();

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn jump_top(&mut self) -> Res<()> {
        while self.cursor_up()? {}

        Ok(())
    }

    fn jump_bottom(&mut self) -> Res<()> {
        while self.cursor_down()? {}

        Ok(())
    }

    fn fix_lines(&mut self) -> Res<()> {
        let (len, height) = (self.lines.len(), self.bounds.height().into());

        match len.cmp(&height) {
            Ordering::Greater => {
                for _ in height..len {
                    let new_line = self.lines.pop_back().context("len > height >= 0")?;
                    self.insert_below(new_line);
                }
            }

            Ordering::Less => {
                for _ in len..height {
                    if let Some(line_from_below) = self.take_from_below()? {
                        self.lines.push_back(line_from_below);
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Up) => {
                if !self.cursor_up()? {
                    self.index = RawIndex::index_front();
                }

                Ok(None)
            }

            pressed!(Key::Down) => {
                if !self.cursor_down()? {
                    self.index = self.current_line()?.index_back(self.index)?.into();
                }

                Ok(None)
            }

            pressed!(Key::Left, ctrl) => {
                let corrected = self.current_line()?.correct_index(self.index);
                let index =
                    if let Some(index) = self.current_line()?.index_backward_word(corrected)? {
                        index
                    } else if self.cursor_up()? {
                        self.current_line()?.index_back(corrected.into())?
                    } else {
                        corrected
                    };

                self.index = index.into();

                Ok(None)
            }

            pressed!(Key::Left) => {
                let corrected = self.current_line()?.correct_index(self.index);

                self.index = if let Some(index) = self.current_line()?.index_backward(corrected)? {
                    index
                } else if self.cursor_up()? {
                    self.current_line()?.index_back(corrected.into())?
                } else {
                    corrected
                }
                .into();

                Ok(None)
            }

            pressed!(Key::Right, ctrl) => {
                let corrected = self.current_line()?.correct_index(self.index);

                self.index =
                    if let Some(index) = self.current_line()?.index_forward_word(corrected)? {
                        index.into()
                    } else if self.cursor_down()? {
                        RawIndex::index_front()
                    } else {
                        corrected.into()
                    };

                Ok(None)
            }

            pressed!(Key::Right) => {
                let corrected = self.current_line()?.correct_index(self.index);

                self.index = if let Some(index) = self.current_line()?.index_forward(corrected)? {
                    index.into()
                } else if self.cursor_down()? {
                    RawIndex::index_front()
                } else {
                    corrected.into()
                };

                Ok(None)
            }

            pressed!(Key::Home, ctrl) => {
                self.jump_top()?;
                self.index = RawIndex::index_front();

                Ok(None)
            }

            pressed!(Key::Home) => {
                self.index = RawIndex::index_front();

                Ok(None)
            }

            pressed!(Key::End, ctrl) => {
                self.jump_bottom()?;
                self.index = self.current_line()?.index_back(self.index)?.into();

                Ok(None)
            }

            pressed!(Key::End) => {
                self.index = self.current_line()?.index_back(self.index)?.into();

                Ok(None)
            }

            &pressed!(Key::Char(c)) => {
                let corrected = self.current_line()?.correct_index(self.index);

                self.current_line_mut()?.insert(corrected, c);
                self.index = self
                    .current_line()?
                    .index_forward(corrected)?
                    .unwrap_or(corrected)
                    .into();

                Ok(None)
            }

            pressed!(Key::Enter, shift + ctrl) => {
                self.lines.insert(self.active, Default::default());
                self.fix_lines()?;

                Ok(None)
            }

            pressed!(Key::Enter, ctrl) => {
                self.cursor_down()?;
                self.lines.insert(self.active, Default::default());
                self.fix_lines()?;

                Ok(None)
            }

            pressed!(Key::Enter) => {
                let corrected = self.current_line()?.correct_index(self.index);
                let new_line = self.current_line_mut()?.split_at(corrected)?;

                self.index = RawIndex::index_front();
                if self.cursor_down()? {
                    self.lines.insert(self.active, new_line);
                } else {
                    self.lines.push_back(new_line);
                }
                self.fix_lines()?;

                Ok(None)
            }

            pressed!(Key::Backspace) => {
                if self.index.at_front() {
                    if !self.at_top() {
                        let line = self.lines.remove(self.active).context("active is valid")?;

                        self.fix_lines()?;
                        self.cursor_up()?;
                        self.index = self.current_line()?.index_back(self.index)?.into();
                        self.current_line_mut()?.append(line);
                    }
                } else {
                    let corrected = self.current_line()?.correct_index(self.index);
                    let index = self
                        .current_line()?
                        .index_backward(corrected)?
                        .unwrap_or_default();

                    self.current_line_mut()?.remove(index);
                    self.index = index.into();
                }

                Ok(None)
            }

            pressed!(Key::Delete) => {
                let corrected = self.current_line()?.correct_index(self.index);

                if self.current_line()?.at_back(corrected) {
                    if !self.at_bottom() {
                        let line = self.lines.remove(self.active).context("active is valid")?;

                        self.fix_lines()?;
                        self.current_line_mut()?.prepend(line);
                    }
                } else {
                    self.current_line_mut()?.remove(corrected);
                }
                self.index = corrected.into();

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn update_status(&self) -> Res<()> {
        status::set(Pos::BottomRight, |status| {
            write!(status, "recycle: {}", self.recycle.len())?;

            Ok(())
        })?
    }

    fn view(&self, out: &mut Out, active: bool) -> Res<()> {
        out::anchor(out, self.bounds)?;

        for (i, line) in self.lines.iter().enumerate() {
            if i != self.active {
                line.view(out, self.bounds.width(), None)?;
            }
            out.queue(MoveDown(1))?
                .queue(MoveToColumn(self.bounds.x0))?;
        }

        if self.lines.len() < self.bounds.height().into() {
            out::clear(
                out,
                Bounds {
                    y0: self.bounds.y0 + u16::try_from(self.lines.len())?,
                    ..self.bounds
                },
            )?;
        }

        out.queue(MoveToRow(self.bounds.y0 + u16::try_from(self.active)?))?;
        self.current_line()?.view(
            out,
            self.bounds.width(),
            if active {
                Some(self.current_line()?.correct_index(self.index))
            } else {
                None
            },
        )?;

        Ok(())
    }
}
