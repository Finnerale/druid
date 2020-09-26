#![allow(dead_code)]

use crate::{common_util::IdleCallback, IdleToken};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/// A handle that can get used to schedule an idle handler. Note that
/// this handle can be cloned and sent between threads.
#[derive(Clone)]
pub struct IdleHandle {
    pub(crate) queue: Arc<Mutex<Vec<IdleKind>>>,
}

pub(crate) enum IdleKind {
    Callback(Box<dyn IdleCallback>),
    Token(IdleToken),
    Redraw,
}

impl IdleHandle {
    fn wake(&self) {
        unimplemented!()
    }

    pub(crate) fn schedule_redraw(&self) {
        self.queue.lock().unwrap().push(IdleKind::Redraw);
        self.wake();
    }

    pub fn add_idle_callback<F>(&self, callback: F)
    where
        F: FnOnce(&dyn Any) + Send + 'static,
    {
        self.queue
            .lock()
            .unwrap()
            .push(IdleKind::Callback(Box::new(callback)));
        self.wake();
    }

    pub fn add_idle_token(&self, token: IdleToken) {
        self.queue.lock().unwrap().push(IdleKind::Token(token));
        self.wake();
    }
}
