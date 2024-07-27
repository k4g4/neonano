use crate::component::buffer::Buffer;

#[derive(Copy, Clone, Default, Debug)]
enum Active {
    #[default]
    First,
    Second,
    Third,
}

#[derive(Clone, Default, Debug)]
pub struct Screen {
    columns: [Option<Column>; 3],
    active: Active,
}

#[derive(Clone, Default, Debug)]
struct Column {
    tiles: [Option<Tile>; 3],
    active: Active,
}

#[derive(Clone, Default, Debug)]
struct Tile {
    buffers: Vec<Buffer>,
    active: usize,
}
