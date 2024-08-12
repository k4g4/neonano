use crate::core::Res;
use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveTo, MoveToColumn, RestorePosition, SavePosition},
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::iter;

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

    pub fn hsplit(self, y: u16) -> [Self; 2] {
        [Bounds { y1: y, ..self }, Bounds { y0: y, ..self }]
    }

    pub fn vsplit(self, x: u16) -> [Self; 2] {
        [Bounds { x1: x, ..self }, Bounds { x0: x, ..self }]
    }

    pub fn hsplit2(self) -> [Self; 2] {
        let mid = self.y0 + self.height() / 2;
        [Bounds { y1: mid, ..self }, Bounds { y0: mid, ..self }]
    }

    pub fn vsplit2(self) -> [Self; 2] {
        let mid = self.x0 + self.width() / 2;
        [Bounds { x1: mid, ..self }, Bounds { x0: mid, ..self }]
    }

    pub fn hsplit3(self) -> [Self; 3] {
        let third = self.height() / 3;
        let (left, right) = (self.y0 + third, self.y1 - third);
        [
            Bounds { y1: left, ..self },
            Bounds {
                y0: left,
                y1: right,
                ..self
            },
            Bounds { y0: right, ..self },
        ]
    }

    pub fn vsplit3(self) -> [Self; 3] {
        let third = self.width() / 3;
        let (above, below) = (self.x0 + third, self.x1 - third);
        [
            Bounds { x1: above, ..self },
            Bounds {
                x0: above,
                x1: below,
                ..self
            },
            Bounds { x0: below, ..self },
        ]
    }
}

pub fn anchor(out: &mut Out, Bounds { x0, y0, .. }: Bounds) -> Res<&mut Out> {
    queue!(out, MoveTo(x0, y0))?;

    Ok(out)
}

pub fn clear(out: &mut Out, bounds: Bounds) -> Res<&mut Out> {
    anchor(out, bounds)?;
    queue!(out, SavePosition)?;

    for _ in bounds.y0..bounds.y1 {
        for _ in bounds.x0..bounds.x1 {
            queue!(out, Print(' '))?;
        }
        queue!(out, MoveDown(1), MoveLeft(bounds.width()))?;
    }

    queue!(out, RestorePosition)?;

    Ok(out)
}

pub fn with_highlighted<'out, F>(out: &'out mut Out, f: F) -> Res<&'out mut Out>
where
    F: FnOnce(&'out mut Out) -> Res<&'out mut Out>,
{
    queue!(
        out,
        SetBackgroundColor(Color::White),
        SetForegroundColor(Color::Black),
    )?;
    let out = f(out)?;
    queue!(out, ResetColor)?;

    Ok(out)
}

pub fn vbar(out: &mut Out, x: u16, down: u16, lefts: u16, rights: u16) -> Res<&mut Out> {
    let multiples = |down, num| {
        let interval = down / num;

        (1..num)
            .flat_map(move |mult| {
                let from = interval * (mult - 1);
                let to = (interval * mult) - 1;

                (from..to).map(|_| false).chain(iter::once(true))
            })
            .chain(iter::repeat(false))
    };
    let left_multiples = multiples(down, lefts);
    let right_multiples = multiples(down, rights);
    let chars = left_multiples
        .zip(right_multiples)
        .map(|left_right| match left_right {
            (true, true) => '┼',
            (true, false) => '┤',
            (false, true) => '├',
            (false, false) => '│',
        })
        .take(down.into());

    for c in chars {
        queue!(out, Print(c), MoveDown(1), MoveToColumn(x))?;
    }

    Ok(out)
}

pub fn hbar(out: &mut Out, right: u16, ups: u16, downs: u16) -> Res<&mut Out> {
    let multiples = |right, num| {
        let interval = right / num;

        (1..num)
            .flat_map(move |mult| {
                let from = interval * (mult - 1);
                let to = (interval * mult) - 1;

                (from..to).map(|_| false).chain(iter::once(true))
            })
            .chain(iter::repeat(false))
    };
    let up_multiples = multiples(right, ups);
    let down_multiples = multiples(right, downs);
    let chars = up_multiples
        .zip(down_multiples)
        .map(|up_down| match up_down {
            (true, true) => '┼',
            (true, false) => '┴',
            (false, true) => '┬',
            (false, false) => '─',
        })
        .take(right.into());

    for c in chars {
        queue!(out, Print(c))?;
    }

    Ok(out)
}
