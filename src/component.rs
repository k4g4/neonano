pub mod state;

use crate::message::Message;
use crate::view::Viewer;

pub trait Component {
    fn update(&mut self, event: Message) -> anyhow::Result<Option<Message>>;

    fn view<'core>(&self, viewer: Viewer<'core>) -> anyhow::Result<Viewer<'core>>;
}
