pub mod content;
pub mod frame;
pub mod screen;
pub mod statusbars;
pub mod window;

use crate::core::Res;
use crate::message::Message;
use crate::utils::out::{Bounds, Out};

pub trait Component {
    fn update(&mut self, message: &Message) -> Res<Option<Message>>;

    fn view(&self, out: &mut Out, bounds: Bounds, active: bool) -> Res<()>;

    fn finally(&mut self) -> Res<()>;
}
