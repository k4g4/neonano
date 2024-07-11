mod core;
mod input;
mod state;

use core::Core;

fn main() -> anyhow::Result<()> {
    Core::new()?.run()
}
