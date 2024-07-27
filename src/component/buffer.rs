#[derive(Clone, Default, Debug)]
pub struct Buffer {
    rows: Vec<Row>,
    active: usize,
    anchor: (usize, usize),
}

#[derive(Clone, Default, Debug)]
struct Row {
    chars: Vec<char>,
    active: Option<usize>,
}
