pub mod frame;

mod content;
mod line;
mod screen;
mod statusbars;
mod window;

use crate::core::Res;
use crate::message::Message;
use crate::utils::out::{Bounds, Out};

pub trait Component {
    fn update(&mut self, message: &Message) -> Res<Option<Message>>;

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()>;

    fn finally(&mut self) -> Res<()>;
}
