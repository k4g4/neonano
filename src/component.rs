pub mod state;

use crate::message::Message;
use crate::view::Viewer;
use state::State;

#[enum_dispatch::enum_dispatch]
pub trait Component {
    fn update(&mut self, message: &Message) -> anyhow::Result<Option<Message>>;

    fn view<'core>(&self, viewer: Viewer<'core>) -> anyhow::Result<Viewer<'core>>;
}

#[enum_dispatch::enum_dispatch(Component)]
#[derive(Debug)]
pub enum ComponentHolder {
    State,
}
