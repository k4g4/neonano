pub mod buffer;
pub mod frame;
pub mod screen;
pub mod statusbars;
pub mod window;

use crate::core::{Out, Res};
use crate::message::Message;

pub struct Update {
    _messages: Vec<Message>,
}

pub trait Component {
    fn update(&mut self, message: &Message) -> Res<Update>;

    fn view<'core>(&self, out: &'core mut Out, width: u16, height: u16) -> Res<&'core mut Out>;
}
