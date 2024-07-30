use std::{cmp, fmt, hash, iter, mem, ops};

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

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            ..Default::default()
        }
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

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let Node { item, prev, .. } = self.items.swap_remove(self.back);

            if !self.is_empty() {
                let swapped = self.back;
                let Node { next, prev, .. } = self.items[swapped];

                self.items[prev].next = swapped;
                if self.front == self.len() - 1 {
                    self.front = swapped;
                } else {
                    self.items[next].prev = swapped;
                }
            }
            self.back = prev;

            Some(item)
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let Node { item, next, .. } = self.items.swap_remove(self.front);

            if !self.is_empty() {
                let swapped = self.front;
                let Node { next, prev, .. } = self.items[swapped];

                self.items[next].prev = swapped;
                if self.back == self.len() - 1 {
                    self.back = swapped;
                } else {
                    self.items[prev].next = swapped;
                }
            }
            self.front = next;

            Some(item)
        }
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

    pub fn cursor_front(&self) -> Option<Cursor<'_, T>> {
        if self.is_empty() {
            None
        } else {
            Some(Cursor {
                list: self,
                at: self.front,
            })
        }
    }

    pub fn cursor_front_mut(&mut self) -> Option<CursorMut<'_, T>> {
        if self.is_empty() {
            None
        } else {
            let at = self.front;
            Some(CursorMut { list: self, at })
        }
    }

    pub fn cursor_back(&self) -> Option<Cursor<'_, T>> {
        if self.is_empty() {
            None
        } else {
            Some(Cursor {
                list: self,
                at: self.back,
            })
        }
    }

    pub fn cursor_back_mut(&mut self) -> Option<CursorMut<'_, T>> {
        if self.is_empty() {
            None
        } else {
            let at = self.back;
            Some(CursorMut { list: self, at })
        }
    }

    pub fn extract_if<F: FnMut(&mut T) -> bool>(&mut self, filter: F) -> ExtractIf<'_, T, F> {
        ExtractIf {
            cursor: self.cursor_front_mut(),
            filter,
        }
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let capacity = match iter.size_hint() {
            (_, Some(upper)) => upper,
            (lower, _) => lower,
        };

        iter.fold(List::with_capacity(capacity), |mut list, item| {
            list.push_back(item);
            list
        })
    }
}

impl<T> Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|item| self.push_back(item));
    }
}

impl<'item, T: Copy> Extend<&'item T> for List<T> {
    fn extend<I: IntoIterator<Item = &'item T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied())
    }
}

pub enum IntoIter<T> {
    Nonempty {
        list: List<T>,
        forward: usize,
        backward: usize,
    },
    Empty,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let IntoIter::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, next, .. } = &list.items[*forward];

        *forward = *next;

        // SAFETY: the iterator moves on to the next item and never visits
        // this one again. When dropped, the inner list's items are forgotten
        // to prevent double-drop.
        let item = unsafe { (item as *const T).read() };

        if finished {
            *self = IntoIter::Empty;
        }

        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            Some(match self {
                IntoIter::Nonempty { list, .. } => list.len(),
                IntoIter::Empty => 0,
            }),
        )
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let IntoIter::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, prev, .. } = &list.items[*backward];

        *backward = *prev;

        // SAFETY: the iterator moves on to the next item and never visits
        // this one again. When dropped, the inner list's items are forgotten
        // to prevent double-drop.
        let item = unsafe { (item as *const T).read() };

        if finished {
            *self = IntoIter::Empty;
        }

        Some(item)
    }
}

impl<T> iter::FusedIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        let IntoIter::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return;
        };

        while *forward != *backward {
            let node = &list.items[*forward];

            // SAFETY: reading and dropping items that were never returned from
            // next() or next_back(). Now that every item has been read and dropped,
            // mem::forget can be called on the entire list.
            drop(unsafe { (&node.item as *const T).read() });
            *forward = node.next;
        }
        let node = &list.items[*forward];
        drop(unsafe { (&node.item as *const T).read() });

        mem::forget(mem::take(list))
    }
}

impl<T> IntoIterator for List<T> {
    type IntoIter = IntoIter<T>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        if self.is_empty() {
            IntoIter::Empty
        } else {
            let forward = self.front;
            let backward = self.back;

            IntoIter::Nonempty {
                list: self,
                forward,
                backward,
            }
        }
    }
}

pub enum Iter<'list, T> {
    Nonempty {
        list: &'list List<T>,
        forward: usize,
        backward: usize,
    },
    Empty,
}

impl<'list, T> Iterator for Iter<'list, T> {
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        let Iter::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, next, .. } = &list.items[*forward];

        *forward = *next;

        if finished {
            *self = Iter::Empty;
        }
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            Some(match self {
                Iter::Nonempty { list, .. } => list.len(),
                Iter::Empty => 0,
            }),
        )
    }
}

impl<'list, T> DoubleEndedIterator for Iter<'list, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let Iter::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, prev, .. } = &list.items[*backward];

        *backward = *prev;

        if finished {
            *self = Iter::Empty;
        }
        Some(item)
    }
}

impl<'list, T> iter::FusedIterator for Iter<'list, T> {}

impl<'list, T> IntoIterator for &'list List<T> {
    type IntoIter = Iter<'list, T>;
    type Item = &'list T;

    fn into_iter(self) -> Self::IntoIter {
        if self.is_empty() {
            Iter::Empty
        } else {
            Iter::Nonempty {
                list: self,
                forward: self.front,
                backward: self.back,
            }
        }
    }
}

pub enum IterMut<'list, T> {
    Nonempty {
        list: &'list mut List<T>,
        forward: usize,
        backward: usize,
    },
    Empty,
}

impl<'list, T> Iterator for IterMut<'list, T> {
    type Item = &'list mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let IterMut::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, next, .. } = &mut list.items[*forward];

        *forward = *next;

        // SAFETY: since 'forward' now points to the next item, this item won't be aliased again
        // by this iterator. Since it lives for 'list, there is no way to get another reference
        // to it until this returned reference is dead.
        let item_extended = unsafe { &mut *(item as *mut _) };

        if finished {
            *self = IterMut::Empty;
        }
        Some(item_extended)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            Some(match self {
                IterMut::Nonempty { list, .. } => list.len(),
                IterMut::Empty => 0,
            }),
        )
    }
}

impl<'list, T> DoubleEndedIterator for IterMut<'list, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let IterMut::Nonempty {
            list,
            forward,
            backward,
        } = self
        else {
            return None;
        };
        let finished = forward == backward;
        let Node { item, prev, .. } = &mut list.items[*backward];

        *backward = *prev;

        // SAFETY: since 'backward' now points to the next item, this item won't be aliased again
        // by this iterator. Since it lives for 'list, there is no way to get another reference
        // to it until this returned reference is dead.
        let item_extended = unsafe { &mut *(item as *mut _) };

        if finished {
            *self = IterMut::Empty;
        }
        Some(item_extended)
    }
}

impl<'list, T> iter::FusedIterator for IterMut<'list, T> {}

impl<'list, T> IntoIterator for &'list mut List<T> {
    type IntoIter = IterMut<'list, T>;
    type Item = &'list mut T;

    fn into_iter(self) -> Self::IntoIter {
        if self.is_empty() {
            IterMut::Empty
        } else {
            let forward = self.front;
            let backward = self.back;

            IterMut::Nonempty {
                list: self,
                forward,
                backward,
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other)
    }
}

impl<T: Eq> Eq for List<T> {}

impl<T: PartialOrd> PartialOrd for List<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for List<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.iter().cmp(other)
    }
}

impl<T: hash::Hash> hash::Hash for List<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.iter().for_each(|item| item.hash(state));
    }
}

#[derive(Clone)]
pub struct Cursor<'list, T> {
    list: &'list List<T>,
    at: usize,
}

impl<'list, T> Cursor<'list, T> {
    pub fn next(&mut self) -> bool {
        if self.at != self.list.back {
            self.at = self.list.items[self.at].next;
            true
        } else {
            false
        }
    }

    pub fn prev(&mut self) -> bool {
        if self.at != self.list.front {
            self.at = self.list.items[self.at].prev;
            true
        } else {
            false
        }
    }
}

impl<'list, T> ops::Deref for Cursor<'list, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.list.items[self.at].item
    }
}

impl<'list, T> PartialEq for Cursor<'list, T> {
    fn eq(&self, other: &Self) -> bool {
        (&self.list as *const _) == (&other.list as *const _) && self.at == other.at
    }
}

impl<'list, T> Eq for Cursor<'list, T> {}

pub struct CursorMut<'list, T> {
    list: &'list mut List<T>,
    at: usize,
}

impl<'list, T> CursorMut<'list, T> {
    pub fn next(&mut self) -> bool {
        if self.at == self.list.back {
            false
        } else {
            self.at = self.list.items[self.at].next;
            true
        }
    }

    pub fn prev(&mut self) -> bool {
        if self.at == self.list.front {
            false
        } else {
            self.at = self.list.items[self.at].prev;
            true
        }
    }

    pub fn insert_after(&mut self, item: T) {
        if self.at == self.list.back {
            self.list.push_back(item);
        } else {
            let items = &mut self.list.items;
            let next = items[self.at].next;
            items.push(Node {
                item,
                next,
                prev: self.at,
            });
            let new = items.len() - 1;
            items[next].prev = new;
            items[self.at].next = new;
        }
    }

    pub fn insert_before(&mut self, item: T) {
        if self.at == self.list.front {
            self.list.push_front(item);
        } else {
            let items = &mut self.list.items;
            let prev = items[self.at].prev;
            items.push(Node {
                item,
                next: self.at,
                prev,
            });
            let new = items.len() - 1;
            items[prev].next = new;
            items[self.at].prev = new;
        }
    }

    pub fn remove(&mut self) -> Option<T> {
        if self.at == self.list.front {
            self.next();
            self.list.pop_front()
        } else if self.at == self.list.back {
            self.prev();
            self.list.pop_back()
        } else {
            let items = &mut self.list.items;
            let Node { item, next, .. } = items.swap_remove(self.at);

            {
                let swapped = self.at;
                let Node { next, prev, .. } = items[swapped];

                items[prev].next = swapped;
                items[next].prev = swapped;
            }

            self.at = next;

            Some(item)
        }
    }
}

impl<'list, T> ops::Deref for CursorMut<'list, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.list.items[self.at].item
    }
}

impl<'list, T> ops::DerefMut for CursorMut<'list, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list.items[self.at].item
    }
}

pub struct ExtractIf<'list, T, F> {
    cursor: Option<CursorMut<'list, T>>,
    filter: F,
}

impl<'list, T, F> Iterator for ExtractIf<'list, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut cursor = self.cursor.as_mut()?;

        if cursor.list.is_empty() {
            None
        } else {
            while !(self.filter)(&mut cursor) {
                if !cursor.next() {
                    return None;
                }
            }
            cursor.remove()
        }
    }
}

impl<'list, T, F> iter::FusedIterator for ExtractIf<'list, T, F> where F: FnMut(&mut T) -> bool {}

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
    fn rev() {
        let mut list = List::new();
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

    #[test]
    fn clone() {
        let list: List<_> = (0..10).filter(|item| *item != 5).collect();

        let list_clone = list.clone();
        assert_eq!(list, list_clone);
    }

    #[test]
    fn ord() {
        let smaller = List::from_iter([1, 2, 3]);
        let mut larger = List::new();
        larger.push_back(2);
        larger.push_back(4);
        larger.push_front(1);

        assert!(smaller < larger);
        assert!(larger > smaller);
    }

    #[test]
    fn extend() {
        let mut list = List::from_iter("the quick brown fox".split_whitespace());
        list.extend("jumps over the lazy dog".split_whitespace());

        let words: Vec<_> = list.iter().copied().collect();
        assert_eq!(
            words.join(" "),
            "the quick brown fox jumps over the lazy dog"
        );
    }

    #[test]
    fn cursor() {
        let list: List<_> = [true, false, true, false, false].into_iter().collect();
        let mut cursor = list.cursor_front().unwrap();
        assert!(*cursor);
        cursor.next();
        assert!(!*cursor);
        cursor.next();
        assert!(*cursor);
        cursor.next();
        assert!(!*cursor);
        cursor.next();
        assert!(!*cursor);
        cursor.prev();
        assert!(!*cursor);
        cursor.prev();
        assert!(*cursor);
    }

    #[test]
    fn cursor_backwards() {
        let list: List<_> = "dlrow olleh".chars().collect();
        let mut cursor = list.cursor_back().unwrap();
        let mut message = String::new();

        message.push(*cursor);
        while cursor.prev() {
            message.push(*cursor);
        }

        assert_eq!(message, "hello world");
    }

    #[test]
    fn cursor_mut() {
        let mut list: List<_> = "hello world".chars().collect();
        let mut cursor = list.cursor_front_mut().unwrap();

        cursor.make_ascii_uppercase();
        while cursor.next() {
            cursor.make_ascii_uppercase();
        }

        let message: String = list.into_iter().collect();
        assert_eq!(message, "HELLO WORLD");
    }

    #[test]
    fn pop_back() {
        let mut list = List::new();

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
}
