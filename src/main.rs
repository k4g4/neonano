mod component;
mod core;
mod input;
mod message;
mod view;

use core::Core;
use std::fs::write;

fn main() {
    write(
        "error",
        if let Err(error) = Core::new().and_then(Core::run) {
            error.to_string()
        } else {
            Default::default()
        },
    )
    .unwrap()
}
