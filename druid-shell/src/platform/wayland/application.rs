//! Implementation of features at the application scope.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    application::AppHandler, error::Error, platform::clipboard::Clipboard, platform::window::Window,
};

#[derive(Clone)]
pub struct Application {
    state: Rc<RefCell<State>>,
}

/// The mutable `Application` state.
#[derive(Default)]
struct State {
    /// Whether `Application::quit` has already been called.
    quitting: bool,
    /// A collection of all the `Application` windows.
    windows: HashMap<u32, Rc<Window>>,
}

impl Application {
    pub fn new() -> Result<Self, Error> {
        Ok(Application {
            state: Rc::new(RefCell::new(State::default())),
        })
    }

    pub fn run(self, handler: Option<Box<dyn AppHandler>>) {}

    pub fn quit(&self) {}

    pub fn clipboard(&self) -> Clipboard {
        Clipboard {}
    }

    pub fn get_locale() -> String {
        "en-US".into()
    }
}
