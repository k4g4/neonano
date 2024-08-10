use crate::{
    component::{
        statusbars::{BottomBar, TopBar},
        window::Window,
        Bounds, Component,
    },
    core::{Res, State, STATE},
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::Out,
};
use crossterm::terminal;

#[derive(Debug)]
pub struct Frame {
    top: TopBar,
    bottom: BottomBar,
    window: Window,
}

impl Frame {
    pub fn new() -> Res<Self> {
        let (width, height) = terminal::size()?;

        STATE.set(State {
            bounds: Bounds {
                x0: 0,
                y0: 0,
                x1: width,
                y1: height,
            },
            ..Default::default()
        });

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
