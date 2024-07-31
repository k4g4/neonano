use crate::{
    component::{Component, Update},
    core::{Out, Res},
    message::Message,
    utils::list::List,
};
use anyhow::Context;
use crossterm::{style::Print, QueueableCommand};

#[derive(Clone, Default, Debug)]
pub struct Buffer {
    rows: List<Row>,
    _active: usize,
    _anchor: (usize, usize),
}

impl Component for Buffer {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        self.rows
            .iter()
            .try_fold(out, |out, row| row.view(out, width, height))
    }
}

#[derive(Clone, Default, Debug)]
struct Row {
    chars: Vec<char>,
    _active: Option<usize>,
}

impl Component for Row {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, _width: u16, _height: u16) -> Res<&'core mut Out> {
        self.chars
            .iter()
            .try_fold(out, |out, c| out.queue(Print(*c)))
            .context("failed to print row")
    }
}
