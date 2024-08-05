use crate::{
    component::{
        statusbars::{BottomBar, TopBar},
        window::Window,
        Bounds, Component,
    },
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::Out,
};

#[derive(Debug)]
pub struct Frame {
    top: TopBar,
    window: Window,
    bottom: BottomBar,
}

impl Frame {
    pub fn new() -> Res<Self> {
        Ok(Self {
            top: TopBar::new()?,
            window: Window::new()?,
            bottom: BottomBar::new()?,
        })
    }
}

impl Component for Frame {
    fn update(&mut self, message: &Message) -> Res<Option<Message>> {
        let update = match message {
            pressed!(Key::Char('c' | 'x'), ctrl) => Some(Message::Quit),
            _ => None,
        };

        if update.is_some() {
            Ok(update)
        } else {
            self.window.update(message)
        }
    }

    fn view(&self, out: &mut Out, bounds: Bounds, _active: bool) -> Res<()> {
        self.window.view(out, bounds, true)?;
        self.top.view(out, bounds, true)?;
        self.bottom.view(out, bounds, true)
    }

    fn finally(&mut self) -> Res<()> {
        self.window.finally()?;
        self.top.finally()?;
        self.bottom.finally()
    }
}
