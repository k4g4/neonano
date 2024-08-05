use anyhow::Context;

use crate::{
    component::{content::Content, Bounds, Component},
    core::Res,
    message::Message,
    utils::out::Out,
};

#[derive(Clone, Debug)]
pub struct Screen {
    columns: [Option<Column>; 3],
    active: usize,
}

impl Screen {
    pub fn new() -> Res<Self> {
        Ok(Self {
            columns: [Some(Column::new()?), None, None],
            active: 0,
        })
    }

    fn len(&self) -> usize {
        self.columns.iter().flatten().count()
    }
}

impl Component for Screen {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.columns[self.active]
            .as_mut()
            .context("column should be Some")?
            .update(message)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        let inactive_columns = self
            .columns
            .iter()
            .flatten()
            .enumerate()
            .filter(|&(i, _)| !active || i != self.active)
            .map(|(_, column)| column);

        for column in inactive_columns {
            column.view(out, bounds, false)?;
        }

        if active {
            self.columns[self.active]
                .as_ref()
                .context("column should be Some")?
                .view(out, bounds, true)?;
        }

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        self.columns
            .iter_mut()
            .flatten()
            .try_for_each(Component::finally)
    }
}

#[derive(Clone, Debug)]
struct Column {
    tiles: [Option<Tile>; 3],
    active: usize,
}

impl Column {
    fn new() -> Res<Self> {
        Ok(Self {
            tiles: [Some(Tile::new()?), None, None],
            active: 0,
        })
    }

    fn len(&self) -> usize {
        self.tiles.iter().flatten().count()
    }
}

impl Component for Column {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.tiles[self.active]
            .as_mut()
            .context("tile should be Some")?
            .update(message)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        let inactive_tiles = self
            .tiles
            .iter()
            .flatten()
            .enumerate()
            .filter(|&(i, _)| !active || i != self.active)
            .map(|(_, tile)| tile);

        for column in inactive_tiles {
            column.view(out, bounds, false)?;
        }

        if active {
            self.tiles[self.active]
                .as_ref()
                .context("tile should be Some")?
                .view(out, bounds, true)?;
        }

        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        self.tiles
            .iter_mut()
            .flatten()
            .try_for_each(Component::finally)
    }
}

#[derive(Clone, Debug)]
struct Tile {
    content: Vec<Content>,
    active: usize,
}

impl Tile {
    fn new() -> Res<Self> {
        Ok(Self {
            content: vec![Content::new()?],
            active: 0,
        })
    }
}

impl Component for Tile {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        self.content[self.active].update(message)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        self.content[self.active].view(out, bounds, active)
    }

    fn finally(&mut self) -> Res<()> {
        self.content.iter_mut().try_for_each(Component::finally)
    }
}
