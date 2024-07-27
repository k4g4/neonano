use crate::core::Res;
use anyhow::anyhow;
use crossterm::event::{self, Event};
use std::{
    cell::Cell,
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
};

pub struct Input(Cell<Option<JoinHandle<Res<()>>>>, Receiver<Event>);

impl Input {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self(
            Cell::new(Some(thread::spawn(move || -> Res<()> {
                loop {
                    sender.send(event::read()?)?;
                }
            }))),
            receiver,
        )
    }

    pub fn read(&self) -> Res<impl Iterator<Item = Event> + '_> {
        if let Some(join_handle) = self.0.take() {
            if join_handle.is_finished() {
                Err(join_handle
                    .join()
                    .expect("input thread exited with error")
                    .expect_err("input thread only returns errors"))
            } else {
                self.0.set(Some(join_handle));
                Ok(self.1.try_iter())
            }
        } else {
            Err(anyhow!("input thread cannot be read from after an error"))
        }
    }
}
