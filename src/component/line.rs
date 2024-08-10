use crate::{
    core::Res,
    message::{Input, Key, KeyCombo, Message},
    pressed,
    utils::out::Out,
};
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

#[derive(Clone, Default, Debug)]
pub struct Line {
    content: String,
    active_byte: usize,
}

impl From<String> for Line {
    fn from(content: String) -> Self {
        Self {
            content,
            active_byte: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct CharIndex {
    disp_index: usize,
    char_index: usize,
    byte_index: usize,
}

impl Line {
    fn indices(&self) -> impl Iterator<Item = CharIndex> + '_ {
        self.content.char_indices().enumerate().scan(
            0,
            |disp_index_state, (char_index, (byte_index, c))| {
                let disp_index = *disp_index_state;
                *disp_index_state += char_width(c);

                Some(CharIndex {
                    disp_index,
                    char_index,
                    byte_index,
                })
            },
        )
    }

    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.content
            .chars()
            .flat_map(|c| {
                iter::repeat(match c {
                    '\t' => ' ',
                    _ => c,
                })
                .take(char_width(c))
            })
            .chain(iter::repeat(' '))
    }

    pub fn active(&self) -> usize {
        self.indices()
            .find_map(|index| (index.byte_index == self.active_byte).then(|| index.disp_index))
            .unwrap_or(self.content.len())
    }

    pub fn set_active(&mut self, display_index: usize) {
        let byte_index = if let Some(index) = self
            .indices()
            .find(|index| index.disp_index >= display_index)
        {
            index.byte_index
        } else {
            self.content.len()
        };

        self.active_byte = byte_index;
    }

    pub fn set_active_front(&mut self) {
        self.active_byte = 0;
    }

    pub fn set_active_back(&mut self) {
        self.active_byte = self.content.len();
    }

    pub fn set_active_next(&mut self) {
        if let Some(forward) = self.content[self.active_byte..]
            .char_indices()
            .skip(1)
            .next()
            .map(|(i, _)| i)
        {
            self.active_byte += forward;
        } else {
            self.set_active_back();
        }
    }

    pub fn set_active_prev(&mut self) {
        self.active_byte = self.content[..self.active_byte]
            .char_indices()
            .rev()
            .next()
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    pub fn split(&mut self) -> Self {
        Self {
            content: self.content.split_off(self.active_byte),
            active_byte: 0,
        }
    }

    pub fn append(&mut self, other: Self) {
        self.content += &other.content;
    }

    pub fn at_front(&self) -> bool {
        self.active_byte == 0
    }

    pub fn at_back(&self) -> bool {
        self.active_byte == self.content.len()
    }

    pub fn insert(&mut self, c: char) {
        self.content.insert(self.active_byte, c);
    }

    pub fn remove(&mut self) {
        self.content.remove(self.active_byte);
    }

    pub fn update(&mut self, message: &Message) -> Res<()> {
        let nonalphanum = |(i, c): (_, char)| (!c.is_alphanumeric()).then(|| i);

        match message {
            pressed!(Key::Left, ctrl) => {
                self.active_byte = self.content[..self.active_byte]
                    .char_indices()
                    .rev()
                    .find_map(nonalphanum)
                    .unwrap_or(0)
                    .saturating_sub(1);
            }

            pressed!(Key::Left) => {
                self.set_active_prev();
            }

            pressed!(Key::Right, ctrl) => {
                if let Some(forward) = self.content[self.active_byte..]
                    .char_indices()
                    .find_map(nonalphanum)
                {
                    self.active_byte += forward + 1;
                } else {
                    self.set_active_back();
                }
            }

            pressed!(Key::Right) => {
                self.set_active_next();
            }

            pressed!(Key::Home) => {
                self.set_active_front();
            }

            pressed!(Key::End) => {
                self.set_active_back();
            }

            &pressed!(Key::Char(c)) => {
                self.insert(c);
                self.set_active_next();
            }

            pressed!(Key::Backspace, ctrl) => {
                let prev_active_byte = self.active_byte;

                self.active_byte = self.content[..self.active_byte]
                    .char_indices()
                    .rev()
                    .find_map(nonalphanum)
                    .unwrap_or(0)
                    .saturating_sub(1);

                self.content.drain(self.active_byte..prev_active_byte);
            }

            pressed!(Key::Backspace) => {
                if !self.at_front() {
                    self.set_active_prev();
                    self.remove();
                }
            }

            pressed!(Key::Delete, ctrl) => {
                if let Some(forward) = self.content[self.active_byte..]
                    .char_indices()
                    .find_map(nonalphanum)
                {
                    let range = self.active_byte..self.active_byte + forward + 1;
                    self.content.drain(range);
                } else {
                    self.content.drain(self.active_byte..);
                };
            }

            pressed!(Key::Delete) => {
                if !self.at_back() {
                    self.remove();
                }
            }

            _ => {}
        }

        Ok(())
    }

    pub fn view(&self, out: &mut Out, width: u16, active: bool) -> Res<()> {
        for c in self.chars().take(width.into()) {
            out.queue(Print(c))?;
        }

        if active {
            out.queue(MoveToColumn(self.active().try_into()?))?
                .queue(Show)?
                .queue(EnableBlinking)?;
        }

        Ok(())
    }
}
