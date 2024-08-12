use crate::{
    component::window::Window,
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::{
        out::{self, Bounds, Out},
        shared::status::{self, Pos},
    },
};
use crossterm::{cursor::MoveRight, queue, style::Print};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Frame {
    top: StatusBar<Top>,
    bottom: StatusBar<Bottom>,
    window: Window,
}

impl Frame {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let [top_bar_bounds, rest] = bounds.hsplit(1);
        let [window_bounds, bottom_bar_bounds] = rest.hsplit(bounds.y1 - 1);

        Ok(Self {
            top: StatusBar::new(top_bar_bounds)?,
            bottom: StatusBar::new(bottom_bar_bounds)?,
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
            let update = self.window.update(message)?;

            Ok(update)
        }
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        self.top.view(out)?;
        self.bottom.view(out)?;
        self.window.view(out)
    }
}

trait Position {
    fn positions() -> [Pos; 3];
}

#[derive(Debug)]
struct Top;

impl Position for Top {
    fn positions() -> [Pos; 3] {
        [Pos::TopLeft, Pos::Top, Pos::TopRight]
    }
}

#[derive(Debug)]
struct Bottom;

impl Position for Bottom {
    fn positions() -> [Pos; 3] {
        [Pos::BottomLeft, Pos::Bottom, Pos::BottomRight]
    }
}

#[derive(Debug)]
struct StatusBar<P> {
    bounds: Bounds,
    status: PhantomData<P>,
}

impl<P: Position> StatusBar<P> {
    fn new(bounds: Bounds) -> Res<Self> {
        Ok(Self {
            bounds,
            status: PhantomData,
        })
    }

    fn view(&self, out: &mut Out) -> Res<()> {
        out::with_highlighted(out, |out| {
            out::clear(out, self.bounds)?;

            for (&pos, bounds) in P::positions().iter().zip(self.bounds.vsplit3()) {
                out::anchor(out, bounds)?;

                status::get(pos, |status| -> Res<_> {
                    let status = status.get(..bounds.width().into()).unwrap_or(status);
                    let indent = (bounds.width() - u16::try_from(status.len())?) / 2;

                    Ok(queue!(out, MoveRight(indent), Print(status))?)
                })??;
            }

            Ok(out)
        })?;

        Ok(())
    }
}
