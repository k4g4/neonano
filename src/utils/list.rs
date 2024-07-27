#[derive(Clone, Default, Debug)]
pub struct List<T>(Vec<Node<T>>);

struct Node<T> {
    item: T,
    next: usize,
    prev: usize,
}

impl List {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

mod tests {
    #[test]
    fn new_is_empty() {
        assert!(List::new().is_empty())
    }

    #[test]
    fn push_back_one() {
        let mut list = List::new();
        list.push_back(1);

        assert_eq!(list.len(), 1);
        assert_eq!(list.front(), 1);
        assert_eq!(list.back(), 1);
    }
}
