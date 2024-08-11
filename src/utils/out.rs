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
    pub fn width(self) -> u16 {
        self.x1 - self.x0
    }

    pub fn height(self) -> u16 {
        self.y1 - self.y0
    }

    pub fn hsplit(self, y: u16) -> (Self, Self) {
        (Bounds { y1: y, ..self }, Bounds { y0: y, ..self })
    }

    pub fn vsplit(self, x: u16) -> (Self, Self) {
        (Bounds { x1: x, ..self }, Bounds { x0: x, ..self })
    }

    pub fn hsplit2(self) -> (Self, Self) {
        let mid = self.y0 + self.height() / 2;
        (Bounds { y1: mid, ..self }, Bounds { y0: mid, ..self })
    }

    pub fn vsplit2(self) -> (Self, Self) {
        let mid = self.x0 + self.width() / 2;
        (Bounds { x1: mid, ..self }, Bounds { x0: mid, ..self })
    }

    pub fn hsplit3(self) -> (Self, Self, Self) {
        let third = self.height() / 3;
        let (left, right) = (self.y0 + third, self.y1 - third);
        (
            Bounds { y1: left, ..self },
            Bounds {
                y0: left,
                y1: right,
                ..self
            },
            Bounds { y0: right, ..self },
        )
    }

    pub fn vsplit3(self) -> (Self, Self, Self) {
        let third = self.width() / 3;
        let (above, below) = (self.x0 + third, self.x1 - third);
        (
            Bounds { x1: above, ..self },
            Bounds {
                x0: above,
                x1: below,
                ..self
            },
            Bounds { x0: below, ..self },
        )
    }
}

pub fn anchor(out: &mut Out, Bounds { x0, y0, .. }: Bounds) -> Res<&mut Out> {
    Ok(out.queue(MoveToRow(y0))?.queue(MoveToColumn(x0))?)
}

pub fn clear(out: &mut Out, bounds: Bounds) -> Res<&mut Out> {
    anchor(out, bounds)?;
    out.queue(SavePosition)?;

    for _ in bounds.y0..bounds.y1 {
        for _ in bounds.x0..bounds.x1 {
            out.queue(Print(' '))?;
        }
        out.queue(MoveDown(1))?.queue(MoveLeft(bounds.width()))?;
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
