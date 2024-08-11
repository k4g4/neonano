use std::cell::RefCell;

#[derive(Default, Debug)]
pub struct Shared {
    pub recycle: usize,
    pub debug: String,
}

thread_local! {
    static SHARED: RefCell<Shared> = Default::default();
}

pub fn get<Ret>(f: impl FnOnce(&Shared) -> Ret) -> Ret {
    SHARED.with_borrow(f)
}

pub fn set<Ret>(f: impl FnOnce(&mut Shared) -> Ret) -> Ret {
    SHARED.with_borrow_mut(f)
}

#[macro_export]
macro_rules! debug {
    () => {
        crate::utils::shared::set(|shared| {
            use std::fmt::Write;
            shared.debug.clear();
            write!(&mut shared.debug, "line: {}", line!());
        })
    };

    ($($arg:tt)*) => {
        crate::utils::shared::set(|shared| {
            use std::fmt::Write;
            shared.debug.clear();
            write!(&mut shared.debug, "line: {} msg: ", line!())?;
            shared.debug.write_fmt(format_args!($($arg)*))?;
            crate::core::Res::Ok(())
        })
    }
}
