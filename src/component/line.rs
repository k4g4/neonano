use crate::{
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::Out,
};
use anyhow::Context;
use crossterm::{
    cursor::{EnableBlinking, MoveToColumn, Show},
    style::Print,
    QueueableCommand,
};
use std::iter;

const TAB_SIZE: usize = 4;

fn char_width(c: char) -> usize {
    match c {
        '\t' => TAB_SIZE,
        _ => 1,
    }
}

fn char_map(c: char) -> char {
    match c {
        '\t' => ' ',
        _ => c,
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
        Ok(self
            .content
            .get(from.byte..)
            .context("byte is on char boundary")?
            .chars()
            .scan(from, |state, c| {
                let index = *state;
                state.display += char_width(c);
                state.byte += c.len_utf8();
                Some(index)
            }))
    }

    fn rindices_from(&self, from: Index) -> Res<impl Iterator<Item = Index> + '_> {
        Ok(self
            .content
            .get(..from.byte)
            .context("byte is on char boundary")?
            .chars()
            .rev()
            .scan(from, |state, c| {
                let index = *state;
                state.display -= char_width(c);
                state.byte -= c.len_utf8();
                Some(index)
            }))
    }

    fn indices(&self) -> impl Iterator<Item = Index> + '_ {
        self.indices_from(Default::default()).expect("0 index")
    }

    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.content
            .chars()
            .flat_map(|c| iter::repeat(char_map(c)).take(char_width(c)))
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
                        Ok(Index {
                            display: valid.display + 1,
                            ..valid
                        })
                    }
                });

            valid
        }
    }

    pub fn index_forward(&self, index: Index) -> Res<Option<Index>> {
        Ok(self.indices_from(index)?.next())
    }

    pub fn index_backward(&self, index: Index) -> Res<Option<Index>> {
        Ok(self.rindices_from(index)?.next())
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

    pub fn append(&mut self, other: Self) {
        self.content += &other.content;
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

    /*
                pressed!(Key::Home) => Ok(Default::default()),

                pressed!(Key::End) => Ok(self.last_index(index.into())?.unwrap_or(index)),

                &pressed!(Key::Char(c)) => {
                    self.insert(index, c);

                    self.next_index(index)?.context("inserted new char")
                }

                pressed!(Key::Backspace, ctrl) => {
                    let start =
                        if let Some(result) = self.rindices_from(index)?.find_map(find_nonalphanum) {
                            result?
                        } else {
                            Default::default()
                        };

                    self.content.drain(start.byte..index.byte);

                    Ok(start)
                }

                pressed!(Key::Backspace) => {
                    if let Some(prev) = self.prev_index(index)? {
                        self.content.remove(prev.byte);

                        Ok(prev)
                    } else {
                        Ok(index)
                    }
                }

                pressed!(Key::Delete, ctrl) => {
                    let end = if let Some(result) = self.indices_from(index)?.find_map(find_nonalphanum)
                    {
                        result?
                    } else {
                        self.last_index(index.into())?.unwrap_or(index)
                    };

                    self.content.drain(index.byte..end.byte);

                    Ok(index)
                }

                pressed!(Key::Delete) => {
                    self.content.remove(index.byte);

                    Ok(index)
                }

                _ => Ok(index),
            }
        }
    */
    pub fn view(&self, out: &mut Out, width: u16, active: Option<Index>) -> Res<()> {
        for c in self.chars().take(width.into()) {
            out.queue(Print(c))?;
        }

        if let Some(index) = active {
            out.queue(MoveToColumn(index.display.try_into()?))?
                .queue(Show)?
                .queue(EnableBlinking)?;
        }

        Ok(())
    }
}
