use anyhow::Context;

use crate::{
    component::content::Content,
    core::Res,
    message::Message,
    utils::out::{Bounds, Out},
};

#[derive(Clone, Debug)]
pub struct Screen {
    columns: [Option<Column>; 3],
    active: usize,
}

impl Screen {
    pub fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self {
            columns: [Some(Column::new(bounds)?), None, None],
            active: 0,
        })
    }

    fn len(&self) -> usize {
        self.columns.iter().flatten().count()
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.columns[self.active]
            .as_mut()
            .context("column should be Some")?
            .update(message)
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        let inactive_columns = self
            .columns
            .iter()
            .flatten()
            .enumerate()
            .filter(|&(i, _)| i != self.active)
            .map(|(_, column)| column);

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

    fn len(&self) -> usize {
        self.tiles.iter().flatten().count()
    }

    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.tiles[self.active]
            .as_mut()
            .context("tile should be Some")?
            .update(message)
    }

    fn view(&self, out: &mut Out, active: bool) -> Res<()> {
        let inactive_tiles = self
            .tiles
            .iter()
            .flatten()
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

    fn view(&self, out: &mut Out, active: bool) -> Res<()> {
        self.content[self.active].view(out, active)
    }
}
