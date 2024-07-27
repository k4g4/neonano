use crate::component::screen::Screen;

#[derive(Default, Debug)]
pub struct Window {
    screens: Vec<Screen>,
    active: usize,
}
