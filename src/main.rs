mod component;
mod core;
mod message;
mod utils;

use core::Core;
use std::fs::write;

fn main() {
    write(
        "error",
        if let Err(error) = Core::new().and_then(Core::run) {
            format!("{error:?}")
        } else {
            "".into()
        },
    )
    .unwrap()
}
