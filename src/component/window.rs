use crate::{
    component::{screen::Screen, Component, Update},
    core::{Out, Res},
    message::Message,
};

#[derive(Default, Debug)]
pub struct Window {
    screens: Vec<Screen>,
    _active: usize,
}

impl Component for Window {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        self.screens
            .iter()
            .try_fold(out, |out, screen| screen.view(out, width, height))
    }
}
