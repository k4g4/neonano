use crate::component::Component;
use crossterm::{cursor::EnableBlinking, style::Print, QueueableCommand};
use std::{
    io::{self, StdoutLock},
    ops::Add,
};

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

    pub fn hsplit(self, components: &[impl Component]) -> anyhow::Result<Self> {
        let Self { from, to, .. } = self;
        let len = components.len() as u16;
        let section_width = (to.x - from.x) / len;
        let section_starts = (0..len).map(|n| n * section_width);
        let section_ends = section_starts.clone().map(|x| x + section_width);
        let mut iter = components.iter().zip(section_starts.zip(section_ends));

        iter.try_fold(self, |viewer, (component, (start, end))| {
            let (from_x, to_x) = (from.x + start, from.x + end);
            let new_from = Point {
                x: from_x,
                y: viewer.from.y,
            };
            let new_to = Point {
                x: to_x,
                y: viewer.to.y,
            };
            viewer.within(new_from, new_to, component)
        })
    }

    pub fn vsplit(self, components: &[impl Component]) -> anyhow::Result<Self> {
        let Self { from, to, .. } = self;
        let len = components.len() as u16;
        let section_height = (to.y - from.y) / len;
        let section_starts = (0..len).map(|n| n * section_height);
        let section_ends = section_starts.clone().map(|x| x + section_height);
        let mut iter = components.iter().zip(section_starts.zip(section_ends));

        iter.try_fold(self, |viewer, (component, (start, end))| {
            let (from_y, to_y) = (from.y + start, from.y + end);
            let new_from = Point {
                x: viewer.from.x,
                y: from_y,
            };
            let new_to = Point {
                x: viewer.to.x,
                y: to_y,
            };
            viewer.within(new_from, new_to, component)
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
