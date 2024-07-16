mod component;
mod core;
mod input;
mod message;

use core::Core;

fn main() -> anyhow::Result<()> {
    Core::new()?.run()
}
