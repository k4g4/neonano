use crossterm::{
    cursor::{EnableBlinking, MoveDown, MoveTo, MoveToColumn},
    style::Print,
    QueueableCommand,
};
use std::io::{self, StdoutLock};

pub type Output = StdoutLock<'static>;

pub fn get_output() -> Output {
    io::stdout().lock()
}

#[derive(Debug)]
pub struct Viewer<'core> {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    row: u16,
    output: &'core mut Output,
}

impl<'core> Viewer<'core> {
    pub fn new(output: &'core mut Output, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            row: 0,
            output,
        }
    }

    pub fn crop(self, right: u16, down: u16, width: u16, height: u16) -> anyhow::Result<Self> {
        let x = self.x + right;
        let y = self.y + down;
        let output = self.output.queue(MoveTo(self.x + right, self.y + down))?;

        Ok(Self {
            x,
            y,
            width,
            height,
            row: 0,
            output,
        })
    }

    pub fn write(self, text: &str) -> anyhow::Result<Self> {
        if self.row < self.height {
            self.output
                .queue(Print(text.get(..self.width.into()).unwrap_or(text)))?
                .queue(EnableBlinking)?;
            Ok(Self {
                row: self.row + 1,
                ..self
            })
        } else {
            Ok(self)
        }
    }

    pub fn writeln(self, line: &str) -> anyhow::Result<Self> {
        if self.row < self.height {
            let viewer = self.write(line)?;
            viewer
                .output
                .queue(MoveToColumn(viewer.x))?
                .queue(MoveDown(1))?;
            Ok(viewer)
        } else {
            Ok(self)
        }
    }
}
