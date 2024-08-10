use crate::core::Res;
use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveToColumn, MoveToRow, RestorePosition, SavePosition},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    QueueableCommand,
};

pub type Out = std::io::StdoutLock<'static>;

#[derive(Copy, Clone, Default, Debug)]
pub struct Bounds {
    pub x0: u16,
    pub y0: u16,
    pub x1: u16,
    pub y1: u16,
}

impl Bounds {
    pub fn hsplit(self, y: u16) -> (Self, Self) {
        (Bounds { y1: y, ..self }, Bounds { y0: y, ..self })
    }

    pub fn vsplit(self, x: u16) -> (Self, Self) {
        (Bounds { x1: x, ..self }, Bounds { x0: x, ..self })
    }

    pub fn width(self) -> u16 {
        self.x1 - self.x0
    }

    pub fn height(self) -> u16 {
        self.y1 - self.y0
    }
}

pub fn anchor(out: &mut Out, Bounds { x0, y0, .. }: Bounds) -> Res<&mut Out> {
    Ok(out.queue(MoveToRow(y0))?.queue(MoveToColumn(x0))?)
}

pub fn clear(out: &mut Out, Bounds { x0, y0, x1, y1 }: Bounds) -> Res<&mut Out> {
    out.queue(SavePosition)?;
    for _ in y0..y1 {
        for _ in x0..x1 {
            out.queue(Print(' '))?;
        }
        out.queue(MoveDown(1))?.queue(MoveLeft(x1 - x0))?;
    }
    Ok(out.queue(RestorePosition)?)
}

pub fn with_highlighted<'out, F>(out: &'out mut Out, f: F) -> Res<&'out mut Out>
where
    F: FnOnce(&'out mut Out) -> Res<&'out mut Out>,
{
    f(out
        .queue(SetBackgroundColor(Color::White))?
        .queue(SetForegroundColor(Color::Black))?)?
    .queue(ResetColor)
    .map_err(Into::into)
}
