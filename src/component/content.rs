use crate::{
    component::{Bounds, Component},
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{clear, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{MoveDown, MoveTo, MoveToColumn},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};
use std::{
    collections::VecDeque,
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
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
            }
            Message::Open(path) => {
                *self = Self::Buffer(Buffer::open(path)?);
            }
            _ => {}
        }

        match self {
            Content::Buffer(buffer) => buffer.update(message),
            Content::FilePicker(filepicker) => filepicker.update(message),
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
struct FilePicker {
    paths: Vec<PathBuf>,
    selected: usize,
    refresh: bool,
}

impl FilePicker {
    fn new() -> Res<Self> {
        Ok(Self::open(env::current_dir()?)?)
    }

    fn open(path: impl AsRef<Path>) -> Res<Self> {
        let paths = fs::read_dir(path)?
            .map(|res| res.map(|dir| dir.path()))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            paths,
            selected: 0,
            refresh: true,
        })
    }
}

impl Component for FilePicker {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        let update = match message {
            pressed!(Key::Up) => {
                self.selected = self.selected.saturating_sub(1);
                None
            }

            pressed!(Key::Down) => {
                self.selected = (self.paths.len() - 1).min(self.selected + 1);
                None
            }

            pressed!(Key::Enter) => {
                //
                Some(Message::Open(self.paths[self.selected].clone()))
            }

            _ => None,
        };

        Ok(update)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            clear(out, bounds)?;
        }

        for (i, path) in self.paths.iter().enumerate() {
            let highlighted = active && i == self.selected;

            if highlighted {
                out.queue(SetBackgroundColor(Color::White))?
                    .queue(SetForegroundColor(Color::Black))?;
            }

            out.queue(Print(path.display()))?
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
            .collect::<Result<_, _>>()?;

        Ok(Self {
            above: VecDeque::new(),
            below,
            refresh: true,
        })
    }
}

impl Component for Buffer {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.below
            .front_mut()
            .context("below is never empty")?
            .update(message)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        if self.refresh {
            clear(out, bounds)?;
        }

        let rows = self.above.iter().chain(&self.below);

        for (i, row) in rows.enumerate().take((bounds.y1 - bounds.y0).into()) {
            row.view(out, bounds, active && i == self.above.len())?;
            out.queue(MoveDown(1))?.queue(MoveToColumn(bounds.x0))?;
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
    active: Option<usize>,
}

impl Component for Row {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        Ok(None)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        self.chars
            .iter()
            .try_for_each(|c| out.queue(Print(*c)).map(|_| ()))
            .context("failed to print row")
    }

    fn finally(&mut self) -> Res<()> {
        Ok(())
    }
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        Self {
            chars: value.chars().collect(),
            active: None,
        }
    }
}
