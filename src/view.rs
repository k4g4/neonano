use crossterm::{
    cursor::{EnableBlinking, MoveDown, MoveLeft},
    style::Print,
    QueueableCommand,
};
use std::io::{stdout, StdoutLock};

pub type Output = StdoutLock<'static>;

pub fn get_output() -> Output {
    stdout().lock()
}

pub struct Viewer<'output> {
    output: &'output mut Output,
    width: u16,
    row: u16,
    height: u16,
}

impl<'output> Viewer<'output> {
    pub fn new(output: &'output mut Output, width: u16, height: u16) -> Self {
        Self {
            output,
            width,
            row: 0,
            height,
        }
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
                .queue(MoveLeft(line.len().try_into()?))?
                .queue(MoveDown(1))?;
            Ok(viewer)
        } else {
            Ok(self)
        }
    }
}
