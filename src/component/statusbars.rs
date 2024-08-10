use crate::{
    core::Res,
    message::Message,
    utils::out::{Bounds, Out},
};

#[derive(Debug)]
pub struct TopBar;

impl TopBar {
    pub fn new(_: Bounds) -> Res<Self> {
        Ok(Self)
    }

    pub fn update(&mut self, message: &Message) -> Res<()> {
        Ok(())
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct BottomBar;

impl BottomBar {
    pub fn new(_: Bounds) -> Res<Self> {
        Ok(Self)
    }

    pub fn update(&mut self, message: &Message) -> Res<()> {
        Ok(())
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        Ok(())
    }
}
