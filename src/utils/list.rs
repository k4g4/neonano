use std::{fmt, mem};

#[derive(Clone)]
pub struct List<T> {
    items: Vec<Node<T>>,
    front: usize,
    back: usize,
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
            front: 0,
            back: 0,
        }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push_back(&mut self, item: T) {
        self.items.push(Node {
            item,
            next: 0,
            prev: 0,
        });
        let new_back = self.len() - 1;
        self.items[self.back].next = new_back;
        self.items[new_back].prev = self.back;
        self.back = new_back;
    }

    pub fn push_front(&mut self, item: T) {
        self.items.push(Node {
            item,
            next: 0,
            prev: 0,
        });
        let new_front = self.len() - 1;
        self.items[self.front].prev = new_front;
        self.items[new_front].next = self.front;
        self.front = new_front;
    }

    pub fn front(&self) -> Option<&T> {
        self.items.get(self.front).map(|node| &node.item)
    }

    pub fn back(&self) -> Option<&T> {
        self.items.get(self.back).map(|node| &node.item)
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.into_iter()
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().fold(List::new(), |mut list, item| {
            list.push_back(item);
            list
        })
    }
}

pub struct IntoIter<T>(List<T>, Option<usize>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let IntoIter(list, at) = self;

        if list.is_empty() {
            None
        } else {
            if let Some(i) = at {
                let Node { item, next, .. } = &list.items[*i];
                *at = if *i == list.back { None } else { Some(*next) };

                // SAFETY: the iterator moves on to the next item and never visits
                // this one again. When dropped, the inner list's items are forgotten
                // to prevent double-drop.
                Some(unsafe { (item as *const T).read() })
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.0.len()))
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        self.0.items.drain(..).for_each(mem::forget);
    }
}

impl<T> IntoIterator for List<T> {
    type IntoIter = IntoIter<T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        let front = self.front;
        IntoIter(self, Some(front))
    }
}

pub struct Iter<'list, T>(&'list List<T>, Option<usize>);

impl<'list, T> Iterator for Iter<'list, T> {
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        let Iter(list, at) = self;

        if list.is_empty() {
            None
        } else {
            if let Some(i) = at {
                let Node { item, next, .. } = &list.items[*i];
                *at = if *i == list.back { None } else { Some(*next) };

                Some(&item)
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.0.len()))
    }
}

impl<'list, T> IntoIterator for &'list List<T> {
    type IntoIter = Iter<'list, T>;
    type Item = &'list T;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self, Some(self.front))
    }
}

pub struct IterMut<'list, T>(Option<&'list mut List<T>>, Option<usize>);

impl<'list, T> Iterator for IterMut<'list, T> {
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let IterMut(list, ref mut at) = self;
        let list = list.take()?;

        if list.is_empty() {
            None
        } else {
            if let Some(i) = at {
                let Node { item, next, .. } = &mut list.items[*i];
                *at = if *i == list.back { None } else { Some(*next) };

                // SAFETY: since 'at' now points to the next item, this item won't be aliased again
                // by this iterator. Since it lives for 'list, there is no way to get another reference
                // to it until this returned reference is dead.
                let item = unsafe { &mut *(item as *mut _) };
                self.0 = Some(list);
                Some(item)
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.0.as_ref().map(|list| list.len()))
    }
}

impl<'list, T> IntoIterator for &'list mut List<T> {
    type IntoIter = IterMut<'list, T>;
    type Item = &'list mut T;

    fn into_iter(self) -> Self::IntoIter {
        let front = self.front;
        IterMut(Some(self), Some(front))
    }
}

impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
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
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&1));
    }

    #[test]
    fn push_front_one() {
        let mut list = List::new();
        list.push_front(1);

        assert_eq!(list.len(), 1);
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&1));
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push_back(2);
        list.push_back(3);
        list.push_front(1);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push_back(2);
        list.push_back(3);
        list.push_front(1);

        for item in list.iter_mut() {
            *item += 1;
        }

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn debug() {
        let mut list = List::new();
        list.push_back(2);
        list.push_back(3);
        list.push_front(1);

        let debug = format!("{list:?}");
        assert_eq!(debug, "[1, 2, 3]");
    }

    #[test]
    fn from_iter() {
        let list: List<_> = (0..10).filter(|item| *item != 5).collect();

        let debug = format!("{list:?}");
        assert_eq!("[0, 1, 2, 3, 4, 6, 7, 8, 9]", debug);
    }
}
