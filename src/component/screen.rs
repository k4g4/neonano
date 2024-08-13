use crate::{
    component::{content::Content, frame::StatusLine},
    core::Res,
    message::Message,
    utils::out::{self, Bounds, Out},
};
use anyhow::Context;
use crossterm::{cursor::MoveTo, queue, style::Print};

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
