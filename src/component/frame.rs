use crate::{
    component::{
        statusbars::{BottomBar, TopBar},
        window::Window,
    },
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{Bounds, Out},
};

#[derive(Debug)]
pub struct Frame {
    top: TopBar,
    bottom: BottomBar,
    window: Window,
}

impl Frame {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let (top_bar_bounds, rest) = bounds.hsplit(1);
        let (window_bounds, bottom_bar_bounds) = rest.hsplit(bounds.y1 - 1);

        Ok(Self {
            top: TopBar::new(top_bar_bounds)?,
            bottom: BottomBar::new(bottom_bar_bounds)?,
            window: Window::new(window_bounds)?,
        })
    }

    pub fn update(&mut self, message: &Message) -> Res<Option<Message>> {
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

    pub fn view(&self, out: &mut Out) -> Res<()> {
        self.top.view(out)?;
        self.bottom.view(out)?;
        self.window.view(out)
    }
}
