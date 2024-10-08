use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter, Result},
    hash::{Hash, Hasher},
    iter::FusedIterator,
    ops::{Index, IndexMut},
};

use slotmap::{DefaultKey, SlotMap};

pub type Key = DefaultKey;

#[derive(Clone)]
pub struct SlotList<T> {
    map: SlotMap<Key, Node<T>>,
    front: Option<Key>,
    back: Option<Key>,
}

#[derive(Clone)]
struct Node<T> {
    next: Option<Key>,
    prev: Option<Key>,
    item: T,
}

impl<T> Default for SlotList<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            front: None,
            back: None,
        }
    }
}

impl<T> FromIterator<T> for SlotList<T> {
    fn from_iter<IntoIter: IntoIterator<Item = T>>(into_iter: IntoIter) -> Self {
        let iter = into_iter.into_iter();
        let mut prev = None;
        let mut front = None;
        let capacity = match iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, _) => lower,
        };
        let mut map = SlotMap::with_capacity(capacity);

        for item in iter {
            let key = map.insert(Node {
                next: None,
                prev,
                item,
            });
            if front.is_none() {
                front = Some(key);
            }
            if let Some(prev_key) = prev {
                map.get_mut(prev_key).unwrap().next = Some(key);
            }
            prev = Some(key);
        }

        Self {
            map,
            front,
            back: prev,
        }
    }
}

impl<T> Index<Key> for SlotList<T> {
    type Output = T;

    fn index(&self, key: Key) -> &Self::Output {
        &self.map[key].item
    }
}

impl<T> IndexMut<Key> for SlotList<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        &mut self.map[key].item
    }
}

impl<T> SlotList<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: SlotMap::with_capacity(capacity),
            front: None,
            back: None,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&self, key: Key) -> Option<(&T, (Option<Key>, Option<Key>))> {
        self.map
            .get(key)
            .map(|node| (&node.item, (node.prev, node.next)))
    }

    pub fn get_mut(&mut self, key: Key) -> Option<(&mut T, (Option<Key>, Option<Key>))> {
        self.map
            .get_mut(key)
            .map(|node| (&mut node.item, (node.prev, node.next)))
    }

    pub fn next(&self, key: Key) -> Option<Key> {
        self.map[key].next
    }

    pub fn prev(&self, key: Key) -> Option<Key> {
        self.map[key].prev
    }

    pub fn back(&self) -> Option<Key> {
        self.back
    }

    pub fn front(&self) -> Option<Key> {
        self.front
    }

    pub fn insert_next(&mut self, key: Key, item: T) -> Key {
        let next = self.next(key);
        let prev = Some(key);
        let new_key = self.map.insert(Node { next, prev, item });

        self.map[key].next = Some(new_key);
        if let Some(next_key) = next {
            self.map[next_key].prev = Some(new_key);
        } else {
            self.back = Some(new_key);
        }

        new_key
    }

    pub fn insert_prev(&mut self, key: Key, item: T) -> Key {
        let prev = self.prev(key);
        let next = Some(key);
        let new_key = self.map.insert(Node { next, prev, item });

        self.map[key].prev = Some(new_key);
        if let Some(prev_key) = prev {
            self.map[prev_key].next = Some(new_key);
        } else {
            self.front = Some(new_key);
        }

        new_key
    }

    pub fn push_back(&mut self, item: T) -> Key {
        let new_key = self.map.insert(Node {
            next: None,
            prev: self.back,
            item,
        });

        if let Some(back_key) = self.back {
            self.map[back_key].next = Some(new_key);
        }
        self.back = Some(new_key);

        new_key
    }

    pub fn push_front(&mut self, item: T) -> Key {
        let new_key = self.map.insert(Node {
            next: self.front,
            prev: None,
            item,
        });

        if let Some(front_key) = self.front {
            self.map[front_key].prev = Some(new_key);
        }
        self.front = Some(new_key);

        new_key
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.front = None;
        self.back = None;
    }

    pub fn remove(&mut self, key: Key) -> Option<(T, (Option<Key>, Option<Key>))> {
        let node = self.map.remove(key)?;

        if let Some(prev_key) = node.prev {
            self.map[prev_key].next = node.next;
        } else {
            self.front = node.next;
        }
        if let Some(next_key) = node.next {
            self.map[next_key].prev = node.prev;
        } else {
            self.back = node.prev;
        }

        Some((node.item, (node.prev, node.next)))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let (item, _) = self.remove(self.back?)?;

        Some(item)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let (item, _) = self.remove(self.front?)?;

        Some(item)
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.into_iter()
    }

    pub fn extract_if<F>(&mut self, filter: F) -> ExtractIf<T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        let Self { front, back, .. } = *self;

        ExtractIf {
            list: self,
            filter,
            front,
            back,
        }
    }
}

impl<T> Extend<T> for SlotList<T> {
    fn extend<IntoIter: IntoIterator<Item = T>>(&mut self, into_iter: IntoIter) {
        let iter = into_iter.into_iter();
        let additional = match iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, _) => lower,
        };

        self.map.reserve(additional);
        iter.for_each(|item| {
            self.push_back(item);
        });
    }
}

impl<'item, T: Copy> Extend<&'item T> for SlotList<T> {
    fn extend<IntoIter: IntoIterator<Item = &'item T>>(&mut self, into_iter: IntoIter) {
        self.extend(into_iter.into_iter().copied())
    }
}

pub struct IntoIter<T>(SlotList<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> IntoIterator for SlotList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

pub struct Iter<'list, T> {
    list: &'list SlotList<T>,
    front: Option<Key>,
    back: Option<Key>,
}

impl<'list, T> Iterator for Iter<'list, T> {
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            None
        } else {
            let node = &self.list.map[self.front?];

            self.front = node.next;

            Some(&node.item)
        }
    }
}

impl<'list, T> DoubleEndedIterator for Iter<'list, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            None
        } else {
            let node = &self.list.map[self.back?];

            self.back = node.prev;

            Some(&node.item)
        }
    }
}

impl<'list, T> FusedIterator for Iter<'list, T> {}

impl<'list, T> IntoIterator for &'list SlotList<T> {
    type Item = &'list T;

    type IntoIter = Iter<'list, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            list: self,
            front: self.front,
            back: self.back,
        }
    }
}

pub struct IterMut<'list, T> {
    list: &'list mut SlotList<T>,
    front: Option<Key>,
    back: Option<Key>,
}

impl<'list, T> Iterator for IterMut<'list, T> {
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            None
        } else {
            let node = &mut self.list.map[self.front?];

            self.front = node.next;

            // SAFETY - borrowck limitation. 'list outlives &mut self
            Some(unsafe { &mut *(&mut node.item as *mut _) })
        }
    }
}

impl<'list, T> DoubleEndedIterator for IterMut<'list, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            None
        } else {
            let node = &mut self.list.map[self.back?];

            self.back = node.prev;

            Some(unsafe { &mut *(&mut node.item as *mut _) })
        }
    }
}

impl<'list, T> FusedIterator for IterMut<'list, T> {}

impl<'list, T> IntoIterator for &'list mut SlotList<T> {
    type Item = &'list mut T;

    type IntoIter = IterMut<'list, T>;

    fn into_iter(self) -> Self::IntoIter {
        let SlotList { front, back, .. } = *self;

        IterMut {
            list: self,
            front,
            back,
        }
    }
}

pub struct ExtractIf<'list, T, F> {
    list: &'list mut SlotList<T>,
    filter: F,
    front: Option<Key>,
    back: Option<Key>,
}

impl<'list, T, F> Iterator for ExtractIf<'list, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            None
        } else {
            while !(self.filter)(&mut self.list[self.front?]) {
                self.front = self.list.next(self.front?);
            }
            let (item, (_, next)) = self.list.remove(self.front?)?;
            self.front = next;

            Some(item)
        }
    }
}

impl<'list, T, F> DoubleEndedIterator for ExtractIf<'list, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            None
        } else {
            while !(self.filter)(&mut self.list[self.back?]) {
                self.back = self.list.prev(self.back?);
            }
            let (item, (prev, _)) = self.list.remove(self.back?)?;
            self.back = prev;

            Some(item)
        }
    }
}

impl<'list, T, F> FusedIterator for ExtractIf<'list, T, F> where F: FnMut(&mut T) -> bool {}

impl<T: Debug> Debug for SlotList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for SlotList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other)
    }
}

impl<T: Eq> Eq for SlotList<T> {}

impl<T: PartialOrd> PartialOrd for SlotList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for SlotList<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for SlotList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        assert!(SlotList::<()>::new().is_empty())
    }

    #[test]
    fn from_iter() {
        let list: SlotList<_> = (0..10).filter(|item| *item != 5).collect();
        let debug = format!("{list:?}");
        assert_eq!("[0, 1, 2, 3, 4, 6, 7, 8, 9]", debug);
    }

    #[test]
    fn push_back_one() {
        let mut list = SlotList::new();
        list.push_back(1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[list.front().unwrap()], 1);
        assert_eq!(&list[list.back().unwrap()], &1);
    }

    #[test]
    fn push_front_one() {
        let mut list = SlotList::new();
        list.push_front(1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[list.front().unwrap()], 1);
        assert_eq!(&list[list.back().unwrap()], &1);
    }

    #[test]
    fn key() {
        let mut list = SlotList::new();
        let key = list.push_back(1);
        assert_eq!(list[key], 1);
        assert_eq!(key, list.front().unwrap());
        assert_eq!(key, list.back().unwrap());
    }

    #[test]
    #[should_panic]
    fn invalid_key() {
        let mut list = SlotList::new();
        let key = list.push_back(1);
        list.clear();
        list[key];
    }

    #[test]
    fn inserting() {
        let mut list: SlotList<_> = [1, 3, 4, 6, 7].into_iter().collect();
        let key = list.front().unwrap();
        let key = list.next(key).unwrap();
        let new_key = list.insert_prev(key, 2);
        assert_eq!(list[new_key], 2);
        let key = list.next(key).unwrap();
        let new_key = list.insert_next(key, 5);
        assert_eq!(list[new_key], 5);
        assert_eq!(list, (1..=7).collect());
    }

    #[test]
    fn iter() {
        let mut list = SlotList::new();
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
        let mut list = SlotList::new();
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
    fn rev() {
        let mut list = SlotList::new();
        list.push_back('C');
        list.push_back('B');
        list.push_back('A');
        list.push_front('D');
        list.push_front('E');
        let letters: String = list.iter().rev().collect();
        assert_eq!(letters, "ABCDE");
    }

    #[test]
    fn debug() {
        let mut list = SlotList::new();
        list.push_back(2);
        list.push_back(3);
        list.push_front(1);
        let debug = format!("{list:?}");
        assert_eq!(debug, "[1, 2, 3]");
    }

    #[test]
    fn clone() {
        let list: SlotList<_> = (0..10).filter(|item| *item != 5).collect();
        let list_clone = list.clone();
        assert_eq!(list, list_clone);
    }

    #[test]
    fn ord() {
        let smaller = SlotList::from_iter([1, 2, 3]);
        let mut larger = SlotList::new();
        larger.push_back(2);
        larger.push_back(4);
        larger.push_front(1);
        assert!(smaller < larger);
        assert!(larger > smaller);
    }

    #[test]
    fn extend() {
        let mut list = SlotList::from_iter("the quick brown fox".split_whitespace());
        list.extend("jumps over the lazy dog".split_whitespace());
        let words: Vec<_> = list.iter().copied().collect();
        assert_eq!(
            words.join(" "),
            "the quick brown fox jumps over the lazy dog"
        );
    }

    #[test]
    fn flatten_lists() {
        let mut list = SlotList::from_iter([
            SlotList::from_iter([3, 4, 5]),
            SlotList::from_iter([6, 7, 8]),
        ]);
        list.push_front(SlotList::from_iter([0, 1, 2]));
        let key = list.back().unwrap();
        list[key].push_back(9);
        let flattened: SlotList<_> = list.into_iter().flatten().collect();
        assert_eq!(flattened.len(), 10);
        for (i, item) in flattened.into_iter().enumerate() {
            assert_eq!(i, item);
        }
    }

    #[test]
    fn pop_back() {
        let mut list = SlotList::new();
        assert!(list.pop_back().is_none());
        list.push_back(0);
        assert_eq!(list.pop_back(), Some(0));
        assert!(list.pop_back().is_none());
        list.push_front(1);
        assert_eq!(list.pop_back(), Some(1));
        assert!(list.pop_back().is_none());
        list.extend([2, 3, 4]);
        assert_eq!(list.pop_back(), Some(4));
        list.push_front(1);
        assert_eq!(list.pop_back(), Some(3));
        list.push_back(5);
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert!(list.pop_back().is_none());
    }

    #[test]
    fn pop_front() {
        let mut list = SlotList::new();
        assert!(list.pop_front().is_none());
        list.push_front(0);
        assert_eq!(list.pop_front(), Some(0));
        assert!(list.pop_front().is_none());
        list.push_back(1);
        assert_eq!(list.pop_front(), Some(1));
        assert!(list.pop_front().is_none());
        list.extend([2, 3, 4]);
        assert_eq!(list.pop_front(), Some(2));
        list.push_back(5);
        assert_eq!(list.pop_front(), Some(3));
        list.push_front(1);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(5));
        assert!(list.pop_front().is_none());
    }

    #[test]
    fn zst() {
        #[derive(PartialEq)]
        struct Nothing;
        let mut list = SlotList::from_iter([Nothing, Nothing, Nothing, Nothing]);
        assert_eq!(list.len(), 4);
        list.push_back(Nothing);
        assert_eq!(list.len(), 5);
        assert!(list.iter().all(|nothing| *nothing == Nothing));
        list.clear();
        assert!(list.is_empty());
    }
}
