#[derive(Clone)]
pub struct List<T> {
    items: Vec<Node<T>>,
    first: usize,
    last: usize,
}

#[derive(Clone)]
struct Node<T> {
    item: T,
    next: usize,
    prev: usize,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            first: 0,
            last: 0,
        }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_back(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        assert!(List::<()>::new().is_empty())
    }

    #[test]
    fn push_back_one() {
        let mut list = List::new();
        list.push_back(1);

        assert_eq!(list.len(), 1);
        assert_eq!(list.front(), 2);
        assert_eq!(list.back(), 1);
    }
}
