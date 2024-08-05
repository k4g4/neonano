use crate::{
    component::{screen::Screen, Bounds, Component},
    core::Res,
    message::Message,
    utils::out::Out,
};

#[derive(Debug)]
pub struct Window {
    screens: Vec<Screen>,
    active: usize,
}

impl Window {
    pub fn new() -> Res<Self> {
        Ok(Self {
            screens: vec![Screen::new()?],
            active: 0,
        })
    }
}

impl Component for Window {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
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

    fn view(&self, out: &mut Out, bounds: Bounds, _active: bool) -> Res<()> {
        self.screens[self.active].view(out, bounds, true)
    }

    fn finally(&mut self) -> Res<()> {
        self.screens
            .iter_mut()
            .try_for_each(|screen| screen.finally())
    }
}
