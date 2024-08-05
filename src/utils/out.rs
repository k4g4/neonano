use crate::core::Res;
use crossterm::{
    cursor::{MoveDown, MoveLeft, RestorePosition, SavePosition},
    style::Print,
    QueueableCommand,
};

pub type Out = std::io::StdoutLock<'static>;

#[derive(Copy, Clone, Debug)]
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
