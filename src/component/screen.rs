use crate::{
    component::{filepicker::FilePicker, frame::StatusLine, portal::Portal},
    core::Res,
    message::{Key, Message},
    pressed,
    utils::out::{self, Bounds, Out},
};
use anyhow::Context;
use crossterm::{cursor::MoveTo, queue, style::Print};
use std::io::{self, ErrorKind};

#[derive(Clone, Debug)]
pub struct Screen {
    columns: [Option<Column>; 3],
    active: usize,
    bounds: Bounds,
}

impl Screen {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let bordered = Bounds {
            x0: bounds.x0 + 1,
            y0: bounds.y0 + 1,
            x1: bounds.x1 - 1,
            y1: bounds.y1 - 1,
        };

        Ok(Self {
            columns: [Some(Column::new(bordered)?), None, None],
            active: 0,
            bounds,
        })
    }

    fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().flatten()
    }

    fn len(&self) -> usize {
        self.columns().count()
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.columns[self.active]
            .as_mut()
            .context("column should be Some")?
            .update(message)
    }

    pub fn status(&self, statuses: &mut StatusLine) -> Res {
        self.columns[self.active]
            .as_ref()
            .context("column should be Some")?
            .status(statuses)
    }

    pub fn view(&self, out: &mut Out) -> Res {
        let columns: u16 = self.len().try_into()?;
        let left_tiles: u16 = self
            .columns()
            .next()
            .context("columns never empty")?
            .len()
            .try_into()?;
        let right_tiles: u16 = self
            .columns()
            .last()
            .context("columns never empty")?
            .len()
            .try_into()?;

        out::anchor(out, self.bounds)?;
        out::vbar(out, self.bounds.x0, self.bounds.height(), 1, left_tiles)?;
        queue!(out, MoveTo(self.bounds.x1, self.bounds.y0))?;
        out::vbar(out, self.bounds.x1, self.bounds.height(), right_tiles, 1)?;
        out::anchor(out, self.bounds)?;
        out::hbar(out, self.bounds.width(), 1, columns)?;
        queue!(out, MoveTo(self.bounds.x0, self.bounds.y1 - 1))?;
        out::hbar(out, self.bounds.width(), columns, 1)?;
        out::anchor(out, self.bounds)?;
        queue!(
            out,
            Print('┌'),
            MoveTo(self.bounds.x0, self.bounds.y1 - 1),
            Print('└'),
            MoveTo(self.bounds.x1 - 1, self.bounds.y1 - 1),
            Print('┘'),
            MoveTo(self.bounds.x1 - 1, self.bounds.y0),
            Print('┐'),
        )?;

        let inactive_columns = self
            .columns()
            .enumerate()
            .filter_map(|(i, column)| (i != self.active).then(|| column));

        for column in inactive_columns {
            column.view(out, false)?;
        }

        self.columns[self.active]
            .as_ref()
            .context("column should be Some")?
            .view(out, true)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Column {
    tiles: [Option<Tile>; 3],
    active: usize,
}

impl Column {
    fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self {
            tiles: [Some(Tile::new(bounds)?), None, None],
            active: 0,
        })
    }

    fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().flatten()
    }

    fn len(&self) -> usize {
        self.tiles().count()
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.tiles[self.active]
            .as_mut()
            .context("tile should be Some")?
            .update(message)
    }

    pub fn status(&self, statuses: &mut StatusLine) -> Res {
        self.tiles[self.active]
            .as_ref()
            .context("tile should be Some")?
            .status(statuses)
    }

    fn view(&self, out: &mut Out, active: bool) -> Res {
        let inactive_tiles = self
            .tiles()
            .enumerate()
            .filter(|&(i, _)| !active || i != self.active)
            .map(|(_, tile)| tile);

        for column in inactive_tiles {
            column.view(out, false)?;
        }

        if active {
            self.tiles[self.active]
                .as_ref()
                .context("tile should be Some")?
                .view(out, true)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Tile {
    content: Vec<Content>,
    active: usize,
}

impl Tile {
    fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self {
            content: vec![Content::new(bounds)?],
            active: 0,
        })
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.content[self.active].update(message)
    }

    pub fn status(&self, statuses: &mut StatusLine) -> Res {
        self.content[self.active].status(statuses)
    }

    fn view(&self, out: &mut Out, active: bool) -> Res {
        self.content[self.active].view(out, active)
    }
}

#[derive(Clone, Debug)]
pub enum Content {
    FilePicker(FilePicker),
    Portal(Portal),
}

impl Content {
    fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self::FilePicker(FilePicker::new(bounds)?))
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        match message {
            pressed!(Key::Esc) => match self {
                Self::Portal(portal) => {
                    let filepicker = FilePicker::new(portal.bounds())?;

                    *self = Self::FilePicker(filepicker);

                    Ok(None)
                }

                Self::FilePicker(filepicker) => filepicker.update(message),
            },

            Message::Open(path) => {
                if let Self::FilePicker(filepicker) = self {
                    match Portal::open(path, filepicker.bounds()) {
                        Ok(buffer) => {
                            *self = Self::Portal(buffer);
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
                Content::Portal(buffer) => buffer.update(message),
                Content::FilePicker(filepicker) => filepicker.update(message),
            },
        }
    }

    fn status(&self, statuses: &mut StatusLine) -> Res {
        match self {
            Content::FilePicker(filepicker) => filepicker.status(statuses),
            Content::Portal(buffer) => buffer.status(statuses),
        }
    }

    fn view(&self, out: &mut Out, active: bool) -> Res {
        match self {
            Content::Portal(buffer) => buffer.view(out, active),
            Content::FilePicker(filepicker) => filepicker.view(out, active),
        }
    }
}
