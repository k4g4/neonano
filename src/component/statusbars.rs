use crate::{
    component::{Bounds, Component},
    core::Res,
    message::Message,
    utils::out::Out,
};

#[derive(Debug)]
pub struct TopBar;

impl TopBar {
    pub fn new() -> Res<Self> {
        Ok(Self)
    }
}

impl Component for TopBar {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        Ok(None)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct BottomBar;

impl BottomBar {
    pub fn new() -> Res<Self> {
        Ok(Self)
    }
}

impl Component for BottomBar {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        Ok(None)
    }

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()> {
        Ok(())
    }

    fn finally(&mut self) -> Res<()> {
        Ok(())
    }
}
