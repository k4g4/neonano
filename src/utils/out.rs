use crate::core::Res;
use crossterm::{
    cursor::{MoveDown, MoveLeft, RestorePosition, SavePosition},
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
