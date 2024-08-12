use crate::{core::Res, utils::out::Out};
use anyhow::Context;
use crossterm::{
    cursor::{EnableBlinking, MoveToColumn, Show},
    queue,
    style::Print,
};
use std::iter::{self, Once, Repeat, Take};

const TAB_SIZE: usize = 4;

fn char_width(c: char) -> usize {
    match c {
        '\t' => TAB_SIZE,
        _ => 1,
    }
}

#[derive(Debug)]
enum CharIter {
    Tab(Take<Repeat<char>>),
    SingleChar(Once<char>),
}

impl CharIter {
    fn new(c: char) -> Self {
        match c {
            '\t' => Self::Tab(iter::repeat(' ').take(TAB_SIZE)),
            _ => Self::SingleChar(iter::once(c)),
        }
    }
}

impl Iterator for CharIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CharIter::Tab(iter) => iter.next(),
            CharIter::SingleChar(iter) => iter.next(),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Line {
    content: String,
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self { content }
    }
}

impl AsRef<str> for Line {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Index {
    display: usize,
    byte: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum RawIndex {
    Valid(Index),
    Invalid { display: usize },
}

impl RawIndex {
    pub fn index_front() -> Self {
        Self::Valid(Default::default())
    }

    fn display(&self) -> usize {
        let &(Self::Valid(Index { display, .. }) | Self::Invalid { display }) = self;
        display
    }

    pub fn at_front(self) -> bool {
        self.display() == 0
    }

    pub fn invalidate(&mut self) {
        *self = match self {
            &mut Self::Valid(Index { display, .. }) => Self::Invalid { display },
            _ => *self,
        };
    }
}

impl From<Index> for RawIndex {
    fn from(valid: Index) -> Self {
        Self::Valid(valid)
    }
}

impl Line {
    fn indices_from(&self, from: Index) -> Res<impl Iterator<Item = Index> + '_> {
        Ok(iter::once(from).chain(
            self.content
                .get(from.byte..)
                .context("byte is on char boundary")?
                .chars()
                .scan(from, |index, c| {
                    index.display += char_width(c);
                    index.byte += c.len_utf8();
                    Some(*index)
                }),
        ))
    }

    fn rindices_from(&self, from: Index) -> Res<impl Iterator<Item = Index> + '_> {
        Ok(iter::once(from).chain(
            self.content
                .get(..from.byte)
                .context("byte is on char boundary")?
                .chars()
                .rev()
                .scan(from, |index, c| {
                    index.display -= char_width(c);
                    index.byte -= c.len_utf8();
                    Some(*index)
                }),
        ))
    }

    fn indices(&self) -> impl Iterator<Item = Index> + '_ {
        self.indices_from(Default::default()).expect("0 index")
    }

    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.content
            .chars()
            .flat_map(CharIter::new)
            .chain(iter::repeat(' '))
    }

    fn get(&self, index: Index) -> Res<char> {
        self.content
            .get(index.byte..)
            .context("byte is on char boundary")?
            .chars()
            .next()
            .context("byte is on char boundary")
    }

    pub fn correct_index(&self, index: RawIndex) -> Index {
        if let RawIndex::Valid(valid) = index {
            valid
        } else {
            let (Ok(valid) | Err(valid)) =
                self.indices().try_fold(Default::default(), |_, valid| {
                    if valid.display >= index.display() {
                        Err(valid)
                    } else {
                        Ok(valid)
                    }
                });

            valid
        }
    }

    pub fn index_forward(&self, index: Index) -> Res<Option<Index>> {
        Ok(self.indices_from(index)?.skip(1).next())
    }

    pub fn index_backward(&self, index: Index) -> Res<Option<Index>> {
        Ok(self.rindices_from(index)?.skip(1).next())
    }

    pub fn index_forward_word(&self, index: Index) -> Res<Option<Index>> {
        Ok(if self.at_back(index) {
            None
        } else {
            let find_nonalphanum = |index| match self.get(index) {
                Ok(c) => (!c.is_alphanumeric()).then(|| Ok(index)),
                Err(error) => Some(Err(error)),
            };

            Some(
                if let Some(result) = self.indices_from(index)?.find_map(find_nonalphanum) {
                    result?
                } else {
                    self.index_back(index.into())?
                },
            )
        })
    }

    pub fn index_backward_word(&self, index: Index) -> Res<Option<Index>> {
        Ok(if RawIndex::from(index).at_front() {
            None
        } else {
            let find_nonalphanum = |index| match self.get(index) {
                Ok(c) => (!c.is_alphanumeric()).then(|| Ok(index)),
                Err(error) => Some(Err(error)),
            };

            Some(
                if let Some(result) = self.rindices_from(index)?.find_map(find_nonalphanum) {
                    result?
                } else {
                    Default::default()
                },
            )
        })
    }

    pub fn index_back(&self, index: RawIndex) -> Res<Index> {
        Ok(match index {
            RawIndex::Valid(index) => self.indices_from(index)?.last().unwrap_or(index),
            _ => self.indices().last().unwrap_or_default(),
        })
    }

    pub fn split_at(&mut self, index: Index) -> Res<Self> {
        Ok(Self {
            content: self.content.split_off(index.byte),
        })
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn append(&mut self, other: impl AsRef<str>) {
        self.content += other.as_ref();
    }

    pub fn prepend(&mut self, other: impl AsRef<str>) {
        self.content.insert_str(0, other.as_ref());
    }

    pub fn at_back(&self, index: Index) -> bool {
        index.byte == self.content.len()
    }

    pub fn insert(&mut self, index: Index, c: char) {
        self.content.insert(index.byte, c);
    }

    pub fn remove(&mut self, index: Index) {
        self.content.remove(index.byte);
    }

    pub fn view(&self, out: &mut Out, x: u16, width: u16, active: Option<Index>) -> Res<()> {
        for c in self.chars().take(usize::from(width) - 1) {
            queue!(out, Print(c))?;
        }

        if let Some(index) = active {
            let column = x + u16::try_from(index.display)?.clamp(0, width - 1);
            queue!(out, MoveToColumn(column), Show, EnableBlinking)?;
        }

        Ok(())
    }
}
