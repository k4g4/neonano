use crate::{
    component::screen::Screen,
    core::Res,
    message::Message,
    utils::out::{Bounds, Out},
};

#[derive(Debug)]
pub struct Window {
    screens: Vec<Screen>,
    active: usize,
}

impl Window {
    pub fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self {
            screens: vec![Screen::new(bounds)?],
            active: 0,
        })
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        let update = match message {
            Message::Input(_) => None,
            _ => None,
        };

        if update.is_some() {
            Ok(update)
        } else {
            self.screens[self.active].update(message)
        }
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        self.screens[self.active].view(out)
    }
}
