#[derive(Debug)]
pub enum Message {
    Event(crossterm::event::Event),
    Quit,
}
