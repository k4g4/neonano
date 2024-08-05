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
    bottom: BottomBar,
    window: Window,
}

impl Frame {
    pub fn new() -> Res<Self> {
        Ok(Self {
            top: TopBar::new()?,
            bottom: BottomBar::new()?,
            window: Window::new()?,
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
        self.top.view(out, bounds, true)?;
        self.bottom.view(out, bounds, true)?;
        self.window.view(out, bounds, true)
    }

    fn finally(&mut self) -> Res<()> {
        self.top.finally()?;
        self.bottom.finally()?;
        self.window.finally()
    }
}
