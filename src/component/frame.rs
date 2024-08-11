use crate::{
    component::window::Window,
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::{
        out::{self, Bounds, Out},
        shared,
    },
};
use crossterm::{style::Print, QueueableCommand};
use std::{fmt::Write, marker::PhantomData};

#[derive(Debug)]
pub struct Frame {
    top: StatusBar<Top>,
    bottom: StatusBar<Bottom>,
    window: Window,
}

impl Frame {
    pub fn new(bounds: Bounds) -> Res<Self> {
        let (top_bar_bounds, rest) = bounds.hsplit(1);
        let (window_bounds, bottom_bar_bounds) = rest.hsplit(bounds.y1 - 1);

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

            self.top.update(message)?;
            self.bottom.update(message)?;

            Ok(update)
        }
    }

    pub fn view(&self, out: &mut Out) -> Res<()> {
        self.top.view(out)?;
        self.bottom.view(out)?;
        self.window.view(out)
    }
}

trait Status {
    fn left(buf: &mut impl Write) -> Res<()>;
    fn middle(buf: &mut impl Write) -> Res<()>;
    fn right(buf: &mut impl Write) -> Res<()>;
}

#[derive(Debug)]
struct Top;

#[derive(Debug)]
struct Bottom;

#[derive(Debug)]
struct StatusBar<S> {
    bounds: Bounds,
    left: String,
    middle: String,
    right: String,
    status: PhantomData<S>,
}

impl<S: Status> StatusBar<S> {
    fn new(bounds: Bounds) -> Res<Self> {
        let mut left = String::new();
        S::left(&mut left)?;

        let mut middle = String::new();
        S::middle(&mut middle)?;

        let mut right = String::new();
        S::right(&mut right)?;

        Ok(Self {
            bounds,
            left,
            middle,
            right,
            status: PhantomData,
        })
    }

    fn update(&mut self, _message: &Message) -> Res<()> {
        self.left.clear();
        S::left(&mut self.left)?;
        self.middle.clear();
        S::middle(&mut self.middle)?;
        self.right.clear();
        S::right(&mut self.right)?;

        Ok(())
    }

    fn view(&self, out: &mut Out) -> Res<()> {
        out::with_highlighted(out, |out| {
            let (left_bounds, middle_bounds, right_bounds) = self.bounds.vsplit3();

            fn write_status<'out>(
                out: &'out mut Out,
                bounds: Bounds,
                status: &str,
            ) -> Res<&'out mut Out> {
                out::anchor(out, bounds)?;
                out.queue(Print(status))?;
                Ok(out)
            }

            out::clear(out, self.bounds)?;
            write_status(out, left_bounds, &self.left)?;
            write_status(out, middle_bounds, &self.middle)?;
            write_status(out, right_bounds, &self.right)?;

            Ok(out)
        })?;

        Ok(())
    }
}

impl Status for Top {
    fn left(status: &mut impl Write) -> Res<()> {
        status.write_str("foo")?;

        Ok(())
    }

    fn middle(status: &mut impl Write) -> Res<()> {
        status.write_str("bar")?;

        Ok(())
    }

    fn right(status: &mut impl Write) -> Res<()> {
        status.write_str("baz")?;

        Ok(())
    }
}

impl Status for Bottom {
    fn left(status: &mut impl Write) -> Res<()> {
        shared::get(|shared| {
            status.write_str(&shared.debug)?;

            Ok(())
        })
    }

    fn middle(_status: &mut impl Write) -> Res<()> {
        Ok(())
    }

    fn right(status: &mut impl Write) -> Res<()> {
        shared::get(|shared| {
            write!(status, "recycle: {}", shared.recycle)?;

            Ok(())
        })
    }
}
