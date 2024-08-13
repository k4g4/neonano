use crate::{
    component::window::Window,
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::{self, Bounds, Out},
};
use crossterm::{cursor::MoveRight, queue, style::Print};

#[derive(Debug)]
pub struct Frame {
    top: StatusBar,
    bottom: StatusBar,
    window: Window,
}

impl Frame {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let [top_bar_bounds, rest] = bounds.hsplit(1);
        let [window_bounds, bottom_bar_bounds] = rest.hsplit(bounds.y1 - 1);
        let window = Window::new(window_bounds)?;

        Ok(Self {
            top: StatusBar::new(top_bar_bounds, StatusLine::top(), &window)?,
            bottom: StatusBar::new(bottom_bar_bounds, StatusLine::bottom(), &window)?,
            window,
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
            let update = self.window.update(message)?;
            self.top.update(&self.window)?;
            self.bottom.update(&self.window)?;

            Ok(update)
        }
    }

    pub fn view(&self, out: &mut Out) -> Res {
        self.top.view(out)?;
        self.bottom.view(out)?;
        self.window.view(out)
    }
}

#[derive(Debug)]
pub enum StatusLine {
    Top(String, String, String),
    Bottom(String, String, String),
}

impl StatusLine {
    fn top() -> Self {
        Self::Top("".into(), "".into(), "".into())
    }

    fn bottom() -> Self {
        Self::Bottom("".into(), "".into(), "".into())
    }

    fn clear(&mut self) {
        let (Self::Top(left, middle, right) | Self::Bottom(left, middle, right)) = self;
        left.clear();
        middle.clear();
        right.clear();
    }

    fn unwrap(&self) -> [&str; 3] {
        let (Self::Top(left, middle, right) | Self::Bottom(left, middle, right)) = self;
        [left, middle, right]
    }
}

#[derive(Debug)]
struct StatusBar {
    bounds: Bounds,
    line: StatusLine,
}

impl StatusBar {
    fn new(bounds: Bounds, mut line: StatusLine, window: &Window) -> Res<Self> {
        window.status(&mut line)?;

        Ok(Self { bounds, line })
    }

    fn update(&mut self, window: &Window) -> Res {
        self.line.clear();
        window.status(&mut self.line)?;

        Ok(())
    }

    fn view(&self, out: &mut Out) -> Res {
        out::with_highlighted(out, |out| {
            out::clear(out, self.bounds)?;

            for (status, bounds) in self.line.unwrap().iter().zip(self.bounds.vsplit3()) {
                out::anchor(out, bounds)?;

                let indent = (bounds.width() - u16::try_from(status.len())?) / 2;

                queue!(out, MoveRight(indent), Print(status))?;
            }

            Ok(out)
        })?;

        Ok(())
    }
}
