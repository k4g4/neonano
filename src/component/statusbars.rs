use crate::{
    component::{Component, Update},
    core::{Out, Res},
    message::Message,
};

#[derive(Default, Debug)]
pub struct TopBar;

impl Component for TopBar {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, _out: &'core mut Out, _width: u16, _height: u16) -> Res<&'core mut Out> {
        todo!()
    }
}

#[derive(Default, Debug)]
pub struct BottomBar;

impl Component for BottomBar {
    fn update(&mut self, message: &Message) -> Res<Update> {
        match message {
            Message::Event(_) => todo!(),
            Message::Quit => todo!(),
        }
    }

    fn view<'core>(&self, _out: &'core mut Out, _width: u16, _height: u16) -> Res<&'core mut Out> {
        todo!()
    }
}
