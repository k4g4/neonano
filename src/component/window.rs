use crate::{
    component::{screen::Screen, Component, Update},
    core::{Out, Res},
    message::Message,
};

#[derive(Default, Debug)]
pub struct Window {
    screens: Vec<Screen>,
    active: usize,
}

impl Component for Window {
    fn update(&mut self, message: &Message) -> Res<Update> {
        todo!()
    }

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out> {
        Ok(out)
    }
}
