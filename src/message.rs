pub enum Message {
    Event(crossterm::event::Event),
    Quit,
}
