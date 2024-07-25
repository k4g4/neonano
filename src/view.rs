use crate::component::Component;
use crossterm::{cursor::EnableBlinking, style::Print, QueueableCommand};
use std::{
    io::{self, StdoutLock},
    ops::Add,
};

pub type Output = StdoutLock<'static>;

pub fn get_output() -> Output {
    io::stdout().lock()
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug)]
pub struct Viewer<'core> {
    output: &'core mut Output,
    from: Point,
    to: Point,
}

impl<'core> Viewer<'core> {
    pub fn new(output: &'core mut Output, from: Point, to: Point) -> Self {
        Self { from, to, output }
    }

    pub fn within(
        self,
        new_from: Point,
        new_to: Point,
        component: &impl Component,
    ) -> anyhow::Result<Self> {
        let Self { from, to, .. } = self;

        let viewer = component.view(Self {
            from: from + new_from,
            to: to + new_to,
            ..self
        })?;

        Ok(Self { from, to, ..viewer })
    }

    pub fn split(self, components: &[impl Component]) -> anyhow::Result<Self> {
        let Self { from, to, .. } = self;

        components.iter().try_fold(self, |viewer, component| {
            //
            todo!()
        })
    }

    pub fn write(self, text: &str) -> anyhow::Result<Self> {
        let width = (self.to.x - self.from.x) as usize;
        Ok(Self {
            output: self
                .output
                .queue(Print(text.get(..width).unwrap_or(text)))?
                .queue(EnableBlinking)?,
            ..self
        })
    }
}
