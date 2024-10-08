use crate::{
    component::{
        frame::StatusLine,
        line::{Line, RawIndex},
    },
    core::Res,
    message::{Input, Key, Message},
    pressed,
    utils::out::{self, Bounds, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{MoveDown, MoveToColumn, MoveToRow},
    queue,
    style::{self, Print, PrintStyledContent, Stylize},
};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

const SCROLL_GRACE: usize = 3;
const SCROLL_DIST: usize = 5;

#[derive(Clone, Debug)]
pub struct Portal {
    lines: VecDeque<Line>,
    above: String,
    below: String,
    active: usize,
    index: RawIndex,
    offset: usize,
    line_num_width: u16,
    bounds: Bounds,
    recycle: Vec<Line>,
}

impl Portal {
    pub fn open(path: impl AsRef<Path>, bounds: Bounds) -> Res<Self> {
        let height = bounds.height().into();
        let file = BufReader::new(File::open(path)?);
        let mut lines = file.lines().collect::<Result<Vec<_>, _>>()?;
        let line_num_width = 3.max(format!("{}", lines.len()).len().try_into()?);
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
            offset: 0,
            line_num_width,
            bounds,
            recycle: vec![],
        })
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
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
            self.offset += 1;

            let new_line_num_width = match self.offset {
                1_000 => 4,
                10_000 => 5,
                100_000 => 6,
                1_000_000 => 7,
                10_000_000 => 8,
                _ => 0,
            };
            self.line_num_width = self.line_num_width.max(new_line_num_width);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scroll_up(&mut self) -> Res<bool> {
        if let Some(line_from_above) = self.take_from_above()? {
            self.lines.push_front(line_from_above);
            self.fix_lines()?;
            self.offset -= 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn cursor_down(&mut self) -> Res<bool> {
        if self.active < self.lines.len() - SCROLL_GRACE {
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
        if self.active > SCROLL_GRACE {
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

    fn jump_top(&mut self) -> Res {
        while self.cursor_up()? {}

        Ok(())
    }

    fn jump_bottom(&mut self) -> Res {
        while self.cursor_down()? {}

        Ok(())
    }

    fn fix_lines(&mut self) -> Res {
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

    fn type_char(&mut self, c: char) -> Res {
        let corrected = self.current_line()?.correct_index(self.index);

        self.current_line_mut()?.insert(corrected, c);
        self.index = self
            .current_line()?
            .index_forward(corrected)?
            .unwrap_or(corrected)
            .into();

        Ok(())
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
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
                self.type_char(c)?;

                Ok(None)
            }

            pressed!(Key::Tab) => {
                self.type_char('\t')?;

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
                    self.cursor_up()?;
                } else {
                    self.lines.push_back(new_line);
                }
                self.fix_lines()?;
                self.cursor_down()?;

                Ok(None)
            }

            pressed!(Key::Backspace, ctrl) => {
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
                        .index_backward_word(corrected)?
                        .unwrap_or_default();

                    self.current_line_mut()?.remove_range(index, corrected);
                    self.index = index.into();
                }

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

            pressed!(Key::Delete, ctrl) => {
                let corrected = self.current_line()?.correct_index(self.index);

                if self.current_line()?.at_back(corrected) {
                    if !self.at_bottom() {
                        let line = self.lines.remove(self.active).context("active is valid")?;

                        self.fix_lines()?;
                        self.current_line_mut()?.prepend(line);
                    }
                } else {
                    let index =
                        if let Some(index) = self.current_line()?.index_forward_word(corrected)? {
                            index
                        } else {
                            self.current_line()?.index_back(corrected.into())?
                        };
                    self.current_line_mut()?.remove_range(corrected, index);
                }
                self.index = corrected.into();

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

            pressed!(Key::PageDown) => {
                for _ in 0..self.lines.len() / 2 {
                    self.scroll_down()?;
                }

                Ok(None)
            }

            pressed!(Key::PageUp) => {
                for _ in 0..self.lines.len() / 2 {
                    self.scroll_up()?;
                }

                Ok(None)
            }

            Message::Input(Input::ScrollDown) => {
                for _ in 0..SCROLL_DIST {
                    self.scroll_down()?;
                }

                Ok(None)
            }

            Message::Input(Input::ScrollUp) => {
                for _ in 0..SCROLL_DIST {
                    self.scroll_up()?;
                }

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    pub fn status(&self, statuses: &mut StatusLine) -> Res {
        match statuses {
            StatusLine::Top(left, middle, right) => {
                write!(left, "Buffer Top Left")?;
                write!(middle, "Buffer Top")?;
                write!(right, "Buffer Top Right")?;

                Ok(())
            }
            StatusLine::Bottom(left, middle, right) => {
                write!(left, "Buffer Bottom Left")?;
                write!(middle, "Buffer Bottom")?;
                write!(right, "Buffer Bottom Right")?;
                Ok(())
            }
        }
    }

    pub fn view(&self, out: &mut Out, active: bool) -> Res {
        out::anchor(out, self.bounds)?;

        let num_width = usize::from(self.line_num_width);

        for (i, line) in self.lines.iter().enumerate() {
            if i != self.active {
                queue!(
                    out,
                    PrintStyledContent(
                        style::style(format_args!("{:num_width$} ", self.offset + i))
                            .with(style::Color::DarkGrey)
                    ),
                )?;
                line.view(
                    out,
                    self.bounds.x0 + self.line_num_width + 1,
                    self.bounds.x1,
                    None,
                )?;
            }

            queue!(out, MoveDown(1), MoveToColumn(self.bounds.x0))?;
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

        let row = self.bounds.y0 + u16::try_from(self.active)?;
        queue!(
            out,
            MoveToRow(row),
            Print(format_args!("{:num_width$} ", self.offset + self.active)),
        )?;
        self.current_line()?.view(
            out,
            self.bounds.x0 + self.line_num_width + 1,
            self.bounds.x1,
            if active {
                Some(self.current_line()?.correct_index(self.index))
            } else {
                None
            },
        )?;

        Ok(())
    }
}
