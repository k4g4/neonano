use crate::core::Res;
use anyhow::Context;
use std::cell::RefCell;

const STATUS_LEN: usize = 6;

#[derive(Default, Debug)]
pub struct Shared {
    pub statuses: [String; STATUS_LEN],
}

thread_local! {
    static SHARED: RefCell<Shared> = Default::default();
}

pub mod status {
    use std::fmt::Write;

    use crossterm::style::Attribute;

    use super::*;

    const TITLE: &str = "neonano";

    #[repr(usize)]
    #[derive(PartialEq, Copy, Clone, Debug)]
    pub enum Pos {
        TopLeft,
        Top,
        TopRight,
        BottomLeft,
        Bottom,
        BottomRight,
    }

    pub fn get<Ret>(pos: Pos, f: impl FnOnce(&str) -> Ret) -> Res<Ret> {
        SHARED.with_borrow(|shared| {
            Ok(f(shared
                .statuses
                .get(pos as usize)
                .context("invalid index")?))
        })
    }

    pub fn set<Ret>(pos: Pos, f: impl FnOnce(&mut String) -> Ret) -> Res<Ret> {
        SHARED.with_borrow_mut(|shared| {
            let status = shared
                .statuses
                .get_mut(pos as usize)
                .context("invalid index")?;
            status.clear();
            Ok(f(status))
        })
    }

    pub fn reset_all() -> Res<()> {
        SHARED.with_borrow_mut(|shared| {
            shared.statuses.iter_mut().for_each(String::clear);
        });

        Ok(status::set(Pos::Top, |status| {
            write!(status, "{}{}", Attribute::Italic, TITLE)
        })??)
    }
}

#[allow(unused_macros)]
macro_rules! debug {
    () => {
        crate::utils::shared::status::set(crate::utils::shared::status::Pos::TopRight, |status| {
            use std::fmt::Write;
            write!(status, "line: {}", line!())?;
            crate::core::Res::Ok(())
        })??
    };

    ($($arg:tt)*) => {
        crate::utils::shared::status::set(crate::utils::shared::status::Pos::TopRight, |status| {
            use std::fmt::Write;
            write!(status, "line: {} msg: ", line!())?;
            status.write_fmt(format_args!($($arg)*))?;
            crate::core::Res::Ok(())
        })??
    }
}
#[allow(unused_imports)]
pub(crate) use debug;
