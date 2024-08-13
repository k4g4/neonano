use std::cell::RefCell;

#[derive(Default, Debug)]
pub struct Shared {
    debug: String,
}

thread_local! {
    static SHARED: RefCell<Shared> = Default::default();
}

pub fn get<Ret>(f: impl FnOnce(&Shared) -> Ret) -> Ret {
    SHARED.with_borrow(|shared| f(shared))
}

pub fn set<Ret>(f: impl FnOnce(&mut Shared) -> Ret) -> Ret {
    SHARED.with_borrow_mut(|shared| f(shared))
}

#[allow(unused_macros)]
macro_rules! debug {
    () => {
        crate::utils::shared::set(|shared| {
            use std::fmt::Write;
            write!(&mut shared.debug, "line: {}", line!())?;
            crate::core::Res::Ok(())
        })??
    };

    ($($arg:tt)*) => {
        crate::utils::shared::set(|shared| {
            use std::fmt::Write;
            write!(&mut shared.debug, "line: {} msg: ", line!())?;
            shared.debug.write_fmt(format_args!($($arg)*))?;
            crate::core::Res::Ok(())
        })??
    }
}
#[allow(unused_imports)]
pub(crate) use debug;
