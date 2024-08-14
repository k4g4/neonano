use crate::{
    component::frame::StatusLine,
    core::Res,
    message::{Key, Message},
    pressed,
    utils::out::{self, Bounds, Out},
};
use anyhow::Context;
use crossterm::{
    cursor::{Hide, MoveDown, MoveToColumn},
    queue,
    style::{self, Color, Print, PrintStyledContent, Stylize},
};
use std::{
    env,
    fmt::Write,
    fs::{self, FileType},
    path::PathBuf,
};

const DIR_ICON: char = '📂';
const FILE_ICON: char = '📄';

#[derive(Clone, Debug)]
pub struct FilePickerEntry {
    path: PathBuf,
    file_type: FileType,
}

#[derive(Clone, Debug)]
pub struct FilePicker {
    entries: Vec<FilePickerEntry>,
    selected: usize,
    history: Vec<PathBuf>,
    bounds: Bounds,
}

impl FilePicker {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let mut filepicker = Self {
            entries: vec![],
            selected: 0,
            history: vec![env::current_dir()?],
            bounds,
        };
        filepicker.open()?;

        Ok(filepicker)
    }

    pub fn open(&mut self) -> Res {
        self.entries = fs::read_dir(self.history.last().context("history never empty")?)?
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

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
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

    pub fn status(&self, statuses: &mut StatusLine) -> Res {
        match statuses {
            StatusLine::Top(left, middle, right) => {
                write!(left, "Filepicker Top Left")?;
                write!(middle, "Filepicker Top")?;
                write!(right, "Filepicker Top Right")?;

                Ok(())
            }
            StatusLine::Bottom(left, middle, right) => {
                write!(left, "Filepicker Bottom Left")?;
                write!(middle, "Filepicker Bottom")?;
                write!(right, "Filepicker Bottom Right")?;
                Ok(())
            }
        }
    }

    pub fn view(&self, out: &mut Out, active: bool) -> Res {
        queue!(out, Hide)?;
        out::anchor(out, self.bounds)?;

        for (i, dir) in self.entries.iter().enumerate() {
            let highlight = active && i == self.selected;

            queue!(
                out,
                Print(format_args!("{:<1$}", ' ', self.bounds.width().into())),
                MoveToColumn(self.bounds.x0),
                PrintStyledContent(
                    style::style(format_args!(
                        "{} {}",
                        if dir.file_type.is_dir() {
                            DIR_ICON
                        } else {
                            FILE_ICON
                        },
                        dir.path.display()
                    ))
                    .with(if highlight {
                        Color::Black
                    } else {
                        Color::White
                    })
                    .on(if highlight {
                        Color::White
                    } else {
                        Color::Reset
                    })
                ),
                MoveDown(1),
                MoveToColumn(self.bounds.x0),
            )?;
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
