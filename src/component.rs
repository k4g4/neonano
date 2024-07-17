pub mod state;

use crate::message::Message;
use crate::view::Viewer;

pub trait Component {
    fn update(&mut self, event: Message) -> anyhow::Result<Option<Message>>;

    fn view<'a>(&self, viewer: Viewer<'a>) -> anyhow::Result<Viewer<'a>>;
}
