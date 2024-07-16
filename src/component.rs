pub mod state;

use crate::message::Message;
use crossterm::QueueableCommand;

pub trait Component {
    fn update(&mut self, event: Message) -> anyhow::Result<Option<Message>>;

    fn view(&self, output: &mut impl QueueableCommand) -> anyhow::Result<()>;
}
