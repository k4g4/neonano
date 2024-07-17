use crossterm::{cursor::MoveToNextLine, style::Print, QueueableCommand};
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

    pub fn write_line(self, line: &str) -> anyhow::Result<Self> {
        if self.row < self.height {
            self.output
                .queue(Print(line.get(..self.width.into()).unwrap_or(line)))?
                .queue(MoveToNextLine(1))?;
        }
        Ok(Self {
            row: self.row + 1,
            ..self
        })
    }
}
