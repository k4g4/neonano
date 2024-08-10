mod component;
mod core;
mod message;
mod utils;

use core::Core;
use std::fs::write;

fn main() {
    write(
        "debug.txt",
        match Core::new().and_then(Core::run) {
            Ok(core) => format!("{core:#?}"),
            Err(error) => format!("{error:?}"),
        },
    )
    .unwrap()
}
