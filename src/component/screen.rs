use crate::{
    component::{buffer::Buffer, Component, Update},
    core::{Out, Res},
    message::Message,
};

#[derive(Copy, Clone, Default, Debug)]
enum Active {
    #[default]
    First,
    _Second,
    _Third,
}

#[derive(Clone, Default, Debug)]
pub struct Screen {
    columns: [Option<Column>; 3],
    _active: Active,
}

impl Component for Screen {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        self.columns
            .iter()
            .flatten()
            .try_fold(out, |out, column| column.view(out, width, height))
    }
}

#[derive(Clone, Default, Debug)]
struct Column {
    tiles: [Option<Tile>; 3],
    _active: Active,
}

impl Component for Column {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        self.tiles
            .iter()
            .flatten()
            .try_fold(out, |out, tile| tile.view(out, width, height))
    }
}

#[derive(Clone, Default, Debug)]
struct Tile {
    buffers: Vec<Buffer>,
    _active: usize,
}

impl Component for Tile {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        self.buffers
            .iter()
            .try_fold(out, |out, buffer| buffer.view(out, width, height))
    }
}
